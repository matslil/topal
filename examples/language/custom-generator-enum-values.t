#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates preservation of nominal enum identity across generator input,
# yield, suspension, resumption, and a distinct alternative returned finally.
Choice is Enum ( First, Second )
choose is generator ( initial : Choice )
  yields Choice
  resumes Unit
  -> Choice

  _ is yield initial
  Second

generated is choose First
generated foreach { choice }
  _ is choice = First
