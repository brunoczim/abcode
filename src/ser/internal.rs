use serde::Serialize;
use tokio::{
    io::{self, AsyncWrite, AsyncWriteExt},
    sync::mpsc,
};

use super::Error;

pub trait SerializationSink {
    fn send_raw_data(&mut self, data: &[u8]) -> Result<(), Error>;

    fn start_var_sized(&mut self, size: Option<usize>) -> Result<(), Error>;

    fn advance_var_sized(&mut self) -> Result<(), Error>;

    fn end_var_sized(&mut self) -> Result<(), Error>;

    fn send_bool(&mut self, value: bool) -> Result<(), Error> {
        self.send_u8(u8::from(value))
    }

    fn send_u8(&mut self, value: u8) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_i8(&mut self, value: i8) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_u16(&mut self, value: u16) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_i16(&mut self, value: i16) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_u32(&mut self, value: u32) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_i32(&mut self, value: i32) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_u64(&mut self, value: u64) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_i64(&mut self, value: i64) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_u128(&mut self, value: u128) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_i128(&mut self, value: i128) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_usize(&mut self, value: usize) -> Result<(), Error> {
        let fixed_int =
            u64::try_from(value).map_err(|_| Error::ExcessiveSize(value))?;
        self.send_u64(fixed_int)
    }

    fn send_isize(&mut self, value: isize) -> Result<(), Error> {
        let fixed_int = i64::try_from(value)
            .map_err(|_| Error::ExcessiveSizeDiff(value))?;
        self.send_i64(fixed_int)
    }

    fn send_f32(&mut self, value: f32) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_f64(&mut self, value: f64) -> Result<(), Error> {
        self.send_raw_data(&value.to_le_bytes())
    }

    fn send_char(&mut self, value: char) -> Result<(), Error> {
        self.send_u32(u32::from(value))
    }

    fn send_bytes(&mut self, value: &[u8]) -> Result<(), Error> {
        self.send_usize(value.len())?;
        self.send_raw_data(value)?;
        Ok(())
    }

    fn send_str(&mut self, value: &str) -> Result<(), Error> {
        self.send_bytes(value.as_bytes())
    }
}

#[derive(Debug)]
pub struct ChannelBackend<W> {
    device: W,
    buf: Vec<u8>,
    buf_limit: usize,
    receiver: mpsc::Receiver<u8>,
}

