#!/usr/bin/env topal
# Demonstrates automatic closure when a transferred Character generator leaves
# function scope unconsumed. The caller continuation remains consumed.
ignore is fn (generated : Generator Character Unit Unit) -> Unit
  ()
generated is characters "Topal"
ignore generated
