use piex::stream::Endian;
use piex::tiff::directory::{DirectoryEntry, TiffContent, TiffDirectory, TiffError, TiffType};
use piex::tiff::tags::gps;
use piex::types::{Rational, SRational};

// ============================================================================
// 1. TiffType Element Size & Parsing Tests
// ============================================================================

#[test]
fn test_tiff_type_element_sizes() {
    assert_eq!(TiffType::Byte.element_size(), 1);
    assert_eq!(TiffType::Ascii.element_size(), 1);
    assert_eq!(TiffType::Short.element_size(), 2);
    assert_eq!(TiffType::Long.element_size(), 4);
    assert_eq!(TiffType::Rational.element_size(), 8);
    assert_eq!(TiffType::SByte.element_size(), 1);
    assert_eq!(TiffType::Undefined.element_size(), 1);
    assert_eq!(TiffType::SShort.element_size(), 2);
    assert_eq!(TiffType::SLong.element_size(), 4);
    assert_eq!(TiffType::SRational.element_size(), 8);
    assert_eq!(TiffType::Float.element_size(), 4);
    assert_eq!(TiffType::Double.element_size(), 8);
    assert_eq!(TiffType::Ifd.element_size(), 4);
}

#[test]
fn test_tiff_type_from_u16_valid_and_invalid() {
    assert_eq!(TiffType::from_u16(1), Some(TiffType::Byte));
    assert_eq!(TiffType::from_u16(2), Some(TiffType::Ascii));
    assert_eq!(TiffType::from_u16(3), Some(TiffType::Short));
    assert_eq!(TiffType::from_u16(4), Some(TiffType::Long));
    assert_eq!(TiffType::from_u16(5), Some(TiffType::Rational));
    assert_eq!(TiffType::from_u16(6), Some(TiffType::SByte));
    assert_eq!(TiffType::from_u16(7), Some(TiffType::Undefined));
    assert_eq!(TiffType::from_u16(8), Some(TiffType::SShort));
    assert_eq!(TiffType::from_u16(9), Some(TiffType::SLong));
    assert_eq!(TiffType::from_u16(10), Some(TiffType::SRational));
    assert_eq!(TiffType::from_u16(11), Some(TiffType::Float));
    assert_eq!(TiffType::from_u16(12), Some(TiffType::Double));
    assert_eq!(TiffType::from_u16(13), Some(TiffType::Ifd));

    // Invalid codes
    assert_eq!(TiffType::from_u16(0), None);
    assert_eq!(TiffType::from_u16(14), None);
    assert_eq!(TiffType::from_u16(100), None);
    assert_eq!(TiffType::from_u16(u16::MAX), None);
}

#[test]
fn test_tiff_type_derives() {
    let t1 = TiffType::Short;
    let t2 = t1; // Copy & Clone
    assert_eq!(t1, t2);
    assert_eq!(format!("{:?}", t1), "Short");
}

// ============================================================================
// 2. TiffDirectory Getters (Big Endian and Little Endian)
// ============================================================================

#[test]
fn test_get_bytes_byte_and_undefined() {
    for endian in [Endian::Little, Endian::Big] {
        let mut dir = TiffDirectory::new(endian);
        assert_eq!(dir.endian(), endian);

        // Add Byte tag
        dir.add_entry(0x01, TiffType::Byte, 4, 100, vec![10, 20, 30, 40]);
        // Add Undefined tag
        dir.add_entry(0x02, TiffType::Undefined, 3, 104, vec![0xAA, 0xBB, 0xCC]);

        assert_eq!(dir.get_bytes(0x01).unwrap(), vec![10, 20, 30, 40]);
        assert_eq!(dir.get_bytes(0x02).unwrap(), vec![0xAA, 0xBB, 0xCC]);
    }
}

