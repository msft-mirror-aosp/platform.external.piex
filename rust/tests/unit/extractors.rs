use piex::extractor::{
    extract_arw, extract_cr2, extract_cr3, extract_dng, extract_nef, extract_orf, extract_pef,
    extract_raf, extract_rw2, extract_srw, get_dng_information, get_olympus_preview_image,
    get_orientation, get_preview_image_data_into, raf_get_dimension, raf_get_orientation,
};
use piex::stream::{ByteSliceStream, Endian, StreamInterface};
use piex::tiff::tags::panasonic::*;
use piex::tiff::tags::pentax::*;
use piex::tiff::tags::tiff::*;
use piex::tiff::TiffType;
use piex::types::{ColorSpace, PiexError, PreviewImageData};

struct TestTiffBuilder {
    buf: Vec<u8>,
    endian: Endian,
}

impl TestTiffBuilder {
    fn new(endian: Endian) -> Self {
        Self { buf: Vec::new(), endian }
    }

    fn write_header(&mut self, first_ifd_offset: u32) {
        match self.endian {
            Endian::Little => {
                self.buf.extend_from_slice(b"II");
                self.buf.extend_from_slice(&42u16.to_le_bytes());
                self.buf.extend_from_slice(&first_ifd_offset.to_le_bytes());
            }
            Endian::Big => {
                self.buf.extend_from_slice(b"MM");
                self.buf.extend_from_slice(&42u16.to_be_bytes());
                self.buf.extend_from_slice(&first_ifd_offset.to_be_bytes());
            }
        }
    }

    fn pack_u16(&self, val: u16) -> [u8; 2] {
        match self.endian {
            Endian::Little => val.to_le_bytes(),
            Endian::Big => val.to_be_bytes(),
        }
    }

    fn pack_u32(&self, val: u32) -> [u8; 4] {
        match self.endian {
            Endian::Little => val.to_le_bytes(),
            Endian::Big => val.to_be_bytes(),
        }
    }

    fn write_entry_inline(&mut self, tag: u16, type_: TiffType, count: u32, inline_bytes: &[u8]) {
        self.buf.extend_from_slice(&self.pack_u16(tag));
        self.buf.extend_from_slice(&self.pack_u16(type_ as u16));
        self.buf.extend_from_slice(&self.pack_u32(count));
        let mut value_field = [0u8; 4];
        let copy_len = inline_bytes.len().min(4);
        value_field[..copy_len].copy_from_slice(&inline_bytes[..copy_len]);
        self.buf.extend_from_slice(&value_field);
    }

    fn write_entry_offset(&mut self, tag: u16, type_: TiffType, count: u32, data_offset: u32) {
        self.buf.extend_from_slice(&self.pack_u16(tag));
        self.buf.extend_from_slice(&self.pack_u16(type_ as u16));
        self.buf.extend_from_slice(&self.pack_u32(count));
        self.buf.extend_from_slice(&self.pack_u32(data_offset));
    }

    fn write_jpeg_header(&mut self, offset: usize, width: u16, height: u16) {
        if self.buf.len() < offset + 15 {
            self.buf.resize(offset + 15, 0);
        }
        self.buf[offset..offset + 2].copy_from_slice(&[0xFF, 0xD8]); // SOI
        self.buf[offset + 2..offset + 4].copy_from_slice(&[0xFF, 0xC0]); // SOF0
        self.buf[offset + 4..offset + 6].copy_from_slice(&[0x00, 0x0B]); // length = 11
        self.buf[offset + 6] = 8; // precision
        self.buf[offset + 7..offset + 9].copy_from_slice(&height.to_be_bytes()); // height
        self.buf[offset + 9..offset + 11].copy_from_slice(&width.to_be_bytes()); // width
        self.buf[offset + 11] = 3; // components
    }
}

