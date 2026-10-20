#![forbid(unsafe_code)]

//! Canon CR3 and ISOBMFF container parsing modules.

pub mod box_reader;
pub mod cr3_parser;

pub use box_reader::{
    find_next_box_with_tag, read_next_box, BoxHeader, BoxType, Uuid, UUID_MOOV, UUID_PRVW,
};
pub use cr3_parser::{
    extract_cr3_orientation, extract_cr3_orientation as get_orientation, extract_cr3_preview_data,
    extract_cr3_preview_data as get_preview_data, extract_cr3_preview_data_into, JpegBoxOffsets,
    PRVW_JPEG_OFFSETS, THMB_JPEG_OFFSETS,
};
