#!/usr/bin/env topal
# Demonstrates automatic closure when a transferred Character generator leaves
# function scope unconsumed. The close signal has root domain and retains
# root.characters separately as generator provenance.
ignore is fn (generated : Generator Character Unit Unit) -> Unit
  ()
generated is characters "Topal"
ignore generated
