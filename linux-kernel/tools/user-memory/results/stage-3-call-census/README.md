# Selected helper-call census and first reach

This checkpoint supports `TOPAL-SYSTEMS-USER-EXTENT-001` and `TK-DEC-283` by
counting actual emitted calls to the two owned-extent helpers and recording
which sites ordinary VM execution reaches. It changes no protection rule or
production selection. Stage 3, production qualification and the original kernel
goal remain incomplete.

The test-only producer records native offsets, the immediate Rust emitter
location and the closest preceding native labels. The location identifies the
helper-emission statement, not its complete Rust call stack. The
[ELF inspector](../../inspect_call_inventory.py) independently checks the
selected function hash, every relative call, external helper relocation and
completed-return-word erasure. LLVM's decoded instruction stream must contain
exactly the same calls, including instructions after interior marker symbols.
Before observation, the host also checks the call instructions and actual
external helper destinations in the linked kernel.

The [census](extent-call-inventory.json) contains **91 sites: 46 readable and
45 writable**. The [static inspection](extent-call-inspection.json) passes.
The [native observations](guest/observations.json) record first reach of
**77 sites: 40 readable and 37 writable**. These breakpoints observe R8/R9 and
the process phase before the helper call, require kernel privilege and CR0.WP,
and disable themselves after the first hit. They do not assign guest registers,
page tables, ownership or application data. First reach does not establish a
successful admission, a validated return, every transfer length/error path, or
dominance of every direct user access by a validator.

Fourteen sites remain unobserved in this cohort:

| Emitted area | Sites | Closest-label descriptions |
| --- | ---: | --- |
| Shared adapter used by bounded fallback paths | 3 | Named-root condition write, exec signal reset, root-directory stat |
| Positional descriptor read | 1 | `fd-pread-count-ready` |
| Descriptor serial read | 1 | `fd-serial-count-ready` |
| Named-directory publication | 1 | `named-directory-publish` |
| Process-owned pipes | 4 | Creation, read, write, ioctl |
| Volatile-file transfers | 4 | Read, pread, write, pwrite |

Exact offsets and source locations remain in the observations' `unreached`
array. These descriptions are navigation aids, not claims of unreachability or
proof that those paths have no historical evidence. The next caller work needs
ordinary source-built requests for the active descriptor/pipe/file paths and
explicit classification of the fallback paths. The 77 reached sites still
require their complete access, direction, length, error and lifetime review.
Direct user loads/stores outside these two-helper calls remain a separate
inventory obligation.

The [native test](native-test.log) passes the existing 120 source-issued syscall
checks and complete enclosing process fixture in 162.20 seconds. Resource peak
was 433545216 bytes with zero validation swap and zero limit/OOM/kill events.
The [six post-checks](post-validation.json) pass: two source-fixture build tests,
workspace formatting, explicit included-file formatting, locked workspace/all-
target checking, the Rust size ceiling and whitespace. Their peak was 806010880
bytes, with the same zero swap/events. Every run used the ordinary bounded
wrapper and retained host limits.

[Readback](readback.json) compares the actual files with the final formatted
producer checkpoint from `0b205e6e6d7243a5aa3c4ebe4889061522719938`. All 17 selected
kernel objects/images, the application image and initramfs are byte-identical;
the complete 7606-byte serial output is unchanged. The intermediate linked
application ELF differs in `.debug_info`, `.debug_line` and `.debug_line_str`;
every allocated section is unchanged. Its new full-file hash is recorded,
rather than claiming the debug artifact itself is identical. Metadata with an
omitted call, wrong direction or wrong function hash is rejected offline without
modifying or executing ELF bytes. The prior
[poll, combined-source and pinned-application evidence](../stage-3-owned-poll/README.md)
retains its original execution scope; no new external-application run is claimed.

Earlier failures are retained separately. The
[compile attempt](earlier-failures/compile/test.log) stopped on a JSON-macro syntax
error in test-only metadata. Its exact input is retained as
[inventory.rs](earlier-failures/compile/inventory.rs). The
[reader attempt](earlier-failures/symbol-reader/test.log) rejected a valid
undefined declaration followed by a definition. The corrected reader prefers
the definition for named lookup, preserves indexed relocation targets and still
rejects conflicting definitions. The original
[reader](earlier-failures/symbol-reader/inspector.py) is retained. Both attempts
ended before VM execution; neither was a safeguard interruption.

Risk in the underlying admission contract remains high. This increment is
test-only metadata and read-only observation, with unchanged emitted executable
bytes verified separately. Review was a single-agent source/authority review
plus independent artifact decoding and before/after comparison; it is not an
independent human review or a complete native security proof.

[Source identity](source-identity.json) retains selected changed inputs,
immediate helper emitters, fixture/observer inputs and authority snapshots.
It is not a complete producer census or cold reconstruction. The
[manifest](manifest.json) records the retained files, and
[scope](scope.json) keeps all remaining acceptance gates explicit.

To repeat the focused experiment, run from the repository root:

```sh
rtk proxy env RUST_MIN_STACK=16777216 scripts/run_bounded.py -- cargo test -p topal-compiler --lib --locked -j1 systems_frame_target::root_tests::owned_source_fixture_checks_poll_transfer_boundaries -- --ignored --exact --test-threads=1 --nocapture
```

The census and observation files are written to the test's printed build path
under `linux-kernel/.cache/x86_64`. This needs the repository's LLVM tools, C
compiler, QEMU and GDB. The retained runners additionally record selected source
hashes and bounded resources; their cache-relative helper paths must be restored
when replaying them. Committed production reconstruction remains a later gate.
