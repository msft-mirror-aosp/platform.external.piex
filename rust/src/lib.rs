//! PIEX (Preview Image EXtractor) pure Rust implementation.
//!
//! Provides types, errors, and stream abstractions for extracting preview images
//! and metadata from RAW image files, along with C-compatible FFI ABI layers.

pub mod c_types;
pub mod cr3;
pub mod cxx_bridge;
pub mod extractor;
pub mod ffi;
pub mod ffi_ext;
pub mod format_recognizer;
pub mod stream;
pub mod tiff;
pub mod types;

pub use c_types::{
    piex_color_space_t, piex_error_t, piex_gps_t, piex_image_format_t, piex_image_t,
    piex_preview_image_data_t, piex_rational_t, piex_raw_image_type_t, piex_read_fn_t,
    piex_stream_t,
};
pub use cr3::{extract_cr3_orientation, extract_cr3_preview_data, BoxHeader, BoxType};
pub use extractor::{
    bytes_required_for_is_raw, extract_arw, extract_cr2, extract_cr3, extract_dng, extract_nef,
    extract_orf, extract_pef, extract_raf, extract_rw2, extract_srw, get_dng_information,
    get_olympus_preview_image, get_orientation, get_preview_image_data, is_raw, raf_get_dimension,
    raf_get_orientation, supported_extensions,
};
pub use ffi::*;
pub use ffi_ext::{
    c_buf_to_bytes, c_buf_to_string, copy_bytes_to_c_buf, copy_str_to_c_buf, CStreamAdapter,
};
pub use format_recognizer::{
    get_number_of_bytes_for_is_of_type, get_number_of_bytes_for_is_raw_lite, is_of_type,
    is_raw_lite, recognize_raw_image_type_lite, MAX_HEADER_INSPECTION_SIZE,
};
pub use stream::{
    ByteSliceStream, Endian, EndianReader, StreamAdapter, StreamError, StreamInterface,
};
pub use tiff::{
    DirectoryEntry, EntryValue, TiffContent, TiffDirectory, TiffError, TiffParser, TiffType,
    THUMBNAIL_MAX_DIMENSION,
};
pub use types::{
    ColorSpace, DngInformation, GpsData, Image, ImageFormat, PiexError, PreviewImageData, Rational,
    RawImageType, SRational,
};
