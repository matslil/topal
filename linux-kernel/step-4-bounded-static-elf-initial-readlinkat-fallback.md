# Step 4 bounded static ELF initial `readlinkat` fallback

The bounded static process now performs the observed optional
`readlinkat(AT_FDCWD, 0x005bfdda, 0x005fe000, 4096)` probe after its stack-limit
query. Native syscall 267 selects a dedicated unsupported-operation path that
checks only the process lifecycle state, returns exact `-ENOSYS`, does not
inspect path or output arguments, retains no state, and leaves the same context
able to complete `exit_group(0)`.

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-READLINKAT-FALLBACK-001` and negative
and structural tests enforce the exact 325-byte clean-room startup sequence,
recoverable result, marker ordering, absence of memory access, state
preservation, and later terminal dispatch. Artifact `/165`, provider `/155`,
provider object `/147`, root `/154`, and gate `/167` identify the contract. The
root has 412 operations, 473 semantic events, 8,996 relocations, and 6,374
console-writer relocations. The systems profile contains 157 rules.

Two independent pinned-QEMU executions reached all 163 observations with
identical evidence SHA-256
`80f516301a1a1cf844a7c8b2658d2fd7da570c79d73e2cf6d4b98565133843e2`.
Key digests are linked kernel
`502122118280ddc55f10cd25e3bbe4db5c5b70aa228adae4645fa0c4e523372c`,
boot image `720bd6731b6659bd898bc211643b080dbb8ab6188c60f411896c53ac9fbc043a`,
boot resource `1c8153d86add347afcca39a50a8ca5adb3355b71487151d0065458fde9dab5af`,
static image `487063d120748dd1b994f2bcd2b68ea00754c55361ef39cb29e6cd76b3c14d5f`,
artifact provenance `069a8c617fb8ae0420fcbed5d4db31458a7e6b82126bc44b01388c6285ebe8bf`,
boot provenance `ac2e6084df631d07f2c5ca29483a588b007837f428e02f4ee4554417b39fec04`,
source `2a90f674fb547be9b5789d9e2e01e48eb285616bdf7b0b7aab589fdc24c74a80`,
and serial output `169745921bc92aa5c44c697ad2062aaaad9c01f07b5e43d9d67c73a0440b12aa`.

Risk is medium because the inline static-process dispatcher gains another
recoverable path, but selection is bounded to syscall 267 after the approved
stack-limit state and performs no argument or memory access. Review covers
exact selection and signed result, state preservation, continued terminal
dispatch, structural inspection, full workspace validation, and duplicate VM
execution. This does not add procfs, VFS lookup, symbolic links, path
resolution, BusyBox redistribution, or a prompt. The next observed startup
dependency is the first process `getrandom` request.
