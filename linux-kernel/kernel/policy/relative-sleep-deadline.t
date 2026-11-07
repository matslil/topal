use language (version is v0.1)
use library support (version is v0.1)
# TK-DEC-258: the native bridge supplies facts, ordinary modules own the policy.
# The native boundary validates interval facts and owns the borrowed clock.
pub sleep-deadline is fn ((seconds : Nat, nanoseconds : Nat, ticks : Nat, wrapped : Boolean)) -> Int
  request is (
    interval is (seconds is seconds, nanoseconds is nanoseconds),
    clock is (ticks is ticks, wrapped is wrapped)
  )
  support clock sleep-deadline request
