use language (version is v0.1)
# Rounding belongs to ordinary numeric policy, never a native representation.
first-quotient is fn (quotient : Int, remainder : Int) -> Int
  quotient

pub rounded is fn (target : Int) -> Int
  ten : Int is 10
  nine : Int is 9
  first-quotient ((target + nine) /% ten)

interval-seconds is fn (parts : Array (2, Int)) -> Int
  array-at? (parts, 0)
    Some value then value
    None then 0

interval-nanoseconds is fn (parts : Array (2, Int)) -> Int
  array-at? (parts, 1)
    Some value then value
    None then 0

duration-from-parts is fn (parts : Array (2, Int)) -> Int
  seconds is interval-seconds parts
  nanoseconds is interval-nanoseconds parts
  maximum : Int is 9223372036854775807
  billion : Int is 1000000000
  seconds >= 9223372036
    true then maximum
    false then seconds * billion + nanoseconds

duration-nanoseconds is fn (interval : Record (seconds : Nat, nanoseconds : Nat)) -> Int
  seconds is interval seconds
  nanoseconds is interval nanoseconds
  seconds-value : Int is seconds
  nanoseconds-value : Int is nanoseconds
  entries : List Int is Entry (seconds-value, Entry (nanoseconds-value, Empty))
  parts : Array (2, Int) is entries collect Array
  duration-from-parts parts

saturate is fn (nanoseconds : Int) -> Int
  maximum : Int is 9223372036854775807
  nanoseconds > maximum
    true then maximum
    false then nanoseconds

ordinary-deadline is fn (request : Record (interval : Record (seconds : Nat, nanoseconds : Nat), clock : Record (ticks : Nat, wrapped : Boolean))) -> Int
  interval is request interval
  clock is request clock
  ticks is clock ticks
  ten : Nat is 10
  duration is duration-nanoseconds interval
  state is (nanoseconds is duration, ticks is ticks)
  requested is ticks * ten + duration
  normalized is state with (nanoseconds is saturate requested)
  target is normalized nanoseconds
  rounded target

pub sleep-deadline is fn (request : Record (interval : Record (seconds : Nat, nanoseconds : Nat), clock : Record (ticks : Nat, wrapped : Boolean))) -> Int
  clock is request clock
  wrapped is clock wrapped
  wrapped
    true then 922337203685477581
    false then ordinary-deadline request