#[test]
fn test_extract_arw_basic() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    // IFD0 at offset 8 with 3 entries
    builder.buf.extend_from_slice(&builder.pack_u16(3));
    builder.write_entry_inline(
        TIFF_TAG_JPEG_OFFSET as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(200),
    );
    builder.write_entry_inline(
        TIFF_TAG_JPEG_BYTE_COUNT as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(500),
    );
    builder.write_entry_inline(EXIF_TAG_WIDTH as u16, TiffType::Long, 1, &builder.pack_u32(6000));
    builder.buf.extend_from_slice(&builder.pack_u32(0)); // next IFD offset = 0

    builder.write_jpeg_header(200, 800, 600);

    if builder.buf.len() < 700 {
        builder.buf.resize(700, 0);
    }

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();

    let res = extract_arw(&stream, &mut preview_data);
    assert!(res.is_ok());
    assert_eq!(preview_data.preview.offset, 200);
    assert_eq!(preview_data.preview.length, 500);
    assert_eq!(preview_data.preview.width, 800);
    assert_eq!(preview_data.preview.height, 600);
}

#[test]
fn test_extract_cr2_basic() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    // IFD0 at offset 8: strip tags
    builder.buf.extend_from_slice(&builder.pack_u16(3));
    builder.write_entry_inline(
        TIFF_TAG_STRIP_OFFSETS as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(100),
    );
    builder.write_entry_inline(
        TIFF_TAG_STRIP_BYTE_COUNTS as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(300),
    );
    builder.write_entry_inline(
        TIFF_TAG_COMPRESSION as u16,
        TiffType::Short,
        1,
        &builder.pack_u16(6),
    );
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    builder.write_jpeg_header(100, 1024, 768);

    if builder.buf.len() < 500 {
        builder.buf.resize(500, 0);
    }

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();

    let res = extract_cr2(&stream, &mut preview_data);
    assert!(res.is_ok());
    assert_eq!(preview_data.preview.offset, 100);
    assert_eq!(preview_data.preview.length, 300);
    assert_eq!(preview_data.preview.width, 1024);
    assert_eq!(preview_data.preview.height, 768);
}

#[test]
fn test_extract_srw_basic() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    // IFD0 at offset 8
    builder.buf.extend_from_slice(&builder.pack_u16(2));
    builder.write_entry_inline(
        TIFF_TAG_JPEG_OFFSET as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(150),
    );
    builder.write_entry_inline(
        TIFF_TAG_JPEG_BYTE_COUNT as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(400),
    );
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    builder.write_jpeg_header(150, 1920, 1080);

    if builder.buf.len() < 600 {
        builder.buf.resize(600, 0);
    }

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();

    let res = extract_srw(&stream, &mut preview_data);
    assert!(res.is_ok());
    assert_eq!(preview_data.preview.offset, 150);
    assert_eq!(preview_data.preview.length, 400);
    assert_eq!(preview_data.preview.width, 1920);
    assert_eq!(preview_data.preview.height, 1080);
}

#[test]
fn test_extract_nef_valid_preview() {
    let mut builder = TestTiffBuilder::new(Endian::Big);
    builder.write_header(8);

    // IFD0 at offset 8: image width = 4000, height = 3000, jpeg offset = 100, count = 500
    builder.buf.extend_from_slice(&builder.pack_u16(4));
    builder.write_entry_inline(
        TIFF_TAG_IMAGE_WIDTH as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(4000),
    );
    builder.write_entry_inline(
        TIFF_TAG_IMAGE_LENGTH as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(3000),
    );
    builder.write_entry_inline(
        TIFF_TAG_JPEG_OFFSET as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(100),
    );
    builder.write_entry_inline(
        TIFF_TAG_JPEG_BYTE_COUNT as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(500),
    );
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    builder.write_jpeg_header(100, 3800, 2800);

    if builder.buf.len() < 700 {
        builder.buf.resize(700, 0);
    }

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();

    let res = extract_nef(&stream, &mut preview_data);
    assert!(res.is_ok());
    // 3800 / 4000 = 0.95 > 0.9 EPSILON, so full_width and full_height update to 3800 and 2800
    assert_eq!(preview_data.full_width, 3800);
    assert_eq!(preview_data.full_height, 2800);
}

