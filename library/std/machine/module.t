#!/usr/bin/env topal
use language (version is v0.1)
### Revision of the finite additive-machine planning namespace.
pub revision is 1
### Sum minimum presses for binary indicators described by bracket targets and indexed buttons.
pub minimum-indicator-presses is fn (manual : String) -> Int
  machine-indicator-minimum-total manual
### Sum minimum presses for nonnegative exact counters described by brace targets and indexed buttons.
pub minimum-counter-presses is fn (manual : String) -> Int
  machine-counter-minimum-total manual
