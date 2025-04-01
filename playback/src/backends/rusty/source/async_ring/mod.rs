use std::{
    fmt::Debug,
    future::Future,
    iter::FusedIterator,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, LazyLock,
    },
    time::Duration,
    u8,
};

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
    /// DEBUG: dont ship
    had_EOS: Arc<AtomicBool>,
}

impl AsyncRingSourceProvider {
    fn new(wrap: ProdWrap, seek_rx: mpsc::Receiver<SeekData>, had_EOS: Arc<AtomicBool>) -> Self {
        AsyncRingSourceProvider {
            inner: wrap,
            seek_rx: Arc::new(RwLock::new(seek_rx)),
            data: None,
            had_EOS,
        }
    }

    /// Check if the consumer ([`AsyncRingSource`]) is still connected and writes are possible
    fn is_closed(&self) -> bool {
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
        // ORIG_ACC.lock().extend_from_slice(buf);
        if self.had_EOS.load(Ordering::SeqCst) {
            error!("WRITING AFTER EOS!");
        }
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
        self.had_EOS.store(true, Ordering::SeqCst);
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

    buf: StaticBuf<32 /* 24000 *//* 48000 */>,
    last_msg: Option<RingMsgParse>,
    handle: Handle,

    // cached information on how to treat current data until a update
    channels: u16,
    rate: u32,
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
        Self::new_eos(
            spec,
            total_duration,
            current_frame_len,
            size,
            handle,
            Arc::new(AtomicBool::new(false)),
        )
    }

    pub fn new_eos(
        spec: SignalSpec,
        total_duration: Option<Duration>,
        current_frame_len: usize,
        size: usize,
        handle: Handle,
        had_EOS: Arc<AtomicBool>,
    ) -> (AsyncRingSourceProvider, Self) {
        let size = size.max(MIN_SIZE);
        let ringbuf = AsyncHeapRb::<u8>::new(size);
        let (prod, cons) = ringbuf.split();
        let (tx, rx) = mpsc::channel(1);

        let async_prod = AsyncRingSourceProvider::new(ProdWrap::new(prod), rx, had_EOS);
        let async_cons = Self {
            inner: ConsWrap::new(cons),
            seek_tx: Some(tx),
            channels: u16::try_from(spec.channels.count())
                .expect("Channel size to be within u16::MAX"),
            rate: spec.rate,
            total_duration,
            current_frame_len,
            last_msg: None,
            buf: StaticBuf::new(),
            handle,
        };

        (async_prod, async_cons)
    }

    /// Ensure there is a complete message in `last_msg`.
    ///
    /// This function assumes there is no current message.
    #[must_use]
    async fn read_msg(&mut self) -> Option<()> {
        debug!("READING MSG");
        let msg = {
            let detect_byte = if self.buf.is_empty() {
                self.inner.pop().await?
            } else {
                let byte = self.buf.get_ref()[0];
                self.buf.advance_beginning(1);
                byte
            };
            let Some((parser, _)) = RingMsgParse::new(&[detect_byte]) else {
                unimplemented!("There is always one byte provided and one is enough for detection");
            };
            self.last_msg = Some(parser);
            self.last_msg.as_mut().unwrap()
        };
        debug!("GOT MSG");

        if !msg.is_fillable() {
            return Some(());
        }

        while !msg.is_done() {
            // "buf.is_empty" is safe here as all messages consume the buffer fully here.
            if self.inner.is_closed() && self.inner.is_empty() && self.buf.is_empty() {
                return None;
            }

            self.buf.maybe_need_move();

            let mut written = 0;
            if self.buf.is_empty() {
                // wait for at least one element being occupied,
                // more elements would mean to wait until they all are there, regardless if they are part of the message or not
                self.inner.wait_occupied(1).await;
                debug!("occupied {:#?}", self.inner.occupied_len());
                written += self.inner.pop_slice(self.buf.get_mut());
                debug!("written {:#?}", written);
                self.buf.set_len(written);
                // DEBUG: Sanity
                assert!(self.buf.len() == written);
            }

            let read = msg.try_fill(&self.buf.get_ref());
            debug!(
                "buf-len {:#}, read {:#?}, written: {:#?}",
                self.buf.len(),
                read,
                written
            );
            self.buf.advance_beginning(read);

            // Sanity, there may be infinite loop because of bad implementation
            debug_assert!(read > 0);
        }

        // we can safely assume "msg" is done beyond here

        debug!("AFTER LOOP");

        // Advance `DataFirst` to `DataActual`
        if msg.is_data_first() {
            *msg = msg.finish_data_first();
        }

        Some(())
    }

