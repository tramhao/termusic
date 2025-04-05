use std::{fmt::Debug, future::Future, iter::FusedIterator, sync::Arc, time::Duration};

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

use messages::{
    MessageDataActual, MessageDataFirst, MessageDataValue, MessageSpec, MessageSpecResult,
    RingMessages, RingMsgParse2, RingMsgWrite2, ValueType,
};
use wrap::{ConsWrap, ProdWrap};

mod messages;
mod wrap;

/// Seek Channel data, the first is the point to seek to in the decoder
/// the second is a callback that the seek is done and how many elements to skip until new data.
pub type SeekData = (Duration, oneshot::Sender<usize>);

/// The minimal size a decode-ringbuffer should have.
///
/// Currently the size is based on 192kHz * 1 seconds, or 4 seconds of 48kHz audio.
const MIN_RING_SIZE: usize = 192_000 * MessageDataValue::MESSAGE_SIZE;

/// A ringbuffer Producer meant for wrapping [`Source`] to make decode & playback async and keep the buffer filled without having audible gaps.
///
/// The implementation of the Producer is meant for async code, awaiting to fully put data on the ringbuffer OR waiting for seek events.
#[derive(Debug)]
pub struct AsyncRingSourceProvider {
    inner: ProdWrap,
    seek_rx: Arc<RwLock<mpsc::Receiver<SeekData>>>,

    data: Option<MessageDataActual>,
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
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Write a new spec.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    ///
    /// # Errors
    ///
    /// Ringbuffer closed and no more data available.
    #[allow(clippy::missing_panics_doc)] // https://github.com/rust-lang/rust-clippy/issues/14534
    pub async fn new_spec(
        &mut self,
        spec: SignalSpec,
        current_frame_len: usize,
    ) -> Result<usize, ()> {
        let mut msg_buf = [0; RingMsgWrite2::get_msg_size(MessageSpec::MESSAGE_SIZE)];
        // SAFETY: we allocaed the exact necessary size, this can never fail
        // #[expect(
        //     clippy::missing_panics_doc,
        //     reason = "static buffer with exact size required created above"
        // )]
        let _ = RingMsgWrite2::try_write_spec(spec, current_frame_len, &mut msg_buf).unwrap();

        self.inner.push_exact(&msg_buf).await.map_err(|_| ())?;

        Ok(msg_buf.len())
    }

    /// Write a new data message, without the buffer yet.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    ///
    /// # Errors
    ///
    /// Ringbuffer closed and no more data available.
    #[allow(clippy::missing_panics_doc)] // https://github.com/rust-lang/rust-clippy/issues/14534
    async fn new_data(&mut self, length: usize) -> Result<usize, ()> {
        let mut msg_buf = [0; RingMsgWrite2::get_msg_size(MessageDataFirst::MESSAGE_SIZE)];
        // SAFETY: we allocaed the exact necessary size, this can never fail
        // #[expect(
        //     clippy::missing_panics_doc,
        //     reason = "static buffer with exact size required created above"
        // )]
        let (data, _written) = RingMsgWrite2::try_write_data_first(length, &mut msg_buf).unwrap();

        self.inner.push_exact(&msg_buf).await.map_err(|_| ())?;

        self.data = Some(data);

        Ok(msg_buf.len())
    }

    /// Write a buffer's content.
    ///
    /// This functions expects a data message to be active.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    ///
    /// # Errors
    ///
    /// Ringbuffer closed and no more data available.
    async fn write_data_inner(&mut self, data: &[u8]) -> Result<usize, ()> {
        let Some(msg) = &mut self.data else {
            unimplemented!("This should be checked outside of the function");
        };

        let buf = &data[msg.get_range()];
        self.inner.push_exact(buf).await.map_err(|_| ())?;
        msg.advance_read(buf.len());

        Ok(buf.len())
    }

    /// Write a given buffer as a data message.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    ///
    /// # Errors
    ///
    /// Ringbuffer closed and no more data available.
    #[allow(clippy::missing_panics_doc)] // https://github.com/rust-lang/rust-clippy/issues/14534
    pub async fn write_data(&mut self, data: &[u8]) -> Result<usize, ()> {
        if data.is_empty() {
            return Err(());
        }

        let mut written = 0;
        if self.data.is_none() {
            written += self.new_data(data.len()).await?;
        }

        // #[expect(
        //     clippy::missing_panics_doc,
        //     reason = "it is ensured to be Some via if and `new_data` above"
        // )]
        while !self.data.as_mut().unwrap().is_done() && !self.inner.is_closed() {
            written += self.write_data_inner(data).await?;
        }

        self.data.take();

        Ok(written)
    }

