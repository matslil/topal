#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates deterministic, locale-independent Unicode uppercase mapping.
# Complete mappings may expand one source character, as German sharp s does.
upper "Straße σς"
