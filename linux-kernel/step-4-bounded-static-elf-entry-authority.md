# Step 4 bounded static ELF entry authority

## Outcome

The qualified x86-64 path now derives one opaque affine entry authority from
the restored static ELF construction, publishes
`TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK`, revokes the authority, and
publishes `TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK` before the
existing detach and frame-return sequence. Checked source never receives the
numeric ELF entry or an executable function value.

The provider revalidates the inactive static root, active bootstrap root,
complete four-frame hierarchy, all 503 sealed leaves, copied ELF `e_entry`,
entry containment at payload offset `0xa6580`, and the exact read/execute,
non-writable leaf at ordinal 166. Authorization and revocation use identical
read-only verification. Neither operation changes `CR3`, page-table bytes,
payload bytes, or metadata, and neither enters the image.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTRY-AUTHORITY-001` adds no core-language
element. Source-negative tests reject substituted owners, policies, markers,
ordering, detachment before revocation, and authority use outside the bounded
lifecycle. The semantic model enforces restored to entry-authorized to
entry-restored to detached ownership. The checked root retains only the
provider-private construction resources and admits teardown after successful
revocation.

Structural provider tests prove identical authorization and revocation
verification, exact ELF-header and entry-page checks, three `CR3` observations,
no `CR3` write, no hierarchy mutation, and the absence of a published numeric
entry. Artifact `/120`, provider and provider-object `/111`, root object `/109`,
boot adapter `/69`, boot-resource `/2`, and gate `/122` identify this executable
contract. The checked root contains 158 fatal edges, a 352-event semantic
trace, 3,777 typed relocations, 3,365 UART calls, and 99 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions derived and
revoked the entry authority, reached both new witnesses in order, completed all
99 observations, remained in the terminal fatal halt, and produced identical
artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `be35128661beb345c649785c097524b7cc46889190712e44562a8b02b1f88123`;
- boot image: `094cadf545c52e9593a422c34df9e1b25ec9f121707d2416afe08ce9684ed781`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `e7532641a094485f7db63439e362f4ddd957e6278597424c912491864331c257`;
- boot provenance: `41eaa74b9cf367e5e6c5777a1c1d5a0f6a9e96480c85869be6f6d4b391701c48`;
- serial output: `2de046f3b05a0987450edbd06aa87419f6e4bbd9a762b197f41ce48b618f3236`;
- evidence: `fa33dcc7ac6c2dff83922981e8e3f8f2ce0eef29472480ce52b360671e0cc0f8`.

## Risk and remaining boundary

Risk is high because accepting a forged entry, writable executable leaf,
stale hierarchy, wrong active root, or authority surviving teardown would
break the intended entry boundary. Review covered exact ELF arithmetic,
leaf permissions, hierarchy and root identity, affine transitions, failure
edges, negative source tests, structural machine-code inspection, and duplicate
physical execution.

This increment deliberately revokes the authority before destroying the
address space. It does not retain the static image as a process, allocate a
user stack, initialize TLS, construct a user context, enter ring 3, or execute
BusyBox.
