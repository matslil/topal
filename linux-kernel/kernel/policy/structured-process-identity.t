use language (version is v0.1)
use library support (version is v0.1)

pub process-identity is fn (
  state : Record (next : Nat, history : Record (creations : Nat, wrapped : Boolean))
) -> Record (allocated : Nat, state : Record (next : Nat, history : Record (creations : Nat, wrapped : Boolean)))
  allocated is support process-identity selected (state next)
  following is support process-identity following allocated
  history is support identity-history following (state history, allocated)
  (allocated is allocated, state is (next is following, history is history))