    /// Apply a new spec from the current message.
    ///
    /// This function assumes the current message is a [`MessageSpec`].
    fn apply_spec_msg(&mut self) {
        let RingMsgParse::Spec(new_spec) = self.last_msg.take().unwrap() else {
            unimplemented!("This should be checked outside of the function");
        };

        let new_spec = new_spec.finish();
        self.channels = new_spec.channels;
        self.rate = new_spec.rate;
    }

    /// Read data from a Data Message.
    ///
    /// This function assumes the current message is a non-finished [`MessageDataActual`].
    #[must_use]
    async fn read_data(&mut self) -> Option<i16> {
        debug!("READ DATA {:#?}", self.buf);
        let RingMsgParse::DataActual(msg) = self.last_msg.as_mut().unwrap() else {
            unimplemented!("This should be checked outside of the function");
        };

        // wait until we have enough data to parse a value
        while self.buf.len() < MessageDataValue::MESSAGE_SIZE {
            if self.inner.is_closed()
                && self.inner.is_empty()
                && self.buf.len() < MessageDataValue::MESSAGE_SIZE
            {
                return None;
            }

            self.buf.maybe_need_move();

            // wait for at least one element being occupied,
            // more elements would mean to wait until they all are there, regardless if they are part of the message or not
            self.inner.wait_occupied(1).await;
            debug!("occupied {:#?}", self.inner.occupied_len());

            // dont overwrite data that may still be in there
            let write_from = self.buf.len();
            let written = self.inner.pop_slice(&mut self.buf.get_mut()[write_from..]);
            debug!("written? {:#?}", written);
            self.buf.set_len(written + write_from);

            // DEBUG: Sanity
            assert!(self.buf.len() == written + write_from);

            // Sanity, there may be infinite loop because of bad implementation
            debug_assert!(written > 0);
        }

        if self.buf.len() < MessageDataValue::MESSAGE_SIZE {
            return None;
        }

        let mut value = MessageDataValue::new();

        let read = value.try_fill(self.buf.get_ref());
        msg.advance_read(read);
        self.buf.advance_beginning(read);

        assert!(value.is_done());

        let sample = value.finish();

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
        // Some(self.buf.len())
        // TODO: implement a sample length cache.
        // None // Infinite for now
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn channels(&self) -> u16 {
        self.channels
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        // if self.buf.is_empty() {
        //     // error!("RATE {:#?}", self.rate);
        // }
        self.rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        debug!("TRY_SEEK");

        // clear the ringbuffer before sending, to potentially unblock the writer.
        self.inner.clear();
        self.last_msg.take();
        self.buf.clear();

        let (cb_tx, cb_rx) = oneshot::channel();
        let _ = self.seek_tx.as_mut().unwrap().blocking_send((pos, cb_tx));
        let to_skip = cb_rx.blocking_recv().map_err(|_| {
            rodio::source::SeekError::Other(
                anyhow!("Seek Callback channel exited unexpectedly").into(),
            )
        })?;

        // skip possible new elements
        let skipped = self.inner.skip(to_skip);
        debug!("AFTER SEEK {:#?}", skipped);

        Ok(())
    }
}

impl Iterator for AsyncRingSource {
    type Item = ValueType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        loop {
            let msg = match self.last_msg.as_ref() {
                Some(v) => v,
                None => {
                    self.handle.clone().block_on(self.read_msg())?;
                    self.last_msg.as_ref().unwrap()
                }
            };

            match msg {
                RingMsgParse::DataFirst(_) => unreachable!("Handled by read_msg"),
                RingMsgParse::DataActual(_) => {
                    let sample = self.handle.clone().block_on(self.read_data());

                    return sample;
                }
                RingMsgParse::Spec(_) => {
                    self.apply_spec_msg();
                }
                RingMsgParse::EOS => {
                    error!("EOS");
                    self.seek_tx.take();
                    return None;
                }
            }
        }
    }
}

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
            debug!("clearing");
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
    Data = 1,
    Spec = 2,
    EOS = 3,
}

