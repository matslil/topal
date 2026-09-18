#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates dynamic zero detection for infinity multiplication through Result.
choose-int is fn (zero : Boolean) -> Int
  zero
    true then 0
    false then -3
choose-rational is fn (zero : Boolean) -> Rational
  zero
    true then 0.0
    false then -0.5
positive : Int is +Infinity
rational-negative : Rational is -Infinity
int-success is positive * (choose-int false)
int-failure is positive * (choose-int true)
rational-success is rational-negative * (choose-rational false)
rational-failure is rational-negative * (choose-rational true)
(int-success, int-failure, rational-success, rational-failure)
