#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;

pub use abstract_bits_derive::abstract_bits;
pub use arbitrary_int::*;
pub use bitvec;
use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;

mod error;
pub use error::{FromBytesError, ReadErrorCause, ToBytesError};

pub trait AbstractBits {
    const MIN_BITS: usize;
    const MAX_BITS: usize;
    /// To get the amount written use [`BitWriter::bits_written`]
    /// or [`BitWriter::bytes_written`]
    fn write_abstract_bits(&self, writer: &mut BitWriter) -> Result<(), ToBytesError>;
    /// To get the amount read use [`BitReader::bits_read`]
    /// or [`BitReader::bytes_read`]
    fn read_abstract_bits(reader: &mut BitReader) -> Result<Self, FromBytesError>
    where
        Self: Sized;

    fn to_abstract_bits(&self) -> Result<Vec<u8>, ToBytesError> {
        let needed_bytes = Self::MAX_BITS.div_ceil(8);
        let mut buffer = alloc::vec![0u8; needed_bytes];
        let mut writer = BitWriter::from(buffer.as_mut_slice());
        self.write_abstract_bits(&mut writer)?;
        let bytes = writer.bytes_written();
        buffer.truncate(bytes);
        Ok(buffer)
    }

    fn from_abstract_bits(bytes: &[u8]) -> Result<Self, FromBytesError>
    where
        Self: Sized,
    {
        let mut reader = BitReader::from(bytes);
        Self::read_abstract_bits(&mut reader)
    }
}

macro_rules! impl_abstract_bits_for_UInt {
    ($base_type:ty, $write_method:ident, $read_method: ident) => {
        impl<const N: usize> AbstractBits for arbitrary_int::UInt<$base_type, N> {
            const MIN_BITS: usize = Self::BITS;
            const MAX_BITS: usize = Self::BITS;

            fn write_abstract_bits(
                &self,
                writer: &mut BitWriter,
            ) -> Result<(), ToBytesError> {
                writer
                    .$write_method(Self::BITS, self.value())
                    .map_err(|cause| ToBytesError::BufferTooSmall {
                        ty: core::any::type_name::<Self>(),
                        cause,
                    })
            }

            fn read_abstract_bits(reader: &mut BitReader) -> Result<Self, FromBytesError>
            where
                Self: Sized,
            {
                use FromBytesError::ReadPrimitive;
                let value = reader.$read_method(Self::BITS).map_err(|cause| {
                    ReadPrimitive(ReadErrorCause::NotEnoughInput {
                        ty: core::any::type_name::<Self>(),
                        cause,
                    })
                })?;
                Ok(Self::new(value))
            }
        }
    };
}

impl_abstract_bits_for_UInt! {u8, write_u8, read_u8}
impl_abstract_bits_for_UInt! {u16, write_u16, read_u16}
impl_abstract_bits_for_UInt! {u32, write_u32, read_u32}
impl_abstract_bits_for_UInt! {u64, write_u64, read_u64}

macro_rules! impl_abstract_bits_for_core_int {
    ($type:ty, $write_method:ident, $read_method:ident, $bits:literal) => {
        impl AbstractBits for $type {
            const MIN_BITS: usize = core::mem::size_of::<Self>() * 8;
            const MAX_BITS: usize = core::mem::size_of::<Self>() * 8;

            fn write_abstract_bits(
                &self,
                writer: &mut BitWriter,
            ) -> Result<(), ToBytesError> {
                writer.$write_method($bits, *self).map_err(|cause| {
                    ToBytesError::BufferTooSmall {
                        ty: core::any::type_name::<Self>(),
                        cause,
                    }
                })
            }

            fn read_abstract_bits(reader: &mut BitReader) -> Result<Self, FromBytesError>
            where
                Self: Sized,
            {
                use FromBytesError::ReadPrimitive;
                reader.$read_method($bits).map_err(|cause| {
                    ReadPrimitive(ReadErrorCause::NotEnoughInput {
                        ty: core::any::type_name::<Self>(),
                        cause,
                    })
                })
            }
        }
    };
}

impl_abstract_bits_for_core_int! {u8, write_u8, read_u8, 8}
impl_abstract_bits_for_core_int! {u16, write_u16, read_u16, 16}
impl_abstract_bits_for_core_int! {u32, write_u32, read_u32, 32}
impl_abstract_bits_for_core_int! {u64, write_u64, read_u64, 64}

impl AbstractBits for bool {
    const MIN_BITS: usize = 1;
    const MAX_BITS: usize = 1;

    fn write_abstract_bits(&self, writer: &mut BitWriter) -> Result<(), ToBytesError> {
        writer
            .write_bit(*self)
            .map_err(|cause| ToBytesError::BufferTooSmall {
                ty: core::any::type_name::<Self>(),
                cause,
            })
    }

    fn read_abstract_bits(reader: &mut BitReader) -> Result<Self, FromBytesError>
    where
        Self: Sized,
    {
        use FromBytesError::ReadPrimitive;
        reader.read_bit().map_err(|cause| {
            ReadPrimitive(ReadErrorCause::NotEnoughInput {
                ty: core::any::type_name::<Self>(),
                cause,
            })
        })
    }
}

impl<const N: usize, T: AbstractBits + Sized> AbstractBits for [T; N] {
    const MIN_BITS: usize = T::MIN_BITS * N;
    const MAX_BITS: usize = T::MAX_BITS * N;

