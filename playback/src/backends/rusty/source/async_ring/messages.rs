use std::ops::Range;

use symphonia::core::audio::SignalSpec;

use crate::backends::rusty::source::SampleType;

/// Types of messages that could appear in the ringbuffer with their id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RingMessages {
    // dont use 0 to differentiate from default buffer values.
    Data = 1,
    Spec = 2,
    Eos = 3,
}

impl RingMessages {
    /// Convert from a byte to a instance of this enum.
    ///
    /// This will panic if a unknown byte is given, as there is only ever expected to be known bytes.
    pub fn from_u8(byte: u8) -> Self {
        match byte {
            1 => Self::Data,
            2 => Self::Spec,
            3 => Self::Eos,
            v => unimplemented!(
                "This should never happen, unless there is de-sync. byte: {}",
                v
            ),
        }
    }

    /// Get the current instance's [`u8`] representation
    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Writer for Ringbuffer messages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RingMsgWrite2;

impl RingMsgWrite2 {
    pub const ID_SIZE: usize = 1;

    /// Add the [`ID_SIZE`](Self::ID_SIZE) to the given size to get the full message size.
    pub const fn get_msg_size(size: usize) -> usize {
        Self::ID_SIZE + size
    }

    /// Write the ID to the buffer and return how much has been written.
    ///
    /// This function assumes there is enough size in the buffer!
    fn write_id(id_type: RingMessages, buf: &mut [u8]) -> PResult<()> {
        if buf.len() < Self::ID_SIZE {
            return Err(Self::ID_SIZE - buf.len());
        }

        // buf[..Self::ID_SIZE] = id_type.as_u8();
        buf[0] = id_type.as_u8();

        Ok(((), Self::ID_SIZE))
    }

    /// Try to write a full [`RingMessages::Spec`] to the buffer, or return how many more bytes are necessary.
    ///
    /// This function will only write anything if there is enough space in the input buffer.
    pub fn try_write_spec(
        spec: SignalSpec,
        current_span_len: usize,
        buf: &mut [u8],
    ) -> PResult<()> {
        let size = Self::get_msg_size(MessageSpec::MESSAGE_SIZE);
        if buf.len() < size {
            return Err(size - buf.len());
        }

        let ((), written) = Self::write_id(RingMessages::Spec, buf).unwrap();
        let buf = &mut buf[written..];

        let ((), _written) = MessageSpec::try_write_buf(spec, current_span_len, buf).unwrap();

        Ok(((), size))
    }

    /// Try to write a full [`RingMessages::EOS`] to the buffer, or return how many more bytes are necessary.
    pub fn try_write_eos(buf: &mut [u8]) -> PResult<()> {
        let ((), written) = Self::write_id(RingMessages::Eos, buf)?;

        Ok(((), written))
    }

    /// Try to write a full [`RingMessages::Data`] (not the data itself) to the buffer, or return how many more bytes are necessary.
    ///
    /// This function will only write anything if there is enough space in the input buffer.
    pub fn try_write_data_first(length: usize, buf: &mut [u8]) -> PResult<MessageDataActual> {
        let size = Self::get_msg_size(MessageDataFirst::MESSAGE_SIZE);
        if buf.len() < size {
            return Err(size - buf.len());
        }

        let ((), written) = Self::write_id(RingMessages::Data, buf).unwrap();
        let buf = &mut buf[written..];

        let (data, _written) = MessageDataFirst::try_write_buf(length, buf).unwrap();

        Ok((data, size))
    }
}

/// Reader for Ringbuffer messages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RingMsgParse2 {
    Spec,
    Data(MessageDataActual),
    Eos,
}

pub type PResult<T> = Result<(T, usize), usize>;

/// The Content and result of a [`RingMessages::Spec`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageSpecResult {
    pub rate: u32,
    pub channels: u16,
    pub current_span_len: usize,
}

/// Read (and Write) a [`RingMessages::Spec`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MessageSpec;

const USIZE_LEN: usize = size_of::<usize>();

// TODO: maybe use nom or similar?
impl MessageSpec {
    pub const MESSAGE_SIZE: usize = size_of::<u32>() + size_of::<u16>() + size_of::<usize>();

