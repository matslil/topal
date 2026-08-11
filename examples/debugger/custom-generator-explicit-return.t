#!/usr/bin/env topal
# Demonstrates reversible explicit generator return with no intervening yield or
# Unit resumption before the final String becomes available.
done is generator ( initial : String )
  yields String
  resumes Unit
  -> String

  return "done"

generated is done "unused"
generated foreach { text }
  _ is empty? text
