#!/usr/bin/env topal
# Demonstrates explicit return before the first yield. Foreach invokes no action
# and directly produces the generator's declared final String.
done is generator ( initial : String )
  yields String
  resumes Unit
  -> String

  return "done"

generated is done "unused"
generated foreach { text }
  _ is empty? text