    fn write_abstract_bits(&self, writer: &mut BitWriter) -> Result<(), ToBytesError> {
        for element in self.iter() {
            element.write_abstract_bits(writer)?;
        }
        Ok(())
    }
    fn read_abstract_bits(reader: &mut BitReader) -> Result<Self, FromBytesError>
    where
        Self: Sized,
    {
        let mut res = Vec::new();
        for _ in 0..N {
            res.push(T::read_abstract_bits(reader)?);
        }

        res.try_into()
            .map_err(|_| unreachable!("for loop ensures vec length matches array's"))
    }
}

pub struct BitReader<'a> {
    pos: usize,
    buf: &'a BitSlice<u8, Lsb0>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error(
    "Need to read beyond end of provided buffer to read {n_bits}. \
    Buffer is missing {bits_needed} bits"
)]
pub struct UnexpectedEndOfBits {
    pub n_bits: usize,
    pub bits_needed: usize,
}

macro_rules! read_primitive {
    ($name:ident, $ty:ty) => {
        fn $name(&mut self, n_bits: usize) -> Result<$ty, UnexpectedEndOfBits> {
            let mut res = <$ty>::default().to_le_bytes();
            let res_bits = BitSlice::<_, Lsb0>::from_slice_mut(&mut res);
            if self.buf.len() < self.pos + n_bits {
                Err(UnexpectedEndOfBits {
                    n_bits,
                    bits_needed: self.pos + n_bits - self.buf.len(),
                })
            } else {
                res_bits[0..n_bits]
                    .copy_from_bitslice(&self.buf[self.pos..self.pos + n_bits]);
                self.pos += n_bits;
                Ok(<$ty>::from_le_bytes(res))
            }
        }
    };
}

impl BitReader<'_> {
    pub fn bits_read(&self) -> usize {
        self.pos
    }
    /// 12 bits read corresponds to 2 bytes read
    pub fn bytes_read(&self) -> usize {
        self.pos.div_ceil(8)
    }
    pub fn skip(&mut self, n_bits: usize) -> Result<(), UnexpectedEndOfBits> {
        if self.pos + n_bits > self.buf.len() {
            Err(UnexpectedEndOfBits {
                n_bits,
                bits_needed: (self.pos + n_bits + 1) - self.buf.len(),
            })
        } else {
            self.pos += n_bits;
            Ok(())
        }
    }
    fn read_bit(&mut self) -> Result<bool, UnexpectedEndOfBits> {
        let Some(res) = self.buf.get(self.pos) else {
            return Err(UnexpectedEndOfBits {
                n_bits: 1,
                bits_needed: 1,
            });
        };
        self.pos += 1;
        Ok(*res)
    }

    read_primitive! {read_u8, u8}
    read_primitive! {read_u16, u16}
    read_primitive! {read_u32, u32}
    read_primitive! {read_u64, u64}
}

impl<'a> From<&'a [u8]> for BitReader<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            pos: 0,
            buf: BitSlice::from_slice(bytes),
        }
    }
}

pub struct BitWriter<'a> {
    pos: usize,
    buf: &'a mut BitSlice<u8, Lsb0>,
}

impl core::fmt::Debug for BitWriter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BitWriter\n")?;
        f.write_fmt(format_args!("\tpos: {}\n", self.pos))?;
        f.write_fmt(format_args!("\tbuf: BitSlice of {} bits\n", self.buf.len()))
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error(
    "Buffer is too small to serialize `{n_bits}` into. \
    Buffer needs to be at least {bits_needed} bits extra"
)]
pub struct BufferTooSmall {
    pub n_bits: usize,
    pub bits_needed: usize,
}

macro_rules! write_primitive {
    ($name:ident, $ty:ty) => {
        fn $name(&mut self, n_bits: usize, val: $ty) -> Result<(), BufferTooSmall> {
            let val = val.to_le_bytes();
            let val = BitSlice::<_, Lsb0>::from_slice(&val);
            if self.pos + n_bits > self.buf.len() {
                Err(BufferTooSmall {
                    n_bits,
                    bits_needed: (self.pos + n_bits) - self.buf.len(),
                })
            } else {
                self.buf[self.pos..self.pos + n_bits].copy_from_bitslice(&val[..n_bits]);
                self.pos += n_bits;
                Ok(())
            }
        }
    };
}

impl BitWriter<'_> {
    pub fn bits_written(&self) -> usize {
        self.pos
    }
    /// 12 bits read corresponds to 2 bytes read
    pub fn bytes_written(&self) -> usize {
        self.pos.div_ceil(8)
    }
    pub fn skip(&mut self, n_bits: usize) -> Result<(), BufferTooSmall> {
        if self.pos + n_bits > self.buf.len() {
            Err(BufferTooSmall {
                n_bits,
                bits_needed: (self.pos + n_bits + 1) - self.buf.len(),
            })
        } else {
            self.pos += n_bits;
            Ok(())
        }
    }
    fn write_bit(&mut self, bit: bool) -> Result<(), BufferTooSmall> {
        if self.pos >= self.buf.len() {
            Err(BufferTooSmall {
                n_bits: 1,
                bits_needed: 1,
            })
        } else {
            self.buf.set(self.pos, bit);
            self.pos += 1;
            Ok(())
        }
    }

    write_primitive!(write_u8, u8);
    write_primitive!(write_u16, u16);
    write_primitive!(write_u32, u32);
    write_primitive!(write_u64, u64);
}

impl<'a> From<&'a mut [u8]> for BitWriter<'a> {
    fn from(buf: &'a mut [u8]) -> Self {
        Self {
            pos: 0,
            buf: BitSlice::from_slice_mut(buf),
        }
    }
}
