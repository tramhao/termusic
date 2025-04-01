use std::{fmt::Debug, future::Future, iter::FusedIterator, sync::Arc, time::Duration, u8};

use anyhow::anyhow;
use async_ringbuf::{
    traits::{AsyncConsumer, AsyncProducer, Consumer, Observer, Split},
    AsyncHeapRb,
};
use parking_lot::RwLock;
use rodio::Source;
use symphonia::core::audio::SignalSpec;
use tokio::{
    runtime::Handle,
    sync::{mpsc, oneshot},
};

use wrap::{ConsWrap, ProdWrap};

mod wrap;

/// Seek Channel data, the first is the point to seek to in the decoder
/// the second is a callback that the seek is done and how many elements to skip until new data.
pub type SeekData = (Duration, oneshot::Sender<usize>);

#[derive(Debug)]
struct ProviderData {
    msg: RingMsgWrite,
}

/// The minimal size a decode-ringbuffer should have.
///
/// Currently the size is based on 192kHz * 2 seconds, equating to ~375kb buffer
const MIN_SIZE: usize = 192000 * 2;

#[derive(Debug)]
pub struct AsyncRingSourceProvider {
    inner: ProdWrap,
    seek_rx: Arc<RwLock<mpsc::Receiver<SeekData>>>,

    data: Option<ProviderData>,
}

impl AsyncRingSourceProvider {
    fn new(wrap: ProdWrap, seek_rx: mpsc::Receiver<SeekData>) -> Self {
        AsyncRingSourceProvider {
            inner: wrap,
            seek_rx: Arc::new(RwLock::new(seek_rx)),
            data: None,
        }
    }

    /// Check if the consumer ([`AsyncRingSource`]) is still connected and writes are possible
    #[allow(dead_code)] // cant use expect as this function is used in tests
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Write a new spec.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    pub async fn new_spec(
        &mut self,
        spec: SignalSpec,
        current_frame_len: usize,
    ) -> Result<usize, ()> {
        let mut msg_buf = [0; RingMsgWrite::get_msg_size(MessageSpec::MESSAGE_SIZE)];
        let mut msg = RingMsgWrite::new_spec(spec, current_frame_len);

        msg.try_write(&mut msg_buf);
        self.inner.push_exact(&msg_buf).await.map_err(|_| ())?;

        Ok(msg_buf.len())
    }

    /// Write a new data message, without the buffer yet.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    async fn new_data(&mut self, length: usize) -> Result<usize, ()> {
        let mut msg_buf = [0; RingMsgWrite::get_msg_size(MessageDataFirst::MESSAGE_SIZE)];
        let mut msg = RingMsgWrite::new_data(length);

        // TODO: avoid extra "msg_buf" or internal msg buffer
        let written = msg.try_write(&mut msg_buf);
        assert!(written == msg_buf.len());
        self.inner.push_exact(&msg_buf).await.map_err(|_| ())?;

        let msg = msg.finish_data_first();
        self.data = Some(ProviderData { msg: msg });

        Ok(msg_buf.len())
    }

    /// Write a buffer's content.
    ///
    /// This functions expects a data message to be active.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    async fn write_data_inner(&mut self, data: &[u8]) -> Result<usize, ()> {
        let Some(pdata) = &mut self.data else {
            unimplemented!("This should be checked outside of the function");
        };
        let RingMsgWrite { msg, .. } = &mut pdata.msg;
        let RingMsgParse::DataActual(msg) = msg else {
            unimplemented!("This should be checked outside of the function");
        };

        let buf = &data[msg.read..msg.length];
        self.inner.push_exact(buf).await.map_err(|_| ())?;
        msg.advance_read(buf.len());

        Ok(buf.len())
    }

    /// Write a given buffer as a data message.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    pub async fn write_data(&mut self, data: &[u8]) -> Result<usize, ()> {
        if data.is_empty() {
            return Err(());
        }

        let mut written = 0;
        if self.data.is_none() {
            written += self.new_data(data.len()).await?
        }

        while !self.data.as_mut().unwrap().msg.is_done() && !self.inner.is_closed() {
            written += self.write_data_inner(data).await?;
        }

        self.data.take();

        Ok(written)
    }

    /// Write a EOS message.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    pub async fn new_eos(&mut self) -> Result<usize, ()> {
        let mut msg_buf = [0; RingMsgWrite::get_msg_size(0)];
        let mut msg = RingMsgWrite::new_eos();

        msg.try_write(&mut msg_buf);
        self.inner.push_exact(&msg_buf).await.map_err(|_| ())?;

        Ok(msg_buf.len())
    }

    /// Wait until the seek channel is dropped([`None`]) or a seek is requested([`Some`]).
    pub fn wait_seek(&mut self) -> WaitSeek {
        WaitSeek(self.seek_rx.clone())
    }

    /// Process a seek and call the Consumer to resume.
    ///
    /// This clear all data state, calls the consumer to resume with how many bytes to skip
    /// and sends the new spec (in case it changed in the seek).
    pub async fn process_seek(
        &mut self,
        spec: SignalSpec,
        current_frame_len: usize,
        cb: oneshot::Sender<usize>,
    ) {
        self.data.take();
        let bytes_to_skip = self.inner.occupied_len();
        let _ = cb.send(bytes_to_skip);
        let _ = self.new_spec(spec, current_frame_len).await;
    }
}

#[derive(Debug)]
pub struct WaitSeek(Arc<RwLock<mpsc::Receiver<SeekData>>>);

impl Future for WaitSeek {
    type Output = Option<SeekData>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.0.write().poll_recv(cx)
    }
}

#[derive(Debug)]
pub struct AsyncRingSource {
    inner: ConsWrap,
    /// Send a seek-to value to the producer.
    /// Also used to indicate to the producer that it should keep active.
    seek_tx: Option<mpsc::Sender<SeekData>>,

    // random default size, 1024*32
    buf: StaticBuf<32768>,
    last_msg: Option<MessageDataActual>,
    handle: Handle,

    // cached information on how to treat current data until a update
    channels: u16,
    rate: u32,
    /// Always positive, unless EOS had been reached.
    current_frame_len: usize,
    total_duration: Option<Duration>,
}

impl AsyncRingSource {
    /// Create a new ringbuffer, with initial channel spec and at least [`MIN_SIZE`].
    pub fn new(
        spec: SignalSpec,
        total_duration: Option<Duration>,
        current_frame_len: usize,
        size: usize,
        handle: Handle,
    ) -> (AsyncRingSourceProvider, Self) {
        let size = size;
        let size = size.max(MIN_SIZE);
        let ringbuf = AsyncHeapRb::<u8>::new(size);
        let (prod, cons) = ringbuf.split();
        let (tx, rx) = mpsc::channel(1);

        let async_prod = AsyncRingSourceProvider::new(ProdWrap::new(prod), rx);
        let async_cons = Self {
            inner: ConsWrap::new(cons),
            seek_tx: Some(tx),
            channels: u16::try_from(spec.channels.count())
                .expect("Channel size to be within u16::MAX"),
            rate: spec.rate,
            total_duration: total_duration,
            current_frame_len: current_frame_len,
            last_msg: None,
            buf: StaticBuf::new(),
            handle: handle,
        };

        (async_prod, async_cons)
    }

