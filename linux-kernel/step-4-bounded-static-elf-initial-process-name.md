# Step 4 bounded static ELF initial process-name query

The bounded static process now supports the exact `prctl(PR_GET_NAME, name)`
request issued by the pinned x86-64 BusyBox after its RELRO protection request.
The native dispatcher verifies the terminal RELRO phase, executable
continuation, active hierarchy, the exact writable and non-executable stack
leaf, and the reusable-random service. It writes the complete 16-byte Linux
process-name field as `busybox` followed by nine zero bytes, returns zero, and
advances the private startup phase without retaining the user pointer.

The process name is immutable in this bounded profile and derives from the
approved initial executable identity. Rule
`TOPAL-SYSTEMS-STATIC-ELF-INITIAL-PROCESS-NAME-001`, negative translation
tests, provider-structure tests, and the exact 616-byte clean-room fixture
enforce the request, full-field write, checked stack authority, lack of hidden
pointer retention, terminal phase, and complete existing rollback. Marker
`TOPAL_KERNEL_STATIC_ELF_INITIAL_PROCESS_NAME_OK` binds runtime evidence to the
path.

Artifact `/172`, provider `/162`, provider object `/154`, root `/161`, boot
adapter `/72`, and gate `/174` identify the contract. The root has 419
operations, 480 semantic events, 9,329 relocations, and 6,705 console-writer
relocations. The systems profile contains 164 rules.

Two independent pinned-QEMU executions reached all 170 observations with
identical evidence SHA-256
`e1c2386a577b374cfd2ac8fffd4e609887e50b24c8f813f0f4f2716f6e0b2b0e`.
Key digests are linked kernel
`4c70834dfe5b50e5b023baa0f67b92ab70d09cd886889c14d724ae74a5359f84`,
boot image `b13f10fb9d0c71f6728190edf1b1c9d10fb0fddec171c78a34028090cf642bfe`,
boot resource `f99a5950e5b5e687efa388694e979f30aecaffa6026b0b526557f5cf2d64b583`,
static image `6227a1b1cc388aacf8b1951f25694646271d9485d02e1d92baba2d9d9dfb16ca`,
artifact provenance
`20066b723b34ed14ca20f20c520238ec425a87614cc826c2d5a08c1e42b036b0`,
boot provenance
`3aeb767000b66dc63d830d0619ffd1a701f3bfe545e9ab7bf8223e445eef639e`,
source `1500c40b8a23e9b89a194163218d72e5dc0f9487e6ef86b49d61a65e69bbfea7`,
and serial output
`badc108b1848e8345627f71c1e5f06ac8a19265da5df9c9d5250bc5c47ed5a4e`.

Risk is high: the logical change is narrow, but it adds a native user-memory
write and another syscall return path whose `rcx` and `r11` registers carry
architectural `SYSRETQ` state. Review covers the exact native arguments,
checked-role derivation, complete 16-byte write and readback, stack authority,
user return-address and RFLAGS preservation, terminal rollback, generic-user
regression coverage, full workspace validation, and duplicate VM execution.

Development qualification initially selected `[0x007fff90, 0x007fffa0)` as a
writable stack scratch interval. Although mapped and writable, that interval
contained retained initial-stack data; clearing it made a later prepared-stack
rollback check fail closed. The final fixture instead reuses the already
qualified, known-zero `[0x007ffe60, 0x007ffe70)` scratch interval, restores the
stack pointer exactly, and leaves every retained initial-stack byte unchanged.

This increment does not add `PR_SET_NAME`, arbitrary names or destinations,
thread-name mutation, other `prctl` operations, dynamic linking, signals,
concurrency, filesystem access, redistributed BusyBox, containers, or a
prompt.