    /// Try to read a message from the given buffer, or return how many bytes are still necessary
    pub fn try_read_buf(buf: &[u8]) -> PResult<MessageSpecResult> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        // read u32
        let rate: [u8; 4] = buf[0..=3].try_into().unwrap();
        // read u16
        let channels: [u8; 2] = buf[4..=5].try_into().unwrap();
        // read usize
        let current_span_len: [u8; USIZE_LEN] = buf[6..6 + USIZE_LEN].try_into().unwrap();

        let rate = u32::from_ne_bytes(rate);
        let channels = u16::from_ne_bytes(channels);
        let current_span_len = usize::from_ne_bytes(current_span_len);

        Ok((
            MessageSpecResult {
                rate,
                channels,
                current_span_len,
            },
            Self::MESSAGE_SIZE,
        ))
    }

    /// Try to write a message to the given buffer, or return how many bytes are still necessary
    pub fn try_write_buf(spec: SignalSpec, current_span_len: usize, buf: &mut [u8]) -> PResult<()> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        (buf[..=3]).copy_from_slice(&spec.rate.to_ne_bytes());
        let channels_u16 = u16::try_from(spec.channels.count()).unwrap();
        (buf[4..=5]).copy_from_slice(&channels_u16.to_ne_bytes());
        (buf[6..6 + USIZE_LEN]).copy_from_slice(&current_span_len.to_ne_bytes());

        Ok(((), Self::MESSAGE_SIZE))
    }
}

/// The first part of a [`RingMessages::Data`], reading the length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MessageDataFirst;

// TODO: maybe use nom or similar?
impl MessageDataFirst {
    pub const MESSAGE_SIZE: usize = size_of::<usize>();

    /// Try to read a message from the given buffer, or return how many bytes are still necessary
    pub fn try_read_buf(buf: &[u8]) -> PResult<MessageDataActual> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        // read usize
        let length_bytes: [u8; USIZE_LEN] = buf[..USIZE_LEN].try_into().unwrap();

        // read usize
        let length = usize::from_ne_bytes(length_bytes);

        Ok((MessageDataActual::new(length), Self::MESSAGE_SIZE))
    }

    /// Try to write a message to the given buffer, or return how many bytes are still necessary
    pub fn try_write_buf(length: usize, buf: &mut [u8]) -> PResult<MessageDataActual> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        buf[..Self::MESSAGE_SIZE].copy_from_slice(&length.to_ne_bytes());

        Ok((MessageDataActual::new_write(length), Self::MESSAGE_SIZE))
    }
}

/// The second part of a [`RingMessage::Data`], a helper to read the actual bytes of a message.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MessageDataActual {
    /// Length in bytes
    length: usize,
    /// Read in bytes
    read: usize,
}

impl MessageDataActual {
    /// Length in bytes
    pub fn new(length: usize) -> Self {
        // assert everything in "length" can be a full "ValueType"
        assert_eq!(length % size_of::<SampleType>(), 0);
        Self { length, read: 0 }
    }

    /// Length in bytes
    #[inline]
    pub fn new_write(length: usize) -> Self {
        Self::new(length)
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.read >= self.length
    }

    #[inline]
    pub fn get_range(&self) -> Range<usize> {
        self.read..self.length
    }

    /// Try to read a message from the given buffer, or return how many bytes are still necessary
    #[inline]
    pub fn try_read_buf(&mut self, buf: &[u8]) -> PResult<SampleType> {
        if self.is_done() {
            return Err(0);
        }

        let (value, read) = MessageDataValue::try_read_buf(buf)?;
        self.read += read;

        Ok((value, read))
    }

    /// Advance the read size when copied outside of [`try_read_buf`](Self::try_read_buf).
    ///
    /// `by` in bytes
    #[inline]
    pub fn advance_read(&mut self, by: usize) {
        self.read += by;
        assert!(self.read <= self.length);
    }

    /// Try to write a message to the given buffer, or return how many bytes are still necessary
    #[allow(dead_code)]
    pub fn try_write_buf(&mut self, val: SampleType, buf: &mut [u8]) -> PResult<()> {
        if self.is_done() {
            return Err(0);
        }

        let ((), written) = MessageDataValue::try_write_buf(val, buf)?;
        self.read += written;

        Ok(((), written))
    }
}

/// The size, in bytes, of a single [`SampleType`]
///
/// Note that this is currently the same as [`MessageDataValue::MESSAGE_SIZE`], but has a different semantic meaning.
pub const SAMPLE_TYPE_SIZE: usize = size_of::<SampleType>();