    /// Ensure there is a complete message in `last_msg`.
    ///
    /// This function assumes there is no current message.
    #[must_use]
    async fn read_msg(&mut self) -> Option<RingMsgParse2> {
        // trace!("Reading a message from the ringbuffer");

        let detected_type = {
            let detect_byte = if self.buf.is_empty() {
                self.inner.pop().await?
            } else {
                let byte = self.buf.get_ref()[0];
                self.buf.advance_beginning(1);
                byte
            };

            RingMessages::from_u8(detect_byte)
        };

        // Eos event does not have more than the id itself
        if detected_type == RingMessages::EOS {
            return Some(RingMsgParse2::Eos);
        }

        let mut wait_for_bytes = 1;
        let mut total = 0;
        loop {
            total += 1;
            // "buf.is_empty" is safe here as all messages consume the buffer fully here.
            if self.inner.is_closed() && self.inner.is_empty() && self.buf.is_empty() {
                return None;
            }

            self.load_more_data(wait_for_bytes).await?;

            // Sanity check against infinite loop
            assert!(total < 10);

            let (msg, read) = match detected_type {
                RingMessages::Data => {
                    let (data_res, read) = match MessageDataFirst::try_read_buf(self.buf.get_ref())
                    {
                        Ok(v) => v,
                        Err(wait_for) => {
                            wait_for_bytes = wait_for + self.buf.len();
                            continue;
                        }
                    };

                    (RingMsgParse2::Data(data_res), read)
                }
                RingMessages::Spec => {
                    let (spec_res, read) = match MessageSpec::try_read_buf(self.buf.get_ref()) {
                        Ok(v) => v,
                        Err(wait_for) => {
                            wait_for_bytes = wait_for + self.buf.len();
                            continue;
                        }
                    };

                    self.apply_spec_msg(spec_res);

                    (RingMsgParse2::Spec, read)
                }
                RingMessages::EOS => unreachable!("Message EOS is returned earlier"),
            };

            assert!(read > 0);

            self.buf.advance_beginning(read);

            return Some(msg);
        }
    }

    /// Loads more data into the current buffer, if the current buffer does not have at least `wait_for_bytes` bytes.
    ///
    /// Returns [`Some`] if the current buffer now has at least `wait_for_bytes` buffered, [`None`] if the buffer closed and not enough can be loaded anymore.
    async fn load_more_data(&mut self, wait_for_bytes: usize) -> Option<()> {
        if self.buf.len() >= wait_for_bytes {
            return Some(());
        }

        self.buf.maybe_need_move();

        // wait for at least one element being occupied,
        // more elements would mean to wait until they all are there, regardless if they are part of the message or not
        self.inner.wait_occupied(wait_for_bytes).await;

        if self.inner.is_closed() && self.inner.is_empty() {
            return None;
        }

        // dont overwrite data that may still be in there
        let write_from = self.buf.len();
        let written = self.inner.pop_slice(&mut self.buf.get_mut()[write_from..]);
        self.buf.set_len(written + write_from);

        // Sanity
        assert!(self.buf.len() == written + write_from);
        // Sanity, there may be infinite loop because of bad implementation
        debug_assert!(written > 0);

        Some(())
    }

    /// Apply a new spec from the current message.
    ///
    /// This function assumes the current message is a [`MessageSpec`].
    fn apply_spec_msg(&mut self, new_spec: MessageSpecResult) {
        self.channels = new_spec.channels;
        self.rate = new_spec.rate;
    }

    /// Read data from a Data Message.
    ///
    /// This function assumes the current message is a non-finished [`MessageDataActual`].
    #[must_use]
    async fn read_data(&mut self) -> Option<i16> {
        // trace!("Reading Data");

        // wait until we have enough data to parse a value
        self.load_more_data(MessageDataValue::MESSAGE_SIZE).await?;

        assert!(self.buf.len() >= MessageDataValue::MESSAGE_SIZE);

        let msg = self.last_msg.as_mut().unwrap();

        // unwrap: should never panic as we explicitly load at least the required amount above.
        let (sample, read) = msg.try_read_buf(self.buf.get_ref()).unwrap();
        self.buf.advance_beginning(read);

        if msg.is_done() {
            self.last_msg.take();
        }

        Some(sample)
    }
}

impl Source for AsyncRingSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.current_frame_len)
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn channels(&self) -> u16 {
        self.channels
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        trace!("Consumer Seek");

        // clear the ringbuffer before sending, in case it is full and to more quickly unblock the producer
        // though this should not be relied upon
        self.inner.clear();
        self.last_msg.take();
        self.buf.clear();

        let (cb_tx, cb_rx) = oneshot::channel();
        let _ = self.seek_tx.as_mut().unwrap().blocking_send((pos, cb_tx));

        // Wait for the Producer to have processed the seek and get the final value of elements to skip
        let to_skip = cb_rx.blocking_recv().map_err(|_| {
            rodio::source::SeekError::Other(
                anyhow!("Seek Callback channel exited unexpectedly").into(),
            )
        })?;

        // skip possible new elements
        let _ = self.inner.skip(to_skip);
        trace!("Consumer Seek Done");

        Ok(())
    }
}

impl Iterator for AsyncRingSource {
    type Item = ValueType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame_len == 0 {
            return None;
        }

        loop {
            if self.last_msg.is_some() {
                let sample = self.handle.clone().block_on(self.read_data());

                return sample;
            }

            if self.inner.is_empty() && self.inner.is_closed() {
                debug!("DONE");
                return None;
            }

            let msg = self.handle.clone().block_on(self.read_msg())?;

            match msg {
                RingMsgParse2::Spec => {}
                RingMsgParse2::Data(message_data_actual) => {
                    self.last_msg = Some(message_data_actual);
                }
                RingMsgParse2::Eos => {
                    if let Some(_) = self.seek_tx {
                        // only write the message once
                        trace!("Reached EOS message");
                    }
                    // this indicates to rodio via Source::current_frame_len that there is no more data
                    // and we also use it to uphold the contract with FusedIterator.
                    self.current_frame_len = 0;
                    return None;
                }
            }
        }
    }
}

// Contract: once reaching a EOS or the ringbuffer being closed & having read all data, it will continue outputting None
impl FusedIterator for AsyncRingSource {}

/// Static Buffer allocated on the stack, which can have a moving area of actual data within
#[derive(Debug, Clone, Copy)]
struct StaticBuf<const N: usize> {
    buf: [u8; N],
    /// The length of the good data. 0 means 0 good elements (like `.len()`)
    used_len: usize,
    /// The length of data to skip until next clear / [`make_beginning`](Self::make_beginning) call.
    data_start_idx: usize,
}