#[test]
fn test_extract_pef_with_makernote() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    // IFD0 at offset 8 with EXIF_TAG_EXIF_IFD pointing to EXIF IFD at offset 100
    builder.buf.extend_from_slice(&builder.pack_u16(3));
    builder.write_entry_inline(
        TIFF_TAG_JPEG_OFFSET as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(500),
    );
    builder.write_entry_inline(
        TIFF_TAG_JPEG_BYTE_COUNT as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(200),
    );
    builder.write_entry_inline(EXIF_TAG_EXIF_IFD as u16, TiffType::Long, 1, &builder.pack_u32(100));
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    if builder.buf.len() < 300 {
        builder.buf.resize(300, 0);
    }

    // EXIF IFD at offset 100 with EXIF_TAG_MAKERNOTES pointing to offset 200 (type Undefined, length 100)
    let exif_ifd_bytes = {
        let mut temp = Vec::new();
        let mut b = TestTiffBuilder::new(Endian::Little);
        temp.extend_from_slice(&b.pack_u16(1));
        b.buf.clear();
        b.write_entry_offset(EXIF_TAG_MAKERNOTES as u16, TiffType::Undefined, 100, 200);
        temp.extend_from_slice(&b.buf);
        temp.extend_from_slice(&[0, 0, 0, 0]);
        temp
    };
    builder.buf[100..100 + exif_ifd_bytes.len()].copy_from_slice(&exif_ifd_bytes);

    // Makernote at offset 200: skip offset is 6 -> Makernote IFD starts at 206
    let mn_ifd_bytes = {
        let mut temp = Vec::new();
        let mut b = TestTiffBuilder::new(Endian::Little);
        temp.extend_from_slice(&b.pack_u16(1));
        b.buf.clear();
        b.write_entry_inline(PENTAX_TAG_COLOR_SPACE as u16, TiffType::Long, 1, &b.pack_u32(0)); // 0 = Srgb
        temp.extend_from_slice(&b.buf);
        temp.extend_from_slice(&[0, 0, 0, 0]);
        temp
    };

    if builder.buf.len() < 206 + mn_ifd_bytes.len() {
        builder.buf.resize(206 + mn_ifd_bytes.len(), 0);
    }
    builder.buf[206..206 + mn_ifd_bytes.len()].copy_from_slice(&mn_ifd_bytes);

    builder.write_jpeg_header(500, 1000, 800);

    if builder.buf.len() < 800 {
        builder.buf.resize(800, 0);
    }

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();

    let res = extract_pef(&stream, &mut preview_data);
    assert!(res.is_ok());
    assert_eq!(preview_data.color_space, ColorSpace::Srgb);
}

struct FailingHeaderStream;

impl StreamInterface for FailingHeaderStream {
    fn get_data(&self, offset: usize, data: &mut [u8]) -> Result<(), PiexError> {
        Err(PiexError::StreamReadFailed { offset, expected: data.len(), available: 0 })
    }

    fn len(&self) -> usize {
        10000
    }
}

#[test]
fn test_get_preview_image_data_propagates_stream_read_error() {
    let stream = FailingHeaderStream;
    let mut preview_data = PreviewImageData::default();
    let res = get_preview_image_data_into(&stream, &mut preview_data);
    assert!(
        matches!(res, Err(PiexError::StreamReadFailed { .. })),
        "Expected StreamReadFailed, got {:?}",
        res
    );
}

#[test]
fn test_get_orientation_propagates_stream_read_error() {
    let stream = FailingHeaderStream;
    let res = get_orientation(&stream);
    assert!(
        matches!(res, Err(PiexError::StreamReadFailed { .. })),
        "Expected StreamReadFailed, got {:?}",
        res
    );
}

