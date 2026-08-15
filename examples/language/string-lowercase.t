#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates deterministic, locale-independent Unicode lowercase mapping.
# Complete mappings may expand one source character into multiple scalars.
lower "İΣ"