impl<const N: usize> StaticBuf<N> {
    const CAPACITY: usize = N;

    /// Create a new buffer.
    ///
    /// Size must be above 0 and and divideable by 2.
    fn new() -> Self {
        const {
            assert!(N > 0);
            assert!(N % 2 == 0);
        }
        Self {
            buf: [0; N],
            used_len: 0,
            data_start_idx: 0,
        }
    }

    /// The length of actual data in the buffer
    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Rest good data length.
    fn clear(&mut self) {
        self.data_start_idx = 0;
        self.used_len = 0;

        // // DEBUG: this should not be necessary, but for debugging the buffer
        // self.buf.fill(u8::MAX);
    }

    /// Get a mutable reference to the slice from data_start until end.
    ///
    /// May contain bad data.
    /// And [`advance_len`](Self::advance_len) needs to be called afterward with the written size.
    fn get_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.data_start_idx..]
    }

    /// Get a reference to the slice which contains good data
    fn get_ref(&self) -> &[u8] {
        &self.buf[self.data_start_idx..self.used_len]
    }

    /// Move the data to the beginning, if start is above half the capacity
    fn maybe_need_move(&mut self) {
        // Fast-path: clear if start idx is above 0 and there are no good elements
        if self.data_start_idx > 0 && self.len() == 0 {
            self.clear();
            return;
        }

        // Only move to the beginning if the start idx is above half the buffer size.
        if self.data_start_idx > Self::CAPACITY / 2 {
            self.make_beginning();
        }
    }

    /// Move the data to beginning if it is not already.
    fn make_beginning(&mut self) {
        let range = self.data_start_idx..self.used_len;
        let range_len = range.len();
        self.buf.copy_within(range, 0);
        self.used_len = range_len;
        self.data_start_idx = 0;

        // // DEBUG: this should not be necessary, but for debugging the buffer
        // self.buf[range_len..].fill(u8::MAX);
    }

    /// Advance the initialized data length.
    #[expect(dead_code)]
    fn advance_len(&mut self, size: usize) {
        self.used_len += size;
        assert!(self.used_len <= self.buf.len());
    }

    /// Set the length of the buffer to the written size plus data start, ie how the buffer was given from [`get_mut`](Self::get_mut)
    fn set_len(&mut self, written: usize) {
        self.used_len = self.data_start_idx + written;
        assert!(self.used_len <= self.buf.len());
    }

    /// Advance the start index.
    ///
    /// Use [`make_beginning`](Self::make_beginning) to move all data to the front again.
    fn advance_beginning(&mut self, by: usize) {
        self.data_start_idx += by;
        assert!(self.data_start_idx <= self.buf.len());

        // // DEBUG: this should not be necessary, but for debugging the buffer
        // self.buf[0..self.data_start_idx].fill(u8::MAX);
    }
}

/// Types of messages that could appear in the ringbuffer with their id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum RingMessages {
    // dont use 0 to differentiate from default buffer values.
    Data = 1,
    Spec = 2,
    EOS = 3,
}

impl RingMessages {
    /// Convert from a byte to a instance of this enum.
    ///
    /// This will panic if a unknown byte is given, as there is only ever expected to be known bytes.
    fn from_u8(byte: u8) -> Self {
        match byte {
            1 => Self::Data,
            2 => Self::Spec,
            3 => Self::EOS,
            v => unimplemented!(
                "This should never happen, unless there is de-sync. byte: {}",
                v
            ),
        }
    }

    /// Get the current instance's [`u8`] representation
    fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Writer for Ringbuffer messages.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RingMsgWrite {
    id_written: bool,
    msg: RingMsgParse,
}

impl RingMsgWrite {
    const ID_SIZE: usize = 1;

    /// Add the [`ID_SIZE`](Self::ID_SIZE) to the given size to get the full message size.
    const fn get_msg_size(size: usize) -> usize {
        Self::ID_SIZE + size
    }

    fn new_spec(spec: SignalSpec, current_frame_len: usize) -> Self {
        Self {
            id_written: false,
            msg: RingMsgParse::Spec(MessageSpec::new_write(spec, current_frame_len)),
        }
    }

    fn new_data(length: usize) -> Self {
        Self {
            id_written: false,
            msg: RingMsgParse::DataFirst(MessageDataFirst::new_write(length)),
        }
    }

    fn new_eos() -> Self {
        Self {
            id_written: false,
            msg: RingMsgParse::EOS,
        }
    }

    /// Is the message fully written to the buffer?
    fn is_done(&self) -> bool {
        if !self.id_written {
            return false;
        }

        match &self.msg {
            RingMsgParse::DataFirst(message_data_first) => message_data_first.is_done(),
            RingMsgParse::DataActual(message_data_actual) => message_data_actual.is_done(),
            RingMsgParse::Spec(message_spec) => message_spec.is_done(),
            RingMsgParse::EOS => true,
        }
    }

    /// Try writing the remaining bytes to the given buffer.
    fn try_write(&mut self, mut buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        let mut written = 0;

        if !self.id_written {
            buf[0] = match self.msg {
                RingMsgParse::DataFirst(_) => RingMessages::Data.as_u8(),
                RingMsgParse::DataActual(_) => unreachable!(),
                RingMsgParse::Spec(_) => RingMessages::Spec.as_u8(),
                RingMsgParse::EOS => RingMessages::EOS.as_u8(),
            };

            self.id_written = true;
            buf = &mut buf[Self::ID_SIZE..];
            written += Self::ID_SIZE;
        }

        written += match &mut self.msg {
            RingMsgParse::DataFirst(message_data_first) => message_data_first.try_write(buf),
            RingMsgParse::DataActual(_message_data_actual) => 0,
            RingMsgParse::Spec(message_spec) => message_spec.try_write(buf),
            RingMsgParse::EOS => 0,
        };

        written
    }

    fn finish_data_first(self) -> Self {
        let RingMsgParse::DataFirst(data) = self.msg else {
            unimplemented!("This should be checked outside of the function");
        };

        let actual = data.finish();

        Self {
            id_written: true,
            msg: RingMsgParse::DataActual(actual),
        }
    }

    fn try_write_data(&mut self, in_buf: &[u8], out_buf: &mut [u8]) -> usize {
        let RingMsgParse::DataActual(data) = &mut self.msg else {
            unimplemented!("This should be checked outside of the function");
        };

        data.try_write(in_buf, out_buf)
    }
}

/// Reader for Ringbuffer messages.
#[derive(Debug, Clone, Copy, PartialEq)]
enum RingMsgParse2 {
    Spec,
    Data(MessageDataActual),
    Eos,
}

/// Reader for Ringbuffer messages.
#[derive(Debug, Clone, Copy, PartialEq)]
enum RingMsgParse {
    DataFirst(MessageDataFirst),
    DataActual(MessageDataActual),
    Spec(MessageSpec),
    EOS,
}

/// Copy from `in_buf` into `out_buf`, returning the bytes copied
fn copy_buffers(in_buf: &[u8], out_buf: &mut [u8]) -> usize {
    // the position to copy to, exclusive
    let copy_to_pos = in_buf.len().min(out_buf.len());

    out_buf[..copy_to_pos].copy_from_slice(&in_buf[..copy_to_pos]);

    copy_to_pos
}

type PResult<T> = Result<(T, usize), usize>;

/// The Content and result of a [`RingMessages::Spec`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MessageSpecResult {
    rate: u32,
    channels: u16,
    current_frame_len: usize,
}