impl<W> ChannelBackend<W>
where
    W: AsyncWrite + Unpin,
{
    pub fn new(
        device: W,
        buf_limit: usize,
        receiver: mpsc::Receiver<u8>,
    ) -> Self {
        Self { device, buf: Vec::with_capacity(buf_limit), buf_limit, receiver }
    }

    pub async fn run(mut self) -> io::Result<()> {
        while self.receiver.recv_many(&mut self.buf, self.buf_limit).await > 0 {
            self.device.write_all(&self.buf[..]).await?;
            self.buf.clear();
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChannelSink {
    sender: mpsc::Sender<u8>,
    fallback_buffer: BufferSink,
    multiplexing: ChannelSinkMultiplexing,
}

impl ChannelSink {
    pub fn new(sender: mpsc::Sender<u8>) -> Self {
        Self {
            sender,
            fallback_buffer: BufferSink::new(),
            multiplexing: ChannelSinkMultiplexing::Channel,
        }
    }
}

impl SerializationSink for ChannelSink {
    fn send_raw_data(&mut self, data: &[u8]) -> Result<(), Error> {
        match self.multiplexing {
            ChannelSinkMultiplexing::Channel => {
                for element in data {
                    self.sender
                        .blocking_send(*element)
                        .map_err(|_| Error::Disconnected)?;
                }
            },

            ChannelSinkMultiplexing::Buffer { .. } => {
                self.fallback_buffer.send_raw_data(data)?
            },
        }

        Ok(())
    }

    fn start_var_sized(&mut self, size: Option<usize>) -> Result<(), Error> {
        match self.multiplexing {
            ChannelSinkMultiplexing::Channel => match size {
                Some(known_len) => self.send_usize(known_len)?,
                None => {
                    self.multiplexing = ChannelSinkMultiplexing::Buffer {
                        outer_seq_size: 0,
                        inner_seqs: 0,
                    };
                },
            },

            ChannelSinkMultiplexing::Buffer { outer_seq_size, inner_seqs } => {
                self.fallback_buffer.start_var_sized(size)?;
                self.multiplexing = ChannelSinkMultiplexing::Buffer {
                    outer_seq_size,
                    inner_seqs: inner_seqs + 1,
                };
            },
        }

        Ok(())
    }

    fn end_var_sized(&mut self) -> Result<(), Error> {
        match self.multiplexing {
            ChannelSinkMultiplexing::Channel => (),

            ChannelSinkMultiplexing::Buffer {
                outer_seq_size,
                inner_seqs: 0,
            } => {
                self.send_usize(outer_seq_size)?;
                for byte in self.fallback_buffer.as_slice() {
                    self.sender
                        .blocking_send(*byte)
                        .map_err(|_| Error::Disconnected)?;
                }
                self.fallback_buffer.clear();
            },

            ChannelSinkMultiplexing::Buffer { outer_seq_size, inner_seqs } => {
                self.fallback_buffer.end_var_sized()?;
                self.multiplexing = ChannelSinkMultiplexing::Buffer {
                    outer_seq_size,
                    inner_seqs: inner_seqs - 1,
                };
            },
        }

        Ok(())
    }

    fn advance_var_sized(&mut self) -> Result<(), Error> {
        match self.multiplexing {
            ChannelSinkMultiplexing::Buffer {
                outer_seq_size,
                inner_seqs: 0,
            } => {
                self.multiplexing = ChannelSinkMultiplexing::Buffer {
                    outer_seq_size: outer_seq_size + 1,
                    inner_seqs: 0,
                };
            },

            _ => (),
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ChannelSinkMultiplexing {
    Channel,
    Buffer { outer_seq_size: usize, inner_seqs: usize },
}

#[derive(Debug, Clone)]
pub struct BufferSink<B = Vec<u8>> {
    buffer: B,
    cursor: usize,
    current_routine: BufferSinkRoutine,
    parent_routines: Vec<BufferSinkRoutine>,
}

impl BufferSink {
    pub fn new() -> Self {
        Self::with_buffer(Vec::new())
    }
}

impl<B> BufferSink<B>
where
    B: AsRef<Vec<u8>> + AsMut<Vec<u8>>,
{
    pub fn with_buffer(buffer: B) -> Self {
        Self {
            buffer,
            cursor: 0,
            current_routine: BufferSinkRoutine::Resolved { seqs: 0 },
            parent_routines: Vec::new(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buffer.as_ref()[..]
    }

    pub fn clear(&mut self) {
        self.buffer.as_mut().clear();
        self.cursor = 0;
    }

    fn push_resolved(&mut self, len: usize) -> Result<(), Error> {
        self.send_usize(len)?;

        self.current_routine = match self.current_routine {
            BufferSinkRoutine::Resolved { seqs } => {
                BufferSinkRoutine::Resolved { seqs: seqs + 1 }
            },
            BufferSinkRoutine::Resolving { .. } => {
                self.parent_routines.push(self.current_routine);
                BufferSinkRoutine::Resolved { seqs: 1 }
            },
        };

        Ok(())
    }

    fn push_resolving(&mut self) -> Result<(), Error> {
        if !matches!(
            self.current_routine,
            BufferSinkRoutine::Resolved { seqs: 0 }
        ) {
            self.parent_routines.push(self.current_routine);
        }
        self.current_routine =
            BufferSinkRoutine::Resolving { cursor: self.cursor, seq_size: 0 };
        self.send_usize(0)?;
        Ok(())
    }

    fn push(&mut self, size: Option<usize>) -> Result<(), Error> {
        match size {
            Some(len) => self.push_resolved(len),
            None => self.push_resolving(),
        }
    }

    fn pop(&mut self) -> Result<(), Error> {
        match self.current_routine {
            BufferSinkRoutine::Resolved { seqs: 1 } => {
                self.current_routine = match self.parent_routines.pop() {
                    Some(routine) => routine,
                    None => BufferSinkRoutine::Resolved { seqs: 0 },
                };
            },

            BufferSinkRoutine::Resolved { seqs } => {
                self.current_routine = BufferSinkRoutine::Resolved {
                    seqs: seqs.saturating_sub(1),
                };
            },

            BufferSinkRoutine::Resolving { cursor, seq_size } => {
                self.current_routine = match self.parent_routines.pop() {
                    Some(routine) => routine,
                    None => BufferSinkRoutine::Resolved { seqs: 0 },
                };
                let previous_cursor = self.cursor;
                self.cursor = cursor;
                self.send_usize(seq_size)?;
                self.cursor = previous_cursor;
            },
        }

        Ok(())
    }

    fn inc_size(&mut self) {
        if let BufferSinkRoutine::Resolving { cursor, seq_size } =
            self.current_routine
        {
            self.current_routine =
                BufferSinkRoutine::Resolving { cursor, seq_size: seq_size + 1 };
        }
    }
}

impl<B> SerializationSink for BufferSink<B>
where
    B: AsRef<Vec<u8>> + AsMut<Vec<u8>>,
{
    fn send_raw_data(&mut self, data: &[u8]) -> Result<(), Error> {
        let mid = data.len().min(self.buffer.as_ref().len() - self.cursor);
        let (overriding, extending) = data.split_at(mid);
        self.buffer.as_mut()[self.cursor .. self.cursor + mid]
            .copy_from_slice(&overriding);
        if extending.is_empty() {
            self.cursor += mid;
        } else {
            self.buffer.as_mut().extend_from_slice(extending);
            self.cursor = self.buffer.as_ref().len();
        }
        Ok(())
    }

    fn start_var_sized(&mut self, size: Option<usize>) -> Result<(), Error> {
        self.push(size)
    }

    fn end_var_sized(&mut self) -> Result<(), Error> {
        self.pop()
    }

    fn advance_var_sized(&mut self) -> Result<(), Error> {
        self.inc_size();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BufferSinkRoutine {
    Resolved { seqs: usize },
    Resolving { cursor: usize, seq_size: usize },
}

#[derive(Debug)]
pub struct Serializer<S> {
    sink: S,
}

impl<S> Serializer<S>
where
    S: SerializationSink,
{
    pub fn new(sink: S) -> Self {
        Self { sink }
    }
}

impl<'a, S> serde::ser::Serializer for &'a mut Serializer<S>
where
    S: SerializationSink,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.sink.send_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.sink.send_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.sink.send_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.sink.send_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.sink.send_i64(v)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.sink.send_i128(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.sink.send_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.sink.send_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.sink.send_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.sink.send_u64(v)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.sink.send_u128(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.sink.send_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.sink.send_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.sink.send_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.sink.send_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.sink.send_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.sink.send_u8(0)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.sink.send_u8(1)?;
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant_index.serialize(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        variant_index.serialize(&mut *self)?;
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        self.sink.start_var_sized(len)?;
        Ok(self)
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.sink.send_u32(variant_index)?;
        Ok(self)
    }

    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error> {
        self.sink.start_var_sized(len)?;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.sink.send_u32(variant_index)?;
        Ok(self)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, S> serde::ser::SerializeSeq for &'a mut Serializer<S>
where
    S: SerializationSink,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.sink.advance_var_sized()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.sink.end_var_sized()?;
        Ok(())
    }
}

impl<'a, S> serde::ser::SerializeMap for &'a mut Serializer<S>
where
    S: SerializationSink,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.sink.advance_var_sized()?;
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.sink.end_var_sized()?;
        Ok(())
    }
}

impl<'a, S> serde::ser::SerializeTuple for &'a mut Serializer<S>
where
    S: SerializationSink,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S> serde::ser::SerializeTupleStruct for &'a mut Serializer<S>
where
    S: SerializationSink,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S> serde::ser::SerializeTupleVariant for &'a mut Serializer<S>
where
    S: SerializationSink,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S> serde::ser::SerializeStruct for &'a mut Serializer<S>
where
    S: SerializationSink,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn skip_field(&mut self, _key: &'static str) -> Result<(), Self::Error> {
        Err(Error::SkipNotAllowed)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S> serde::ser::SerializeStructVariant for &'a mut Serializer<S>
where
    S: SerializationSink,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn skip_field(&mut self, _key: &'static str) -> Result<(), Self::Error> {
        Err(Error::SkipNotAllowed)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
