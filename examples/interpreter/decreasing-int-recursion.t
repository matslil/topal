#!/usr/bin/env topal
# Demonstrates structurally proven Int recursion: values at or below zero stop,
# while every recursive call above zero passes exactly value - 1.
sum-down is fn (value : Int) -> Int
  value
    <= 0 then 0
    otherwise value + (sum-down (value - 1))
sum-down 5