#[test]
fn test_get_string_ascii_null_trimming_and_invalid_utf8() {
    for endian in [Endian::Little, Endian::Big] {
        let mut dir = TiffDirectory::new(endian);

        // NUL-terminated ASCII string
        dir.add_entry(0x10, TiffType::Ascii, 10, 0, b"Canon\0EOS".to_vec());
        assert_eq!(dir.get_string(0x10).unwrap(), "Canon");

        // ASCII string without NUL byte
        dir.add_entry(0x11, TiffType::Ascii, 5, 0, b"Nikon".to_vec());
        assert_eq!(dir.get_string(0x11).unwrap(), "Nikon");

        // Lossy UTF-8 sequence
        dir.add_entry(0x12, TiffType::Ascii, 3, 0, vec![0xFF, 0xFE, 0xFD]);
        assert_eq!(dir.get_string(0x12).unwrap(), "\u{FFFD}\u{FFFD}\u{FFFD}");
    }
}

#[test]
fn test_get_ascii_bytes_returns_raw_value_like_cpp() {
    let mut dir = TiffDirectory::new(Endian::Little);
    dir.add_entry(0x10, TiffType::Ascii, 10, 0, b"Canon\0EOS\0".to_vec());
    assert_eq!(dir.get_ascii_bytes(0x10).unwrap(), b"Canon\0EOS\0");

    dir.add_entry(0x12, TiffType::Ascii, 4, 0, vec![0xB6, 0xFF, b'P', 0]);
    assert_eq!(dir.get_ascii_bytes(0x12).unwrap(), [0xB6, 0xFF, b'P', 0]);

    dir.add_entry(0x13, TiffType::Byte, 1, 0, vec![b'N']);
    assert!(matches!(dir.get_ascii_bytes(0x13), Err(TiffError::TypeMismatch { .. })));
}

#[test]
fn test_get_u32_and_u32_vec_short_and_long() {
    // Little Endian
    let mut dir_le = TiffDirectory::new(Endian::Little);
    // Short: 0x1234, 0x5678 -> LE bytes [0x34, 0x12, 0x78, 0x56]
    dir_le.add_entry(0x100, TiffType::Short, 2, 0, vec![0x34, 0x12, 0x78, 0x56]);
    // Long: 0x12345678 -> LE bytes [0x78, 0x56, 0x34, 0x12]
    dir_le.add_entry(0x101, TiffType::Long, 1, 0, vec![0x78, 0x56, 0x34, 0x12]);

    assert_eq!(dir_le.get_u32_vec(0x100).unwrap(), vec![0x1234, 0x5678]);
    assert_eq!(dir_le.get_u32(0x101).unwrap(), 0x12345678);
    assert_eq!(dir_le.get_u32_vec(0x101).unwrap(), vec![0x12345678]);

    // Count mismatch when calling get_u32 on tag with count != 1
    assert_eq!(
        dir_le.get_u32(0x100),
        Err(TiffError::InvalidCount { tag: 0x100, expected: 1, actual: 2 })
    );

    // Big Endian
    let mut dir_be = TiffDirectory::new(Endian::Big);
    // Short: 0x1234, 0x5678 -> BE bytes [0x12, 0x34, 0x56, 0x78]
    dir_be.add_entry(0x100, TiffType::Short, 2, 0, vec![0x12, 0x34, 0x56, 0x78]);
    // Long: 0x12345678 -> BE bytes [0x12, 0x34, 0x56, 0x78]
    dir_be.add_entry(0x101, TiffType::Long, 1, 0, vec![0x12, 0x34, 0x56, 0x78]);

    assert_eq!(dir_be.get_u32_vec(0x100).unwrap(), vec![0x1234, 0x5678]);
    assert_eq!(dir_be.get_u32(0x101).unwrap(), 0x12345678);
    assert_eq!(dir_be.get_u32_vec(0x101).unwrap(), vec![0x12345678]);
}

