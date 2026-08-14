#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that normalization is explicit and canonical equality stays exact.
preserved is "é"
normalized is preserved normalize NFC
different-before is preserved != "é"
same-after is normalized = "é"
(preserved, normalized, different-before, same-after)