/// Read a single Data value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MessageDataValue;

// TODO: maybe use nom or similar?
impl MessageDataValue {
    pub const MESSAGE_SIZE: usize = SAMPLE_TYPE_SIZE;

    /// Try to read a message from the given buffer, or return how many bytes are still necessary
    #[inline]
    pub fn try_read_buf(buf: &[u8]) -> PResult<SampleType> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        // read ValueType
        let value_bytes: [u8; Self::MESSAGE_SIZE] = buf[..Self::MESSAGE_SIZE].try_into().unwrap();

        // read ValueType
        let value = SampleType::from_ne_bytes(value_bytes);

        Ok((value, Self::MESSAGE_SIZE))
    }

    /// Try to write a message to the given buffer, or return how many bytes are still necessary
    pub fn try_write_buf(val: SampleType, buf: &mut [u8]) -> PResult<()> {
        if buf.len() < Self::MESSAGE_SIZE {
            return Err(Self::MESSAGE_SIZE - buf.len());
        }

        buf[..Self::MESSAGE_SIZE].copy_from_slice(&val.to_ne_bytes());

        Ok(((), Self::MESSAGE_SIZE))
    }
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)] // make sure that tests fail if expectations change
mod tests {
    // Simple test to verify that the expectations have not changed, and only having one single place to change once it does
    #[test]
    fn expect_sampletype() {
        use super::SAMPLE_TYPE_SIZE;
        assert_eq!(SAMPLE_TYPE_SIZE, 4);
    }

    mod parse_message_spec {
        use crate::backends::rusty::source::async_ring::{MessageSpec, MessageSpecResult};

        #[test]
        fn should_read_complete_once() {
            let input: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes())
                .chain(10usize.to_ne_bytes())
                .collect();
            assert_eq!(input.len(), MessageSpec::MESSAGE_SIZE);

