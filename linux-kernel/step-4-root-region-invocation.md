# Step 4: ordinary region helpers on actual kernel-owned bytes

`TK-DEC-271`; implements the [root loan design](design/root-borrowed-region-invocation.md)
and `TOPAL-SYSTEMS-REGION-ROOT-{BINDING,CALL,RECLAIM}-001`. Step 4 remains
in progress; distribution replacement, Docker/Podman and VM hosting remain
unqualified.

## Executed boundary

The [primary root](kernel/arch/x86_64/toolchain-gate.t) selects
[ordinary source](kernel/policy/bootstrap-byte-copy.t) through its declared
`region-library`, initializes owner byte 0 to 90, calls the imported
[byte helper](kernel/support/bytes.t), and checks byte 1 through the resumed
owner. The helper executes ordinary Topal function lowering; the model records
unknown output bytes and a dynamic guard instead of pretending to execute it.
A source-free control restores the preceding store/load witness and owns no
region source/runtime binding.

The native bridge binds the actual 64-byte owner, generation, source code and
exclusive temporary storage with interrupts disabled. Dedicated root hooks
use explicit control, without borrowing process/GS state. The allocation hook
is a true leaf with only its eight-byte return address; abort never returns.
The source adapter revokes its views/control. The root bridge then erases and
reads back every byte of the 65536-byte arena, 64-byte control and 4096-byte
retired source-stack reserve, clears volatile registers and restores the native
callee-saved registers and caller stack. Only scalar history remains until
terminal cleanup, which erases it separately. Successful owner stores survive.

All ordinary bootstrap allocations exclude the provider's private low 4096
bytes. Their complete physical footprint, including padding, must fit the
pool. Atomic access derives from the actual owner plus its source-relative
offset. Context stacks retain their actual allocated offsets and are checked
for extent, alignment, disjointness and lifecycle instead of old fixed physical
positions.

The region object now uses schema `topal.kernel-region-source-object.x86_64/2`
and dedicated `topal_kernel_region_allocate_v1` / `topal_kernel_region_abort_v1`
imports. Shared token-aware LLVM namespace adaptation isolates the region's
numeric runtime and `memset`. Publication binds the full source/module/object
closure, actual addresses/extents and relocated bytes of every ordinary source/
runtime function, exact native root templates, storage and combined bounds. Boot
packaging repeats linked inspection and rejects inconsistent complete call,
allocation, hook, placement or output records. Invalid source never publishes
a partial destination. A changed ordinary helper instruction or native hook is
rejected even when its outer kernel digest is corrected; changed linked source
byte bindings reject as well. Existing standalone source replay/native tests retain
their independent boundary checks with the new private hooks.

## Physical evidence

The [QEMU harness](labs/qemu/x86_64/region_gate.py),
[GDB inspector](labs/qemu/x86_64/region_inspect.py) and
[record](labs/qemu/x86_64/results/topal-root-region-gate.json) qualify seven
executions: primary, owner-byte-24 variant, wrong helper result, three invalid
entry controls and source absence.

The primary kernel SHA-256 is
`3f0edeee5a801512c1dd9c44d3b8cdf58c5357a2ea3237a6de44bd038fd55c85`.
Artifact/root/provider/object/boot/gate revisions are 245/231/232/224/145/245.
Actual helper parameters identify the live owner at bootstrap-pool+4096,
extent 64, tag 4097 and epoch 1. GDB observes helper -> ordinary entry -> source
adapter -> native bridge -> `_topal_kernel_entry`; the final frame PC equals
the return address independently read from the native stack. Private
nonallocated `.debug_frame` CFI describes the bridge's partial prologue,
alignment and epilogue; it supplies no runtime exception service. Imported
module debug locations resolve against the selected library root and exist
at observation time.

The primary and byte-24 variant each observe 136 arena bytes against the
136-byte proof and 464 source-stack bytes at instrumented function/hook entries
against the 528-byte combined source/caller proof. The source proof is 496 bytes; native alignment allowance
is 32 bytes. Full payload, complete cleanup, private-prefix preservation,
callee-saved state, restored RSP and cleared volatile registers are checked
before the root's resumed byte guard.

The byte-24 variant writes 37 through the same ordinary 64-byte view and still
preserves all 4096 provider-private bytes. The wrong helper copies to byte 2;
its real resulting payload is observed and its byte-1 guard fails before
terminal history cleanup or subsequent workload completion. The three entry
controls use a private-prefix owner, enabled interrupts, or a dirty arena.
All reach the terminal region abort without entering source or changing the
captured owner/control/arena/history bytes. They do not claim successful
cleanup or owner resumption. Absence runs the preceding witness successfully.

## Regression and review

The full bounded serial Cargo workspace passes, including 618 language tests,
251 compiler-library tests, 261 compiler CLI tests and 170 semantic-model tests.
The source/object native byte, parameter/effect-order, helper-frame and complete
certificate-replay tests pass as part of that suite. Both the shared frontend
and semantic publisher enforce the declaration/call pairing. The all-target workspace check, workspace formatting and explicit included-file
format checks also pass.

One earlier workspace build was killed by an automatically reduced 665 MiB
cgroup limit; the system journal identifies rustc as the killed process. The
wrapper's requested limit and host reserve were unchanged. The final run uses
serial compilation/tests, disabled host test debug/incremental state and two
host allocator arenas; its automatically selected budget is 1015 MiB. Generated
Topal DWARF remains enabled and is checked by the actual debugger observations.

The fresh full cohort passes 122 VM observations. The preceding 115 comprise
70 primary-kernel positives, 21 source/interface variants, eight controlled
observations and 16 expected refusals. Seven new region cases add three
successful runs (primary, byte 24 and absence), three pre-entry refusals and
the wrong-helper byte-guard refusal. The corpus includes all 40 unmodified
pinned BusyBox workloads, from command-line and interactive prompt cycles to
file redirection, named files, exec/child operations and pipes. The full cohort
is regenerated against the new primary and its explicitly recorded variants;
the older reports retain their historical kernel identities.

All 37 Python lab contract tests, 286 unique normative systems rules, relative
links and aggregate code/binding observations pass. Final all-target, formatting, whitespace and evidence checks also pass.

Compatibility harness repairs preserve the independent region declaration and
copy its source into reduced policy fixtures. A structured history
counterexample exposed a debugger shutdown race: QEMU termination could
precede GDB disconnect after the result file appeared. A 200 ms delayed GDB
exit reproduced the broken pipe. Controlled structured runs now wait for
successful debugger disconnect and independently query QMP for
`running=false`, `status=debug` before teardown. The product wrong-history
control retains the 200 ms delay; all three controlled structured observations
prove the guest stays stopped. Normal successful runs detach and resume.

Risk is high at privilege, provenance, lifecycle, physical placement,
model knowledge, ABI and stack bounds. The implementing agent reviewed the
higher-level resource/effect intent, ownership transitions, emitted branches,
complete code/cost closure and independent physical refusal oracles. Human PR
review remains required; no independent review result is claimed.

This supplies a real ordinary effectful-source foothold for kernel subsystems.
It does not implement general persistent allocation, scheduling/signals,
VFS/devices, networking/isolation or distribution/container/VM-host support.
See the [status ledger](step-4-status.md) for the initial-plan acceptance work.
