#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates binding a successful Unit resumption. `resumed` does not exist
# while suspended; foreach supplies Unit, after which it becomes the return.
bind-resume is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  resumed is yield initial
  resumed

generated is bind-resume "T"
generated foreach { character }
  _ is String character
