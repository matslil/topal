# Step 4 increment: bounded static-process shell input refill

## Outcome

This increment replaces the single combined shell-input publication with two
ordered publications through one checked process buffer. Both qualified
profiles receive exact `echo topal\n`, advance affine input ownership, and
later replace the consumed publication with exact `exit\n`. The clean fixture
publishes a second `/ # ` prompt between the commands; real BusyBox requests
the refill directly while every optional clean observation remains absent.

The provider validates both sealed process-header variants, complete prior
profile state, the checked buffer and hierarchy, live dispositions, exact
request bounds, and input ownership before either publication. It writes and
reads back exact command prefixes with zero tails, returns 11 then 5, retains
no user pointer, validates clean fixture bytes before replacement, and permits
the external application to parse or mutate its process-owned first input.
Terminal handling requires the second owner, validates the sealed exit value
where applicable, erases the complete command buffer, and consumes ownership.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-REFILL-INPUT-001` and decision
`TK-DEC-220` bind the transition. Structural tests inspect both dispatches,
the two process variants, checked buffer and requested bound, exact prefixes
and zero tails, results, affine owner states, the clean second-prompt fact,
external optional-state absence, terminal erasure, and rollback.

Artifact `/199`, provider `/189`, provider object `/181`, root object `/188`,
boot adapter `/99`, and gate `/201` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and
carries the same exact 2,168-byte continuation with two reads and two clean
prompt operations.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`9dfc77624a8e7ac3b7b875b5cd5d5dee55ac5c41603ac001843443fcd5ff6039`
and
`324fcec2113bb63974f8889f491da8853e04177564417f93b44c6e45c3ae6737`.
They bind boot image
`f76172447f27493d443228e9d7fb3cb9d59871f4bf2523e8417881bac8fdbb0f`,
distinguish the clean second-prompt transition from the prompt-absent external
refill, complete all ordered markers, remain running afterward, and remain
serially quiescent. Their serial transcript hashes are
`cf4d6a1003caa3b15f1c7cea0638a9d9cfe372a795cb65855f06894434dfaf44`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, compiler and language suites, semantic
suite, full Cargo workspace suite, formatting check, diff check, and repeated
clean plus external QEMU executions pass through the bounded validation
workflow where applicable.

## Risk and remaining boundary

Risk is high because a refill replaces process-visible input across two real
application profiles and terminal rollback consumes its authority. Review
combines exact variant, predecessor, request, buffer, publication, ownership,
prompt, result, terminal, erasure, and physical runtime evidence.

This does not provide arbitrary input, partial reads, blocking or wakeup
policy, a general command loop, dynamic applications, filesystems, processes,
containers, or VM-host support.
