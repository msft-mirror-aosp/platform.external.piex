//! Layer 1 FFI C-ABI entry points for PIEX.
//!
//! Provides `extern "C"` exported functions with panic isolation boundaries,
//! converting between C types/handles and pure Rust domain types.

use core::ffi::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::c_types::{
    piex_error_t, piex_preview_image_data_t, piex_raw_image_type_t, piex_stream_t,
};
use crate::cr3;
use crate::extractor;
use crate::ffi_ext::{copy_str_to_c_buf, CStreamAdapter};
use crate::format_recognizer;
use crate::stream::StreamInterface;
use crate::types::RawImageType;

/// Returns the number of header bytes required to identify whether a stream is a RAW image.
///
/// # Safety
///
/// This function is safe to call from any C context as it takes no parameters, dereferences
/// no pointers, and accesses no shared mutable state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_bytes_required_for_is_raw() -> usize {
    catch_unwind(AssertUnwindSafe(|| extractor::bytes_required_for_is_raw())).unwrap_or(0)
}

/// Checks if the provided stream contains a recognized RAW image format header.
///
/// # Safety
///
/// The caller must ensure that `stream`, if non-null (`Some`), contains a valid `read_fn`
/// callback pointer and `user_data` context pointer that remain valid for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_is_raw(stream: Option<&piex_stream_t>) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(stream) = stream else {
            return false;
        };
        let adapter = CStreamAdapter::new(stream, usize::MAX);
        let req_bytes = extractor::bytes_required_for_is_raw();
        let mut header = vec![0u8; req_bytes];
        if adapter.get_data(0, &mut header).is_err() {
            return false;
        }
        format_recognizer::recognize_raw_image_type_lite(&header) != RawImageType::NonRaw
    }))
    .unwrap_or(false)
}

/// Extracts preview image metadata and raw image type from the given stream.
///
/// # Safety
///
/// The caller must ensure that:
/// - `stream`, if non-null (`Some`), contains a valid `read_fn` callback and `user_data` context.
/// - `out_data` and `out_type`, if non-null (`Some`), point to valid, dereferenceable memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_get_preview_image_data(
    stream: Option<&piex_stream_t>,
    out_data: Option<&mut piex_preview_image_data_t>,
    out_type: Option<&mut piex_raw_image_type_t>,
) -> piex_error_t {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(stream) = stream else {
            return piex_error_t::PIEX_INVALID_ARGUMENT;
        };
        if out_data.is_none() && out_type.is_none() {
            return piex_error_t::PIEX_INVALID_ARGUMENT;
        }

        let mut out_type = out_type;
        let adapter = CStreamAdapter::new(stream, usize::MAX);
        let req_bytes = extractor::bytes_required_for_is_raw();
        let mut header = vec![0u8; req_bytes];
        if adapter.get_data(0, &mut header).is_ok() {
            let recognized = format_recognizer::recognize_raw_image_type_lite(&header);
            if let Some(ref mut out_t) = out_type {
                **out_t = piex_raw_image_type_t::from(recognized);
            }
        }

        match extractor::get_preview_image_data(&adapter) {
            Ok((data, raw_type)) => {
                if let Some(out_d) = out_data {
                    *out_d = piex_preview_image_data_t::from(data);
                }
                if let Some(out_t) = out_type {
                    *out_t = piex_raw_image_type_t::from(raw_type);
                }
                piex_error_t::PIEX_OK
            }
            Err(err) => piex_error_t::from(err),
        }
    }))
    .unwrap_or(piex_error_t::PIEX_PANIC)
}

/// Retrieves DNG image dimensions and CFA pattern dimensions from the stream.
///
/// # Safety
///
/// The caller must ensure that:
/// - `stream`, `out_width`, `out_height`, and `out_cfa_dim`, if non-null, point to valid, dereferenceable memory locations.
/// - `out_width` and `out_height` point to distinct, non-aliasing memory locations per the C API caller contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_get_dng_information(
    stream: Option<&piex_stream_t>,
    out_width: Option<&mut u32>,
    out_height: Option<&mut u32>,
    out_cfa_dim: Option<&mut [u32; 2]>,
) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(stream) = stream else {
            return false;
        };
        let adapter = CStreamAdapter::new(stream, usize::MAX);
        let mut width = 0u32;
        let mut height = 0u32;
        let mut cfa = [0u32; 2];
        let status =
            extractor::get_dng_information_out(&adapter, &mut width, &mut height, &mut cfa);
        if let Some(w) = out_width {
            *w = width;
        }
        if let Some(h) = out_height {
            *h = height;
        }
        if let Some(cfa_out) = out_cfa_dim {
            *cfa_out = cfa;
        }
        status
    }))
    .unwrap_or(false)
}

