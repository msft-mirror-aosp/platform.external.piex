#![forbid(unsafe_code)]

//! Stream abstractions and byte reading helper utilities.

use std::io::{Read, Seek, SeekFrom};

use crate::types::{PiexError, Rational};

/// Alias for stream operation errors.
pub type StreamError = PiexError;

/// Endianness byte order specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Endian {
    /// Little-endian byte order.
    Little,
    /// Big-endian byte order.
    Big,
}

impl Endian {
    /// Determines byte order from TIFF header prefix bytes ("II" for Little, "MM" for Big).
    pub fn from_tiff_prefix(source: &[u8]) -> Option<Self> {
        if source.len() < 2 {
            return None;
        }
        match &source[0..2] {
            b"II" => Some(Endian::Little),
            b"MM" => Some(Endian::Big),
            _ => None,
        }
    }
}

/// Abstract interface for reading raw bytes from a stream.
pub trait StreamInterface {
    /// Reads data from `offset` into `data`.
    fn get_data(&self, offset: usize, data: &mut [u8]) -> Result<(), PiexError>;

    /// Returns the total length of the stream in bytes.
    fn len(&self) -> usize;

    /// Returns `true` if the stream contains no bytes.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Stream implementation backed by an in-memory byte slice.
#[derive(Debug, Clone, Copy)]
pub struct ByteSliceStream<'a> {
    data: &'a [u8],
}

impl<'a> ByteSliceStream<'a> {
    /// Creates a new stream from a byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Creates a sub-stream referencing a slice from `offset` with `len`.
    pub fn sub_stream(&self, offset: usize, len: usize) -> Result<Self, PiexError> {
        if offset > self.data.len() {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size: len,
                total_len: self.data.len(),
            });
        }
        if len == 0 {
            return Ok(Self::new(&[]));
        }
        let end = offset.checked_add(len).ok_or_else(|| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: len,
            total_len: self.data.len(),
        })?;
        if end > self.data.len() {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size: len,
                total_len: self.data.len(),
            });
        }
        Ok(Self::new(&self.data[offset..end]))
    }
}

impl StreamInterface for ByteSliceStream<'_> {
    fn get_data(&self, offset: usize, data: &mut [u8]) -> Result<(), PiexError> {
        let requested_size = data.len();
        if offset > self.data.len() {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size,
                total_len: self.data.len(),
            });
        }
        if requested_size == 0 {
            return Ok(());
        }
        let end = offset.checked_add(requested_size).ok_or_else(|| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size,
            total_len: self.data.len(),
        })?;
        if end > self.data.len() {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size,
                total_len: self.data.len(),
            });
        }
        data.copy_from_slice(&self.data[offset..end]);
        Ok(())
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

/// Stream implementation wrapping any standard `std::io::Read + std::io::Seek` reader.
///
/// Uses `std::sync::Mutex` for thread-safe interior mutability, ensuring multi-threaded safety
/// when accessed concurrently across threads or through FFI.
pub struct StreamAdapter<R: Read + Seek> {
    reader: std::sync::Mutex<R>,
    total_len: usize,
}

impl<R: Read + Seek> StreamAdapter<R> {
    /// Creates a new `StreamAdapter` wrapping `reader`.
    ///
    /// Determines total stream size by seeking to the end and restoring original position.
    pub fn new(mut reader: R) -> Result<Self, PiexError> {
        let current_pos = reader.stream_position().map_err(PiexError::from)?;
        let end_pos = reader.seek(SeekFrom::End(0)).map_err(PiexError::from)?;
        reader.seek(SeekFrom::Start(current_pos)).map_err(PiexError::from)?;
        let total_len = usize::try_from(end_pos).map_err(|_| {
            PiexError::GenericFail(
                "Stream length exceeds system pointer width capacity".to_string(),
            )
        })?;
        Ok(Self { reader: std::sync::Mutex::new(reader), total_len })
    }
}

impl<R: Read + Seek> StreamInterface for StreamAdapter<R> {
    fn get_data(&self, offset: usize, data: &mut [u8]) -> Result<(), PiexError> {
        let requested_size = data.len();
        if offset > self.total_len {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size,
                total_len: self.total_len,
            });
        }
        if requested_size == 0 {
            return Ok(());
        }
        let end = offset.checked_add(requested_size).ok_or_else(|| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size,
            total_len: self.total_len,
        })?;
        if end > self.total_len {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size,
                total_len: self.total_len,
            });
        }
        let mut reader = self
            .reader
            .lock()
            .map_err(|_| PiexError::GenericFail("Mutex poisoned during stream read".to_string()))?;
        reader.seek(SeekFrom::Start(offset as u64)).map_err(PiexError::from)?;
        reader.read_exact(data).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                PiexError::StreamReadFailed {
                    offset,
                    expected: requested_size,
                    available: self.total_len.saturating_sub(offset),
                }
            } else {
                PiexError::from(e)
            }
        })?;
        Ok(())
    }

    fn len(&self) -> usize {
        self.total_len
    }
}

