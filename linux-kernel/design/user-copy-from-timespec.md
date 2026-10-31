# Fault-contained `copy_from_user` and zero-duration `nanosleep` contract

This record resolves `TK-DEC-034` as the input-direction refinement of the
approved `TOPAL-SYSTEMS-RECOVERY-001` boundary. It changes only the systems
feature and qualified kernel target; the Topal core language is unchanged.

## Semantic boundary

`copy_from_user` uses the same opaque affine
`UserTransferScope P A T B I` as `copy_to_user`. The scope retains the exact
processor, user address space, thread, ABI personality, and live syscall-entry
identity. Opening it suspends disposition of that entry, and every complete or
fault result returns its uniquely refined successor before the entry can be
returned or terminated.

The source candidate remains an untrusted ABI machine word rather than a
Topal reference. A transfer combines it with an overflow-checked extent, a
selected input layout, destination-owned kernel staging storage, and an
explicit progress policy. Its closed result is either:

- `Complete(value, copied = extent)`, which exposes a value only after every
  byte was copied and the selected layout was validated; or
- `Fault(copied-prefix)`, which exposes progress but no staging bytes or
  partially initialized value.

The staging destination is private to the operation. On fault its initialized
prefix and uninitialized suffix are discarded together; neither can be read by
ordinary source. This avoids inventing a general uninitialized-memory feature
and prevents a caller from accidentally decoding a partial structure. Linux
interfaces that deliberately consume a valid prefix require a later explicit
byte-policy refinement rather than weakening this layout contract.

Candidate overflow, wrong direction, layout, address-space or entry identity,
duplicate scope use, disposition with a live scope, and direct candidate or
staging access are rejected before lowering or fail closed. The backend owns
target access-enable state and binds only generated load sites to generated
recovery continuations. Unexpected kernel faults and faults outside the scope
remain fatal.

## First executable profile

The initial x86-64 profile extends the sealed probe with native Linux syscall
35 (`nanosleep`) after the two qualified `uname` calls and before
`exit_group`. The selected input layout is the native 16-byte Linux
`timespec`, two signed 64-bit fields with `tv_sec >= 0` and
`0 <= tv_nsec < 1_000_000_000`.

The valid call places `{ tv_sec = 0, tv_nsec = 0 }` at user candidate
`0x601ff0`, passes a null remainder candidate, copies and validates the entire
layout, and returns zero immediately. A second call starts at `0x601ff8`: its
first eight bytes are readable, while its second field crosses into the
qualification guard page at `0x602000`. Recovery reports an eight-byte prefix,
discards the staging value, and returns Linux `-EFAULT` (`-14`).

The private x86 page-fault entry distinguishes generated user writes from
generated user reads. A `copy_from_user` recovery accepts only the active
read-direction scope, a non-present supervisor read at the exact generated
load site, and `CR2 == 0x602000`; any mismatch remains fatal.

Zero duration is intentionally terminal without arming a deadline. This slice
does not implement nonzero sleep, remainder publication, interruption, or
restart semantics.

## Cross-architecture boundary

X86-64 may use supervisor loads with target-qualified SMAP state and private
page-fault fixup. AArch64 may use PAN/UAO-qualified loads and synchronous-abort
return. RISC-V may use SUM-qualified loads and supervisor-trap recovery. All
targets expose the same affine entry-bound scope, untrusted candidate,
all-bytes-before-value rule, prefix-only fault result, and closed
recovery/fatal distinction.

The portable contract does not require byte-at-a-time loads, one instruction
family, or prevalidation. A provider may copy in larger safe units but must
report an allowed prefix and must never expose a value from an incomplete
layout.

## Rejected alternatives

- Returning a partially initialized `timespec` would introduce unsafe
  uninitialized storage and make validation depend on a fault path.
- Treating the candidate as a reference would confer unchecked authority.
- Using `write` would couple the transfer primitive to descriptors, console
  routing, partial I/O, blocking, and interruption.
- Using a private syscall would not qualify the pinned Linux ABI.
- Implementing nonzero sleep now would combine user-copy qualification with a
  second user scheduling and signal boundary.

## Deliberate exclusions

This increment qualifies one fixed input layout and zero-duration result. It
does not add arbitrary input layouts or byte streams, dynamic user mappings,
concurrent mapping changes, demand paging, multiple processes or threads,
signals, restart, nonzero sleep, remainder output, file descriptors, ELF
loading, or a complete syscall table.
