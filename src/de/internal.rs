use serde::{de::IntoDeserializer, Deserialize};
use smallvec::SmallVec;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    sync::mpsc,
};

use super::Error;

pub trait DeserializationSource {
    fn recv_raw_data(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    fn recv_u64(&mut self) -> Result<u64, Error> {
        let mut buf = [0; 8];
        self.recv_raw_data(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn recv_i64(&mut self) -> Result<i64, Error> {
        let mut buf = [0; 8];
        self.recv_raw_data(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    fn recv_usize(&mut self) -> Result<usize, Error> {
        let bits = self.recv_u64()?;
        usize::try_from(bits).map_err(|_| Error::ExcessiveSize(bits))
    }

    fn recv_isize(&mut self) -> Result<isize, Error> {
        let bits = self.recv_i64()?;
        isize::try_from(bits).map_err(|_| Error::ExcessiveSizeDiff(bits))
    }
}

pub type ChannelBytes = SmallVec<[u8; 16]>;

#[derive(Debug)]
pub struct ChannelBackend<R> {
    device: R,
    hard_eof: bool,
    response_sender: mpsc::Sender<ChannelBytes>,
    request_receiver: mpsc::Receiver<usize>,
}

impl<R> ChannelBackend<R>
where
    R: AsyncRead + Unpin,
{
    pub fn new(
        device: R,
        response_sender: mpsc::Sender<ChannelBytes>,
        request_receiver: mpsc::Receiver<usize>,
    ) -> Self {
        Self { device, hard_eof: false, response_sender, request_receiver }
    }

    pub fn set_hard_eof(&mut self, on: bool) {
        self.hard_eof = on;
    }

    pub async fn run(mut self) -> Result<(), Error> {
        while let Some(size) = self.request_receiver.recv().await {
            let mut bytes = ChannelBytes::from_elem(0, size);
            let mut cursor = &mut bytes[..];
            while !cursor.is_empty() {
                let count = self.device.read(&mut cursor).await?;
                if self.hard_eof && count == 0 {
                    Err(Error::PrematureEof)?
                }
                cursor = &mut cursor[count ..];
            }
            self.response_sender
                .send(bytes)
                .await
                .map_err(|_| Error::Disconnected)?;
        }
        if self.hard_eof {
            let mut buf = [0];
            if self.device.read(&mut buf).await? != 0 {
                Err(Error::ExpectedEof(buf[0]))?
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChannelSource {
    request_sender: mpsc::Sender<usize>,
    response_receiver: mpsc::Receiver<ChannelBytes>,
}

impl ChannelSource {
    pub fn new(
        request_sender: mpsc::Sender<usize>,
        response_receiver: mpsc::Receiver<ChannelBytes>,
    ) -> Self {
        Self { request_sender, response_receiver }
    }
}

impl DeserializationSource for ChannelSource {
    fn recv_raw_data(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.request_sender
            .blocking_send(buf.len())
            .map_err(|_| Error::PrematureEof)?;
        let vector = self
            .response_receiver
            .blocking_recv()
            .ok_or(Error::PrematureEof)?;
        buf.copy_from_slice(&vector[..]);
        Ok(())
    }
}

#[derive(Debug)]
pub struct BufferSource<B = Vec<u8>> {
    buffer: B,
    cursor: usize,
}

impl<B> BufferSource<B>
where
    B: AsRef<[u8]>,
{
    pub fn new(buffer: B) -> Self {
        Self { buffer, cursor: 0 }
    }

    pub fn ensure_eof(&self) -> Result<(), Error> {
        match self.buffer.as_ref().get(self.cursor) {
            None => Ok(()),
            Some(found) => Err(Error::ExpectedEof(*found)),
        }
    }
}

impl<B> DeserializationSource for BufferSource<B>
where
    B: AsRef<[u8]>,
{
    fn recv_raw_data(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let new_cursor = self.cursor + buf.len();
        let source = self
            .buffer
            .as_ref()
            .get(self.cursor .. new_cursor)
            .ok_or(Error::PrematureEof)?;
        buf.copy_from_slice(source);
        self.cursor = new_cursor;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Deserializer<S> {
    source: S,
}

impl<S> Deserializer<S>
where
    S: DeserializationSource,
{
    pub fn new(source: S) -> Self {
        Self { source }
    }

    pub fn source(&self) -> &S {
        &self.source
    }
}

impl<'a, 'de, S> serde::de::Deserializer<'de> for &'a mut Deserializer<S>
where
    S: DeserializationSource,
{
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::UnsupportedAny)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_bool(buf[0] != 0)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_i8(i8::from_le_bytes(buf))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 2];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_i16(i16::from_le_bytes(buf))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 4];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_i32(i32::from_le_bytes(buf))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 8];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_i64(i64::from_le_bytes(buf))
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 16];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_i128(i128::from_le_bytes(buf))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_u8(u8::from_le_bytes(buf))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 2];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_u16(u16::from_le_bytes(buf))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 4];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_u32(u32::from_le_bytes(buf))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 8];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_u64(u64::from_le_bytes(buf))
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 16];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_u128(u128::from_le_bytes(buf))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 4];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_f32(f32::from_le_bytes(buf))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut buf = [0; 8];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_f64(f64::from_le_bytes(buf))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let codepoint = u32::deserialize(self)?;
        let ch = char::try_from(codepoint)
            .map_err(|_| Error::InvalidCodePoint(codepoint))?;
        visitor.visit_char(ch)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let string = String::deserialize(self)?;
        visitor.visit_str(&string[..])
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let buf = Vec::<u8>::deserialize(self)?;
        let string = String::from_utf8(buf).map_err(Error::Utf8)?;
        visitor.visit_string(string)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let buf = Vec::<u8>::deserialize(self)?;
        visitor.visit_bytes(&buf[..])
    }

