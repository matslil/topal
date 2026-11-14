# Step 4 bounded static ELF initial `getuid`

The bounded static process now supports the exact pointer-free `getuid()`
request issued by the pinned x86-64 BusyBox after its immutable process-name
query. The native dispatcher verifies the process-name phase, executable
continuation, active hierarchy, retained RW/NX stack leaf, and the complete
zero-valued `AT_UID`, `AT_EUID`, `AT_GID`, and `AT_EGID` group. It returns the
sealed real UID zero, ignores native argument registers, and advances the
private startup phase without retaining an argument, pointer, credential
object, or identity cache.

This path refines the existing `TOPAL-SYSTEMS-USER-IDENTITY-001` contract.
Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-GETUID-001`, negative source tests,
provider-structure tests, and the exact 628-byte clean-room fixture enforce the
ordered query, credential coherence, zero result, nonretention, terminal phase,
and complete existing rollback. Marker
`TOPAL_KERNEL_STATIC_ELF_INITIAL_GETUID_OK` binds runtime evidence to the path.

Artifact `/173`, provider `/163`, provider object `/155`, root `/162`, boot
adapter `/73`, and gate `/175` identify the contract. The root has 420
operations, 481 semantic events, 9,370 relocations, and 6,746 console-writer
relocations. The systems profile contains 165 rules.

Two independent pinned-QEMU executions reached all 171 observations with
identical evidence SHA-256
`5ccbb94007b00cc51487969331466f7740cf9bf8915d01b07081ab27b8a4c3a3`.
Key digests are linked kernel
`8b30a0a2742483eb431fc1034ef36d59e5dd7fcce868d55102bab35ceb333229`,
boot image `4a59a85fe51891e696103dc0d5fd6f51ee29ae38b489d84769f342e666b42463`,
boot resource `a2456d5d37bd5ff516d2ad70305db13994ba6c82ad66f199266178d59300b9db`,
static image `7a3996177202c0773ad66131849d1fea6581c66316913e698dbfa003e301c2f1`,
artifact provenance
`2d2782d5028440175749c04029d041fb56945fbac27c5428d7bb69ccf8e1e024`,
boot provenance
`c0a9ac0a6ecd2b9bf5d548a65eb6f0f68a4560c911f3a5254a292102f60c1211`,
source `0a734bdec94221db20c93557ddbd62699bfe38c548853a36e9f5e24d98a720c0`,
and serial output
`0abbae8284185b4e1afe6dbe2ee872cf381e9daa464bae198bde657db5579a52`.

Risk is high: the logical query is pointer-free, but it adds another native
syscall return path whose `rcx` and `r11` registers carry architectural
`SYSRETQ` state and whose coherence proof reads live user-stack metadata.
Review covers exact native dispatch, active hierarchy and leaf validation,
all eight credential words, ignored arguments, zero return, RFLAGS and
continuation preservation, terminal rollback, generic-user regressions, full
workspace validation, and duplicate VM execution.

Development qualification initially attempted to read the validated stack
frame through its physical address while the static user hierarchy was active.
That address was not an admitted virtual alias under the active CR3, so the VM
failed closed before the process-entry witnesses. The final path first proves
the exact hierarchy and stack leaf, then validates the credential words through
that leaf's provider-fixed user virtual alias. The second and duplicate boots
completed every process and rollback witness.

This increment does not add `geteuid`, `getgid`, `getegid`, process or thread
identity queries, credential mutation, supplementary groups, user namespaces,
capabilities, permission checks, multiple processes or threads, dynamic
linking, redistributed BusyBox, containers, or a prompt.
