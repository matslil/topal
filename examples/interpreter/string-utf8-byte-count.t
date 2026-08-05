#!/usr/bin/env topal
# Demonstrates that UTF-8 byte count observes encoding size, not characters.
ascii-bytes is "Topal" byte-count Utf8
accent-bytes is "é" byte-count Utf8
emoji-bytes is "👩‍🔬" byte-count Utf8
(ascii-bytes, accent-bytes, emoji-bytes)
