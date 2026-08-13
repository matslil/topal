#!/usr/bin/env topal
# Demonstrates a labeled sum with both Unit and recursively composed product
# payloads. Matching a payload-bearing alternative binds its complete product.
Message is Union
  Stop
  Move : (Int, (Int, Int))

Scalar is Variant (String, Int)

describe is fn (message : Message) -> (Int, (Int, Int))
  message
    Move coordinates then coordinates
    Stop then (0, (0, 0))

show-scalar is fn (scalar : Scalar) -> String
  scalar
    Scalar at 0 text then text
    Scalar at 1 number then "number"

(describe (Move (10, (20, 30))), describe Stop, show-scalar (Scalar at 0 "text"), show-scalar (Scalar at 1 42))
