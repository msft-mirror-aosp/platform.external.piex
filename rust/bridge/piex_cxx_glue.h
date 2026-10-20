// Copyright 2026 Google LLC
//
// PIEX CXX Bridge Glue Helpers

#ifndef PIEX_RUST_BRIDGE_PIEX_CXX_GLUE_H_
#define PIEX_RUST_BRIDGE_PIEX_CXX_GLUE_H_

#include <cstddef>
#include <cstdint>

#include "src/piex_types.h"

namespace piex_bridge {

using ColorSpace = ::piex::PreviewImageData::ColorSpace;
using Image = ::piex::Image;
using ImageFormat = ::piex::Image::Format;
using Rational = ::piex::PreviewImageData::Rational;
using StreamInterface = ::piex::StreamInterface;

inline int32_t StreamGetData(StreamInterface& stream, size_t offset,
                             size_t length, uint8_t* data) {
  return static_cast<int32_t>(stream.GetData(offset, length, data));
}

}  // namespace piex_bridge
#endif  // PIEX_RUST_BRIDGE_PIEX_CXX_GLUE_H_
