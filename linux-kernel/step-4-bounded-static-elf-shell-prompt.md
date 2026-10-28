# Step 4 increment: bounded static-process root prompt

## Outcome

This increment publishes the clean shell's exact `/ # ` root prompt after the
system-profile description has been explicitly consumed and before command
input. The clean fixture stages the four-byte segment, writes it to standard
error, requires result four, erases the complete candidate, and proves erasure.

The x86-64 provider accepts the write only at the exact unchanged shell-input
phase with every prior interactive observation complete, the profile
description absent, and closed-state history live. It validates the complete
stack candidate, copies and checks all bytes in private staging before UART
publication, erases and proves staging empty, retains no pointer, and publishes
a fresh prompt-completion fact. Sealed input and terminal rollback require the
fact; rollback clears it with the retained profile history. The pinned external
BusyBox echo profile requires the complete optional state to remain absent.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-PROMPT-001` and decision `TK-DEC-219`
bind the transition. Structural tests inspect native syscall 1, descriptor two,
the exact four-byte checked stack segment, private staging and validation,
ordered output, exact result, candidate and staging erasure, completion-fact
lifecycle, successor gating, terminal clearing, and full context restoration.

Artifact `/198`, provider `/188`, provider object `/180`, root object `/187`,
boot adapter `/98`, and gate `/200` identify the qualified contract. The Python
physical mirror retains the exact 2,024,544-byte fixture extent and carries the
same exact 2,113-byte continuation shape and 48-byte prompt sequence.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`0ad29489f000d652f6fb3f76ee5099042e414cf1ff637a69e6b0b6f22e8253f8`
and
`0c93b46a821abafc42714074f18ba6257ffa82fc2527d3999f38b9d20114a4e8`.
They bind boot image
`2662de2efaacc93c07f128b40460546659f92196deaaec40bc9647b3e4931b23`,
distinguish the clean exact-root-prompt witness from the absent external
observation, complete all ordered markers, remain running afterward, and
remain serially quiescent. Both profiles passed twice. Their serial transcript
hashes are
`46667a2244f641863308520b728096e04fbc49dc05c0deaa01ec637c913932a2`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler and language suites,
semantic suite, full Cargo workspace suite, formatting check, and diff check
all pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because application-supplied bytes cross into observable console
output and prompt completion becomes a predecessor for input. Review combines
exact phase, description state, candidate, staging, byte value, publication,
result, successor, and erasure checks; physical evidence; full-workspace
validation; and repeated clean plus external QEMU runs.

This does not provide terminal cursor queries, prompt editing, refillable input,
general commands, filesystems, processes, containers, or VM-host support.
