#!/usr/bin/env topal
# Demonstrates explicit completion evidence alongside an inert Effect value.
finish is fn () -> (Completed, Effect)
  (Completed, Effects ())
finish ()
