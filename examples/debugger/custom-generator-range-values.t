#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible preservation of an inclusive Range Int while its
# generator continuation is suspended and resumed.
narrow is generator ( initial : Range Int )
  yields Range Int
  resumes Unit
  -> Range Int

  _ is yield initial
  initial and (5 .. 15)

generated is narrow (0 .. 10)
generated foreach { interval }
  _ is 5 in interval