/// Read (and Write) a [`RingMessages::Spec`].
#[derive(Debug, Clone, Copy, PartialEq)]
struct MessageSpec {
    buffer: [u8; Self::MESSAGE_SIZE],
    read: usize,
}

const USIZE_LEN: usize = size_of::<usize>();

// TODO: maybe use nom or similar?
impl MessageSpec {
    const MESSAGE_SIZE: usize = size_of::<u32>() + size_of::<u16>() + size_of::<usize>();

    /// Create a new instance with the buffer filled already for writing.
    fn new_write(spec: SignalSpec, current_frame_len: usize) -> Self {
        let mut buf = [0; Self::MESSAGE_SIZE];
        (buf[..=3]).copy_from_slice(&spec.rate.to_ne_bytes());
        let channels_u16 = u16::try_from(spec.channels.count()).unwrap();
        (buf[4..=5]).copy_from_slice(&channels_u16.to_ne_bytes());
        (buf[6..6 + USIZE_LEN]).copy_from_slice(&current_frame_len.to_ne_bytes());
        Self {
            buffer: buf,
            read: 0,
        }
    }

    fn is_done(&self) -> bool {
        self.read == Self::MESSAGE_SIZE
    }

    /// Try to read a message from the given buffer, or return how many bytes are still necessary
    fn try_read_buf(buf: &[u8]) -> PResult<MessageSpecResult> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        // read u32
        let rate: [u8; 4] = buf[0..=3].try_into().unwrap();
        // read u16
        let channels: [u8; 2] = buf[4..=5].try_into().unwrap();
        // read usize
        let current_frame_len: [u8; USIZE_LEN] = buf[6..6 + USIZE_LEN].try_into().unwrap();

        let rate = u32::from_ne_bytes(rate);
        let channels = u16::from_ne_bytes(channels);
        let current_frame_len = usize::from_ne_bytes(current_frame_len);

        Ok((
            MessageSpecResult {
                rate,
                channels,
                current_frame_len,
            },
            Self::MESSAGE_SIZE,
        ))
    }

    /// Try to write the current buffer into the given buffer.
    ///
    /// This function will always return how many bytes have been writter to the buffer.
    fn try_write(&mut self, buf: &mut [u8]) -> usize {
        if self.is_done() || buf.is_empty() {
            return 0;
        }

        let written = copy_buffers(&self.buffer[self.read..], buf);
        self.read += written;

        written
    }
}

/// The first part of a [`RingMessages::Data`], reading the length.
#[derive(Debug, Clone, Copy, PartialEq)]
struct MessageDataFirst {
    buffer: [u8; Self::MESSAGE_SIZE],
    read: usize,
}

// TODO: maybe use nom or similar?
impl MessageDataFirst {
    const MESSAGE_SIZE: usize = size_of::<usize>();

    /// Create a new instance with the buffer filled already for writing.
    fn new_write(length: usize) -> Self {
        let mut buf = [0; Self::MESSAGE_SIZE];
        buf.copy_from_slice(&length.to_ne_bytes());
        Self {
            buffer: buf,
            read: 0,
        }
    }

    fn is_done(&self) -> bool {
        self.read == Self::MESSAGE_SIZE
    }

    /// Try to read a message from the given buffer, or return how many bytes are still necessary
    fn try_read_buf(buf: &[u8]) -> PResult<MessageDataActual> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        // read usize
        let length_bytes: [u8; USIZE_LEN] = buf[..USIZE_LEN].try_into().unwrap();

        // read usize
        let length = usize::from_ne_bytes(length_bytes);

        Ok((MessageDataActual::new(length), Self::MESSAGE_SIZE))
    }

    /// Read the current buffer into the resulting type
    fn finish(&self) -> MessageDataActual {
        // read usize
        let length = usize::from_ne_bytes(self.buffer);

        MessageDataActual::new(length)
    }

    /// Try to write the current buffer into the given buffer.
    ///
    /// This function will always return how many bytes have been writter to the buffer.
    fn try_write(&mut self, buf: &mut [u8]) -> usize {
        if self.is_done() || buf.is_empty() {
            return 0;
        }

        let written = copy_buffers(&self.buffer[self.read..], buf);
        self.read += written;

        written
    }
}

/// The second part of a [`RingMessage::Data`], a helper to read the actual bytes of a message.
#[derive(Debug, Clone, Copy, PartialEq)]
struct MessageDataActual {
    length: usize,
    read: usize,
}

impl MessageDataActual {
    fn new(length: usize) -> Self {
        Self { length, read: 0 }
    }

    fn new_write(length: usize) -> Self {
        Self::new(length)
    }

    fn is_done(&self) -> bool {
        self.read >= self.length
    }

    /// Try to read a message from the given buffer, or return how many bytes are still necessary
    fn try_read_buf(&mut self, buf: &[u8]) -> PResult<ValueType> {
        if self.is_done() {
            return Err(0);
        }

        let (value, read) = MessageDataValue::try_read_buf(buf)?;
        self.read += read;

        Ok((value, read))
    }

    /// Advance the read size when copied outside of [`try_read`](Self::try_read)
    fn advance_read(&mut self, by: usize) {
        self.read += by;
        assert!(self.read <= self.length);
    }

    /// Try to write the current buffer into the given buffer.
    ///
    /// This function will always return how many bytes have been writter to the buffer.
    fn try_write(&mut self, in_buf: &[u8], out_buf: &mut [u8]) -> usize {
        let this = &mut *self;
        if this.is_done() || in_buf.is_empty() || out_buf.is_empty() {
            return 0;
        }

        // the length left to copy until the message has ended
        let remainder = this.length - this.read;
        let remainder = remainder.min(in_buf.len());
        let read = copy_buffers(&in_buf[..remainder], out_buf);
        this.read += read;

        read
    }
}

/// The Value type we have in the actual data
type ValueType = i16;

/// Read a single Data value.
#[derive(Debug, Clone, Copy, PartialEq)]
struct MessageDataValue {
    buffer: [u8; Self::MESSAGE_SIZE],
    read: usize,
}

// TODO: maybe use nom or similar?
impl MessageDataValue {
    const MESSAGE_SIZE: usize = size_of::<ValueType>();

    #[allow(dead_code)] // data is already in a buffer
    fn new_write(value: ValueType) -> Self {
        let mut buf = [0; Self::MESSAGE_SIZE];
        buf.copy_from_slice(&value.to_ne_bytes());
        Self {
            buffer: buf,
            read: 0,
        }
    }