/// Retrieves the EXIF orientation value from the stream.
///
/// # Safety
///
/// The caller must ensure that `stream` and `out_orientation`, if non-null,
/// point to valid, dereferenceable memory locations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_get_orientation(
    stream: Option<&piex_stream_t>,
    out_orientation: Option<&mut u32>,
) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(stream) = stream else {
            return false;
        };
        let Some(out_orient) = out_orientation else {
            return false;
        };
        let adapter = CStreamAdapter::new(stream, usize::MAX);
        match extractor::get_orientation(&adapter) {
            Ok(orient) => {
                *out_orient = orient;
                true
            }
            Err(_) => false,
        }
    }))
    .unwrap_or(false)
}

/// Copies supported RAW file extension names into a destination C character buffer.
///
/// # Safety
///
/// The caller must ensure that:
/// - If `out_buffer` is non-null and `buffer_len > 0`, `out_buffer` points to a valid,
///   writable memory buffer of at least `buffer_len` bytes.
/// - `out_written`, if non-null (`Some`), points to a valid, dereferenceable memory location.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_supported_extensions(
    out_buffer: *mut c_char,
    buffer_len: usize,
    out_written: Option<&mut usize>,
) -> piex_error_t {
    catch_unwind(AssertUnwindSafe(|| {
        let exts = extractor::supported_extensions();
        let joined = exts.join(",");

        if let Some(written) = out_written {
            *written = joined.len();
        }

        if !out_buffer.is_null() && buffer_len > 0 {
            // SAFETY: `out_buffer` is non-null and `buffer_len` > 0; caller guarantees memory is valid for writing `buffer_len` bytes.
            let dst_slice = unsafe { core::slice::from_raw_parts_mut(out_buffer, buffer_len) };
            copy_str_to_c_buf(&joined, dst_slice);
        }

        piex_error_t::PIEX_OK
    }))
    .unwrap_or(piex_error_t::PIEX_PANIC)
}

/// Retrieves orientation specifically for Canon CR3 image streams.
///
/// # Safety
///
/// The caller must ensure that `stream` and `out_orientation`, if non-null,
/// point to valid, dereferenceable memory locations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_cr3_get_orientation(
    stream: Option<&piex_stream_t>,
    out_orientation: Option<&mut u32>,
) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(stream) = stream else {
            return false;
        };
        let Some(out_orient) = out_orientation else {
            return false;
        };
        let adapter = CStreamAdapter::new(stream, usize::MAX);
        if let Ok(orient) = cr3::extract_cr3_orientation(&adapter) {
            *out_orient = orient;
            true
        } else {
            false
        }
    }))
    .unwrap_or(false)
}

/// Extracts preview image data specifically for Canon CR3 image streams.
///
/// # Safety
///
/// The caller must ensure that `stream` and `out_data`, if non-null,
/// point to valid, dereferenceable memory locations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_cr3_get_preview_data(
    stream: Option<&piex_stream_t>,
    out_data: Option<&mut piex_preview_image_data_t>,
) -> piex_error_t {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(stream) = stream else {
            return piex_error_t::PIEX_INVALID_ARGUMENT;
        };
        let Some(out_d) = out_data else {
            return piex_error_t::PIEX_INVALID_ARGUMENT;
        };
        let adapter = CStreamAdapter::new(stream, usize::MAX);
        match cr3::extract_cr3_preview_data(&adapter) {
            Ok(preview_data) => {
                *out_d = piex_preview_image_data_t::from(preview_data);
                piex_error_t::PIEX_OK
            }
            Err(err) => piex_error_t::from(err),
        }
    }))
    .unwrap_or(piex_error_t::PIEX_PANIC)
}

/// Fast light-weight check to determine whether an in-memory byte buffer contains RAW image data.
///
/// # Safety
///
/// If `data` is non-null and `len > 0`, the caller must ensure `data` points to a readable
/// memory buffer containing at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_is_raw_lite(data: *const u8, len: usize) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        if data.is_null() || len == 0 {
            return false;
        }
        // SAFETY: `data` is verified non-null and `len` > 0; caller guarantees `data` points to `len` readable bytes.
        let slice = unsafe { core::slice::from_raw_parts(data, len) };
        format_recognizer::is_raw_lite(slice)
    }))
    .unwrap_or(false)
}

/// Fast light-weight identification of the specific RAW image type from an in-memory byte buffer.
///
/// # Safety
///
/// If `data` is non-null and `len > 0`, the caller must ensure `data` points to a readable
/// memory buffer containing at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn piex_recognize_raw_image_type_lite(
    data: *const u8,
    len: usize,
) -> piex_raw_image_type_t {
    catch_unwind(AssertUnwindSafe(|| {
        if data.is_null() || len == 0 {
            return piex_raw_image_type_t::PIEX_RAW_TYPE_NON_RAW;
        }
        // SAFETY: `data` is verified non-null and `len` > 0; caller guarantees `data` points to `len` readable bytes.
        let slice = unsafe { core::slice::from_raw_parts(data, len) };
        let raw_type = format_recognizer::recognize_raw_image_type_lite(slice);
        piex_raw_image_type_t::from(raw_type)
    }))
    .unwrap_or(piex_raw_image_type_t::PIEX_RAW_TYPE_NON_RAW)
}
