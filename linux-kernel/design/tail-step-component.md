# Private ordinary source steps

This continues `TK-DEC-282` Stage 2. Conditional carried-state bounds alone
do not qualify execution. The next component lowers the selected source cycle
to an explicit private step function and a calling bridge. This is a systems
provider convention; source function syntax, values and calls remain ordinary.

## Return boundary and source evaluation

The step emitter walks only checked tail positions. It delegates ordinary
expressions, bindings, comparisons, Optional payloads and all acyclic helpers
to the shared emitter. A terminal action copies its actual result into private
result storage and returns a terminal tag. A certified self edge evaluates all
arguments once in source order, transfers every successor scalar into the
other owned bank, and returns a successor tag. The invariant borrowed region
is checked rather than reconstructed from a number. Tags are private control
outcomes, not source values or a source ABI.

Returning to the bridge makes the step's source/helper frames dead before
reclamation. A loop inside a live source LLVM frame would additionally need
to prove that no compiler spill, debug location or register still depends on
retiring storage. The explicit return boundary avoids that live-frame erasure
problem while retaining independent obligations for the complete private
stack, register state, bridge callers and fatal cleanup. A C test bridge by
itself is not a qualified kernel stack-switch/reclamation provider.

Both banks coexist during transfer. Copy full immutable integer values,
including zero, sign, all admitted limbs, constants and aliases. Record every
read-only root's exact extent from the generated constants and reviewed
runtime; a numeric address cannot authorize reading an arbitrary object.
Transfer must finish before work/old-bank erasure, and the next step reloads
its roots from owned storage. An allocation or copy failure after source stores
preserves those stores and enters trusted fatal reclamation.

## Helper and view history

New-component helpers receive a hidden private context operand. Entry/exit
hooks maintain the exact closed call graph, current logical frame, fresh
nonwrapping identities and scoped views; source does not gain an extra operand
or context value. Each region operation validates its current scoped view.
The bridge admits total possible issuance against the actual next counters
before mutation. Reusing a descriptor cannot reset or borrow an earlier label.

The initial component can reuse the reviewed byte algorithms under a distinct
tail-region runtime. This runtime admits the caller's complete mapped owned
extent through checked pointer/end and view containment rather than the old
65536-byte extent cap. It retains the old primitive algorithms and records its
own exact runtime identity. Logical helper loans use the private context's
history separately from derived-window accounting; this source family admits
no Window construction. Existing region runtimes, count/view caps, certificates
and namespace remain unchanged. A larger extent is never source authority:
the owning native caller must establish its mapped, exclusive, disjoint span.

## Component evidence and publication boundary

Source components bind the rebuilt plan/ranges, helper identities, actual
generated LLVM, readonly-root extents, private hooks and region-runtime bytes.
They are not accepted by the kernel artifact publisher as executable library
certificates. External constructed IR receives no source-origin attestation.
Debug information names the synthetic tail step and actual source values;
its private outcome tag must not appear as an ordinary source return value.

Independent native clients must check source values/effects/order, unchanged
caller input/owner controls, complete next-root copies, both banks, work reuse,
helper/loan history, refused inputs and every post-store fatal boundary. The
actual context/descriptor/result/root layout must be included in a recomputed
private bound; the earlier 968-byte proposed layout is not that certificate.
Full guarded stack/register/erasure observations and source/native/runtime
binding remain required before Stage 2 is complete or a real kernel owner can
invoke the new family. The existing 65536 temporary/4096 stack limits remain.

The same semantic return/transfer boundary can be implemented on AArch64 and
RISC-V, but their register convention, stack isolation, pointer/granule checks
and cleanup need independent provider evidence. Only x86-64 is implemented.