            let (res, read) = MessageSpec::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageSpec::MESSAGE_SIZE);
            assert_eq!(
                res,
                MessageSpecResult {
                    rate: 44000,
                    channels: 2,
                    current_span_len: 10,
                }
            );
        }

        #[test]
        fn should_report_additional() {
            let input: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes())
                .chain(10usize.to_ne_bytes())
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
                    current_span_len: 10,
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

            let msg_spec = MessageSpec::try_write_buf(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                10,
                out_buf,
            );

            assert_eq!(msg_spec, Ok(((), MessageSpec::MESSAGE_SIZE)));

            let expected: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes())
                .chain(10usize.to_ne_bytes())
                .collect();

            assert_eq!(out_buf, expected.as_slice());
        }

        #[test]
        fn should_write_across_calls() {
            let out_buf = &mut [0; MessageSpec::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageSpec::MESSAGE_SIZE);

            let msg_spec = MessageSpec::try_write_buf(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                10,
                &mut out_buf[..=3],
            );

            assert_eq!(msg_spec, Err(MessageSpec::MESSAGE_SIZE - 4));

            // check the 0 length buffer given path
            let msg_spec = MessageSpec::try_write_buf(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                10,
                &mut [],
            );

            assert_eq!(msg_spec, Err(MessageSpec::MESSAGE_SIZE));

            // finish with the last bytes
            let msg_spec = MessageSpec::try_write_buf(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                10,
                out_buf,
            );

            assert_eq!(msg_spec, Ok(((), MessageSpec::MESSAGE_SIZE)));

            let expected: Vec<u8> = 44000u32
                .to_ne_bytes()
                .into_iter()
                .chain(2u16.to_ne_bytes())
                .chain(10usize.to_ne_bytes())
                .collect();

            assert_eq!(out_buf, expected.as_slice());
        }
    }

    mod parse_message_data_first {
        use crate::{
            __bench::async_ring::messages::SAMPLE_TYPE_SIZE,
            backends::rusty::source::async_ring::{MessageDataActual, MessageDataFirst},
        };

        #[test]
        fn should_read_complete_once() {
            let input: Vec<u8> = SAMPLE_TYPE_SIZE.to_ne_bytes().into_iter().collect();
            assert_eq!(input.len(), MessageDataFirst::MESSAGE_SIZE);

            let (res, read) = MessageDataFirst::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageDataFirst::MESSAGE_SIZE);
            assert_eq!(res, MessageDataActual::new(4));
        }

        #[test]
        fn should_report_additional() {
            let input: Vec<u8> = SAMPLE_TYPE_SIZE.to_ne_bytes().into_iter().collect();
            assert_eq!(input.len(), MessageDataFirst::MESSAGE_SIZE);

            let additional = MessageDataFirst::try_read_buf(&input[0..=1]).unwrap_err();
            assert_eq!(additional, MessageDataFirst::MESSAGE_SIZE - 2);

            // check the 0 length buffer given path
            let additional = MessageDataFirst::try_read_buf(&[]).unwrap_err();
            assert_eq!(additional, MessageDataFirst::MESSAGE_SIZE);

            // finish with the last bytes
            let (res, read) = MessageDataFirst::try_read_buf(&input).unwrap();

            assert_eq!(read, MessageDataFirst::MESSAGE_SIZE);
            assert_eq!(res, MessageDataActual::new(SAMPLE_TYPE_SIZE));
        }
    }

    mod write_message_data_first {
        use crate::backends::rusty::source::async_ring::{MessageDataActual, MessageDataFirst};

        #[test]
        fn should_write_complete_once() {
            let out_buf = &mut [0; MessageDataFirst::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageDataFirst::MESSAGE_SIZE);

            let msg_spec = MessageDataFirst::try_write_buf(4, out_buf);

            assert_eq!(
                msg_spec,
                Ok((MessageDataActual::new(4), MessageDataFirst::MESSAGE_SIZE))
            );
        }

        #[test]
        fn should_write_across_calls() {
            let out_buf = &mut [0; MessageDataFirst::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageDataFirst::MESSAGE_SIZE);

            let msg_spec = MessageDataFirst::try_write_buf(4, &mut out_buf[..=1]);

            assert_eq!(msg_spec, Err(MessageDataFirst::MESSAGE_SIZE - 2));

            // check the 0 length buffer given path
            let msg_spec = MessageDataFirst::try_write_buf(4, &mut []);

            assert_eq!(msg_spec, Err(MessageDataFirst::MESSAGE_SIZE));

            // finish with the last bytes
            let msg_spec = MessageDataFirst::try_write_buf(4, out_buf);

            assert_eq!(
                msg_spec,
                Ok((MessageDataActual::new(4), MessageDataFirst::MESSAGE_SIZE))
            );
        }
    }

    mod parse_message_data_actual {
        use crate::{
            __bench::async_ring::messages::SAMPLE_TYPE_SIZE,
            backends::rusty::source::async_ring::{MessageDataActual, MessageDataValue},
        };

        #[expect(clippy::float_cmp)] // we dont to arthihmatic, we store and parse bytes, which *are* exact
        #[test]
        fn should_read_complete_once() {
            let input: Vec<u8> = vec![1; SAMPLE_TYPE_SIZE];

            let mut msg = MessageDataActual::new(SAMPLE_TYPE_SIZE);
            let (res, read) = msg.try_read_buf(&input).unwrap();
            assert_eq!(res, f32::from_ne_bytes([1; SAMPLE_TYPE_SIZE]));
            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.read, MessageDataValue::MESSAGE_SIZE);
            assert!(msg.is_done());
        }

        #[test]
        fn should_report_additional() {
            // check the 0 length buffer given path
            let mut msg = MessageDataActual::new(SAMPLE_TYPE_SIZE * 2);
            let additional = msg.try_read_buf(&[]).unwrap_err();
            assert_eq!(additional, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.read, 0);
            assert!(!msg.is_done());

            // some actual partial data
            let mut msg = MessageDataActual::new(SAMPLE_TYPE_SIZE * 2);
            let additional = msg.try_read_buf(&[1; 1]).unwrap_err();
            assert_eq!(additional, MessageDataValue::MESSAGE_SIZE - 1);
            assert_eq!(msg.read, 0);
            assert!(!msg.is_done());
        }

        #[expect(clippy::float_cmp)] // we dont to arthihmatic, we store and parse bytes, which *are* exact
        #[test]
        fn should_resume_additional() {
            let input: Vec<u8> = vec![1; SAMPLE_TYPE_SIZE * 2];

            let mut msg = MessageDataActual::new(SAMPLE_TYPE_SIZE * 2);
            let (res, read) = msg.try_read_buf(&input).unwrap();
            assert_eq!(res, f32::from_ne_bytes([1; SAMPLE_TYPE_SIZE]));
            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.read, MessageDataValue::MESSAGE_SIZE);
            assert!(!msg.is_done());

            // finish with the last bytes
            let (res, read) = msg.try_read_buf(&input).unwrap();
            assert_eq!(res, f32::from_ne_bytes([1; SAMPLE_TYPE_SIZE]));
            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(msg.read, MessageDataValue::MESSAGE_SIZE * 2);
            assert!(msg.is_done());
        }
    }

    mod write_message_data_actual {
        use crate::{
            __bench::async_ring::messages::SAMPLE_TYPE_SIZE,
            backends::rusty::source::async_ring::{MessageDataActual, MessageDataValue},
        };

        #[test]
        fn should_write_complete_once() {
            let out_buf = &mut [0; SAMPLE_TYPE_SIZE];

            let mut msg_data = MessageDataActual::new_write(SAMPLE_TYPE_SIZE);

            assert!(!msg_data.is_done());

            let res = msg_data.try_write_buf(1f32, out_buf);

            assert_eq!(res, Ok(((), SAMPLE_TYPE_SIZE)));
            assert_eq!(out_buf, &1f32.to_ne_bytes());
            assert!(msg_data.is_done());
        }

        #[test]
        fn should_write_across_calls() {
            let out_buf = &mut [0; SAMPLE_TYPE_SIZE];

            let mut msg_data = MessageDataActual::new_write(SAMPLE_TYPE_SIZE);

            let res = msg_data.try_write_buf(1f32, &mut out_buf[0..1]);

            assert_eq!(res, Err(MessageDataValue::MESSAGE_SIZE - 1));
            assert!(!msg_data.is_done());

            // check the 0 length buffer given path
            let res = msg_data.try_write_buf(1f32, &mut []);

            assert_eq!(res, Err(MessageDataValue::MESSAGE_SIZE));
            assert!(!msg_data.is_done());

            // finish with the last bytes
            let res = msg_data.try_write_buf(1f32, out_buf);

            assert_eq!(res, Ok(((), SAMPLE_TYPE_SIZE)));
            assert_eq!(out_buf, &1f32.to_ne_bytes());
            assert!(msg_data.is_done());
        }
    }

    mod parse_message_value {
        use crate::backends::rusty::source::async_ring::MessageDataValue;

        #[expect(clippy::float_cmp)] // we dont to arthihmatic, we store and parse bytes, which *are* exact
        #[test]
        fn should_read_complete_once() {
            let input: &[u8] = &10f32.to_ne_bytes();
            assert_eq!(input.len(), MessageDataValue::MESSAGE_SIZE);

            let (res, read) = MessageDataValue::try_read_buf(input).unwrap();

            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(res, 10f32);
        }

        #[expect(clippy::float_cmp)] // we dont to arthihmatic, we store and parse bytes, which *are* exact
        #[test]
        fn should_report_additional() {
            let input: &[u8] = &10f32.to_ne_bytes();
            assert_eq!(input.len(), MessageDataValue::MESSAGE_SIZE);

            assert!(MessageDataValue::MESSAGE_SIZE > 1);

            let additional = MessageDataValue::try_read_buf(&input[0..1]).unwrap_err();
            assert_eq!(additional, MessageDataValue::MESSAGE_SIZE - 1);

            // check the 0 length buffer given path
            let additional = MessageDataValue::try_read_buf(&[]).unwrap_err();
            assert_eq!(additional, MessageDataValue::MESSAGE_SIZE);

            // finish with the last bytes
            let (res, read) = MessageDataValue::try_read_buf(input).unwrap();

            assert_eq!(read, MessageDataValue::MESSAGE_SIZE);
            assert_eq!(res, 10f32);
        }
    }

    mod write_message_value {
        use crate::backends::rusty::source::async_ring::MessageDataValue;

        #[test]
        fn should_write_complete_once() {
            let out_buf = &mut [0; MessageDataValue::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageDataValue::MESSAGE_SIZE);

            let res = MessageDataValue::try_write_buf(10f32, out_buf);

            assert_eq!(res, Ok(((), MessageDataValue::MESSAGE_SIZE)));

            let expected: &[u8] = &10f32.to_ne_bytes();
            assert_eq!(out_buf, expected);
        }

        #[test]
        fn should_write_across_calls() {
            let out_buf = &mut [0; MessageDataValue::MESSAGE_SIZE];
            assert_eq!(out_buf.len(), MessageDataValue::MESSAGE_SIZE);

            let res = MessageDataValue::try_write_buf(10f32, &mut out_buf[..1]);

            assert_eq!(res, Err(MessageDataValue::MESSAGE_SIZE - 1));

            // check the 0 length buffer given path
            let res = MessageDataValue::try_write_buf(10f32, &mut []);

            assert_eq!(res, Err(MessageDataValue::MESSAGE_SIZE));

            // finish with the last bytes
            let res = MessageDataValue::try_write_buf(10f32, out_buf);

            assert_eq!(res, Ok(((), MessageDataValue::MESSAGE_SIZE)));

            let expected: &[u8] = &10f32.to_ne_bytes();
            assert_eq!(out_buf, expected);
        }
    }

    mod write_ring_messages {
        use symphonia::core::audio::{Channels, SignalSpec};

        use crate::{
            __bench::async_ring::messages::SAMPLE_TYPE_SIZE,
            backends::rusty::source::async_ring::{
                MessageDataActual, MessageDataFirst, MessageDataValue, MessageSpec, RingMessages,
                RingMsgWrite2,
            },
        };

        #[test]
        fn should_write_complete_once_spec() {
            let out_buf = &mut [0; RingMsgWrite2::get_msg_size(MessageSpec::MESSAGE_SIZE)];
            assert_eq!(
                out_buf.len(),
                MessageSpec::MESSAGE_SIZE + RingMsgWrite2::ID_SIZE
            );

            let res = RingMsgWrite2::try_write_spec(
                SignalSpec::new(44000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
                10,
                out_buf,
            );

            assert_eq!(
                res,
                Ok(((), RingMsgWrite2::ID_SIZE + MessageSpec::MESSAGE_SIZE))
            );

            let expected: Vec<u8> = [RingMessages::Spec.as_u8()]
                .into_iter()
                .chain(44000u32.to_ne_bytes())
                .chain(2u16.to_ne_bytes())
                .chain(10usize.to_ne_bytes())
                .collect();
            assert_eq!(out_buf, expected.as_slice());
        }

        #[test]
        fn should_write_complete_once_data() {
            let out_buf = &mut [0; RingMsgWrite2::get_msg_size(
                MessageDataFirst::MESSAGE_SIZE + SAMPLE_TYPE_SIZE,
            )];
            assert_eq!(
                out_buf.len(),
                MessageDataFirst::MESSAGE_SIZE + SAMPLE_TYPE_SIZE + RingMsgWrite2::ID_SIZE
            );

            let res = RingMsgWrite2::try_write_data_first(SAMPLE_TYPE_SIZE, out_buf);

            assert_eq!(
                res,
                Ok((
                    MessageDataActual::new_write(4),
                    RingMsgWrite2::ID_SIZE + MessageDataFirst::MESSAGE_SIZE
                ))
            );
            let expected: Vec<u8> = [RingMessages::Data.as_u8()]
                .into_iter()
                .chain(SAMPLE_TYPE_SIZE.to_ne_bytes())
                .chain([0; SAMPLE_TYPE_SIZE])
                .collect();
            assert_eq!(out_buf, expected.as_slice());

            let (mut data, written) = res.unwrap();

            assert!(!data.is_done());

            let res = data.try_write_buf(1f32, &mut out_buf[written..]);

            assert_eq!(res, Ok(((), MessageDataValue::MESSAGE_SIZE)));
            assert!(data.is_done());

            let expected: Vec<u8> = [RingMessages::Data.as_u8()]
                .into_iter()
                .chain(SAMPLE_TYPE_SIZE.to_ne_bytes())
                .chain(1f32.to_ne_bytes())
                .collect();

            assert_eq!(out_buf, expected.as_slice());
        }

        #[test]
        fn should_write_complete_once_eos() {
            let out_buf = &mut [0; RingMsgWrite2::ID_SIZE];
            assert_eq!(out_buf.len(), RingMsgWrite2::ID_SIZE);

            let res = RingMsgWrite2::try_write_eos(out_buf);

            assert_eq!(res, Ok(((), RingMsgWrite2::ID_SIZE)));

            let expected: &[u8] = &[RingMessages::Eos.as_u8()];
            assert_eq!(out_buf, expected);
        }
    }
}
