use language (version is v0.1)
# A parcel locker has several externally visible modes. Modeling the controller
# as a Union is clearer than a cluster of flags such as `door-open`,
# `payment-pending`, and `faulted`, because impossible combinations cannot be
# constructed accidentally.
LockerState is Union
  Idle : Int
  Authorizing : Int
  DoorOpen : Int
  Faulted : Int

# The transition is deliberately total over the current state and a requested
# event. Adding a new state makes its missing transition visible.
authorization-result is fn (pending : Int, unlock-approved : Boolean) -> LockerState
  unlock-approved
    true then DoorOpen pending
    false then Idle pending

next-state is fn (state : LockerState, unlock-approved : Boolean) -> LockerState
  state
    Idle idle then Authorizing idle
    Authorizing pending then authorization-result (pending, unlock-approved)
    DoorOpen open-count then Idle open-count
    Faulted error-count then Faulted error-count

state : LockerState is Idle 0
first is next-state (state, true)
second is next-state (first, true)
third is next-state (second, true)
(first, second, third)
