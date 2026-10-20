// Copyright 2026 Google LLC
//
// Public header for the Rust implementation of PIEX.
// Provides drop-in Rust implementations of PIEX extraction and type recognition
// under namespace piex_rust, consuming canonical piex:: types.

#ifndef PIEX_RUST_INCLUDE_PIEX_RUST_H_
#define PIEX_RUST_INCLUDE_PIEX_RUST_H_

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

#include "src/binary_parse/range_checked_byte_ptr.h"
#include "src/image_type_recognition/image_type_recognition_lite.h"
#include "src/piex_types.h"

namespace piex_rust {

// Returns the maximum number of bytes IsRaw() will read from the stream.
size_t BytesRequiredForIsRaw();

// Returns true if 'data' contains a RAW file format, even if it is not
// supported by Piex, false otherwise. Reads at most BytesRequiredForIsRaw()
// from the stream.
bool IsRaw(::piex::StreamInterface* data);

// Gets the largest JPEG compressed preview image data using the Rust
// implementation.
::piex::Error GetPreviewImageData(
    ::piex::StreamInterface* data, ::piex::PreviewImageData* preview_image_data,
    ::piex::image_type_recognition::RawImageTypes* output_type = nullptr);

// Returns true if the full width and height and the mosaic pattern dimension of
// a DNG image could be obtained. False otherwise.
bool GetDngInformation(::piex::StreamInterface* data, std::uint32_t* width,
                       std::uint32_t* height,
                       std::vector<std::uint32_t>* cfa_pattern_dim);

// Returns true if Exif orientation for the image can be obtained. False
// otherwise.
bool GetOrientation(::piex::StreamInterface* data, std::uint32_t* orientation);

// Returns a vector of upper case file extensions, which are used as a first
// step to quickly guess a supported file format.
std::vector<std::string> SupportedExtensions();

// CR3 APIs
bool Cr3GetOrientation(::piex::StreamInterface* stream,
                       std::uint32_t* orientation);
::piex::Error Cr3GetPreviewData(::piex::StreamInterface* stream,
                                ::piex::PreviewImageData* preview_image_data);

namespace image_type_recognition {

::piex::image_type_recognition::RawImageTypes RecognizeRawImageTypeLite(
    const ::piex::binary_parse::RangeCheckedBytePtr& source);
bool IsRaw(::piex::image_type_recognition::RawImageTypes type);
bool IsOfType(const ::piex::binary_parse::RangeCheckedBytePtr& source,
              ::piex::image_type_recognition::RawImageTypes type);
size_t GetNumberOfBytesForIsRawLite();
size_t GetNumberOfBytesForIsOfType(
    ::piex::image_type_recognition::RawImageTypes type);
bool IsRawLite(const ::piex::binary_parse::RangeCheckedBytePtr& source);

}  // namespace image_type_recognition

}  // namespace piex_rust

#endif  // PIEX_RUST_INCLUDE_PIEX_RUST_H_
