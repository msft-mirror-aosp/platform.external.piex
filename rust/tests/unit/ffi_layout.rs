use std::ffi::c_char;
use std::mem::{offset_of, size_of};

use piex::c_types::{
    piex_color_space_t, piex_error_t, piex_gps_t, piex_image_format_t, piex_image_t,
    piex_preview_image_data_t, piex_rational_t, piex_raw_image_type_t, piex_stream_t,
};
use piex::ffi_ext::{c_buf_to_string, copy_str_to_c_buf, CStreamAdapter};
use piex::stream::StreamInterface;
use piex::types::{
    ColorSpace, GpsData, Image, ImageFormat, PiexError, PreviewImageData, Rational, RawImageType,
};

#[test]
fn test_piex_rational_t_layout() {
    assert_eq!(size_of::<piex_rational_t>(), 8);
    assert_eq!(offset_of!(piex_rational_t, numerator), 0);
    assert_eq!(offset_of!(piex_rational_t, denominator), 4);
}

#[test]
fn test_piex_image_t_layout() {
    assert_eq!(size_of::<piex_image_t>(), 16);
    assert_eq!(offset_of!(piex_image_t, width), 0);
    assert_eq!(offset_of!(piex_image_t, height), 2);
    assert_eq!(offset_of!(piex_image_t, length), 4);
    assert_eq!(offset_of!(piex_image_t, offset), 8);
    assert_eq!(offset_of!(piex_image_t, format), 12);
}

#[test]
fn test_piex_gps_t_layout() {
    let gps_size = size_of::<piex_gps_t>();
    assert!(gps_size > 0);

    assert_eq!(offset_of!(piex_gps_t, is_valid), 0);
    assert_eq!(offset_of!(piex_gps_t, latitude_ref), 1);
    assert_eq!(offset_of!(piex_gps_t, latitude), 4);
    assert_eq!(offset_of!(piex_gps_t, longitude_ref), 28);
    assert_eq!(offset_of!(piex_gps_t, longitude), 32);
    assert_eq!(offset_of!(piex_gps_t, altitude_ref), 56);
    assert_eq!(offset_of!(piex_gps_t, altitude), 60);
    assert_eq!(offset_of!(piex_gps_t, time_stamp), 68);
    assert_eq!(offset_of!(piex_gps_t, date_stamp), 92);
}

#[test]
fn test_piex_preview_image_data_t_layout() {
    let preview_size = size_of::<piex_preview_image_data_t>();
    assert!(preview_size > 0);

    assert_eq!(offset_of!(piex_preview_image_data_t, preview), 0);
    assert_eq!(offset_of!(piex_preview_image_data_t, thumbnail), 16);
    assert_eq!(offset_of!(piex_preview_image_data_t, exif_orientation), 32);
    assert_eq!(offset_of!(piex_preview_image_data_t, color_space), 36);
    assert_eq!(offset_of!(piex_preview_image_data_t, full_width), 40);
    assert_eq!(offset_of!(piex_preview_image_data_t, full_height), 44);
    assert_eq!(offset_of!(piex_preview_image_data_t, maker), 48);
    assert_eq!(offset_of!(piex_preview_image_data_t, model), 112);
    assert_eq!(offset_of!(piex_preview_image_data_t, date_time), 176);
    assert_eq!(offset_of!(piex_preview_image_data_t, iso), 208);
    assert_eq!(offset_of!(piex_preview_image_data_t, exposure_time), 212);
    assert_eq!(offset_of!(piex_preview_image_data_t, fnumber), 220);
    assert_eq!(offset_of!(piex_preview_image_data_t, focal_length), 228);
    assert_eq!(offset_of!(piex_preview_image_data_t, gps), 236);
    assert_eq!(offset_of!(piex_preview_image_data_t, cfa_pattern_dim), 340);
}

#[test]
fn test_piex_stream_t_layout() {
    assert_eq!(size_of::<piex_stream_t>(), 16);
    assert_eq!(offset_of!(piex_stream_t, read_fn), 0);
    assert_eq!(offset_of!(piex_stream_t, user_data), 8);
}

#[test]
fn test_c_string_buffer_helpers() {
    let mut buf = [0 as c_char; 16];
    copy_str_to_c_buf("Canon EOS R", &mut buf);
    let str_out = c_buf_to_string(&buf);
    assert_eq!(str_out, "Canon EOS R");

    // Test truncation on overflow
    let mut small_buf = [0 as c_char; 5];
    copy_str_to_c_buf("LongString", &mut small_buf);
    let trunc_out = c_buf_to_string(&small_buf);
    assert_eq!(trunc_out, "Long");
}