/// Type alias for thread-safe stream adapter.
pub type SyncStreamAdapter<R> = StreamAdapter<R>;

/// Helper trait extending byte slices with endian-aware binary decoding methods.
pub trait EndianReader {
    /// Reads a `u8` value from `offset`.
    fn read_u8(&self, offset: usize) -> Result<u8, PiexError>;

    /// Reads a `u16` value from `offset` with specified endianness.
    fn read_u16(&self, offset: usize, endian: Endian) -> Result<u16, PiexError>;

    /// Reads an `i16` value from `offset` with specified endianness.
    fn read_i16(&self, offset: usize, endian: Endian) -> Result<i16, PiexError>;

    /// Reads a `u32` value from `offset` with specified endianness.
    fn read_u32(&self, offset: usize, endian: Endian) -> Result<u32, PiexError>;

    /// Reads an `i32` value from `offset` with specified endianness.
    fn read_i32(&self, offset: usize, endian: Endian) -> Result<i32, PiexError>;

    /// Reads a `Rational` fraction (two 32-bit uints: numerator, denominator) from `offset`.
    fn read_rational(&self, offset: usize, endian: Endian) -> Result<Rational, PiexError>;

    /// Returns a slice of `length` bytes from `offset`.
    fn read_bytes(&self, offset: usize, length: usize) -> Result<&[u8], PiexError>;

    /// Reads a string of up to `length` bytes from `offset`, terminating at the first NUL byte.
    fn read_string(&self, offset: usize, length: usize) -> Result<String, PiexError>;
}

impl EndianReader for [u8] {
    fn read_u8(&self, offset: usize) -> Result<u8, PiexError> {
        self.get(offset).copied().ok_or_else(|| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: 1,
            total_len: self.len(),
        })
    }

    fn read_u16(&self, offset: usize, endian: Endian) -> Result<u16, PiexError> {
        let bytes = self.read_bytes(offset, 2)?;
        let arr: [u8; 2] = bytes.try_into().map_err(|_| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: 2,
            total_len: self.len(),
        })?;
        Ok(match endian {
            Endian::Little => u16::from_le_bytes(arr),
            Endian::Big => u16::from_be_bytes(arr),
        })
    }

    fn read_i16(&self, offset: usize, endian: Endian) -> Result<i16, PiexError> {
        let bytes = self.read_bytes(offset, 2)?;
        let arr: [u8; 2] = bytes.try_into().map_err(|_| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: 2,
            total_len: self.len(),
        })?;
        Ok(match endian {
            Endian::Little => i16::from_le_bytes(arr),
            Endian::Big => i16::from_be_bytes(arr),
        })
    }

    fn read_u32(&self, offset: usize, endian: Endian) -> Result<u32, PiexError> {
        let bytes = self.read_bytes(offset, 4)?;
        let arr: [u8; 4] = bytes.try_into().map_err(|_| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: 4,
            total_len: self.len(),
        })?;
        Ok(match endian {
            Endian::Little => u32::from_le_bytes(arr),
            Endian::Big => u32::from_be_bytes(arr),
        })
    }

    fn read_i32(&self, offset: usize, endian: Endian) -> Result<i32, PiexError> {
        let bytes = self.read_bytes(offset, 4)?;
        let arr: [u8; 4] = bytes.try_into().map_err(|_| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: 4,
            total_len: self.len(),
        })?;
        Ok(match endian {
            Endian::Little => i32::from_le_bytes(arr),
            Endian::Big => i32::from_be_bytes(arr),
        })
    }

    fn read_rational(&self, offset: usize, endian: Endian) -> Result<Rational, PiexError> {
        let numerator = self.read_u32(offset, endian)?;
        let offset_den = offset.checked_add(4).ok_or_else(|| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: 8,
            total_len: self.len(),
        })?;
        let denominator = self.read_u32(offset_den, endian)?;
        Ok(Rational::new(numerator, denominator))
    }

    fn read_bytes(&self, offset: usize, length: usize) -> Result<&[u8], PiexError> {
        if offset > self.len() {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size: length,
                total_len: self.len(),
            });
        }
        if length == 0 {
            return Ok(&[]);
        }
        let end = offset.checked_add(length).ok_or_else(|| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: length,
            total_len: self.len(),
        })?;
        self.get(offset..end).ok_or_else(|| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: length,
            total_len: self.len(),
        })
    }

    fn read_string(&self, offset: usize, length: usize) -> Result<String, PiexError> {
        let bytes = self.read_bytes(offset, length)?;
        let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Ok(String::from_utf8_lossy(&bytes[..null_pos]).into_owned())
    }
}
