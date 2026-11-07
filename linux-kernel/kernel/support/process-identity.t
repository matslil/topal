use language (version is v0.1)

pub selected is fn (cursor : Nat) -> Nat
  initial : Nat is 2
  cursor = 0
    true then initial
    false then cursor

pub following is fn (allocated : Nat) -> Nat
  one : Nat is 1
  initial : Nat is 2
  allocated = 2147483647
    true then initial
    false then allocated + one