#[test]
fn test_get_rational_and_rational_vec() {
    // Little Endian
    let mut dir_le = TiffDirectory::new(Endian::Little);

    // Short converted to Rational
    dir_le.add_entry(0x200, TiffType::Short, 1, 0, vec![0x0A, 0x00]); // 10
    assert_eq!(dir_le.get_rational(0x200).unwrap(), Rational::new(10, 1));

    // Long converted to Rational
    dir_le.add_entry(0x201, TiffType::Long, 1, 0, vec![0x64, 0x00, 0x00, 0x00]); // 100
    assert_eq!(dir_le.get_rational(0x201).unwrap(), Rational::new(100, 1));

    // Rational: 100 / 2 -> num: 100, den: 2
    let mut rat_bytes_le = Vec::new();
    rat_bytes_le.extend_from_slice(&100u32.to_le_bytes());
    rat_bytes_le.extend_from_slice(&2u32.to_le_bytes());
    dir_le.add_entry(0x202, TiffType::Rational, 1, 0, rat_bytes_le);

    assert_eq!(dir_le.get_rational(0x202).unwrap(), Rational::new(100, 2));
    assert_eq!(dir_le.get_rational_vec(0x202).unwrap(), vec![Rational::new(100, 2)]);

    // Big Endian
    let mut dir_be = TiffDirectory::new(Endian::Big);
    let mut rat_bytes_be = Vec::new();
    rat_bytes_be.extend_from_slice(&100u32.to_be_bytes());
    rat_bytes_be.extend_from_slice(&4u32.to_be_bytes());
    dir_be.add_entry(0x202, TiffType::Rational, 1, 0, rat_bytes_be);

    assert_eq!(dir_be.get_rational(0x202).unwrap(), Rational::new(100, 4));
}

#[test]
fn test_zero_denominator_rational_and_srational_handling() {
    let mut dir = TiffDirectory::new(Endian::Little);

    // Rational with zero denominator
    let mut rat_bytes = Vec::new();
    rat_bytes.extend_from_slice(&100u32.to_le_bytes());
    rat_bytes.extend_from_slice(&0u32.to_le_bytes());
    dir.add_entry(0x203, TiffType::Rational, 1, 0, rat_bytes);

    let res_rat = dir.get_rational(0x203);
    assert_eq!(res_rat, Err(TiffError::ZeroDenominator { tag: 0x203 }));

    let res_rat_vec = dir.get_rational_vec(0x203);
    assert_eq!(res_rat_vec, Err(TiffError::ZeroDenominator { tag: 0x203 }));

    // SRational with zero denominator
    let mut srat_bytes = Vec::new();
    srat_bytes.extend_from_slice(&(-50i32).to_le_bytes());
    srat_bytes.extend_from_slice(&0i32.to_le_bytes());
    dir.add_entry(0x303, TiffType::SRational, 1, 0, srat_bytes);

    let res_srat = dir.get_srational(0x303);
    assert_eq!(res_srat, Err(TiffError::ZeroDenominator { tag: 0x303 }));

    let res_srat_vec = dir.get_srational_vec(0x303);
    assert_eq!(res_srat_vec, Err(TiffError::ZeroDenominator { tag: 0x303 }));
}

#[test]
fn test_get_srational_and_srational_vec() {
    // Little Endian
    let mut dir_le = TiffDirectory::new(Endian::Little);

    // SShort converted to SRational (-5)
    dir_le.add_entry(0x300, TiffType::SShort, 1, 0, (-5i16).to_le_bytes().to_vec());
    assert_eq!(dir_le.get_srational(0x300).unwrap(), SRational::new(-5, 1));

    // SLong converted to SRational (-1000)
    dir_le.add_entry(0x301, TiffType::SLong, 1, 0, (-1000i32).to_le_bytes().to_vec());
    assert_eq!(dir_le.get_srational(0x301).unwrap(), SRational::new(-1000, 1));

    // SRational: -50 / 5
    let mut srat_bytes_le = Vec::new();
    srat_bytes_le.extend_from_slice(&(-50i32).to_le_bytes());
    srat_bytes_le.extend_from_slice(&5i32.to_le_bytes());
    dir_le.add_entry(0x302, TiffType::SRational, 1, 0, srat_bytes_le);

    assert_eq!(dir_le.get_srational(0x302).unwrap(), SRational::new(-50, 5));
    assert_eq!(dir_le.get_srational_vec(0x302).unwrap(), vec![SRational::new(-50, 5)]);

    // Big Endian
    let mut dir_be = TiffDirectory::new(Endian::Big);
    let mut srat_bytes_be = Vec::new();
    srat_bytes_be.extend_from_slice(&(-100i32).to_be_bytes());
    srat_bytes_be.extend_from_slice(&10i32.to_be_bytes());
    dir_be.add_entry(0x302, TiffType::SRational, 1, 0, srat_bytes_be);

    assert_eq!(dir_be.get_srational(0x302).unwrap(), SRational::new(-100, 10));
}

