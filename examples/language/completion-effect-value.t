#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates explicit completion evidence alongside an inert Effect value.
finish is fn () -> (Completed, Effect)
  (Completed, Effects ())
finish ()