#[test]
fn test_extract_pef_corrupt_makernote_fails() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    // IFD0 at offset 8 with EXIF_TAG_EXIF_IFD pointing to EXIF IFD at offset 100
    builder.buf.extend_from_slice(&builder.pack_u16(3));
    builder.write_entry_inline(
        TIFF_TAG_JPEG_OFFSET as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(500),
    );
    builder.write_entry_inline(
        TIFF_TAG_JPEG_BYTE_COUNT as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(200),
    );
    builder.write_entry_inline(EXIF_TAG_EXIF_IFD as u16, TiffType::Long, 1, &builder.pack_u32(100));
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    if builder.buf.len() < 300 {
        builder.buf.resize(300, 0);
    }

    // EXIF IFD at offset 100 with EXIF_TAG_MAKERNOTES pointing to offset 200
    let exif_ifd_bytes = {
        let mut temp = Vec::new();
        let mut b = TestTiffBuilder::new(Endian::Little);
        temp.extend_from_slice(&b.pack_u16(1));
        b.buf.clear();
        b.write_entry_offset(EXIF_TAG_MAKERNOTES as u16, TiffType::Undefined, 100, 200);
        temp.extend_from_slice(&b.buf);
        temp.extend_from_slice(&[0, 0, 0, 0]);
        temp
    };
    builder.buf[100..100 + exif_ifd_bytes.len()].copy_from_slice(&exif_ifd_bytes);

    // Corrupt Makernote at offset 200: skip offset is 6 -> Makernote IFD starts at 206
    // Write invalid entry count (0xFFFF) at offset 206
    if builder.buf.len() < 210 {
        builder.buf.resize(210, 0);
    }
    builder.buf[206..208].copy_from_slice(&0xFFFFu16.to_le_bytes());

    builder.write_jpeg_header(500, 1000, 800);

    if builder.buf.len() < 800 {
        builder.buf.resize(800, 0);
    }

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();

    let res = extract_pef(&stream, &mut preview_data);
    assert!(
        res.is_err(),
        "extract_pef must return Err when Pentax Makernote is corrupted, but got Ok"
    );
}

#[test]
fn test_get_dng_information_basic() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    // IFD0 at offset 8 with 3 entries
    let crop_offset = 200u32;
    let cfa_bytes = [2u16.to_le_bytes(), 2u16.to_le_bytes()].concat();

    builder.buf.extend_from_slice(&builder.pack_u16(3));
    builder.write_entry_offset(EXIF_TAG_DEFAULT_CROP_SIZE as u16, TiffType::Long, 2, crop_offset);
    builder.write_entry_inline(TIFF_TAG_CFA_PATTERN_DIM as u16, TiffType::Short, 2, &cfa_bytes);
    builder.write_entry_inline(
        TIFF_TAG_IMAGE_WIDTH as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(4000),
    );
    builder.buf.extend_from_slice(&builder.pack_u32(0)); // next IFD = 0

    if builder.buf.len() < 210 {
        builder.buf.resize(210, 0);
    }
    // Crop data [4000, 3000]
    builder.buf[crop_offset as usize..crop_offset as usize + 8]
        .copy_from_slice(&[4000u32.to_le_bytes(), 3000u32.to_le_bytes()].concat());

    let stream = ByteSliceStream::new(&builder.buf);
    let dng_info = get_dng_information(&stream).expect("Failed to parse DNG information");

    assert_eq!(dng_info.width, 4000);
    assert_eq!(dng_info.height, 3000);
    assert_eq!(dng_info.cfa_pattern_dim, [2, 2]);
}

