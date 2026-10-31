# Bounded static ELF initial `brk(0)` query

## Decision

The bounded static-process entry profile may require one initial program-break
query before terminal return. The process entry operation consumes the same
randomized static-process owner, enters the sealed executable, accepts exactly
one Linux `brk` request whose requested address is zero, returns the initial
break derived from the validated writable image, then accepts only the existing
`exit_group(0)` terminal result.

The initial break is the base-page ceiling of the highest byte in the validated
writable load segment. It is not a source integer and is not independently
embedded process policy. For the pinned executable shape the writable memory
interval ends at `0x005f61c8`, so the x86-64 provider returns `0x005f7000`.

The clean-room qualification payload performs `brk(0)`, compares the complete
native result with that derived break, and only then invokes `exit_group(0)`.
The completed-process owner proves that the query occurred exactly once in that
order and that no process byte, mapping, stack fact, random byte, or program-
break state was changed.

## Portable boundary

Source names an initial program-break query as part of the selected Linux
process-entry service profile. It does not name a syscall number, register,
instruction, virtual address, page-table value, or rounding implementation.
The portable behavior is a zero-request observation of a provider-derived
initial byte boundary.

This abstraction also fits AArch64 and RISC-V: their providers use their native
Linux syscall ABI while deriving the same result from the checked executable
plan and target base-page contract. Architectures with non-integer address
authority may return the ABI integer while retaining the underlying mapping
authority privately.

## X86-64 lowering

The pinned provider shall:

1. revalidate the randomized process, writable load interval, entry, and stack;
2. derive `0x005f7000` by checked 4 KiB rounding of the writable memory end;
3. enter CPL3 through the existing private process context;
4. accept syscall 12 exactly once with a zero request and an executable return;
5. return the derived break in `rax` without changing process state;
6. accept only syscall 231 with status zero after the successful query; and
7. restore the exact privileged state and completed-process owner as before.

The qualification-only entry inverse clears the private query-completion bit
with the rest of the terminal context before restoring the randomized owner.

## Rejected alternatives

- Returning a source constant would duplicate loader authority and allow the
  executable plan and process state to diverge.
- Treating the writable segment's unrounded end as the initial break would not
  match the pinned Linux process observed for this executable.
- Admitting arbitrary `brk` requests would introduce mutable heap policy,
  growth, shrink, and failure behavior before the real startup query is proven.
- Exposing syscall numbers or registers would make portable source x86-specific.

## Scope

This is a systems-profile refinement only. It changes no core-language rule.
It admits one read-only `brk(0)` startup query and no other nonterminal syscall.
It does not grow or shrink the break, initialize TLS, execute the redistributed
BusyBox bytes in qualification, provide a prompt, or establish general memory-
management compatibility.
