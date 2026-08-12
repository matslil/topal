#!/usr/bin/env topal
# Demonstrates preservation of inclusive Range Int endpoints across generator
# input, yield, suspension, and a narrowed final return.
narrow is generator ( initial : Range Int )
  yields Range Int
  resumes Unit
  -> Range Int

  _ is yield initial
  initial and (5 .. 15)

generated is narrow (0 .. 10)
generated foreach { interval }
  _ is 5 in interval
