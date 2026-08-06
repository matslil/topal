#!/usr/bin/env topal
# Demonstrates lossless narrowing when a closed exact division has denominator
# one and its immediate binding explicitly requires Int.
fifty : Int is 100 / 2
negative-three : Int is -9 / 3
(fifty, negative-three)