    /// Write a EOS message.
    ///
    /// Returns [`Ok(count)`](Ok) if the message is written, with the length, [`Err`] if closed.
    ///
    /// # Errors
    ///
    /// Ringbuffer closed and no more data available.
    #[allow(clippy::missing_panics_doc)] // https://github.com/rust-lang/rust-clippy/issues/14534
    pub async fn new_eos(&mut self) -> Result<usize, ()> {
        let mut msg_buf = [0; RingMsgWrite2::get_msg_size(0)];
        // SAFETY: we allocaed the exact necessary size, this can never fail
        // #[expect(
        //     clippy::missing_panics_doc,
        //     reason = "static buffer with exact size required created above"
        // )]
        let _ = RingMsgWrite2::try_write_eos(&mut msg_buf).unwrap();

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

/// Struct to wait on a channel, without the compiler complaining that `self` cant be borrowed mutably multiple times.
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

/// A ringbuffer Consumer meant for wrapping [`Source`] to make decode & playback async and keep the buffer filled without having audible gaps.
///
/// The implementation of the Consumer is meant for sync code, only blocking when having to wait for more data.
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
    ///
    /// # Panics
    ///
    /// If the given channels in `spec` cannot be converted to a u16 (which should never happen as there are only 26 defined channels)
    #[must_use]
    pub fn new(
        spec: SignalSpec,
        total_duration: Option<Duration>,
        current_frame_len: usize,
        size: usize,
        handle: Handle,
    ) -> (AsyncRingSourceProvider, Self) {
        let size = size.max(MIN_RING_SIZE);
        let ringbuf = AsyncHeapRb::<u8>::new(size);
        let (prod, cons) = ringbuf.split();
        // Channel if exactly 1 message size, as seeks should not happen often, and if they do, only one can be processed at once.
        let (tx, rx) = mpsc::channel(1);

        let async_prod = AsyncRingSourceProvider::new(ProdWrap::new(prod), rx);
        let async_cons = Self {
            inner: ConsWrap::new(cons),
            seek_tx: Some(tx),
            // SAFETY: as of symphonia 0.5.4, there can only be at most 26 channels (Channels::all().count())
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
    fn read_msg(&mut self) -> Option<RingMsgParse2> {
        // trace!("Reading a message from the ringbuffer");

        self.load_more_data(1)?;

        assert!(!self.buf.is_empty());

        let detected_type = {
            let detect_byte = {
                // SAFETY: we loaded and asserted that there is at least one byte
                let byte = self.buf.get_ref()[0];
                self.buf.advance_beginning(1);
                byte
            };

            RingMessages::from_u8(detect_byte)
        };

        // Eos event does not have more than the id itself
        if detected_type == RingMessages::Eos {
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

            self.load_more_data(wait_for_bytes)?;

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
                RingMessages::Eos => unreachable!("Message EOS is returned earlier"),
            };

            assert!(read > 0);

            self.buf.advance_beginning(read);

            return Some(msg);
        }
    }

    /// Loads more data into the current buffer, if the current buffer does not have at least `wait_for_bytes` bytes.
    ///
    /// This function will **not** block when there is `wait_for_bytes` available, but **will** block otherwise.
    ///
    /// Returns [`Some`] if the current buffer now has at least `wait_for_bytes` buffered, [`None`] if the buffer closed and not enough can be loaded anymore.
    fn load_more_data(&mut self, wait_for_bytes: usize) -> Option<()> {
        if self.buf.len() >= wait_for_bytes {
            return Some(());
        }

        self.buf.maybe_need_move();

        // wait for at least one element being occupied,
        // more elements would mean to wait until they all are there, regardless if they are part of the message or not

        // Avoid calling into async-runtime and async code if the ringbuffer knowingly already contains enough bytes
        // as reading can be done sync, non-blocking for buffer copies.
        // Also SAFETY: "occupied_len" says it "could be more or less" but we are in the consumer here, so we known it wont *decrease*
        // between here and actually reading it.
        if self.inner.occupied_len() < wait_for_bytes {
            // Avoid having to call async stuff for as long as possible, as that can heavily increase CPU load in a hot path.
            // When not doing this, cpu load can be 1.0~1.4 on average.
            // When doing the current way, the load is ~0.5~0.6 on average, the same as if running the decoder directly as
            // as source instead of using this ringbuffer.
            self.handle
                .block_on(self.inner.wait_occupied(wait_for_bytes));
        }

        if self.inner.is_closed() && self.inner.is_empty() {
            return None;
        }

        let written = self.inner.pop_slice(self.buf.get_spare_mut());
        self.buf.advance_len(written);

        // Sanity, this point should never be reached if the buffer contains enough data or there is no more data to read
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
    fn read_data(&mut self) -> Option<i16> {
        // trace!("Reading Data");

        // wait until we have enough data to parse a value
        self.load_more_data(MessageDataValue::MESSAGE_SIZE)?;

        assert!(self.buf.len() >= MessageDataValue::MESSAGE_SIZE);

        let msg = self.last_msg.as_mut().unwrap();

        // unwrap: should never panic as we explicitly load at least the required amount above.
        #[allow(clippy::missing_panics_doc)]
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
                let sample = self.read_data();

                return sample;
            }

            if self.inner.is_empty() && self.inner.is_closed() {
                debug!("DONE");
                return None;
            }

            let msg = self.read_msg()?;

            match msg {
                RingMsgParse2::Spec => {}
                RingMsgParse2::Data(message_data_actual) => {
                    self.last_msg = Some(message_data_actual);
                }
                RingMsgParse2::Eos => {
                    if self.seek_tx.take().is_some() {
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
    #[inline]
    fn len(&self) -> usize {
        self.get_ref().len()
    }

    /// Returns `true` if there is currently no good data in the buffer
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Rest good data length.
    #[inline]
    fn clear(&mut self) {
        self.data_start_idx = 0;
        self.used_len = 0;

        // // DEBUG: this should not be necessary, but for debugging the buffer
        // self.buf.fill(u8::MAX);
    }

    /// Get a mutable reference to the slice from `data_start` until end.
    ///
    /// May contain bad data.
    /// And [`advance_len`](Self::advance_len) needs to be called afterward with the written size.
    #[inline]
    #[allow(unused)]
    fn get_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.data_start_idx..]
    }

    /// Get a mutable reference to the unused portion of the buffer (starting from `len`)
    ///
    /// May contain bad data.
    /// And [`advance_len`](Self::advance_len) needs to be called afterward with the written size.
    #[inline]
    fn get_spare_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.used_len..]
    }

    /// Get a reference to the slice which contains good data
    #[inline]
    fn get_ref(&self) -> &[u8] {
        &self.buf[self.data_start_idx..self.used_len]
    }

    /// Move the data to the beginning, if start is above half the capacity
    #[inline]
    fn maybe_need_move(&mut self) {
        // Fast-path: clear if start idx is above 0 and there are no good elements
        if self.data_start_idx > 0 && self.is_empty() {
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

    /// Advance the initialized data length by the given count.
    ///
    /// # Panics
    ///
    /// If the given `by` count plus the current `length` will result in higher than `capacity`.
    #[inline]
    fn advance_len(&mut self, by: usize) {
        self.set_len(self.len() + by);
    }

    /// Set the length of the buffer to the written size plus data start, ie how the buffer was given from [`get_mut`](Self::get_mut).
    ///
    /// # Panics
    ///
    /// If the given `written` plus the current `start_idx` will result in higher than `capacity`.
    #[inline]
    fn set_len(&mut self, written: usize) {
        self.used_len = self.data_start_idx + written;
        assert!(self.used_len <= self.buf.len());
    }

    /// Advance the start index.
    ///
    /// Use [`make_beginning`](Self::make_beginning) to move all data to the front again.
    ///
    /// # Panics
    ///
    /// If the given `by` count plus the current `start_idx` will result in higher than `capacity`.
    #[inline]
    fn advance_beginning(&mut self, by: usize) {
        self.data_start_idx += by;
        assert!(self.data_start_idx <= self.buf.len());

        // // DEBUG: this should not be necessary, but for debugging the buffer
        // self.buf[0..self.data_start_idx].fill(u8::MAX);
    }
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)] // make sure that tests fail if expectations change
mod tests {

    mod static_buffer {
        use crate::backends::rusty::source::async_ring::StaticBuf;

        #[test]
        fn should_work() {
            let mut buf = StaticBuf::<32>::new();
            assert_eq!(buf.len(), 0);
            assert_eq!(buf.get_ref().len(), 0);
            assert_eq!(buf.len(), buf.used_len);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[0u8; 0]);

            buf.get_mut()[0] = u8::MAX;
            buf.set_len(1);

            assert_eq!(buf.len(), 1);
            assert_eq!(buf.get_ref().len(), 1);
            assert_eq!(buf.len(), buf.used_len);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[u8::MAX; 1]);

            buf.advance_beginning(1);
            assert_eq!(buf.len(), 0);
            assert_eq!(buf.used_len, 1);
            assert_ne!(buf.len(), buf.used_len);
            assert_eq!(buf.get_mut().len(), 31);
            assert_eq!(buf.get_ref(), &[0u8; 0]);

            buf.get_mut().fill(u8::MAX);
            buf.set_len(31);

            assert_eq!(buf.len(), 31);
            assert_eq!(buf.used_len, 32);
            assert_ne!(buf.len(), buf.used_len);
            assert_eq!(buf.get_mut().len(), 31);
            assert_eq!(buf.get_ref(), &[u8::MAX; 31]);

            buf.make_beginning();

            assert_eq!(buf.len(), 31);
            assert_eq!(buf.len(), buf.used_len);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[u8::MAX; 31]);

            buf.advance_beginning(15);

            assert_eq!(buf.len(), 16);
            assert_ne!(buf.len(), buf.used_len);
            assert_eq!(buf.used_len, 31);
            assert_eq!(buf.get_mut().len(), 17);
            assert_eq!(buf.get_ref(), &[u8::MAX; 16]);

            buf.clear();

            assert_eq!(buf.len(), 0);
            assert_eq!(buf.len(), buf.used_len);
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

        #[test]
        fn advance_length() {
            let mut buf = StaticBuf::<32>::new();
            assert_eq!(buf.len(), 0);
            assert_eq!(buf.get_ref().len(), 0);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_spare_mut().len(), 32);
            assert_eq!(buf.get_ref(), &[0u8; 0]);

            buf.get_mut()[..4].fill(4);
            buf.advance_len(4);

            assert_eq!(buf.len(), 4);
            assert_eq!(buf.get_ref().len(), 4);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_spare_mut().len(), 28);
            let expected = &[4u8; 4];
            assert_eq!(buf.get_ref(), expected);

            buf.get_spare_mut()[..4].fill(6);
            buf.advance_len(4);

            assert_eq!(buf.len(), 8);
            assert_eq!(buf.get_ref().len(), 8);
            assert_eq!(buf.get_mut().len(), 32);
            assert_eq!(buf.get_spare_mut().len(), 24);
            let expected: Vec<_> = [4u8; 4].into_iter().chain([6u8; 4]).collect();
            assert_eq!(buf.get_ref(), &expected);

            buf.advance_beginning(5);

            assert_eq!(buf.len(), 3);
            assert_eq!(buf.get_ref().len(), 3);
            assert_eq!(buf.get_mut().len(), 27);
            assert_eq!(buf.get_spare_mut().len(), 24);
            let expected = &[6u8; 3];
            assert_eq!(buf.get_ref(), expected);
        }
    }

    mod ringbuffer {
        use std::{sync::Arc, time::Duration};

        use async_ringbuf::traits::Observer;
        use parking_lot::Mutex;
        use symphonia::core::audio::{Channels, SignalSpec};

        use crate::backends::rusty::source::async_ring::{
            AsyncRingSource, MessageDataFirst, MessageSpec, RingMsgWrite2, ValueType, MIN_RING_SIZE,
        };

        #[tokio::test]
        async fn should_work() {
            let mut send = Vec::new();
            let recv = Arc::new(Mutex::new(Vec::new()));

            let (mut prod, mut cons) = AsyncRingSource::new(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                None,
                1024,
                0,
                tokio::runtime::Handle::current(),
            );

            assert_eq!(prod.inner.capacity().get(), MIN_RING_SIZE);

            let recv_c = recv.clone();
            let handle = tokio::task::spawn_blocking(move || {
                let mut lock = recv_c.lock();
                for num in cons.by_ref() {
                    lock.extend_from_slice(&num.to_ne_bytes());
                }
                assert_eq!(cons.inner.occupied_len(), 0);
            });

            let first_data = 1i16.to_le_bytes().repeat(1024);
            let written = prod.write_data(&first_data).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite2::get_msg_size(MessageDataFirst::MESSAGE_SIZE + first_data.len())
            );
            send.extend_from_slice(&first_data);

            let new_spec = SignalSpec::new(48000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT);
            let written = prod.new_spec(new_spec, 1024).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite2::get_msg_size(MessageSpec::MESSAGE_SIZE)
            );

            let second_data = 2i16.to_le_bytes().repeat(1024);
            let written = prod.write_data(&second_data).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite2::get_msg_size(MessageDataFirst::MESSAGE_SIZE + second_data.len())
            );
            send.extend_from_slice(&second_data);

            let written = prod.new_eos().await.unwrap();
            assert_eq!(written, RingMsgWrite2::get_msg_size(0));

            prod.write_data(&[]).await.unwrap_err();

            // just to prevent a inifinitely running test due to a deadlock
            let res = tokio::time::timeout(Duration::from_secs(3), handle)
                .await
                .is_ok();
            assert!(res, "Read Task did not complete within 3 seconds");

            assert!(prod.is_closed());
            assert!(prod.inner.is_empty());

            let recv_lock = recv.lock();
            let value_size = size_of::<ValueType>();
            assert_eq!(send.len(), value_size * 1024 * 2);
            assert_eq!(recv_lock.len(), value_size * 1024 * 2);

            assert_eq!(*send, *recv_lock);
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

            assert!(obsv.read_is_held());
            assert!(obsv.write_is_held());
            let order_c = order.clone();

            let cons_handle = tokio::task::spawn_blocking(move || {
                for num in cons.by_ref() {
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
                    RingMsgWrite2::get_msg_size(MessageDataFirst::MESSAGE_SIZE + first_data.len())
                );

                assert!(obsv_c.read_is_held());
                assert!(obsv_c.write_is_held());

                let written = prod.new_eos().await.unwrap();
                assert_eq!(written, RingMsgWrite2::get_msg_size(0));

                let _ = prod.wait_seek().await;
                order_c.lock().push("prod");
            });

            // just to prevent a inifinitely running test due to a deadlock
            let res = tokio::time::timeout(Duration::from_secs(3), cons_handle)
                .await
                .is_ok();
            assert!(res, "Read Task did not complete within 3 seconds");

            assert!(!obsv.read_is_held());

            // just to prevent a inifinitely running test due to a deadlock
            let res = tokio::time::timeout(Duration::from_secs(3), prod_handle)
                .await
                .is_ok();
            assert!(res, "Write Task did not complete within 3 seconds");

            assert!(!obsv.write_is_held());

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
                for num in cons.by_ref() {
                    lock.extend_from_slice(&num.to_ne_bytes());
                }
                assert_eq!(cons.inner.occupied_len(), 0);
                assert!(!cons.inner.write_is_held());
            });

            let first_data = 1i16.to_le_bytes().repeat(1024);
            let written = prod.write_data(&first_data).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite2::get_msg_size(MessageDataFirst::MESSAGE_SIZE + first_data.len())
            );

            let written = prod.new_eos().await.unwrap();
            assert_eq!(written, RingMsgWrite2::get_msg_size(0));

            let obsv = prod.inner.observe();
            drop(prod);

            assert!(!obsv.write_is_held());
            // dont check read as that *could* have consumed and exited already
            // assert_eq!(obsv.read_is_held(), true);

            // just to prevent a inifinitely running test due to a deadlock
            let res = tokio::time::timeout(Duration::from_secs(3), handle)
                .await
                .is_ok();
            assert!(res, "Read Task did not complete within 3 seconds");

            assert!(!obsv.write_is_held());
            assert!(!obsv.read_is_held());

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
                for num in cons.by_ref() {
                    lock.extend_from_slice(&num.to_ne_bytes());
                }
                assert_eq!(cons.inner.occupied_len(), 0);
                assert!(!cons.inner.write_is_held());
            });

            let first_data = 1i16.to_le_bytes().repeat(1024);
            let written = prod.write_data(&first_data).await.unwrap();
            assert_eq!(
                written,
                RingMsgWrite2::get_msg_size(MessageDataFirst::MESSAGE_SIZE + first_data.len())
            );

            let obsv = prod.inner.observe();
            drop(prod);

            assert!(!obsv.write_is_held());
            // dont check read as that *could* have consumed and exited already
            // assert_eq!(obsv.read_is_held(), true);

            // just to prevent a inifinitely running test due to a deadlock
            let res = tokio::time::timeout(Duration::from_secs(3), handle)
                .await
                .is_ok();
            assert!(res, "Read Task did not complete within 3 seconds");

            assert!(!obsv.write_is_held());
            assert!(!obsv.read_is_held());

            let recv_lock = recv.lock();
            let value_size = size_of::<ValueType>();
            assert_eq!(recv_lock.len(), value_size * 1024);

            assert_eq!(*recv_lock, first_data.as_slice());
        }
    }
}
