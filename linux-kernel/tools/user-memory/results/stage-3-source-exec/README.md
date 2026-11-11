# Stage 3 source-built exec vector transfers

This selected checkpoint exercises `TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-ARGUMENTS-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-ENVIRONMENT-001` through compiled application
code and ordinary `execve` arguments. The [native run](native-run.json) passes
four source invocations: 116 rejected exec requests and twelve heap calls.
The enclosing ABI witness completes its existing successful image replacements
and final process retirement. The 128 observed calls are this cohort's source
requests, not a count of all syscalls issued by the enclosing application.

The [C program](exec-cases.c) checks:

- Unmapped, supervisor, overflowing and partially mapped pathname/vector inputs,
  a missing pathname before unused vectors, and a 256-byte unterminated pathname.
- Unaligned argv entries crossing a directory boundary and a page boundary
  beyond page 34; nested strings crossing pages; and readable code-resident
  pathname, string and vector data. A later invalid environment forces rollback
  after complete argv copying.
- A nested string fault after a copied argv prefix, a vector whose terminator
  lies in an unmapped page, an empty argv, count overflow and aggregate overflow.
- Environment entry/string faults after complete argv and nonempty environment
  prefixes, an inaccessible terminator, sixteen entries followed by a seventeenth
  non-null pointer, and both single-string and aggregate byte-limit overflow.

Every failed exec is observed to erase all 1536 bytes of private exec staging.
The complete 8192-byte process table, signal/thread registrations, shell/file
facts, auxiliary-vector identity and all 512 private command bytes are compared
before/after each request. The observer also checks refusal phase preservation,
source callee registers, stack shape, restored original heap contents and final
heap, owner-token, command and thread retirement. It never assigns guest
registers, memory or translations. [Actual observations](guest/observations.json)
retain the request/result matrix and selected readable input prefixes; this is
not a full native transcript of every nested string load.

[Separate readback](independent-readback.json) parses the linked ELF and all 9893
source text bytes in its read-only executable load, the syscall adapter and the
ordinary runtime call. It compares code-resident vector/string input with actual
ELF bytes, verifies the observed boundary requests and recomputes C stack costs.
The exec entry cannot call the other C entries. Its conservative 1304-byte bound
includes all its C frames/returns and the existing scratch/adapter allowance,
within the unchanged 2048-byte fixture bound. This is separate from the kernel
caller reserve. The other C objects are linked but are not executed by this
cohort; the original transfer cohort remains a separate test.

Seventeen exact comparisons preserve the preceding kernel objects, linked
kernel and boot image. All 7606 serial bytes match the preceding clean witness.
A fresh build also preserves the preceding source-transfer fixture image
exactly. The [six post-check commands](post-validation.json) pass both build-only
tests, formatting, locked workspace/all-target compilation, source size and
whitespace. No full-workspace test execution or new unchanged-BusyBox run is
claimed for this checkpoint.

The native test finishes in 149.70 seconds, within the unchanged 300-second
guest deadline. Its bounded command, including compilation, takes 162.21 seconds
and peaks at 1131925504 bytes. Limit/OOM/kill events and validation swap are zero.
The [readback correction](readback-correction.json) records an initial verifier
length error: it requested 42 bytes for a 41-byte expected sequence. The corrected
comparison passes on the same completed native artifacts. This is a verifier
error, not a guest assertion or AI safeguard decision.

[Source identity](source-identity.json) freezes fourteen selected executable/test
inputs and the updated agent guidance. This is not a complete producer census,
clean-checkout run or cold production reconstruction. Documentation and evidence
were recorded after execution. [Artifact digests](artifacts.json) retain identity
without executable payloads; [the manifest](manifest.json) indexes the retained
files. Logs preserve original whitespace through the local Git attribute.
Reproduce with the [explicit native command](../../../../labs/qemu/x86_64/README.md#source-built-exec-vector-cases).

Review is single-agent authority/full-diff self-review with separate artifact
parsing and native observations, not independent human review. Risk remains high
for kernel memory boundaries; the kernel implementation and production selection
are unchanged. Header-poll negative cases need a source-built sectioned-profile
lifecycle; a header-tuple change alone does not reproduce that startup sequence
and cannot establish the pinned external binary's identity. Its separate digest
gate remains required. Specialized signal/reset and vector-output cases,
complete caller and native ownership/mode/stability qualification, canonical
production binding and cold reconstruction remain acceptance blockers. Stage 3,
initial Step 4 and the full original kernel objective remain unfinished.