    fn deserialize_byte_buf<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let len = self.source.recv_usize()?;
        let mut buf = vec![0; len];
        self.source.recv_raw_data(&mut buf)?;
        visitor.visit_byte_buf(buf)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let tag = u8::deserialize(&mut *self)?;
        if tag == 0 {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let len = self.source.recv_usize()?;
        visitor.visit_seq(ProductAccess { remaining: len, deserializer: self })
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(ProductAccess { remaining: len, deserializer: self })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(ProductAccess { remaining: len, deserializer: self })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let len = self.source.recv_usize()?;
        visitor.visit_map(ProductAccess { remaining: len, deserializer: self })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(ProductAccess {
            remaining: fields.len(),
            deserializer: self,
        })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(SumAccess { deserializer: self })
    }

    fn deserialize_identifier<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_ignored_any<V>(
        self,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::UnsupportedAny)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
struct ProductAccess<'a, S> {
    remaining: usize,
    deserializer: &'a mut Deserializer<S>,
}

impl<'a, 'de, S> serde::de::SeqAccess<'de> for ProductAccess<'a, S>
where
    S: DeserializationSource,
{
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let Some(adjusted_remaining) = self.remaining.checked_sub(1) else {
            return Ok(None);
        };

        let element = seed.deserialize(&mut *self.deserializer)?;
        self.remaining = adjusted_remaining;
        Ok(Some(element))
    }
}

impl<'a, 'de, S> serde::de::MapAccess<'de> for ProductAccess<'a, S>
where
    S: DeserializationSource,
{
    type Error = Error;

    fn next_key_seed<K>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let Some(adjusted_remaining) = self.remaining.checked_sub(1) else {
            return Ok(None);
        };

        let element = seed.deserialize(&mut *self.deserializer)?;
        self.remaining = adjusted_remaining;
        Ok(Some(element))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.deserializer)
    }
}

#[derive(Debug)]
struct SumAccess<'a, S> {
    deserializer: &'a mut Deserializer<S>,
}

impl<'a, 'de, S> serde::de::EnumAccess<'de> for SumAccess<'a, S>
where
    S: DeserializationSource,
{
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let tag: u32 = u32::deserialize(&mut *self.deserializer)?;
        let result: Result<_, Error> =
            seed.deserialize(tag.into_deserializer());
        let val = result?;
        Ok((val, self))
    }
}

impl<'a, 'de, S> serde::de::VariantAccess<'de> for SumAccess<'a, S>
where
    S: DeserializationSource,
{
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.deserializer)
    }

    fn tuple_variant<V>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(ProductAccess {
            remaining: len,
            deserializer: &mut *self.deserializer,
        })
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(ProductAccess {
            remaining: fields.len(),
            deserializer: &mut *self.deserializer,
        })
    }
}
