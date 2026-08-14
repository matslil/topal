#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible nominal enum flow through a suspended generator,
# preserving Choice identity separately from First and Second alternatives.
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
