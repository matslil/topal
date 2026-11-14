# Step 4 runtime-derived executable extent

## Outcome

Executable installation and post-protection verification now derive their
bound from the external ELF code program header at runtime. The provider no
longer emits one load/store or load/compare pair for every qword in the
generated sample payload. Both paths use bounded loops against the same
validated immutable resource.

Portable source observes only the ordered
`TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK` witness. ELF field offsets, byte
counts, qword loops, addresses, and instructions remain provider details. This
is a systems-feature refinement and changes no core-language rule.

## Model and x86-64 lowering

The checked model requires the runtime-extent witness exactly once after
external-copy verification and before data/BSS, transfer, syscall, retention,
or teardown observations. Negative tests reject missing and renamed witnesses,
while the ordered continuation rejects early, duplicate, or late placement.

Provider revision `/46` excludes executable `p_filesz` and `p_memsz` from the
immediate metadata oracle. It loads both from the external ELF, requires them
to be equal, requires the extent to include the selected entry qword, limits it
to one 4 KiB base page, and requires qword completeness. Installation copies
exactly that bound; after RO/NX publication, verification compares exactly the
same bound before RO/X publication. Any zero/truncated, unequal, misaligned,
over-page, incomplete, or mismatched extent fails closed.

Artifact and root revisions `/47` record 62 placements, a 194-transition
trace, and 1,675 typed relocations, including 1,489 UART writes. Boot resource
schema `/1` and boot adapter `/8` are unchanged.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 46 ordered markers. Both copied and verified the external code
through the runtime-derived loop bound, completed data/BSS and every prior
process-entry and Linux syscall check, remained in the live fatal halt, and
left serial output quiescent. Gate schema `/49` records:

- linked kernel: `2b86c10a8aabdf680228b0fee733ef8fa351f5d381664e3de46f464ad99b1098`;
- boot image: `1fd88c00973614c284a3dc8b04bcb79eb695c98aa767d7a9924da293adb9f93f`;
- boot resource: `fd2be78bfb6a99467fdd8d9b9d7c8cc0e3c07412c1b9122e8fee45a385d3fb2b`;
- initial user image: `079d7bcaca0d4cbfa91777ac8a15bf1eda1bf52f48116f2addcf8059c819407b`;
- artifact provenance: `7aaa1a532db05abe7d82a339582105edafd312ca7fc84b599aad0ddcb758cc6e`;
- boot provenance: `ed77474d2b064e67572639fc52899cab08d67ca45976bd234e364aeb0a5779d4`;
- serial: `be0c00d566a2820c3f01f2d0b9c6b242704367de28c3ed0a2ab2db468f93be03`;
- evidence record: `fe8a121f6afbd9081415268e29cce7dbcbc19f6bc6372c60bae2c9838b751dfa`.

## Risk and review

Risk is high because a resource-controlled loop bound governs executable
population and executable-authority publication. Review covered field-oracle
removal, relational checks, lower and upper bounds, qword completeness, exact
loop termination, same-resource provenance, post-protection comparison,
fail-closed mismatch handling, negative model tests, structural instruction
tests, full workspace validation, and two independent physical runs.

## Remaining boundary

Placement, entry, resource size, data extent, and the generated qualification
payload remain sealed. The profile does not yet admit writable data, arbitrary
segments, relocations, interpreters, dynamic linking, TLS, PIE, ASLR, demand
paging, path lookup, or a general process loader.
