# Step 4 bounded static ELF post-random `brk(0)` query

The bounded static process now supports the read-only `brk(0)` query issued by
the pinned x86-64 BusyBox after its repeated `getrandom` startup work. The
native syscall dispatcher admits the query only after the established initial
heap growth and startup-service sequence, validates that the saved return
address remains inside the sealed executable mapping, returns the current
break `0x005f7d80`, and leaves the lifecycle and reusable entropy service
unchanged.

Rule `TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-QUERY-001`, negative translation
tests, provider-structure tests, and the exact 438-byte clean-room fixture
enforce syscall 12, a zero request, the fixed current-break result, ordering
after the second bounded random request, and fail-closed handling of every
other lifecycle state. Marker
`TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_QUERY_OK` binds runtime evidence to the
new path.

Artifact `/168`, provider `/158`, provider object `/150`, root `/157`, and gate
`/170` identify the contract. The root has 415 operations, 476 semantic events,
9,135 relocations, and 6,511 console-writer relocations. The systems profile
contains 160 rules.

Two independent pinned-QEMU executions reached all 166 observations with
identical evidence SHA-256
`1a3f349a8dca6a816422245c697fc8d6db0e15a327adebd07a91d1e85ef84caa`.
Key digests are linked kernel
`3537da989e5fcdb301e7be471b1e6f29e219a61be1cb4b429595d43e1fe587fa`,
boot image `32fb264173bdd10ea57f9cbdbd6b1a0998ee6a6120418c6fa3c858a7a6b5ef77`,
boot resource `6bcbbd8fad03c3898a2af4f199ba1187cc86e2e60efb19273cc51c4448f5c1d5`,
static image `2717c88b3c566d1937c514d3bda52449095dae21262c672370bd70d6395bbc44`,
artifact provenance
`c79681b978ad3692a84ee6da6cfdabe5672646906ec091c74e5dce8b26f974d4`,
boot provenance
`876a934db34b6b9a2e77fe4d48a3bb6dd087fe9b351130463449cc7897ac0fa9`,
source `87d05447da03e2387691537a8b3516dfc15cfa1a202169b1eafa88dbb6beeca6`,
and serial output
`1c28bdcfe8aa50b9eda803e33b22ecf3ac31745062959e81d87d394f51a4fb4a`.

Risk is medium: the change broadens a native syscall at a precisely bounded
lifecycle point but does not alter heap mappings, entropy state, or core
language semantics. Review covers initial-versus-current query separation,
sealed-return validation, state preservation, exact fixture parity, negative
translation cases, full workspace validation, and duplicate VM execution. One
earlier cold VM run reached the new marker and the existing terminal scheduler
boundary but timed out before later unrelated observations; both subsequent
runs completed with identical evidence.

This increment does not add further heap growth, shrinking, dynamic mappings,
signals, concurrency, filesystem access, redistributed BusyBox, containers, or
a prompt.
