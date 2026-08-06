#!/usr/bin/env topal
# Demonstrates zero-based indexing by user-perceived Unicode characters. The
# combining sequence and emoji sequence stay whole; invalid indexes return None.
text is "á👩‍🔬🇸🇪"
(text character-at 0, text character-at 1, text character-at 2, text character-at -1, text character-at 3)
