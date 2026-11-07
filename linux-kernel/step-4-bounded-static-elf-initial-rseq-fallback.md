# Step 4 bounded static ELF initial `rseq` fallback

The bounded static process now performs the observed
`rseq(0x005f7300, 33, 0, 0x53053053)` probe after empty robust-list
registration. Native syscall 334 selects a dedicated unsupported-operation
path that checks only the process lifecycle state, returns exact `-ENOSYS`,
does not inspect any argument or user memory, retains no state, and leaves the
same context able to complete `exit_group(0)`.

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-RSEQ-FALLBACK-001` and negative and
structural tests enforce the exact 226-byte clean-room startup sequence,
recoverable result, marker ordering, state preservation, and later terminal
dispatch. Artifact `/163`, provider `/153`, provider object `/145`, root `/152`,
and gate `/165` identify the contract. The root has 410 operations, 471 semantic
events, 8,896 relocations, and 6,274 console-writer relocations. The systems
profile contains 155 rules.

Three successful pinned-QEMU executions reached all 161 observations with
identical evidence. The committed evidence SHA-256 is
`c7ed0fc8937f227bbf966fbe841f9db835f7bc45d283aec1fe945a8c4370995c`.
Key digests are linked kernel
`999b5360850dcaec4c4647156b01df22a08dc5c057d2ee7a07e6842d36da3c47`,
boot image `ceaca8e52201bcc62598020839815283c033a4881dc36d75081a4972099868d0`,
boot resource `d56c5b77d2e20b72494f0ea3c1caa486b30bfb5196f61924388ac46474be668d`,
static image `6f281518a9f39beeed09577ed7fbc5c80a069e62d56ade4aa3aa1cf5ba7152ec`,
artifact provenance `eb0111832f039704722f926fe401c245dd4ad3bd6d81ccce9e7fe44a310b314b`,
boot provenance `4a90eaae386f561ee66df515ce3224f4e387ce9152d268aa34f1dab5d576a2e2`,
source `cc76f429f1ad2b1d5632f7dbc8bb746b4927b322dbc0bfa741bc296211a8ae97`,
and serial output `b34764aad4502b2bb025c83c802b75322a9daacbdbe9c7bf3b0c2262a8a9e87c`.

One earlier qualification attempt reached the new marker and completed static
process restoration, then timed out in the pre-existing later context schedule;
it was not accepted as evidence. Repeated successful runs excluded a stable
regression in this startup slice.

Risk is medium because the inline static-process dispatcher gains another
recoverable path. Review covers syscall selection, exact signed result, absence
of argument access and state mutation, continued terminal dispatch, fixture
identity, full workspace validation, and repeated VM execution. This does not
implement restartable sequences, scheduling integration, signals, BusyBox
redistribution, or a prompt. The next observed startup dependency is
`prlimit64` for the stack resource limit.
