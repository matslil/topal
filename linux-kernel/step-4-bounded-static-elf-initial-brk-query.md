# Step 4 bounded static ELF initial `brk(0)` query

## Outcome

The bounded static-process entry now crosses one real nonterminal Linux startup
syscall before terminal return. The clean-room executable invokes `brk(0)`,
checks the complete returned value `0x005f7000`, and only then requests
`exit_group(0)`. The provider derives that result by checked 4 KiB rounding of
the validated writable load segment's memory end `0x005f61c8`; source contains
neither address nor syscall number.

The x86-64 syscall entry admits native selector 12 exactly once with a zero
request and an executable continuation, records completion in private context,
and returns through `SYSRETQ`. Terminal selector 231 is accepted only after the
query. Kernel address-space, MSR, GS, stack, continuation, mapping-status, and
process-owner restoration remain exact, and the qualification inverse clears
the enlarged private context.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-BRK-QUERY-001` refines the existing
process-entry lifecycle with the exact startup service and policy. The checked
source emits `TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK` only after the
completed owner proves query-before-terminal ordering. Negative tests reject a
different ABI, startup service, terminal result, policy, or either marker.

Structural qualification proves the clean-room query/compare/exit sequence,
native `brk` and `exit_group` selectors, derived break result, executable-return
bounds, one `SYSRETQ`, two CR3 switches, private completion state, fourteen-
qword inverse clearing, and the existing Accessed/Dirty normalization.

Artifact schema `/158`, provider `/148`, provider object `/140`, root `/147`,
boot adapter `/69`, boot resource `/2`, and gate `/160` identify the changed
contract. The generated root has 405 operations, a 466-event semantic trace,
8,660 relocations, and 6,039 console-writer relocations. The formal systems
profile now contains 150 rules.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` executions produced identical
canonical evidence with SHA-256
`797d18e5db0730e2b4a2e3ab68b8233aad0eb06dc282ac2009790c9d88e6c692`.
Both reached all 156 ordered observations, including the initial-break query,
terminal return, process-entry restoration, complete static-process rollback,
and the unchanged later kernel/user probes. The evidence records:

- linked kernel: `de6b6581aba6bfca4b7204b00385782ded372efa682ea113f7aa1c5f3c113501`;
- boot image: `8553f2ea9c4fdd4ba6050ed66fc17957d0fbcbcd4a01e8fdb62825fc21349850`;
- boot resource: `9ae7a092c381253a55b51890aec10e1e61ff42ababe0f97e7817398e566a4724`;
- static clean-room image: `896c95d9dacaec6afc4d3397629d66edeeae5d4a62acb1b982d126a0678f3428`;
- artifact provenance: `4ec43ecb0159f0144e78996680bd4a1ce10114fa0a6777429a230ff35b384aa5`;
- boot provenance: `1340365ff7c02532a6a2df754464b6d7eb6470d5c8b2fbc5dd8703efbf5e982c`;
- source: `dc1749707b91cef89f2fe0af5ccf9d4850977ae246da976e8456e28f99adc984`;
- serial output: `2d7865e6ab50d4d543285411f3bf00992d79a98b45cac71d5b341bd2ed1641cb`.

## Risk and review

Risk is high because this adds a resumable userspace syscall path to the static
process boundary. Review covered `SWAPGS` pairing, native return-state
preservation, executable continuation bounds, query ordering and uniqueness,
loader-derived arithmetic, immutable process state, terminal gating, private
context lifetime, rollback, fail-closed paths, structural code inspection,
full workspace validation, and duplicate physical execution.

## Remaining boundary

This increment admits only `brk(0)`. It does not grow or shrink the program
break, initialize TLS, execute redistributed BusyBox bytes in qualification,
support another nonterminal startup syscall, provide a prompt, or establish
general memory-management or application compatibility. The next static-
process increment can follow the pinned executable's observed startup syscall
sequence from this proven resumable boundary.
