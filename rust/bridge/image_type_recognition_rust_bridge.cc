// Copyright 2026 Google LLC
//
// Drop-in C++ replacement implementation of image_type_recognition_lite
// wrapping pure Rust implementation via CXX bridge.

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <vector>

#include "rust/include/piex_rust.h"
#include "rust/src/cxx_bridge.rs.h"
#include "src/binary_parse/range_checked_byte_ptr.h"
#include "src/image_type_recognition/image_type_recognition_lite.h"

namespace piex_rust {
namespace image_type_recognition {

using ::piex::binary_parse::RangeCheckedBytePtr;
using ::piex::image_type_recognition::kNonRawImage;
using ::piex::image_type_recognition::RawImageTypes;

namespace bridge = ::piex_bridge;

bool IsRaw(const RawImageTypes type) {
  return bridge::rust_is_raw_type(static_cast<uint32_t>(type));
}

bool IsOfType(const RangeCheckedBytePtr& source, const RawImageTypes type) {
  if (source.errorOccurred()) {
    return false;
  }
  const size_t req = bridge::rust_get_number_of_bytes_for_is_of_type(
      static_cast<uint32_t>(type));
  const size_t len = std::min(source.remainingLength(), req);
  if (len == 0 && req > 0) {
    return false;
  }
  const std::vector<uint8_t> buf = source.extractBytes(0, len);
  if (buf.size() != len) {
    return false;
  }
  const rust::Slice<const uint8_t> slice(buf.data(), buf.size());
  return bridge::rust_is_of_type(slice, static_cast<uint32_t>(type));
}

RawImageTypes RecognizeRawImageTypeLite(const RangeCheckedBytePtr& source) {
  if (source.errorOccurred()) {
    return kNonRawImage;
  }
  const size_t req = bridge::rust_get_number_of_bytes_for_is_raw_lite();
  const size_t len = std::min(source.remainingLength(), req);
  if (len == 0) {
    return kNonRawImage;
  }
  const std::vector<uint8_t> buf = source.extractBytes(0, len);
  if (buf.size() != len) {
    return kNonRawImage;
  }
  const rust::Slice<const uint8_t> slice(buf.data(), buf.size());
  const uint32_t type_val = bridge::rust_recognize_raw_image_type_lite(slice);
  return static_cast<RawImageTypes>(type_val);
}

size_t GetNumberOfBytesForIsRawLite() {
  return bridge::rust_get_number_of_bytes_for_is_raw_lite();
}

size_t GetNumberOfBytesForIsOfType(const RawImageTypes type) {
  return bridge::rust_get_number_of_bytes_for_is_of_type(
      static_cast<uint32_t>(type));
}

bool IsRawLite(const RangeCheckedBytePtr& source) {
  return bridge::rust_is_raw_type(
      static_cast<uint32_t>(RecognizeRawImageTypeLite(source)));
}

}  // namespace image_type_recognition
}  // namespace piex_rust