    fn is_done(&self) -> bool {
        self.read == Self::MESSAGE_SIZE
    }

    /// Try to read a message from the given buffer, or return how many bytes are still necessary
    fn try_read_buf(buf: &[u8]) -> PResult<ValueType> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        // read ValueType
        let value_bytes: [u8; Self::MESSAGE_SIZE] = buf[..Self::MESSAGE_SIZE].try_into().unwrap();

        // read ValueType
        let value = ValueType::from_ne_bytes(value_bytes);

        Ok((value, Self::MESSAGE_SIZE))
    }

    /// Try to write the current buffer into the given buffer.
    ///
    /// This function will always return how many bytes have been writter to the buffer.
    #[allow(dead_code)]
    fn try_write(&mut self, buf: &mut [u8]) -> usize {
        if self.is_done() || buf.is_empty() {
            return 0;
        }

        let written = copy_buffers(&self.buffer[self.read..], buf);
        self.read += written;

        written
    }
}

#[cfg(test)]
mod tests {
    mod parse_message_spec {
        use crate::backends::rusty::source::async_ring::{MessageSpec, MessageSpecResult};

        #[test]
        fn should_read_complete_once() {
            let input: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes().into_iter())
                .chain(10usize.to_ne_bytes().into_iter())
                .collect();
            assert_eq!(input.len(), MessageSpec::MESSAGE_SIZE);

