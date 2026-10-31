use language (version is v0.1)
# Rounding belongs to ordinary numeric policy, never a native representation.
first-quotient is fn (quotient : Int, remainder : Int) -> Int
  quotient

pub rounded is fn (target : Nat) -> Int
  ten : Nat is 10
  nine : Nat is 9
  first-quotient ((target + nine) /% ten)
