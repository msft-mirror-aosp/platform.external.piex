//! Top-level hermetic unit test binary for PIEX Rust implementation.
//!
//! All unit tests in this binary use synthetic, in-memory buffers and do not
//! depend on external camera RAW sample files.

#[path = "unit/stream_and_types.rs"]
mod stream_and_types;

#[path = "unit/format_recognizer.rs"]
mod format_recognizer;

#[path = "unit/tiff_directory.rs"]
mod tiff_directory;

#[path = "unit/tiff_parser.rs"]
mod tiff_parser;

#[path = "unit/cr3_box_reader.rs"]
mod cr3_box_reader;

#[path = "unit/cr3_parser.rs"]
mod cr3_parser;

#[path = "unit/extractors.rs"]
mod extractors;

#[path = "unit/ffi_layout.rs"]
mod ffi_layout;