            let (res, read) = MessageSpec::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageSpec::MESSAGE_SIZE);
            assert_eq!(
                res,
                MessageSpecResult {
                    rate: 44000,
                    channels: 2,
                    current_frame_len: 10,
                }
            );
        }

        #[test]
        fn should_report_additional() {
            let input: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes().into_iter())
                .chain(10usize.to_ne_bytes().into_iter())
                .collect();
            assert_eq!(input.len(), MessageSpec::MESSAGE_SIZE);

            let additional = MessageSpec::try_read_buf(&input[0..=1]).unwrap_err();
            assert_eq!(additional, MessageSpec::MESSAGE_SIZE - 2);

            // check the 0 length buffer given path
            let additional = MessageSpec::try_read_buf(&[]).unwrap_err();
            assert_eq!(additional, MessageSpec::MESSAGE_SIZE);

            // finish with the last bytes
            let (res, read) = MessageSpec::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageSpec::MESSAGE_SIZE);
            assert_eq!(
                res,
                MessageSpecResult {
                    rate: 44000,
                    channels: 2,
                    current_frame_len: 10,
                }
            );
        }
    }

    mod write_message_spec {
        use symphonia::core::audio::{Channels, SignalSpec};

        use crate::backends::rusty::source::async_ring::MessageSpec;

        #[test]
        fn should_write_complete_once() {
            let out_buf = &mut [0; MessageSpec::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageSpec::MESSAGE_SIZE);

            let mut msg_spec = MessageSpec::new_write(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                10,
            );

            assert_eq!(msg_spec.is_done(), false);

            let written = msg_spec.try_write(out_buf);
            let expected: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes().into_iter())
                .chain(10usize.to_ne_bytes().into_iter())
                .collect();

            assert_eq!(written, MessageSpec::MESSAGE_SIZE);
            assert_eq!(out_buf, expected.as_slice());
            assert_eq!(msg_spec.is_done(), true);
        }

        #[test]
        fn should_write_across_calls() {
            let out_buf = &mut [0; MessageSpec::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageSpec::MESSAGE_SIZE);

            let mut msg_spec = MessageSpec::new_write(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                10,
            );

            assert_eq!(msg_spec.is_done(), false);

            let written = msg_spec.try_write(&mut out_buf[..=3]);
            let expected: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(0u16.to_ne_bytes().into_iter())
                .chain(0usize.to_ne_bytes().into_iter())
                .collect();

            assert_eq!(written, 4);
            assert_eq!(out_buf, expected.as_slice());
            assert_eq!(msg_spec.is_done(), false);

            // check the 0 length buffer given path
            let written = msg_spec.try_write(&mut [0; 0]);

            assert_eq!(written, 0);
            assert_eq!(msg_spec.is_done(), false);

            // finish with the last bytes
            let written = msg_spec.try_write(&mut out_buf[4..]);
            let expected: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes().into_iter())
                .chain(10usize.to_ne_bytes().into_iter())
                .collect();

            assert_eq!(written, MessageSpec::MESSAGE_SIZE - 4);
            assert_eq!(out_buf, expected.as_slice());
            assert_eq!(msg_spec.is_done(), true);
        }
    }

    mod parse_message_data_first {
        use crate::backends::rusty::source::async_ring::{MessageDataActual, MessageDataFirst};

        #[test]
        fn should_read_complete_once() {
            let input: Vec<u8> = 2usize.to_ne_bytes().into_iter().collect();
            assert_eq!(input.len(), MessageDataFirst::MESSAGE_SIZE);

            let (res, read) = MessageDataFirst::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageDataFirst::MESSAGE_SIZE);
            assert_eq!(res, MessageDataActual::new(2));
        }

        #[test]
        fn should_report_additional() {
            let input: Vec<u8> = 2usize.to_ne_bytes().into_iter().collect();
            assert_eq!(input.len(), MessageDataFirst::MESSAGE_SIZE);

            let additional = MessageDataFirst::try_read_buf(&input[0..=1]).unwrap_err();
            assert_eq!(additional, MessageDataFirst::MESSAGE_SIZE - 2);

            // check the 0 length buffer given path
            let additional = MessageDataFirst::try_read_buf(&[]).unwrap_err();
            assert_eq!(additional, MessageDataFirst::MESSAGE_SIZE);

            // finish with the last bytes
            let (res, read) = MessageDataFirst::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageDataFirst::MESSAGE_SIZE);
            assert_eq!(res, MessageDataActual::new(2));
        }
    }

    mod write_message_data_first {
        use crate::backends::rusty::source::async_ring::MessageDataFirst;

        #[test]
        fn should_write_complete_once() {
            let out_buf = &mut [0; MessageDataFirst::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageDataFirst::MESSAGE_SIZE);

            let mut msg_spec = MessageDataFirst::new_write(4);

            assert_eq!(msg_spec.is_done(), false);

            let written = msg_spec.try_write(out_buf);
            let expected: Vec<u8> = 4usize.to_ne_bytes().to_vec();

            assert_eq!(written, MessageDataFirst::MESSAGE_SIZE);
            assert_eq!(out_buf, expected.as_slice());
            assert_eq!(msg_spec.is_done(), true);
        }

        #[test]
        fn should_write_across_calls() {
            let out_buf = &mut [0; MessageDataFirst::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageDataFirst::MESSAGE_SIZE);

            let mut msg_spec = MessageDataFirst::new_write(4);

            assert_eq!(msg_spec.is_done(), false);

            let usize_bytes = 0usize.to_le_bytes().len();
            let written = msg_spec.try_write(&mut out_buf[..=1]);
            let expected: Vec<u8> = 4usize
                .to_ne_bytes()
                .into_iter()
                .take(2)
                .chain([0u8; 1].repeat(usize_bytes - 2))
                .collect();

            assert_eq!(written, 2);
            assert_eq!(out_buf, expected.as_slice());
            assert_eq!(msg_spec.is_done(), false);

            // check the 0 length buffer given path
            let written = msg_spec.try_write(&mut [0; 0]);

            assert_eq!(written, 0);
            assert_eq!(msg_spec.is_done(), false);

            // finish with the last bytes
            let written = msg_spec.try_write(&mut out_buf[2..]);
            let expected: Vec<u8> = 4usize.to_ne_bytes().to_vec();

            assert_eq!(written, usize_bytes - 2);
            assert_eq!(out_buf, expected.as_slice());
            assert_eq!(msg_spec.is_done(), true);
        }
    }

    mod parse_message_data_actual {
        use crate::backends::rusty::source::async_ring::{MessageDataActual, MessageDataValue};

        #[test]
        fn should_read_complete_once() {
            let input: Vec<u8> = vec![1; 2];
            assert_eq!(input.len(), 2);

            let mut msg = MessageDataActual::new(2);
            let (res, read) = msg.try_read_buf(&input).unwrap();
            assert_eq!(res, i16::from_ne_bytes([1; 2]));
            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.is_done(), true);
        }

        #[test]
        fn should_report_additional() {
            // check the 0 length buffer given path
            let mut msg = MessageDataActual::new(6);
            let additional = msg.try_read_buf(&[]).unwrap_err();
            assert_eq!(additional, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.read, 0);
            assert_eq!(msg.is_done(), false);

            // some actual partial data
            let mut msg = MessageDataActual::new(6);
            let additional = msg.try_read_buf(&[1; 1]).unwrap_err();
            assert_eq!(additional, MessageDataValue::MESSAGE_SIZE - 1);
            assert_eq!(msg.read, 0);
            assert_eq!(msg.is_done(), false);
        }

        #[test]
        fn should_resume_additional() {
            let input: Vec<u8> = vec![1; 4];
            assert_eq!(input.len(), 4);

            let mut msg = MessageDataActual::new(4);
            let (res, read) = msg.try_read_buf(&input).unwrap();
            assert_eq!(res, i16::from_ne_bytes([1; 2]));
            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.is_done(), false);

            // finish with the last bytes
            let (res, read) = msg.try_read_buf(&input).unwrap();
            assert_eq!(res, i16::from_ne_bytes([1; 2]));
            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.read, MessageDataValue::MESSAGE_SIZE * 2);
            assert_eq!(msg.is_done(), true);
        }
    }

    mod write_message_data_actual {
        use crate::backends::rusty::source::async_ring::MessageDataActual;

        #[test]
        fn should_write_complete_once() {
            let out_buf = &mut [0; 6];
            assert_eq!(out_buf.len(), 6);

            let mut msg_data = MessageDataActual::new_write(6);

            let in_buf = &[1; 6];
            let written = msg_data.try_write(in_buf, out_buf);
            let expected = [1; 6];

            assert_eq!(written, 6);
            assert_eq!(out_buf, &expected);
        }

        #[test]
        fn should_write_across_calls() {
            let out_buf = &mut [0; 6];
            assert_eq!(out_buf.len(), 6);

            let mut msg_data = MessageDataActual::new_write(6);

            let in_buf = &[1; 2];
            let written = msg_data.try_write(&in_buf[0..=1], out_buf);
            let expected: Vec<u8> = [1; 2].into_iter().chain([0; 4].into_iter()).collect();

            assert_eq!(written, 2);
            assert_eq!(out_buf, expected.as_slice());

            // check the 0 length buffer given path
            let written = msg_data.try_write(&[0; 0], out_buf);

            assert_eq!(written, 0);

            // finish with the last bytes
            let in_buf = &[2; 4];
            let written = msg_data.try_write(in_buf, &mut out_buf[2..]);
            let expected: Vec<u8> = [1; 2].into_iter().chain([2; 4].into_iter()).collect();

            assert_eq!(written, 4);
            assert_eq!(out_buf, expected.as_slice());
        }
    }

    mod parse_message_value {
        use crate::backends::rusty::source::async_ring::MessageDataValue;

        #[test]
        fn should_read_complete_once() {
            let input: &[u8] = &10i16.to_ne_bytes();
            assert_eq!(input.len(), MessageDataValue::MESSAGE_SIZE);

            let (res, read) = MessageDataValue::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(res, 10);
        }

        #[test]
        fn should_report_additional() {
            let input: &[u8] = &10i16.to_ne_bytes();
            assert_eq!(input.len(), MessageDataValue::MESSAGE_SIZE);

            assert!(MessageDataValue::MESSAGE_SIZE > 1);

            let additional = MessageDataValue::try_read_buf(&input[0..1]).unwrap_err();
            assert_eq!(additional, MessageDataValue::MESSAGE_SIZE - 1);

            // check the 0 length buffer given path
            let additional = MessageDataValue::try_read_buf(&[]).unwrap_err();
            assert_eq!(additional, MessageDataValue::MESSAGE_SIZE);

            // finish with the last bytes
            let (res, read) = MessageDataValue::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(res, 10);
        }
    }

    mod write_message_value {
        use crate::backends::rusty::source::async_ring::MessageDataValue;

        #[test]
        fn should_write_complete_once() {
            let out_buf = &mut [0; MessageDataValue::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageDataValue::MESSAGE_SIZE);

            let mut msg_value = MessageDataValue::new_write(10);

            assert_eq!(msg_value.is_done(), false);

            let written = msg_value.try_write(out_buf);
            let expected: &[u8] = &10i16.to_ne_bytes();

            assert_eq!(written, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(out_buf, expected);
            assert_eq!(msg_value.is_done(), true);
        }

        #[test]
        fn should_write_across_calls() {
            let out_buf = &mut [0; MessageDataValue::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageDataValue::MESSAGE_SIZE);

            let mut msg_spec = MessageDataValue::new_write(10);

            assert_eq!(msg_spec.is_done(), false);

            let written = msg_spec.try_write(&mut out_buf[..1]);
            let expected: Vec<u8> = 10i16
                .to_ne_bytes()
                .into_iter()
                .take(1)
                .chain([0u8; 1].into_iter())
                .collect();

            assert_eq!(written, 1);
            assert_eq!(out_buf, expected.as_slice());
            assert_eq!(msg_spec.is_done(), false);

            // check the 0 length buffer given path
            let written = msg_spec.try_write(&mut [0; 0]);

            assert_eq!(written, 0);
            assert_eq!(msg_spec.is_done(), false);

            // finish with the last bytes
            let written = msg_spec.try_write(&mut out_buf[1..]);
            let expected: &[u8] = &10i16.to_ne_bytes();

            assert_eq!(written, 1);
            assert_eq!(out_buf, expected);
            assert_eq!(msg_spec.is_done(), true);
        }
    }

    mod write_ring_messages {
        use symphonia::core::audio::{Channels, SignalSpec};

        use crate::backends::rusty::source::async_ring::{
            MessageDataFirst, MessageSpec, RingMessages, RingMsgWrite,
        };

        #[test]
        fn should_write_complete_once_spec() {
            let out_buf = &mut [0; RingMsgWrite::get_msg_size(MessageSpec::MESSAGE_SIZE)];
            assert_eq!(out_buf.len(), MessageSpec::MESSAGE_SIZE + 1);

            let mut writer = RingMsgWrite::new_spec(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                10,
            );

            assert_eq!(writer.id_written, false);
            assert_eq!(writer.is_done(), false);

            let written = writer.try_write(out_buf);

            assert_eq!(written, MessageSpec::MESSAGE_SIZE + 1);
            assert_eq!(writer.id_written, true);
            assert_eq!(writer.is_done(), true);

            let expected: Vec<u8> = [RingMessages::Spec.as_u8()]
                .into_iter()
                .chain(44000u32.to_ne_bytes().into_iter())
                .chain(2u16.to_ne_bytes().into_iter())
                .chain(10usize.to_ne_bytes().into_iter())
                .collect();
            assert_eq!(out_buf, expected.as_slice());
        }

        #[test]
        fn should_write_complete_once_data() {
            let out_buf = &mut [0; RingMsgWrite::get_msg_size(MessageDataFirst::MESSAGE_SIZE + 2)];
            assert_eq!(out_buf.len(), MessageDataFirst::MESSAGE_SIZE + 2 + 1);

            let input_data = &[2; 2];

            let mut writer = RingMsgWrite::new_data(2);

            assert_eq!(writer.id_written, false);
            assert_eq!(writer.is_done(), false);

            let written = writer.try_write(out_buf);
            let expected: Vec<u8> = [RingMessages::Data.as_u8()]
                .into_iter()
                .chain(2usize.to_ne_bytes().into_iter())
                .chain([0; 2].into_iter())
                .collect();

            assert_eq!(written, MessageDataFirst::MESSAGE_SIZE + 1);
            assert_eq!(out_buf, expected.as_slice());
            assert_eq!(writer.id_written, true);
            assert_eq!(writer.is_done(), true);

            let mut writer = writer.finish_data_first();

            assert_eq!(writer.id_written, true);
            assert_eq!(writer.is_done(), false);

            let written = writer.try_write_data(input_data, &mut out_buf[written..]);
            let expected: Vec<u8> = [RingMessages::Data.as_u8()]
                .into_iter()
                .chain(2usize.to_ne_bytes().into_iter())
                .chain([2; 2].into_iter())
                .collect();

            assert_eq!(written, 2);
            assert_eq!(writer.is_done(), true);
            assert_eq!(out_buf, expected.as_slice());
        }

        #[test]
        fn should_write_complete_once_eos() {
            let out_buf = &mut [0; RingMsgWrite::ID_SIZE];
            assert_eq!(out_buf.len(), RingMsgWrite::ID_SIZE);

            let mut writer = RingMsgWrite::new_eos();

            assert_eq!(writer.id_written, false);
            assert_eq!(writer.is_done(), false);

            let written = writer.try_write(out_buf);

            assert_eq!(written, RingMsgWrite::ID_SIZE);
            assert_eq!(writer.id_written, true);
            assert_eq!(writer.is_done(), true);

            let expected: &[u8] = &[RingMessages::EOS.as_u8()];
            assert_eq!(out_buf, expected);
        }
    }

    mod static_buffer {
        use crate::backends::rusty::source::async_ring::StaticBuf;

        #[test]
        fn should_work() {
            let mut buf = StaticBuf::<32>::new();
            assert_eq!(buf.len(), 0);
            assert_eq!(buf.get_ref().len(), 0);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[0u8; 0]);

            buf.get_mut()[0] = u8::MAX;
            buf.set_len(1);

            assert_eq!(buf.len(), 1);
            assert_eq!(buf.get_ref().len(), 1);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[u8::MAX; 1]);

            buf.advance_beginning(1);
            assert_eq!(buf.len(), 0);
            assert_eq!(buf.get_mut().len(), 31);
            assert_eq!(buf.get_ref(), &[0u8; 0]);

            buf.get_mut().fill(u8::MAX);
            buf.set_len(31);

            assert_eq!(buf.len(), 31);
            assert_eq!(buf.get_mut().len(), 31);
            assert_eq!(buf.get_ref(), &[u8::MAX; 31]);

            buf.make_beginning();

            assert_eq!(buf.len(), 31);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[u8::MAX; 31]);

            buf.advance_beginning(15);

            assert_eq!(buf.len(), 16);
            assert_eq!(buf.get_mut().len(), 17);
            assert_eq!(buf.get_ref(), &[u8::MAX; 16]);

            buf.clear();

            assert_eq!(buf.len(), 0);
            assert_eq!(buf.get_ref().len(), 0);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[0u8; 0]);
        }

        #[test]
        #[should_panic(expected = "self.used_len <= self.buf.len()")]
        fn length_capacity() {
            let mut buf = StaticBuf::<16>::new();
            buf.set_len(17);
        }

        #[test]
        #[should_panic(expected = "self.data_start_idx <= self.buf.len()")]
        fn beginning_capacity() {
            let mut buf = StaticBuf::<16>::new();
            buf.advance_beginning(17);
        }
    }

    mod ringbuffer {
        use std::{sync::Arc, time::Duration};

        use async_ringbuf::traits::Observer;
        use parking_lot::Mutex;
        use symphonia::core::audio::{Channels, SignalSpec};

        use crate::backends::rusty::source::async_ring::{
            AsyncRingSource, MessageDataFirst, MessageSpec, RingMsgWrite, ValueType, MIN_SIZE,
        };

        #[tokio::test]
        async fn should_work() {
            let send = Arc::new(Mutex::new(Vec::new()));
            let recv = Arc::new(Mutex::new(Vec::new()));

            let (mut prod, mut cons) = AsyncRingSource::new(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                None,
                1024,
                0,
                tokio::runtime::Handle::current(),
            );

            assert_eq!(prod.inner.capacity().get(), MIN_SIZE);

            let recv_c = recv.clone();
            let handle = tokio::task::spawn_blocking(move || {
                let mut lock = recv_c.lock();
                while let Some(num) = cons.next() {
                    lock.extend_from_slice(&num.to_ne_bytes());
                }
                assert_eq!(cons.inner.occupied_len(), 0);
            });

            let mut lock = send.lock();
            let first_data = 1i16.to_le_bytes().repeat(1024);
            let written = prod.write_data(&first_data).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite::get_msg_size(MessageDataFirst::MESSAGE_SIZE + first_data.len())
            );
            lock.extend_from_slice(&first_data);

            let new_spec = SignalSpec::new(48000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT);
            let written = prod.new_spec(new_spec, 1024).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite::get_msg_size(MessageSpec::MESSAGE_SIZE)
            );

            let second_data = 2i16.to_le_bytes().repeat(1024);
            let written = prod.write_data(&second_data).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite::get_msg_size(MessageDataFirst::MESSAGE_SIZE + second_data.len())
            );
            lock.extend_from_slice(&second_data);

            let written = prod.new_eos().await.unwrap();
            assert_eq!(written, RingMsgWrite::get_msg_size(0));

            prod.write_data(&[]).await.unwrap_err();

            // just to prevent a inifinitely running test due to a deadlock
            if let Err(_) = tokio::time::timeout(Duration::from_secs(3), handle).await {
                panic!("Read Task did not complete within 3 seconds");
            }

            assert!(prod.is_closed());
            assert!(prod.inner.is_empty());

            let send_lock = lock;
            let recv_lock = recv.lock();
            let value_size = size_of::<ValueType>();
            assert_eq!(send_lock.len(), value_size * 1024 * 2);
            assert_eq!(recv_lock.len(), value_size * 1024 * 2);

            assert_eq!(*send_lock, *recv_lock);
        }

        // the producer should not exit before the consumer in a actual use-case
        // as the producer may need to still process and output a seek request
        #[tokio::test]
        async fn prod_should_not_exist_before_cons() {
            let order = Arc::new(Mutex::new(Vec::new()));

            let (mut prod, mut cons) = AsyncRingSource::new(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                None,
                1024,
                0,
                tokio::runtime::Handle::current(),
            );

            let obsv = prod.inner.observe();

            assert_eq!(obsv.read_is_held(), true);
            assert_eq!(obsv.write_is_held(), true);
            let order_c = order.clone();

            let cons_handle = tokio::task::spawn_blocking(move || {
                while let Some(num) = cons.next() {
                    let _ = num;
                }
                assert_eq!(cons.inner.occupied_len(), 0);
                order_c.lock().push("cons");
            });

            let obsv_c = obsv.clone();
            let order_c = order.clone();

            let prod_handle = tokio::task::spawn(async move {
                let first_data = 1i16.to_le_bytes().repeat(1024);
                let written = prod.write_data(&first_data).await.unwrap();
                assert_eq!(
                    written,
                    RingMsgWrite::get_msg_size(MessageDataFirst::MESSAGE_SIZE + first_data.len())
                );

                assert_eq!(obsv_c.read_is_held(), true);
                assert_eq!(obsv_c.write_is_held(), true);

                let written = prod.new_eos().await.unwrap();
                assert_eq!(written, RingMsgWrite::get_msg_size(0));

                let _ = prod.wait_seek().await;
                order_c.lock().push("prod");
            });

            // just to prevent a inifinitely running test due to a deadlock
            if let Err(_) = tokio::time::timeout(Duration::from_secs(3), cons_handle).await {
                panic!("Read Task did not complete within 3 seconds");
            }

            assert_eq!(obsv.read_is_held(), false);

            // just to prevent a inifinitely running test due to a deadlock
            if let Err(_) = tokio::time::timeout(Duration::from_secs(3), prod_handle).await {
                panic!("Read Task did not complete within 3 seconds");
            }

            assert_eq!(obsv.write_is_held(), false);

            assert_eq!(*order.lock(), &["cons", "prod"]);
        }

        // even if the producer (due to some error or otherwise) exits with eos, the consumer should consume everything still available
        #[tokio::test]
        async fn should_consume_on_prod_exit_eos() {
            let recv = Arc::new(Mutex::new(Vec::new()));

            let (mut prod, mut cons) = AsyncRingSource::new(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                None,
                1024,
                0,
                tokio::runtime::Handle::current(),
            );

            let recv_c = recv.clone();
            let handle = tokio::task::spawn_blocking(move || {
                let mut lock = recv_c.lock();
                while let Some(num) = cons.next() {
                    lock.extend_from_slice(&num.to_ne_bytes());
                }
                assert_eq!(cons.inner.occupied_len(), 0);
                assert_eq!(cons.inner.write_is_held(), false);
            });

            let first_data = 1i16.to_le_bytes().repeat(1024);
            let written = prod.write_data(&first_data).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite::get_msg_size(MessageDataFirst::MESSAGE_SIZE + first_data.len())
            );

            let written = prod.new_eos().await.unwrap();
            assert_eq!(written, RingMsgWrite::get_msg_size(0));

            let obsv = prod.inner.observe();
            drop(prod);

            assert_eq!(obsv.write_is_held(), false);
            // dont check read as that *could* have consumed and exited already
            // assert_eq!(obsv.read_is_held(), true);

            // just to prevent a inifinitely running test due to a deadlock
            if let Err(_) = tokio::time::timeout(Duration::from_secs(3), handle).await {
                panic!("Read Task did not complete within 3 seconds");
            }

            assert_eq!(obsv.write_is_held(), false);
            assert_eq!(obsv.read_is_held(), false);

            let recv_lock = recv.lock();
            let value_size = size_of::<ValueType>();
            assert_eq!(recv_lock.len(), value_size * 1024);

            assert_eq!(*recv_lock, first_data.as_slice());
        }

        // even if the producer (due to some error or otherwise) exits without, the consumer should consume everything still available
        #[tokio::test]
        async fn should_consume_on_prod_exit() {
            let recv = Arc::new(Mutex::new(Vec::new()));

            let (mut prod, mut cons) = AsyncRingSource::new(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                None,
                1024,
                0,
                tokio::runtime::Handle::current(),
            );

            let recv_c = recv.clone();
            let handle = tokio::task::spawn_blocking(move || {
                let mut lock = recv_c.lock();
                while let Some(num) = cons.next() {
                    lock.extend_from_slice(&num.to_ne_bytes());
                }
                assert_eq!(cons.inner.occupied_len(), 0);
                assert_eq!(cons.inner.write_is_held(), false);
            });

            let first_data = 1i16.to_le_bytes().repeat(1024);
            let written = prod.write_data(&first_data).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite::get_msg_size(MessageDataFirst::MESSAGE_SIZE + first_data.len())
            );

            let obsv = prod.inner.observe();
            drop(prod);

            assert_eq!(obsv.write_is_held(), false);
            // dont check read as that *could* have consumed and exited already
            // assert_eq!(obsv.read_is_held(), true);

            // just to prevent a inifinitely running test due to a deadlock
            if let Err(_) = tokio::time::timeout(Duration::from_secs(3), handle).await {
                panic!("Read Task did not complete within 3 seconds");
            }

            assert_eq!(obsv.write_is_held(), false);
            assert_eq!(obsv.read_is_held(), false);

            let recv_lock = recv.lock();
            let value_size = size_of::<ValueType>();
            assert_eq!(recv_lock.len(), value_size * 1024);

            assert_eq!(*recv_lock, first_data.as_slice());
        }
    }
}
