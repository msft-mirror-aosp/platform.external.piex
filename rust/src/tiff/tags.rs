#![forbid(unsafe_code)]

//! TIFF and vendor tag constants definitions.

/// Standard TIFF and EXIF tags.
pub mod tiff {
    /// Subfile type.
    pub const TIFF_TAG_SUB_FILE_TYPE: u32 = 0x00FE;
    /// Image width in pixels.
    pub const TIFF_TAG_IMAGE_WIDTH: u32 = 0x0100;
    /// Image length (height) in pixels.
    pub const TIFF_TAG_IMAGE_LENGTH: u32 = 0x0101;
    /// Number of bits per sample component.
    pub const TIFF_TAG_BITS_PER_SAMPLE: u32 = 0x0102;
    /// Compression scheme used.
    pub const TIFF_TAG_COMPRESSION: u32 = 0x0103;
    /// Photometric interpretation.
    pub const TIFF_TAG_PHOTOMETRIC: u32 = 0x0106;
    /// Image description string.
    pub const TIFF_TAG_IMAGE_DESCRIPTION: u32 = 0x010E;
    /// Camera manufacturer name.
    pub const TIFF_TAG_MAKE: u32 = 0x010F;
    /// Camera model name.
    pub const TIFF_TAG_MODEL: u32 = 0x0110;
    /// Byte offsets to image data strips.
    pub const TIFF_TAG_STRIP_OFFSETS: u32 = 0x0111;
    /// Image orientation.
    pub const TIFF_TAG_ORIENTATION: u32 = 0x0112;
    /// Number of samples per pixel.
    pub const TIFF_TAG_SAMPLES_PER_PIXEL: u32 = 0x0115;
    /// Number of rows per strip.
    pub const TIFF_TAG_ROWS_PER_STRIP: u32 = 0x0116;
    /// Byte counts for each image data strip.
    pub const TIFF_TAG_STRIP_BYTE_COUNTS: u32 = 0x0117;
    /// Horizontal resolution.
    pub const TIFF_TAG_XRESOLUTION: u32 = 0x011A;
    /// Vertical resolution.
    pub const TIFF_TAG_YRESOLUTION: u32 = 0x011B;
    /// Planar configuration.
    pub const TIFF_TAG_PLANAR_CONFIG: u32 = 0x011C;
    /// Unit of measure for resolution.
    pub const TIFF_TAG_RESOLUTION_UNIT: u32 = 0x0128;
    /// Software used to generate image.
    pub const TIFF_TAG_SOFTWARE: u32 = 0x0131;
    /// Date and time string.
    pub const TIFF_TAG_DATE_TIME: u32 = 0x0132;
    /// Artist or author string.
    pub const TIFF_TAG_ARTIST: u32 = 0x013B;
    /// Byte offset to SubIFDs.
    pub const EXIF_TAG_SUB_IFDS: u32 = 0x014A;
    /// JPEG preview byte offset.
    pub const TIFF_TAG_JPEG_OFFSET: u32 = 0x0201;
    /// JPEG preview byte count.
    pub const TIFF_TAG_JPEG_BYTE_COUNT: u32 = 0x0202;
    /// Tile width in pixels.
    pub const TIFF_TAG_TILE_WIDTH: u32 = 0x0142;
    /// Tile length in pixels.
    pub const TIFF_TAG_TILE_LENGTH: u32 = 0x0143;
    /// Tile byte offsets.
    pub const TIFF_TAG_TILE_OFFSETS: u32 = 0x0144;
    /// Tile byte counts.
    pub const TIFF_TAG_TILE_BYTE_COUNTS: u32 = 0x0145;
    /// Color Filter Array pattern dimensions.
    pub const TIFF_TAG_CFA_PATTERN_DIM: u32 = 0x828D;
    /// Exposure time in seconds.
    pub const EXIF_TAG_EXPOSURE_TIME: u32 = 0x829A;
    /// F-number aperture value.
    pub const EXIF_TAG_FNUMBER: u32 = 0x829D;
    /// Offset to EXIF IFD.
    pub const EXIF_TAG_EXIF_IFD: u32 = 0x8769;
    /// Offset to GPS IFD.
    pub const EXIF_TAG_GPS: u32 = 0x8825;
    /// ISO speed rating.
    pub const EXIF_TAG_ISO_SPEED: u32 = 0x8827;
    /// Original date and time string.
    pub const EXIF_TAG_DATE_TIME_ORIGINAL: u32 = 0x9003;
    /// Focal length in millimeters.
    pub const EXIF_TAG_FOCAL_LENGTH: u32 = 0x920A;
    /// Camera maker notes offset or data.
    pub const EXIF_TAG_MAKERNOTES: u32 = 0x927C;
    /// Color space indication.
    pub const EXIF_TAG_COLOR_SPACE: u32 = 0xA001;
    /// Pixel width.
    pub const EXIF_TAG_WIDTH: u32 = 0xA002;
    /// Pixel height.
    pub const EXIF_TAG_HEIGHT: u32 = 0xA003;
    /// Default crop size.
    pub const EXIF_TAG_DEFAULT_CROP_SIZE: u32 = 0xC620;
}