impl RingMessages {
    /// Try to detect which type of message is at the beginning of the given buffer
    fn try_detect(buf: &[u8]) -> Option<(Self, usize)> {
        let byte = buf.iter().next()?;

        Some((Self::from_u8(*byte), 1))
    }

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

#[derive(Debug, Clone, Copy, PartialEq)]
enum RingMsgParse {
    DataFirst(MessageDataFirst),
    DataActual(MessageDataActual),
    Spec(MessageSpec),
    EOS,
}

impl RingMsgParse {
    fn new(buf: &[u8]) -> Option<(Self, usize)> {
        let (detected, read) = RingMessages::try_detect(buf)?;

        let new = match detected {
            RingMessages::Data => Self::DataFirst(MessageDataFirst::new()),
            RingMessages::Spec => Self::Spec(MessageSpec::new()),
            RingMessages::EOS => Self::EOS,
        };

        Some((new, read))
    }

    fn is_done(&self) -> bool {
        match self {
            RingMsgParse::DataFirst(message_data_first) => message_data_first.is_done(),
            RingMsgParse::DataActual(message_data_actual) => message_data_actual.is_done(),
            RingMsgParse::Spec(message_spec) => message_spec.is_done(),
            RingMsgParse::EOS => true,
        }
    }

    fn is_fillable(&self) -> bool {
        match self {
            RingMsgParse::DataFirst(_) => true,
            RingMsgParse::DataActual(_) => false,
            RingMsgParse::Spec(_) => true,
            RingMsgParse::EOS => false,
        }
    }

    fn is_data_first(&self) -> bool {
        if let Self::DataFirst(_) = self {
            return true;
        }

        false
    }

    fn try_fill(&mut self, buf: &[u8]) -> usize {
        match self {
            RingMsgParse::DataFirst(message_data_first) => message_data_first.try_fill(buf),
            RingMsgParse::DataActual(_message_data_actual) => 0,
            RingMsgParse::Spec(message_spec) => message_spec.try_fill(buf),
            RingMsgParse::EOS => 0,
        }
    }

    fn finish_spec(self) -> MessageSpecResult {
        let Self::Spec(spec) = self else {
            unimplemented!("This should be checked outside of the function");
        };

        spec.finish()
    }

    fn finish_data_first(self) -> Self {
        let Self::DataFirst(data_first) = self else {
            unimplemented!("This should be checked outside of the function");
        };

        Self::DataActual(data_first.finish())
    }

