#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible transfer and closure history with root domain and
# separate root.characters generator provenance.
ignore is fn (generated : Generator Character Unit Unit) -> Unit
  ()
generated is characters "Topal"
ignore generated
