#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates Boolean composition expressed entirely as ordinary Topal source.
pub implies is fn (premise : Boolean, consequence : Boolean) -> Boolean
  (not premise) or consequence

pub equivalent is fn (left : Boolean, right : Boolean) -> Boolean
  left = right
