#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an exact nominal Sum value as one complete package field.
Message is Union
  Stop
  Move : Int

make-message is fn () -> Message
  Move 42

fallback-value is fn () -> Int
  42

offset-value is fn () -> Int
  0

score is fn ((message : Message default Stop, fallback : Int default 0)) -> Int
  message
    Move amount then amount + fallback
    Stop then fallback

shift-score is fn ((message : Message), offset : Int) -> Int
  message
    Move amount then amount + offset
    Stop then offset

(
  score (fallback is 0, message is make-message ()),
  score (fallback is fallback-value ()),
  score (Move 42, 0),
  (message is make-message ()) shift-score (offset-value ())
)
