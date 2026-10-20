#![forbid(unsafe_code)]

//! TIFF container parsing types, tags, and directory abstractions.

pub mod directory;
pub mod parser;
pub mod tags;

pub use directory::{DirectoryEntry, EntryValue, TiffContent, TiffDirectory, TiffError, TiffType};
pub use parser::{TiffParser, THUMBNAIL_MAX_DIMENSION};
pub use tags::*;