/// GPS metadata tags.
pub mod gps {
    /// Latitude reference ('N' or 'S').
    pub const GPS_TAG_LATITUDE_REF: u32 = 0x0001;
    /// Latitude values (degrees, minutes, seconds).
    pub const GPS_TAG_LATITUDE: u32 = 0x0002;
    /// Longitude reference ('E' or 'W').
    pub const GPS_TAG_LONGITUDE_REF: u32 = 0x0003;
    /// Longitude values (degrees, minutes, seconds).
    pub const GPS_TAG_LONGITUDE: u32 = 0x0004;
    /// Altitude reference (0 = above sea level, 1 = below sea level).
    pub const GPS_TAG_ALTITUDE_REF: u32 = 0x0005;
    /// Altitude value in meters.
    pub const GPS_TAG_ALTITUDE: u32 = 0x0006;
    /// Time stamp (hours, minutes, seconds).
    pub const GPS_TAG_TIME_STAMP: u32 = 0x0007;
    /// Date stamp string formatted as "YYYY:MM:DD".
    pub const GPS_TAG_DATE_STAMP: u32 = 0x001D;

    /// Array containing all defined GPS tag IDs.
    pub const ALL_GPS_TAGS: [u32; 8] = [
        GPS_TAG_LATITUDE_REF,
        GPS_TAG_LATITUDE,
        GPS_TAG_LONGITUDE_REF,
        GPS_TAG_LONGITUDE,
        GPS_TAG_ALTITUDE_REF,
        GPS_TAG_ALTITUDE,
        GPS_TAG_TIME_STAMP,
        GPS_TAG_DATE_STAMP,
    ];

    /// Returns a slice containing all defined GPS tag IDs.
    pub fn all_gps_tags() -> &'static [u32] {
        &ALL_GPS_TAGS
    }
}

/// Olympus vendor specific tags.
pub mod olympus {
    /// Aspect frame dimensions.
    pub const OLYMPUS_TAG_ASPECT_FRAME: u32 = 0x1113;
    /// Camera settings data block.
    pub const OLYMPUS_TAG_CAMERA_SETTINGS: u32 = 0x2020;
    /// RAW processing options block.
    pub const OLYMPUS_TAG_RAW_PROCESSING: u32 = 0x2040;
}

/// Panasonic vendor specific tags.
pub mod panasonic {
    /// Sensor top border coordinate.
    pub const PANASONIC_TAG_TOP_BORDER: u32 = 0x0004;
    /// Sensor left border coordinate.
    pub const PANASONIC_TAG_LEFT_BORDER: u32 = 0x0005;
    /// Sensor bottom border coordinate.
    pub const PANASONIC_TAG_BOTTOM_BORDER: u32 = 0x0006;
    /// Sensor right border coordinate.
    pub const PANASONIC_TAG_RIGHT_BORDER: u32 = 0x0007;
    /// ISO speed tag.
    pub const PANASONIC_TAG_ISO: u32 = 0x0017;
    /// JPEG embedded preview image tag.
    pub const PANASONIC_TAG_JPEG_IMAGE: u32 = 0x002E;
}

/// Pentax vendor specific tags.
pub mod pentax {
    /// Color space configuration tag.
    pub const PENTAX_TAG_COLOR_SPACE: u32 = 0x0037;
}
