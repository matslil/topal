use language (version is v0.1, features is (systems))

# TK-DEC-282, TOPAL-SYSTEMS-TAIL-PLAN-001.
# Ordinary source traversal; native resource/lifetime qualification is pending.
# Checked callers must admit the complete requested interval before source entry.
# Status is an ordinary carried fact: one means no store has failed.
# A false final result preserves earlier stores and never authorizes resumption.
store-status is fn ((region : KernelByteRegion, offset : Nat)) -> Nat
  region byte store (offset-bytes is offset, value is 0)
    true then 1
    false then 0

next-status is fn ((region : KernelByteRegion, offset : Nat, status : Nat)) -> Nat
  status = 1
    true then store-status (region, offset)
    false then 0

pub clear-prefix is fn ((region : KernelByteRegion, remaining : Nat, offset : Nat, status : Nat)) -> Boolean : Decreases remaining
  remaining
    <= 0 then status = 1
    otherwise clear-prefix (region, remaining - 1, offset + 1, next-status (region, offset, status))
