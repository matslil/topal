use language (version is v0.1)
use library support (version is v0.1)

pub process-identity is fn (state : Record (next : Nat)) -> Record (allocated : Nat, next : Nat)
  cursor is state next
  allocated is support process-identity selected cursor
  following is support process-identity following allocated
  (allocated is allocated, next is following)