#[test]
fn test_get_offset_and_length() {
    let mut dir = TiffDirectory::new(Endian::Little);
    dir.add_entry(0x400, TiffType::Long, 5, 1024, vec![0; 20]);

    // Matching type
    let (offset, length) = dir.get_offset_and_length(0x400, TiffType::Long).unwrap();
    assert_eq!(offset, 1024);
    assert_eq!(length, 20); // 5 elements * 4 bytes/element

    // Type mismatch
    let err = dir.get_offset_and_length(0x400, TiffType::Short);
    assert_eq!(
        err,
        Err(TiffError::TypeMismatch {
            tag: 0x400,
            expected: TiffType::Short,
            actual: TiffType::Long
        })
    );
}

// ============================================================================
// 3. TiffDirectory Error Handling
// ============================================================================

#[test]
fn test_tag_not_found_error() {
    let dir = TiffDirectory::new(Endian::Little);

    assert_eq!(dir.get_bytes(0x9999), Err(TiffError::TagNotFound { tag: 0x9999 }));
    assert_eq!(dir.get_string(0x9999), Err(TiffError::TagNotFound { tag: 0x9999 }));
    assert_eq!(dir.get_u32(0x9999), Err(TiffError::TagNotFound { tag: 0x9999 }));
    assert_eq!(dir.get_rational(0x9999), Err(TiffError::TagNotFound { tag: 0x9999 }));
    assert_eq!(dir.get_srational(0x9999), Err(TiffError::TagNotFound { tag: 0x9999 }));
}

#[test]
fn test_type_mismatch_error() {
    let mut dir = TiffDirectory::new(Endian::Little);
    dir.add_entry(0x500, TiffType::Short, 1, 0, vec![0x01, 0x02]);

    assert_eq!(
        dir.get_bytes(0x500),
        Err(TiffError::TypeMismatch {
            tag: 0x500,
            expected: TiffType::Byte,
            actual: TiffType::Short,
        })
    );

    assert_eq!(
        dir.get_string(0x500),
        Err(TiffError::TypeMismatch {
            tag: 0x500,
            expected: TiffType::Ascii,
            actual: TiffType::Short,
        })
    );

    assert_eq!(
        dir.get_rational_vec(0x500), // Short is valid for get_rational, so test Byte
        Ok(vec![Rational::new(0x0201, 1)])
    );

    dir.add_entry(0x501, TiffType::Byte, 1, 0, vec![0x01]);
    assert_eq!(
        dir.get_u32_vec(0x501),
        Err(TiffError::TypeMismatch {
            tag: 0x501,
            expected: TiffType::Long,
            actual: TiffType::Byte,
        })
    );
}

#[test]
fn test_truncated_buffer_handling() {
    let mut dir = TiffDirectory::new(Endian::Little);
    // Entry claims count = 2 for Long (expected 8 bytes), but value is only 4 bytes
    dir.add_entry(0x600, TiffType::Long, 2, 0, vec![0x01, 0x00, 0x00, 0x00]);

    // get_u32_vec fails with InvalidCount because value is shorter than count * elem_size
    assert_eq!(
        dir.get_u32_vec(0x600),
        Err(TiffError::InvalidCount { tag: 0x600, expected: 2, actual: 1 })
    );

    // get_u32 fails with InvalidCount because count in entry is 2 != 1
    assert_eq!(
        dir.get_u32(0x600),
        Err(TiffError::InvalidCount { tag: 0x600, expected: 1, actual: 2 })
    );
}