#[test]
fn test_gps_data_roundtrip_conversion() {
    let mut gps = GpsData::default();
    gps.is_valid = true;
    gps.latitude_ref = b'N';
    gps.latitude = [Rational::new(37, 1), Rational::new(46, 1), Rational::new(29, 1)];
    gps.longitude_ref = b'W';
    gps.longitude = [Rational::new(122, 1), Rational::new(25, 1), Rational::new(10, 1)];
    gps.altitude_ref = true;
    gps.altitude = Rational::new(100, 1);
    gps.time_stamp = [Rational::new(14, 1), Rational::new(30, 1), Rational::new(0, 1)];
    gps.date_stamp = b"2024:05:15".to_vec();

    let c_gps = piex_gps_t::from(&gps);
    let rust_gps = GpsData::from(&c_gps);
    assert_eq!(gps, rust_gps);
}

#[test]
fn test_preview_image_data_roundtrip_conversion() {
    let mut data = PreviewImageData::default();
    data.preview = Image::new(1920, 1080, 50000, 1024, ImageFormat::JpegCompressed);
    data.thumbnail = Image::new(160, 120, 5000, 512, ImageFormat::JpegCompressed);
    data.exif_orientation = 6;
    data.color_space = ColorSpace::AdobeRgb;
    data.full_width = 6000;
    data.full_height = 4000;
    data.maker = b"Canon".to_vec();
    data.model = b"Canon EOS 5D Mark IV".to_vec();
    data.date_time = b"2023:10:01 12:00:00".to_vec();
    data.iso = 100;
    data.exposure_time = Rational::new(1, 250);
    data.fnumber = Rational::new(28, 10);
    data.focal_length = Rational::new(50, 1);
    data.cfa_pattern_dim = [2, 2];

    let c_data = piex_preview_image_data_t::from(&data);
    let rust_data = PreviewImageData::from(&c_data);
    assert_eq!(data, rust_data);
}

#[test]
fn test_raw_image_type_discriminant_exact_values() {
    let expected_mappings: [(RawImageType, piex_raw_image_type_t, i32); 25] = [
        (RawImageType::NonRaw, piex_raw_image_type_t::PIEX_RAW_TYPE_NON_RAW, 0),
        (RawImageType::Arw, piex_raw_image_type_t::PIEX_RAW_TYPE_ARW, 1),
        (RawImageType::Cr2, piex_raw_image_type_t::PIEX_RAW_TYPE_CR2, 2),
        (RawImageType::Cr3, piex_raw_image_type_t::PIEX_RAW_TYPE_CR3, 3),
        (RawImageType::Crw, piex_raw_image_type_t::PIEX_RAW_TYPE_CRW, 4),
        (RawImageType::Dcr, piex_raw_image_type_t::PIEX_RAW_TYPE_DCR, 5),
        (RawImageType::Dng, piex_raw_image_type_t::PIEX_RAW_TYPE_DNG, 6),
        (RawImageType::Erf, piex_raw_image_type_t::PIEX_RAW_TYPE_ERF, 7),
        (RawImageType::Kdc, piex_raw_image_type_t::PIEX_RAW_TYPE_KDC, 8),
        (RawImageType::Mos, piex_raw_image_type_t::PIEX_RAW_TYPE_MOS, 9),
        (RawImageType::Mrw, piex_raw_image_type_t::PIEX_RAW_TYPE_MRW, 10),
        (RawImageType::Nef, piex_raw_image_type_t::PIEX_RAW_TYPE_NEF, 11),
        (RawImageType::Nrw, piex_raw_image_type_t::PIEX_RAW_TYPE_NRW, 12),
        (RawImageType::Orf, piex_raw_image_type_t::PIEX_RAW_TYPE_ORF, 13),
        (RawImageType::Pef, piex_raw_image_type_t::PIEX_RAW_TYPE_PEF, 14),
        (RawImageType::PhaseOne, piex_raw_image_type_t::PIEX_RAW_TYPE_PHASE_ONE, 15),
        (RawImageType::Pxn, piex_raw_image_type_t::PIEX_RAW_TYPE_PXN, 16),
        (RawImageType::Qtk, piex_raw_image_type_t::PIEX_RAW_TYPE_QTK, 17),
        (RawImageType::Raf, piex_raw_image_type_t::PIEX_RAW_TYPE_RAF, 18),
        (RawImageType::Raw3fr, piex_raw_image_type_t::PIEX_RAW_TYPE_RAW_3FR, 19),
        (RawImageType::RawContaxN, piex_raw_image_type_t::PIEX_RAW_TYPE_RAW_CONTAX_N, 20),
        (RawImageType::Rdc, piex_raw_image_type_t::PIEX_RAW_TYPE_RDC, 21),
        (RawImageType::Rw2, piex_raw_image_type_t::PIEX_RAW_TYPE_RW2, 22),
        (RawImageType::Srw, piex_raw_image_type_t::PIEX_RAW_TYPE_SRW, 23),
        (RawImageType::X3f, piex_raw_image_type_t::PIEX_RAW_TYPE_X3F, 24),
    ];

    for (rust_ty, c_ty, expected_val) in expected_mappings {
        assert_eq!(rust_ty as i32, expected_val);
        assert_eq!(c_ty as i32, expected_val);
        assert_eq!(piex_raw_image_type_t::from(rust_ty), c_ty);
        assert_eq!(RawImageType::from(c_ty), rust_ty);
    }
}

