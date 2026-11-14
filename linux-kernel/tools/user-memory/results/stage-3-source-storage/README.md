# Source-built owned file and directory transfers

This checkpoint exercises `TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001` and
`TOPAL-SYSTEMS-POSITIONAL-FILE-TRANSFER-001` through ordinary application
requests. It changes test construction and observation only. Production
selection, protection rules and core-language meaning remain unchanged.

The new [C component](storage-cases.c) executes four times in the enclosing
process fixture. Each execution grows the heap through `brk`, creates an
anonymous `/tmp` file, exercises ordinary and positional transfers, closes it,
reads the immutable application file with pread, reads `/tmp` directory records,
and restores its descriptors, heap bytes and break. The component is compiled
and linked before construction of the first-party ABI witness. No finished
application is patched or host workload substituted.

The [native run](native-test.log) passes **208 source-issued syscalls**, including
196 storage requests. Cases include 112 `EFAULT` buffer refusals, four negative
offset refusals, twenty successful positional returns preserving shared
positions, twelve fresh descriptions and four exact storage retirements.
Buffer cases cover read-only and supervisor outputs, noncanonical/wrapped and
unmapped candidates, partial mapped prefixes, page/directory crossings beyond
the predecessor heap, read-only write input and unused unmapped EOF outputs.
The source separately checks actual stored/replaced bytes and directory records.

The [observer](inputs/linux-kernel/labs/qemu/x86_64/user_extent_storage_observer.py.gz)
compares all 8192 process-table bytes and all 0x9000 selected filesystem bytes
with each operation's permitted transition. Rejected copies preserve the
complete selected table/store, including positions, object identity, resident
pages and data. Creation verifies previously free description/object storage,
the lowest free descriptor, fresh identity and mode. A newly associated page
must have been free and clean, acquire the exact object/logical-page association,
and preserve all other associations and payload. Writes publish only their
returned prefix; reads compare actual output with retained contents. Positional
successes leave shared positions unchanged, and complete table comparisons also
cover rejected operations.

The observer checks the readable portions of user copy windows on 164 requests,
including unchanged surrounding bytes where mapped. It does not read or claim
observation of unmapped portions. Immutable pread compares with the actual
booted application's header supplied by the host-side image/archive check.
Directory output is independently constructed from native inode, offset, type,
name and padding rules. Transfer staging and directory staging return to their
preceding clean state. Timestamp checks permit only the contract's designated
fields to change and require positive observations; they add no independent RTC
qualification.

Each execution retires the anonymous file's page, association, allocation bit,
object and description. The complete table and filesystem return to their
preceding state except for the next-identity counter advancing once. Existing
other filesystem owners remain unchanged. The enclosing fixture also checks
user registers/stack, heap prefix, command state and terminal resource cleanup.
This is selected copy/position/ownership-transition evidence, not all sparse,
capacity, append, alias, fork, namespace, device or filesystem behavior.

[Independent readback](readback.json) verifies all **10080 linked text bytes** in
the executable user load, the native four-argument syscall adapter, C source
identity and a conservative **1320-of-2048-byte** source-stack bound. All 17
selected kernel objects/images are byte-identical to the
[preceding pipe checkpoint](../stage-3-source-pipes/README.md). The application
image intentionally changes for this cohort; its initial ELF header and the
complete 7606-byte serial result retain their preceding bytes. The default ABI
fixture still passes its exact committed digest test.

This cohort [reaches 81 sites](guest/observations.json) in the same 91-site emitted
call census. Six are newly observed: volatile read/write/pread/pwrite,
immutable-file pread and named-directory publication. Exact kernel/census
identity checks establish an **87-of-91 union** with the preceding poll and pipe
cohorts. The four remaining sites are serial descriptor read and three bounded
fallback inputs. First reach is not admission/return qualification of every
length, error path or direct user access; no checks are deleted on that basis.

The native test passes in **142.38 seconds**. Peak use, including compilation,
was 1027272704 bytes. All [six post-checks](post-validation.json) pass, with peak
565854208 bytes: default/source-fixture build tests, workspace formatting,
explicit included-file formatting, locked workspace/all-target checking, Rust
file sizes and whitespace. Both runs retain the ordinary bounded wrapper and
host reserve, with zero validation swap and zero limit/OOM/kill events.

Risk remains high for the underlying access/ownership contract. This stage is
test-only and separate artifact comparison verifies unchanged kernel bytes.
Review was single-agent authority/source review plus independent state and ELF
readback; no independent human security review is claimed. The first native
attempt and post-checks passed without a failed retry in this checkpoint.

[Selected input snapshots](source-identity.json), [scope](scope.json) and the
[manifest](manifest.json) retain exact evidence without claiming a complete
producer census or committed cold reconstruction. Stage 3, native ownership/
mode/stability qualification, canonical production binding and the original
kernel objective remain unfinished.

To repeat from the repository root with LLVM tools, a C compiler, QEMU and GDB:

```sh
rtk proxy env RUST_MIN_STACK=16777216 scripts/run_bounded.py -- cargo test -p topal-compiler --lib --locked -j1 systems_frame_target::root_tests::owned_source_fixture_checks_storage_transfer_boundaries -- --ignored --exact --test-threads=1 --nocapture
```

The test prints its build path under `linux-kernel/.cache/x86_64`. Retained
runners additionally capture selected source/resource identities; restore their
original cache-relative helper paths when replaying them.