#[test]
fn test_extract_dng_basic() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    let crop_offset = 200u32;
    builder.buf.extend_from_slice(&builder.pack_u16(4));
    builder.write_entry_offset(EXIF_TAG_DEFAULT_CROP_SIZE as u16, TiffType::Long, 2, crop_offset);
    builder.write_entry_inline(
        TIFF_TAG_STRIP_OFFSETS as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(300),
    );
    builder.write_entry_inline(
        TIFF_TAG_STRIP_BYTE_COUNTS as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(1000),
    );
    builder.write_entry_inline(
        TIFF_TAG_COMPRESSION as u16,
        TiffType::Short,
        1,
        &7u16.to_le_bytes(),
    );
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    if builder.buf.len() < 1300 {
        builder.buf.resize(1300, 0);
    }
    builder.buf[crop_offset as usize..crop_offset as usize + 8]
        .copy_from_slice(&[6000u32.to_le_bytes(), 4000u32.to_le_bytes()].concat());
    builder.write_jpeg_header(300, 1920, 1080);

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();
    extract_dng(&stream, &mut preview_data).expect("extract_dng failed");

    assert_eq!(preview_data.full_width, 6000);
    assert_eq!(preview_data.full_height, 4000);
    assert_eq!(preview_data.preview.offset, 300);
    assert_eq!(preview_data.preview.length, 1000);
}

#[test]
fn test_extract_cr3_corrupt_stream() {
    let buf = vec![0u8; 100];
    let stream = ByteSliceStream::new(&buf);
    let mut preview_data = PreviewImageData::default();

    let res = extract_cr3(&stream, &mut preview_data);
    assert!(res.is_err());
}

#[test]
fn test_raf_get_orientation_and_dimension() {
    let mut buf = vec![0u8; 300];
    // Offset 84 = preview offset = 100
    buf[84..88].copy_from_slice(&100u32.to_be_bytes());
    // Offset 88 = preview length = 200
    buf[88..92].copy_from_slice(&200u32.to_be_bytes());
    // Offset 92 = CFA header offset = 50
    buf[92..96].copy_from_slice(&50u32.to_be_bytes());

    // CFA header at offset 50: 1 entry
    buf[50..54].copy_from_slice(&1u32.to_be_bytes());
    // CFA entry 0 at offset 54: id = 0x0111, length = 4, height = 3000, width = 4000
    buf[54..56].copy_from_slice(&0x0111u16.to_be_bytes());
    buf[56..58].copy_from_slice(&4u16.to_be_bytes());
    buf[58..60].copy_from_slice(&3000u16.to_be_bytes());
    buf[60..62].copy_from_slice(&4000u16.to_be_bytes());

    // Write TIFF EXIF at preview offset + 12 = 112
    let mut builder = TestTiffBuilder::new(Endian::Big);
    builder.write_header(8);
    builder.buf.extend_from_slice(&builder.pack_u16(1));
    builder.write_entry_inline(
        TIFF_TAG_ORIENTATION as u16,
        TiffType::Short,
        1,
        &builder.pack_u16(6),
    );
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    if buf.len() < 112 + builder.buf.len() {
        buf.resize(112 + builder.buf.len(), 0);
    }
    buf[112..112 + builder.buf.len()].copy_from_slice(&builder.buf);

    let stream = ByteSliceStream::new(&buf);
    let (w, h) = raf_get_dimension(&stream).expect("raf_get_dimension failed");
    assert_eq!(w, 4000);
    assert_eq!(h, 3000);

    let orient = raf_get_orientation(&stream).expect("raf_get_orientation failed");
    assert_eq!(orient, 6);

    let mut preview_data = PreviewImageData::default();
    extract_raf(&stream, &mut preview_data).expect("extract_raf failed");
    assert_eq!(preview_data.full_width, 4000);
    assert_eq!(preview_data.full_height, 3000);
    assert_eq!(preview_data.preview.offset, 100);
    assert_eq!(preview_data.preview.length, 200);
}

