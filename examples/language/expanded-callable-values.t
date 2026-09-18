#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates every remaining symbolic callable as a retained Function value,
# including private Function result passage without tag-based dispatch.
equal is =
not-equal is !=
less is <
greater is >
at-most is <=
at-least is >=
multiply is *
divide is /
quotient-modulo is /%
modulo is %
power is ^
half-open is ..
open is <..
closed is ..=
lower-open is <..=

select is fn (operation : Function) -> Function
  operation

selected is select *
(equal (42, 42), not-equal (41, 42), less (1, 2), greater (2, 1), at-most (2, 2), at-least (2, 2), multiply (6, 7), divide (6, 8), quotient-modulo (17, 5), modulo (-17, 5), power (2, 10), half-open (0, 3), open (0, 3), closed (0, 3), lower-open (0, 3), selected (7, 6))