    fn try_read_data(&mut self, in_buf: &[u8], out_buf: &mut [u8]) -> usize {
        let Self::DataActual(data) = self else {
            unimplemented!("This should be checked outside of the function");
        };

        data.try_read(in_buf, out_buf)
    }
}

/// Copy from `in_buf` into `out_buf`, returning the bytes copied
fn copy_buffers(in_buf: &[u8], out_buf: &mut [u8]) -> usize {
    // the position to copy to, exclusive
    let copy_to_pos = in_buf.len().min(out_buf.len());

    out_buf[..copy_to_pos].copy_from_slice(&in_buf[..copy_to_pos]);

    copy_to_pos
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MessageSpecResult {
    rate: u32,
    channels: u16,
    current_frame_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MessageSpec {
    buffer: [u8; Self::MESSAGE_SIZE],
    read: usize,
}

const USIZE_LEN: usize = size_of::<usize>();

// TODO: maybe use nom or similar?
impl MessageSpec {
    const MESSAGE_SIZE: usize = size_of::<u32>() + size_of::<u16>() + size_of::<usize>();

    fn new() -> Self {
        Self {
            buffer: [0; Self::MESSAGE_SIZE],
            read: 0,
        }
    }

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

    /// Try to read from the given buffer into a [`MessageSpecResult`].
    ///
    /// This function will always return how many bytes have been consumed from the buffer.
    fn try_fill(&mut self, buf: &[u8]) -> usize {
        if self.is_done() || buf.is_empty() {
            return 0;
        }

        let read = copy_buffers(buf, &mut self.buffer[self.read..]);
        self.read += read;

        read
    }

    /// Read the current buffer into the resulting type
    fn finish(&self) -> MessageSpecResult {
        // read u32
        let rate: [u8; 4] = self.buffer[0..=3].try_into().unwrap();
        // read u16
        let channels: [u8; 2] = self.buffer[4..=5].try_into().unwrap();
        // read usize
        let current_frame_len: [u8; USIZE_LEN] = self.buffer[6..6 + USIZE_LEN].try_into().unwrap();

        let rate = u32::from_ne_bytes(rate);
        let channels = u16::from_ne_bytes(channels);
        let current_frame_len = usize::from_ne_bytes(current_frame_len);

        MessageSpecResult {
            rate,
            channels,
            current_frame_len,
        }
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct MessageDataFirst {
    buffer: [u8; Self::MESSAGE_SIZE],
    read: usize,
}

// TODO: maybe use nom or similar?
impl MessageDataFirst {
    const MESSAGE_SIZE: usize = size_of::<usize>();

    fn new() -> Self {
        Self {
            buffer: [0; Self::MESSAGE_SIZE],
            read: 0,
        }
    }

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

    /// Try to read from the given buffer into a [`MessageDataActual`].
    ///
    /// This function will always return how many bytes have been consumed from the buffer.
    fn try_fill(&mut self, buf: &[u8]) -> usize {
        if self.is_done() || buf.is_empty() {
            return 0;
        }

        let read = copy_buffers(buf, &mut self.buffer[self.read..]);
        self.read += read;

        read
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

    /// Try to read from the given buffer into a given out-buffer.
    /// This function will put new data into the out-buffer from the beginning.
    ///
    /// This function will always return how many bytes have been consumed from the buffer.
    // TODO: move code to "try_write" and remove "try_read"
    fn try_read(&mut self, in_buf: &[u8], out_buf: &mut [u8]) -> usize {
        if self.is_done() || in_buf.is_empty() || out_buf.is_empty() {
            return 0;
        }

        // the length left to copy until the message has ended
        let remainder = self.length - self.read;
        let remainder = remainder.min(in_buf.len());
        let read = copy_buffers(&in_buf[..remainder], out_buf);
        self.read += read;

        read
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
        self.try_read(in_buf, out_buf)
    }
}

/// The Value type we have in the actual data
type ValueType = i16;

#[derive(Debug, Clone, Copy, PartialEq)]
struct MessageDataValue {
    buffer: [u8; Self::MESSAGE_SIZE],
    read: usize,
}

// TODO: maybe use nom or similar?
impl MessageDataValue {
    const MESSAGE_SIZE: usize = size_of::<ValueType>();

    fn new() -> Self {
        Self {
            buffer: [0; Self::MESSAGE_SIZE],
            read: 0,
        }
    }

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

    /// Try to read from the given buffer into a [`MessageDataActual`].
    ///
    /// This function will always return how many bytes have been consumed from the buffer.
    fn try_fill(&mut self, buf: &[u8]) -> usize {
        if self.is_done() || buf.is_empty() {
            return 0;
        }

        let read = copy_buffers(buf, &mut self.buffer[self.read..]);
        self.read += read;

        read
    }

    /// Read the current buffer into the resulting type
    fn finish(&self) -> ValueType {
        // read ValueType
        let value = ValueType::from_ne_bytes(self.buffer);

        value
    }

    /// Try to write the current buffer into the given buffer.
    ///
    /// This function will always return how many bytes have been writter to the buffer.
    #[expect(dead_code)] // data is directly copied from the incoming buffer
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

            let mut msg_spec = MessageSpec::new();

            let read = msg_spec.try_fill(&input);
            let res = msg_spec.finish();
            let expected = {
                let mut buf = [0; MessageSpec::MESSAGE_SIZE];
                buf.copy_from_slice(&input);
                buf
            };
            assert_eq!(
                msg_spec,
                MessageSpec {
                    buffer: expected,
                    read: MessageSpec::MESSAGE_SIZE
                }
            );
            assert_eq!(msg_spec.is_done(), true);

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
        fn should_resume_across_calls() {
            let input: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes().into_iter())
                .chain(10usize.to_ne_bytes().into_iter())
                .collect();
            assert_eq!(input.len(), MessageSpec::MESSAGE_SIZE);

            let mut msg_spec = MessageSpec::new();

            let read = msg_spec.try_fill(&input[0..=1]);
            let expected = {
                let mut buf = [0; MessageSpec::MESSAGE_SIZE];
                (buf[..=1]).copy_from_slice(&input[0..=1]);
                buf
            };
            assert_eq!(
                msg_spec,
                MessageSpec {
                    buffer: expected,
                    read: 2
                }
            );

            assert_eq!(read, 2);
            assert_eq!(msg_spec.is_done(), false);

            // check the 0 length buffer given path
            let read = msg_spec.try_fill(&[0; 0]);
            let expected = {
                let mut buf = [0; MessageSpec::MESSAGE_SIZE];
                (buf[..=1]).copy_from_slice(&input[0..=1]);
                buf
            };
            assert_eq!(
                msg_spec,
                MessageSpec {
                    buffer: expected,
                    read: 2
                }
            );

            assert_eq!(read, 0);
            assert_eq!(msg_spec.is_done(), false);

            // finish with the last bytes
            let read = msg_spec.try_fill(&input[2..]);
            let res = msg_spec.finish();
            let expected = {
                let mut buf = [0; MessageSpec::MESSAGE_SIZE];
                buf.copy_from_slice(&input);
                buf
            };
            assert_eq!(
                msg_spec,
                MessageSpec {
                    buffer: expected,
                    read: MessageSpec::MESSAGE_SIZE
                }
            );
            assert_eq!(msg_spec.is_done(), true);

            assert_eq!(read, MessageSpec::MESSAGE_SIZE - 2);
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

        use crate::backends::rusty::source::async_ring::{MessageSpec, USIZE_LEN};

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

            let mut msg_data = MessageDataFirst::new();

            let read = msg_data.try_fill(&input);
            let res = msg_data.finish();
            let expected = {
                let mut buf = [0; MessageDataFirst::MESSAGE_SIZE];
                buf.copy_from_slice(&input);
                buf
            };
            assert_eq!(
                msg_data,
                MessageDataFirst {
                    buffer: expected,
                    read: MessageDataFirst::MESSAGE_SIZE
                }
            );
            assert_eq!(msg_data.is_done(), true);

            assert_eq!(read, MessageDataFirst::MESSAGE_SIZE);
            assert_eq!(res, MessageDataActual::new(2));
        }

        #[test]
        fn should_resume_across_calls() {
            let input: Vec<u8> = 2usize.to_ne_bytes().into_iter().collect();
            assert_eq!(input.len(), MessageDataFirst::MESSAGE_SIZE);

            let mut msg_data = MessageDataFirst::new();

            let read = msg_data.try_fill(&input[0..=1]);
            let expected = {
                let mut buf = [0; MessageDataFirst::MESSAGE_SIZE];
                (buf[..=1]).copy_from_slice(&input[0..=1]);
                buf
            };
            assert_eq!(
                msg_data,
                MessageDataFirst {
                    buffer: expected,
                    read: 2
                }
            );

            assert_eq!(read, 2);
            assert_eq!(msg_data.is_done(), false);

            // check the 0 length buffer given path
            let read = msg_data.try_fill(&[0; 0]);
            let expected = {
                let mut buf = [0; MessageDataFirst::MESSAGE_SIZE];
                (buf[..=1]).copy_from_slice(&input[0..=1]);
                buf
            };
            assert_eq!(
                msg_data,
                MessageDataFirst {
                    buffer: expected,
                    read: 2
                }
            );

            assert_eq!(read, 0);
            assert_eq!(msg_data.is_done(), false);

            // finish with the last bytes
            let read = msg_data.try_fill(&input[2..]);
            let res = msg_data.finish();
            let expected = {
                let mut buf = [0; MessageDataFirst::MESSAGE_SIZE];
                buf.copy_from_slice(&input);
                buf
            };
            assert_eq!(
                msg_data,
                MessageDataFirst {
                    buffer: expected,
                    read: MessageDataFirst::MESSAGE_SIZE
                }
            );
            assert_eq!(msg_data.is_done(), true);

            assert_eq!(read, MessageDataFirst::MESSAGE_SIZE - 2);
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
        use crate::backends::rusty::source::async_ring::MessageDataActual;

        #[test]
        fn should_read_complete_once() {
            let input: Vec<u8> = vec![1; 6];
            assert_eq!(input.len(), 6);

            let mut msg_data = MessageDataActual::new(6);

            let out_buf = &mut [0; 6];
            let read = msg_data.try_read(&input, out_buf);
            let expected = {
                let mut buf = [0; 6];
                buf.copy_from_slice(&input);
                buf
            };
            assert_eq!(msg_data, MessageDataActual { length: 6, read: 6 });

            assert_eq!(read, 6);
            assert_eq!(out_buf, &expected);
        }

        #[test]
        fn should_resume_across_calls() {
            let input: Vec<u8> = vec![1; 6];
            assert_eq!(input.len(), 6);

            let mut msg_data = MessageDataActual::new(6);

            let out_buf = &mut [0; 6];
            let read = msg_data.try_read(&input[0..=1], out_buf);
            let expected = {
                let mut buf = [0; 6];
                (buf[..=1]).copy_from_slice(&input[0..=1]);
                buf
            };
            assert_eq!(msg_data, MessageDataActual { length: 6, read: 2 });

            assert_eq!(read, 2);
            assert_eq!(out_buf, &expected);

            // check the 0 length buffer given path
            let out_buf = &mut [0; 6];
            let read = msg_data.try_read(&[0; 0], out_buf);
            let expected = [0; 6];
            assert_eq!(msg_data, MessageDataActual { length: 6, read: 2 });

            assert_eq!(read, 0);
            assert_eq!(out_buf, &expected);

            // finish with the last bytes
            let out_buf = &mut [0; 6];
            let read = msg_data.try_read(&input[2..], out_buf);
            let expected = {
                let mut buf = [0; 6];
                (buf[..4]).copy_from_slice(&input[2..]);
                buf
            };
            assert_eq!(msg_data, MessageDataActual { length: 6, read: 6 });

            assert_eq!(read, 4);
            assert_eq!(out_buf, &expected);
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
            let written = msg_data.try_read(&in_buf[0..=1], out_buf);
            let expected: Vec<u8> = [1; 2].into_iter().chain([0; 4].into_iter()).collect();

            assert_eq!(written, 2);
            assert_eq!(out_buf, expected.as_slice());

            // check the 0 length buffer given path
            let written = msg_data.try_read(&[0; 0], out_buf);

            assert_eq!(written, 0);

            // finish with the last bytes
            let in_buf = &[2; 4];
            let written = msg_data.try_read(in_buf, &mut out_buf[2..]);
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

            let mut msg_value = MessageDataValue::new();

            let read = msg_value.try_fill(&input);
            let res = msg_value.finish();
            let expected = {
                let mut buf = [0; MessageDataValue::MESSAGE_SIZE];
                buf.copy_from_slice(&input);
                buf
            };
            assert_eq!(
                msg_value,
                MessageDataValue {
                    buffer: expected,
                    read: MessageDataValue::MESSAGE_SIZE
                }
            );
            assert_eq!(msg_value.is_done(), true);

            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(res, 10);
        }

        #[test]
        fn should_resume_across_calls() {
            let input: &[u8] = &10i16.to_ne_bytes();
            assert_eq!(input.len(), MessageDataValue::MESSAGE_SIZE);

            let mut msg_value = MessageDataValue::new();

            let read = msg_value.try_fill(&input[0..1]);
            let expected = {
                let mut buf = [0; MessageDataValue::MESSAGE_SIZE];
                (buf[..1]).copy_from_slice(&input[0..1]);
                buf
            };
            assert_eq!(
                msg_value,
                MessageDataValue {
                    buffer: expected,
                    read: 1
                }
            );

            assert_eq!(read, 1);
            assert_eq!(msg_value.is_done(), false);

            // check the 0 length buffer given path
            let read = msg_value.try_fill(&[0; 0]);
            let expected = {
                let mut buf = [0; MessageDataValue::MESSAGE_SIZE];
                (buf[..1]).copy_from_slice(&input[0..1]);
                buf
            };
            assert_eq!(
                msg_value,
                MessageDataValue {
                    buffer: expected,
                    read: 1
                }
            );

            assert_eq!(read, 0);
            assert_eq!(msg_value.is_done(), false);

            // finish with the last bytes
            let read = msg_value.try_fill(&input[1..]);
            let res = msg_value.finish();
            let expected = {
                let mut buf = [0; MessageDataValue::MESSAGE_SIZE];
                buf.copy_from_slice(&input);
                buf
            };
            assert_eq!(
                msg_value,
                MessageDataValue {
                    buffer: expected,
                    read: MessageDataValue::MESSAGE_SIZE
                }
            );
            assert_eq!(msg_value.is_done(), true);

            assert_eq!(read, 1);
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

    mod parse_ring_messages {
        use crate::backends::rusty::source::async_ring::{
            MessageDataActual, MessageDataFirst, MessageSpec, MessageSpecResult, RingMessages,
            RingMsgParse,
        };

        #[test]
        fn should_read_complete_once_spec() {
            let input: Vec<u8> = [RingMessages::Spec.as_u8()]
                .into_iter()
                .chain([1; MessageSpec::MESSAGE_SIZE].into_iter())
                .collect();
            assert_eq!(input.len(), MessageSpec::MESSAGE_SIZE + 1);

            let (mut parser, read) = RingMsgParse::new(&input).unwrap();

            assert_eq!(read, 1);
            assert_eq!(parser, RingMsgParse::Spec(MessageSpec::new()));
            assert_eq!(parser.is_done(), false);
            assert_eq!(parser.is_fillable(), true);

            let input = &input[read..];

            let read = parser.try_fill(&input);

            assert_eq!(read, MessageSpec::MESSAGE_SIZE);
            assert_eq!(
                parser,
                RingMsgParse::Spec(MessageSpec {
                    buffer: [1; MessageSpec::MESSAGE_SIZE],
                    read: MessageSpec::MESSAGE_SIZE
                })
            );
            assert_eq!(parser.is_done(), true);
            assert_eq!(parser.is_fillable(), true);

            let res = parser.finish_spec();

            assert_eq!(
                res,
                MessageSpecResult {
                    rate: u32::from_ne_bytes([1; 4]),
                    channels: u16::from_ne_bytes([1; 2]),
                    current_frame_len: usize::from_ne_bytes([1; 8])
                }
            );
        }

        #[test]
        fn should_read_complete_once_data() {
            let input: Vec<u8> = [RingMessages::Data.as_u8()]
                .into_iter()
                .chain(2usize.to_ne_bytes().into_iter())
                .chain([1; 2].into_iter())
                .collect();
            assert_eq!(input.len(), MessageDataFirst::MESSAGE_SIZE + 2 + 1);

            let (mut parser, read) = RingMsgParse::new(&input).unwrap();

            assert_eq!(read, 1);
            assert_eq!(parser, RingMsgParse::DataFirst(MessageDataFirst::new()));
            assert_eq!(parser.is_done(), false);
            assert_eq!(parser.is_fillable(), true);

            let input = &input[read..];

            let read = parser.try_fill(&input);

            assert_eq!(read, MessageDataFirst::MESSAGE_SIZE);
            assert_eq!(
                parser,
                RingMsgParse::DataFirst(MessageDataFirst {
                    buffer: 2usize.to_ne_bytes(),
                    read: MessageDataFirst::MESSAGE_SIZE
                })
            );
            assert_eq!(parser.is_done(), true);
            assert_eq!(parser.is_fillable(), true);

            let mut parser = parser.finish_data_first();

            assert_eq!(parser, RingMsgParse::DataActual(MessageDataActual::new(2)));
            assert_eq!(parser.is_done(), false);
            assert_eq!(parser.is_fillable(), false);

            let out_buf = &mut [0; 2];
            let input = &input[read..];
            let read = parser.try_read_data(&input, out_buf);

            assert_eq!(read, 2);
            assert_eq!(parser.is_done(), true);
            assert_eq!(parser.is_fillable(), false);
            assert_eq!(out_buf, &[1; 2]);
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
            buf.advance_len(1);

            assert_eq!(buf.len(), 1);
            assert_eq!(buf.get_ref().len(), 1);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[u8::MAX; 1]);

            buf.advance_beginning(1);
            assert_eq!(buf.len(), 0);
            assert_eq!(buf.get_mut().len(), 31);
            assert_eq!(buf.get_ref(), &[0u8; 0]);

            buf.get_mut().fill(u8::MAX);
            buf.advance_len(31);

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
            buf.advance_len(17);
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

/// F
pub async fn playground() {
    use std::time::Duration;

    use symphonia::core::audio::{Channels, SignalSpec};

    let (mut prod, mut cons) = AsyncRingSource::new(
        SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
        None,
        REPEAT,
        MIN_SIZE,
        tokio::runtime::Handle::current(),
    );

    const REPEAT: usize = 20;

    tokio::task::spawn_blocking(move || {
        error!("SPAWNED");
        let mut old_spec = cons.rate;
        let mut total_nums = 0;
        while let Some(num) = cons.next() {
            info!("NUM: {:#?}", num);
            total_nums += 1;

            if total_nums <= REPEAT / 2 {
                assert_eq!(num, 10, "not 10, at {}", total_nums);
            } else {
                assert_eq!(num, 20, "not 20, at {}", total_nums);
            }

            if old_spec != cons.rate {
                error!("NEW SPEC: {:#?}", cons);
                old_spec = cons.rate;
            }

            if total_nums == REPEAT / 2 {
                let _ = cons.try_seek(Duration::from_secs(1));
            }
        }
        error!("EXITED total: {:#?}", total_nums);
    });

    let new_spec = SignalSpec::new(48000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT);

    let written = prod.new_spec(new_spec, REPEAT).await.unwrap();
    debug!("Written SPEC {:#?}", written);

    let data_buf = 10i16.to_ne_bytes().repeat(REPEAT / 2 + 10);

    let written = prod.write_data(&data_buf).await.unwrap();
    debug!("Written DATA {:#?}", written);

    let written = prod.new_eos().await.unwrap();
    debug!("Written EOS {:#?}", written);

    // drop(prod);

    // tokio::time::sleep(Duration::from_secs(2)).await;

    debug!("ORDER {:#?}", 10i16.to_ne_bytes());

    let res = prod.wait_seek().await.unwrap();

    debug!("AFTER SEEK RECV: {:#?}", res);

    let new_spec = SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT);

    prod.process_seek(new_spec, REPEAT, res.1).await;

    debug!("Written SPEC again");

    let data_buf = 20i16.to_ne_bytes().repeat(REPEAT / 2);

    let written = prod.write_data(&data_buf).await.unwrap();
    debug!("Written DATA {:#?}", written);

    let written = prod.new_eos().await.unwrap();
    debug!("Written EOS {:#?}", written);

    let res = prod.wait_seek().await;

    debug!("AFTER SEEK RECV: {:#?}", res);
}

pub static CALL_COUNT: LazyLock<Arc<AtomicUsize>> = LazyLock::new(|| Arc::new(AtomicUsize::new(0)));
