#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates full, locale-independent Unicode case folding. It expands sharp
# s and folds both Greek sigma forms to the same caseless-comparison basis.
case-fold "Straße Σς"
