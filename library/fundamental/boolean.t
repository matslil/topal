#!/usr/bin/env topal
use language (
  version is v0.1
)

# Logical implication is a named, reusable Boolean relation whose intent is
# clearer than repeating its primitive expansion at every call site.
pub implies is fn (premise : Boolean, consequence : Boolean) -> Boolean
  (not premise) or consequence
