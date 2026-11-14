use language (version is v0.1)

retain-wrap is fn (previous : Boolean, allocated : Nat) -> Boolean
  previous
    true then true
    false then allocated = 2147483647

pub following is fn (
  current : Record (creations : Nat, wrapped : Boolean),
  allocated : Nat
) -> Record (creations : Nat, wrapped : Boolean)
  one : Nat is 1
  creations is (current creations) + one
  previous is current wrapped
  wrapped is retain-wrap (previous, allocated)
  (creations is creations, wrapped is wrapped)