#[test]
fn test_extract_rw2_basic() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    // IFD0 with border tags, ISO, and JPEG image
    builder.buf.extend_from_slice(&builder.pack_u16(6));
    builder.write_entry_inline(
        PANASONIC_TAG_LEFT_BORDER as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(0),
    );
    builder.write_entry_inline(
        PANASONIC_TAG_TOP_BORDER as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(0),
    );
    builder.write_entry_inline(
        PANASONIC_TAG_RIGHT_BORDER as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(4000),
    );
    builder.write_entry_inline(
        PANASONIC_TAG_BOTTOM_BORDER as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(3000),
    );
    builder.write_entry_inline(
        PANASONIC_TAG_ISO as u16,
        TiffType::Short,
        1,
        &builder.pack_u16(200),
    );
    builder.write_entry_offset(PANASONIC_TAG_JPEG_IMAGE as u16, TiffType::Undefined, 1000, 200);
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    builder.write_jpeg_header(200, 1920, 1080);

    // EXIF header at preview_offset + 12 = 212
    let mut exif_builder = TestTiffBuilder::new(Endian::Little);
    exif_builder.write_header(8);
    exif_builder.buf.extend_from_slice(&exif_builder.pack_u16(2));
    exif_builder.write_entry_inline(
        TIFF_TAG_JPEG_OFFSET as u16,
        TiffType::Long,
        1,
        &exif_builder.pack_u32(500),
    );
    exif_builder.write_entry_inline(
        TIFF_TAG_JPEG_BYTE_COUNT as u16,
        TiffType::Long,
        1,
        &exif_builder.pack_u32(200),
    );
    exif_builder.buf.extend_from_slice(&exif_builder.pack_u32(0));

    if builder.buf.len() < 212 + exif_builder.buf.len() {
        builder.buf.resize(212 + exif_builder.buf.len(), 0);
    }
    builder.buf[212..212 + exif_builder.buf.len()].copy_from_slice(&exif_builder.buf);

    if builder.buf.len() < 1200 {
        builder.buf.resize(1200, 0);
    }

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();
    extract_rw2(&stream, &mut preview_data).expect("extract_rw2 failed");

    assert_eq!(preview_data.full_width, 4000);
    assert_eq!(preview_data.full_height, 3000);
    assert_eq!(preview_data.iso, 200);
    assert_eq!(preview_data.preview.offset, 200);
    assert_eq!(preview_data.preview.length, 1000);
}

#[test]
fn test_get_olympus_preview_image_non_olympus() {
    let buf = vec![0u8; 100];
    let stream = ByteSliceStream::new(&buf);
    let mut preview_data = PreviewImageData::default();

    let res = get_olympus_preview_image(&stream, &mut preview_data);
    assert!(res.is_err());
}

#[test]
fn test_extract_orf_basic() {
    let mut builder = TestTiffBuilder::new(Endian::Little);
    builder.write_header(8);

    builder.buf.extend_from_slice(&builder.pack_u16(2));
    builder.write_entry_inline(
        TIFF_TAG_JPEG_OFFSET as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(200),
    );
    builder.write_entry_inline(
        TIFF_TAG_JPEG_BYTE_COUNT as u16,
        TiffType::Long,
        1,
        &builder.pack_u32(500),
    );
    builder.buf.extend_from_slice(&builder.pack_u32(0));

    builder.write_jpeg_header(200, 640, 480);
    if builder.buf.len() < 700 {
        builder.buf.resize(700, 0);
    }

    let stream = ByteSliceStream::new(&builder.buf);
    let mut preview_data = PreviewImageData::default();
    extract_orf(&stream, &mut preview_data).expect("extract_orf failed");

    assert_eq!(preview_data.preview.offset, 200);
    assert_eq!(preview_data.preview.length, 500);
}

#[test]
fn test_regression_crash_02_dng_orientation_fallback() {
    // TIFF header "II*\0" followed by trailer box signature "ftypcrx "
    let payload = b"\x9fII*\x00ftypcrx \x00\x00\x00\x00\x00\x00C\x03R\x02\x00\x00\x00\x00\x00\x04\x00\x00\x00\x03\x03";
    let stream = ByteSliceStream::new(payload);

    // Orientation lookup should fallback or extract orientation gracefully without failing due to pseudo-CR3 trailer
    let res = get_orientation(&stream);
    // Verified against C++ reference behavior: invalid byte order rejected with CorruptData
    assert!(matches!(res, Err(PiexError::CorruptData(_))));
}