#[test]
fn test_enum_conversions() {
    let types = [
        RawImageType::NonRaw,
        RawImageType::Arw,
        RawImageType::Cr2,
        RawImageType::Cr3,
        RawImageType::Crw,
        RawImageType::Dcr,
        RawImageType::Dng,
        RawImageType::Erf,
        RawImageType::Kdc,
        RawImageType::Mos,
        RawImageType::Mrw,
        RawImageType::Nef,
        RawImageType::Nrw,
        RawImageType::Orf,
        RawImageType::Pef,
        RawImageType::PhaseOne,
        RawImageType::Pxn,
        RawImageType::Qtk,
        RawImageType::Raf,
        RawImageType::Raw3fr,
        RawImageType::RawContaxN,
        RawImageType::Rdc,
        RawImageType::Rw2,
        RawImageType::Srw,
        RawImageType::X3f,
    ];

    for &ty in &types {
        let c_ty = piex_raw_image_type_t::from(ty);
        let back_ty = RawImageType::from(c_ty);
        assert_eq!(ty, back_ty);
    }

    assert_eq!(
        piex_color_space_t::from(ColorSpace::Srgb),
        piex_color_space_t::PIEX_COLOR_SPACE_SRGB
    );
    assert_eq!(
        piex_color_space_t::from(ColorSpace::AdobeRgb),
        piex_color_space_t::PIEX_COLOR_SPACE_ADOBE_RGB
    );

    assert_eq!(
        piex_image_format_t::from(ImageFormat::JpegCompressed),
        piex_image_format_t::PIEX_FORMAT_JPEG_COMPRESSED
    );
    assert_eq!(
        piex_image_format_t::from(ImageFormat::UncompressedRgb),
        piex_image_format_t::PIEX_FORMAT_UNCOMPRESSED_RGB
    );
    assert_eq!(
        piex_image_format_t::from(ImageFormat::HevcCompressed),
        piex_image_format_t::PIEX_FORMAT_HEVC_COMPRESSED
    );
}

#[test]
fn test_piex_error_conversions() {
    let err_unsupported = PiexError::UnsupportedFormat;
    assert_eq!(piex_error_t::from(err_unsupported), piex_error_t::PIEX_UNSUPPORTED);

    let err_oob = PiexError::OutOfBounds { requested_offset: 10, requested_size: 5, total_len: 12 };
    assert_eq!(piex_error_t::from(err_oob), piex_error_t::PIEX_FAIL);
}

/// Helper struct passed as user_data context in mock C callback.
struct MockBufferContext<'a> {
    data: &'a [u8],
}

/// # Safety
///
/// `user_data` must point to a valid `MockBufferContext` instance, and `out_data` must point to
/// a writeable memory buffer of at least `length` bytes.
unsafe extern "C" fn mock_read_fn(
    user_data: *mut core::ffi::c_void,
    offset: usize,
    length: usize,
    out_data: *mut u8,
) -> i32 {
    if user_data.is_null() || out_data.is_null() {
        return -1;
    }

    // SAFETY: caller guarantees `user_data` is a valid pointer to `MockBufferContext`.
    let ctx = unsafe { &*(user_data as *const MockBufferContext) };

    let end = match offset.checked_add(length) {
        Some(e) if e <= ctx.data.len() => e,
        _ => return -1,
    };

    // SAFETY: caller guarantees `out_data` points to a mutable slice of at least `length` bytes,
    // and `offset..end` is within bounds of `ctx.data`.
    unsafe {
        core::ptr::copy_nonoverlapping(ctx.data[offset..end].as_ptr(), out_data, length);
    }

    0
}

#[test]
fn test_c_stream_adapter_functional_read() {
    let raw_data = b"Hello, PIEX C Stream Adapter test payload!";
    let mut ctx = MockBufferContext { data: raw_data };

    let c_stream = piex_stream_t {
        read_fn: Some(mock_read_fn),
        user_data: (&mut ctx as *mut MockBufferContext) as *mut core::ffi::c_void,
    };

    let adapter = CStreamAdapter::new(&c_stream, raw_data.len());
    assert_eq!(adapter.len(), raw_data.len());
    assert!(!adapter.is_empty());

    let mut buf = [0u8; 12];
    let res = adapter.get_data(7, &mut buf);
    assert!(res.is_ok());
    assert_eq!(&buf, b"PIEX C Strea");

    // Test out of bounds read
    let mut oob_buf = [0u8; 100];
    let oob_res = adapter.get_data(0, &mut oob_buf);
    assert!(oob_res.is_err());
}
