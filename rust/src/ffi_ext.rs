//! Safe FFI accessor bridge and stream adapter implementations.

use crate::c_types::piex_stream_t;
use crate::stream::StreamInterface;
use crate::types::PiexError;
use core::ffi::c_char;

/// Safe adapter wrapping a C `piex_stream_t` reference and total stream length to implement
/// `StreamInterface`.
#[derive(Debug, Clone, Copy)]
pub struct CStreamAdapter<'a> {
    stream: &'a piex_stream_t,
    length: usize,
}

impl<'a> CStreamAdapter<'a> {
    /// Creates a new `CStreamAdapter` wrapping `stream` with a known total byte `length`.
    pub fn new(stream: &'a piex_stream_t, length: usize) -> Self {
        Self { stream, length }
    }
}

impl StreamInterface for CStreamAdapter<'_> {
    fn get_data(&self, offset: usize, data: &mut [u8]) -> Result<(), PiexError> {
        let req_len = data.len();
        if offset > self.length {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size: req_len,
                total_len: self.length,
            });
        }
        if req_len == 0 {
            return Ok(());
        }

        let end = offset.checked_add(req_len).ok_or_else(|| PiexError::OutOfBounds {
            requested_offset: offset,
            requested_size: req_len,
            total_len: self.length,
        })?;

        if end > self.length {
            return Err(PiexError::OutOfBounds {
                requested_offset: offset,
                requested_size: req_len,
                total_len: self.length,
            });
        }

        let Some(read_fn) = self.stream.read_fn else {
            return Err(PiexError::GenericFail(
                "Read function callback in piex_stream_t is null".to_string(),
            ));
        };

        // SAFETY: `read_fn` is a valid C callback function pointer provided by the caller,
        // `self.stream.user_data` is passed through unchanged as an opaque pointer, and
        // `data.as_mut_ptr()` points to a valid, mutable memory buffer of at least
        // `data.len()` bytes for the duration of the call.
        let status = unsafe { read_fn(self.stream.user_data, offset, req_len, data.as_mut_ptr()) };

        if status == 0 {
            Ok(())
        } else {
            Err(PiexError::StreamReadFailed {
                offset,
                expected: req_len,
                available: self.length.saturating_sub(offset),
            })
        }
    }

    fn len(&self) -> usize {
        self.length
    }
}

/// Copies a Rust UTF-8 string into a destination C `c_char` buffer, ensuring NUL-termination.
pub fn copy_str_to_c_buf(src: &str, dst: &mut [c_char]) {
    copy_bytes_to_c_buf(src.as_bytes(), dst);
}

/// Copies raw bytes into a destination C `c_char` buffer, truncating if needed and
/// ensuring NUL-termination.
pub fn copy_bytes_to_c_buf(src_bytes: &[u8], dst: &mut [c_char]) {
    if dst.is_empty() {
        return;
    }
    let max_bytes = dst.len() - 1;
    let copy_len = src_bytes.len().min(max_bytes);

    src_bytes[..copy_len].iter().enumerate().for_each(|(i, &b)| {
        dst[i] = b as c_char;
    });

    dst[copy_len] = 0;
}

/// Converts a NUL-terminated C `c_char` buffer into an owned Rust `String`.
pub fn c_buf_to_string(src: &[c_char]) -> String {
    String::from_utf8_lossy(&c_buf_to_bytes(src)).into_owned()
}

/// Returns the bytes of a C `c_char` buffer up to (not including) the first NUL.
pub fn c_buf_to_bytes(src: &[c_char]) -> Vec<u8> {
    let nul_pos = src.iter().position(|&c| c == 0).unwrap_or(src.len());
    src[..nul_pos].iter().map(|&c| c as u8).collect()
}
