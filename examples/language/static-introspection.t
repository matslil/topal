#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates qualified, compile-time inspection of language objects. The
# operations produce typed static views; they do not reflect over runtime data.
integer-identity is lang identity Int
integer-view is lang view Int
current-context is lang context
(Int lang same-object Int, Int lang equivalent-type Rational, lang version)
