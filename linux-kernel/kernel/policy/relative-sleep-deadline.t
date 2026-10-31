use language (version is v0.1)
# TK-DEC-253: ordinary numeric policy for the qualified ten-nanosecond clock view.

duration-nanoseconds is fn ((seconds : Nat, nanoseconds : Nat)) -> Nat
  maximum : Nat is 9223372036854775807
  billion : Nat is 1000000000
  seconds >= 9223372036
    true then maximum
    false then seconds * billion + nanoseconds

saturate is fn (nanoseconds : Nat) -> Nat
  maximum : Nat is 9223372036854775807
  nanoseconds > maximum
    true then maximum
    false then nanoseconds

first-quotient is fn (quotient : Int, remainder : Int) -> Int
  quotient

ordinary-deadline is fn ((seconds : Nat, nanoseconds : Nat, ticks : Nat)) -> Int
  ten : Nat is 10
  nine : Nat is 9
  duration is duration-nanoseconds (seconds, nanoseconds)
  target is saturate (ticks * ten + duration)
  first-quotient ((target + nine) /% ten)

# The native boundary validates interval facts and owns the borrowed clock.
pub sleep-deadline is fn ((seconds : Nat, nanoseconds : Nat, ticks : Nat, wrapped : Boolean)) -> Int
  wrapped
    true then 922337203685477581
    false then ordinary-deadline (seconds, nanoseconds, ticks)