#[test]
fn test_vector_getters_payload_underflow_all_types() {
    let mut dir = TiffDirectory::new(Endian::Little);

    // 1. get_u32_vec with Short (count = 3, expects 6 bytes, given 4)
    dir.add_entry(0x100, TiffType::Short, 3, 0, vec![1, 0, 2, 0]);
    assert_eq!(
        dir.get_u32_vec(0x100),
        Err(TiffError::InvalidCount { tag: 0x100, expected: 3, actual: 2 })
    );

    // 2. get_u32_vec with Long (count = 4, expects 16 bytes, given 8)
    dir.add_entry(0x101, TiffType::Long, 4, 0, vec![1, 0, 0, 0, 2, 0, 0, 0]);
    assert_eq!(
        dir.get_u32_vec(0x101),
        Err(TiffError::InvalidCount { tag: 0x101, expected: 4, actual: 2 })
    );

    // 3. get_rational_vec with Rational (count = 2, expects 16 bytes, given 8)
    dir.add_entry(0x200, TiffType::Rational, 2, 0, vec![10, 0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(
        dir.get_rational_vec(0x200),
        Err(TiffError::InvalidCount { tag: 0x200, expected: 2, actual: 1 })
    );

    // 4. get_rational_vec with Short (count = 3, expects 6 bytes, given 4)
    dir.add_entry(0x201, TiffType::Short, 3, 0, vec![1, 0, 2, 0]);
    assert_eq!(
        dir.get_rational_vec(0x201),
        Err(TiffError::InvalidCount { tag: 0x201, expected: 3, actual: 2 })
    );

    // 5. get_srational_vec with SRational (count = 2, expects 16 bytes, given 8)
    dir.add_entry(0x300, TiffType::SRational, 2, 0, vec![10, 0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(
        dir.get_srational_vec(0x300),
        Err(TiffError::InvalidCount { tag: 0x300, expected: 2, actual: 1 })
    );

    // 6. get_srational_vec with SShort (count = 3, expects 6 bytes, given 4)
    dir.add_entry(0x301, TiffType::SShort, 3, 0, vec![1, 0, 2, 0]);
    assert_eq!(
        dir.get_srational_vec(0x301),
        Err(TiffError::InvalidCount { tag: 0x301, expected: 3, actual: 2 })
    );

    // 7. get_srational_vec with SLong (count = 3, expects 12 bytes, given 8)
    dir.add_entry(0x302, TiffType::SLong, 3, 0, vec![1, 0, 0, 0, 2, 0, 0, 0]);
    assert_eq!(
        dir.get_srational_vec(0x302),
        Err(TiffError::InvalidCount { tag: 0x302, expected: 3, actual: 2 })
    );
}

// ============================================================================
// 4. TiffDirectory Mutators & Hierarchy
// ============================================================================

#[test]
fn test_directory_mutators_and_subdirectories() {
    let mut parent = TiffDirectory::new(Endian::Little);
    assert!(!parent.has(0x100));

    // add_entry and add_directory_entry
    parent.add_entry(0x100, TiffType::Short, 1, 0, vec![0x01, 0x00]);
    assert!(parent.has(0x100));

    let entry = DirectoryEntry::new(TiffType::Long, 1, 0, vec![0x00, 0x01, 0x00, 0x00]);
    parent.add_directory_entry(0x101, entry);
    assert!(parent.has(0x101));

    // Subdirectory handling
    assert_eq!(parent.sub_directories().len(), 0);

    let child = TiffDirectory::new(Endian::Little);
    parent.add_sub_directory(child);

    assert_eq!(parent.sub_directories().len(), 1);
    assert_eq!(parent.sub_directories()[0].endian(), Endian::Little);
}

#[test]
fn test_directory_entry_struct() {
    let entry = DirectoryEntry::new(TiffType::Long, 1, 1024, vec![1, 2, 3, 4]);
    assert_eq!(entry.type_, TiffType::Long);
    assert_eq!(entry.count, 1);
    assert_eq!(entry.offset, 1024);
    assert_eq!(entry.value.as_slice(), &[1, 2, 3, 4]);

    let cloned = entry.clone();
    assert_eq!(entry, cloned);
}

#[test]
fn test_tiff_content_struct() {
    let content = TiffContent::default();
    assert!(content.tiff_directories.is_empty());
    assert!(content.exif_directory.is_none());
    assert!(content.gps_directory.is_none());

    let mut content_new = TiffContent::new();
    content_new.tiff_directories.push(TiffDirectory::new(Endian::Big));
    content_new.exif_directory = Some(TiffDirectory::new(Endian::Big));
    assert_eq!(content_new.tiff_directories.len(), 1);
    assert!(content_new.exif_directory.is_some());
}

// ============================================================================
// 5. GPS Tag Set Validation
// ============================================================================

#[test]
fn test_gps_all_gps_tags() {
    let tags = gps::all_gps_tags();
    assert_eq!(tags.len(), 8);

    assert!(tags.contains(&gps::GPS_TAG_LATITUDE_REF));
    assert!(tags.contains(&gps::GPS_TAG_LATITUDE));
    assert!(tags.contains(&gps::GPS_TAG_LONGITUDE_REF));
    assert!(tags.contains(&gps::GPS_TAG_LONGITUDE));
    assert!(tags.contains(&gps::GPS_TAG_ALTITUDE_REF));
    assert!(tags.contains(&gps::GPS_TAG_ALTITUDE));
    assert!(tags.contains(&gps::GPS_TAG_TIME_STAMP));
    assert!(tags.contains(&gps::GPS_TAG_DATE_STAMP));
}

// ============================================================================
// 6. SRational Initialization and Operations
// ============================================================================

#[test]
fn test_srational_struct_operations() {
    let sr = SRational::new(-10, 2);
    assert_eq!(sr.numerator, -10);
    assert_eq!(sr.denominator, 2);
    assert_eq!(sr.as_f64(), -5.0);

    let sr_zero_den = SRational::new(10, 0);
    assert_eq!(sr_zero_den.as_f64(), 0.0);

    let sr_def = SRational::default();
    assert_eq!(sr_def.numerator, 0);
    assert_eq!(sr_def.denominator, 1);
    assert_eq!(sr_def.as_f64(), 0.0);
}

// ============================================================================
// 7. Error Display Tests
// ============================================================================

#[test]
fn test_tiff_error_display() {
    let e1 = TiffError::InvalidByteOrder;
    assert_eq!(format!("{}", e1), "Invalid byte order prefix");

    let e2 = TiffError::InvalidMagicNumber(0x1234);
    assert_eq!(format!("{}", e2), "Invalid TIFF magic number: 0x1234, expected 0x002a");

    let e3 = TiffError::TagNotFound { tag: 0x0100 };
    assert_eq!(format!("{}", e3), "Tag 0x0100 not found in TIFF directory");

    let e4 =
        TiffError::TypeMismatch { tag: 0x0100, expected: TiffType::Short, actual: TiffType::Long };
    assert_eq!(format!("{}", e4), "Type mismatch for tag 0x0100: expected Short, actual Long");

    let e5 = TiffError::InvalidCount { tag: 0x0100, expected: 1, actual: 3 };
    assert_eq!(format!("{}", e5), "Invalid count for tag 0x0100: expected 1, actual 3");

    let e6 = TiffError::ZeroDenominator { tag: 0x0100 };
    assert_eq!(format!("{}", e6), "Zero denominator encountered in rational for tag 0x0100");

    let e7 = TiffError::MaxIfdLimitReached(10);
    assert_eq!(format!("{}", e7), "Maximum IFD limit reached: 10");

    let e8 = TiffError::CorruptedEntry { offset: 0x1000 };
    assert_eq!(format!("{}", e8), "Corrupted directory entry at offset 0x00001000");

    let e9 = TiffError::InvalidTypeId(99);
    assert_eq!(format!("{}", e9), "Invalid TIFF type ID: 99");

    let e10 = TiffError::InvalidStringEncoding { tag: 0x0100 };
    assert_eq!(format!("{}", e10), "Invalid string encoding for tag 0x0100");
}
