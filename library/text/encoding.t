#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates that binary conversion is explicit and version selected. Native
# serialization produces a validated stream rather than pretending bytes are
# Unicode text.
pub encode-int is fn (value : Int) -> SerializationStream
  serialize is lang version (lang serialize)
  serialize value

pub decode-int is fn (stream : SerializationStream) -> Int
  lang deserialize stream
