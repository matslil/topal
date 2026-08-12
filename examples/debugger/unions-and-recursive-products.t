#!/usr/bin/env topal-debug
# Demonstrates reversible construction and matching of a payload Union and a
# positional Variant whose payload is a recursively nested product.
Message is Union
  Stop
  Move : (Int, (Int, Int))

describe is fn (message : Message) -> (Int, (Int, Int))
  message
    Move coordinates then coordinates
    Stop then (0, (0, 0))

describe (Move (10, (20, 30)))
