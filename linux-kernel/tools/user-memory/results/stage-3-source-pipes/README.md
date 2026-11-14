# Source-built owned FIFO transfer cases

This checkpoint exercises `TOPAL-SYSTEMS-USER-EXTENT-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-{001,TRANSFER-001}` through ordinary application
syscalls. It adds no kernel protection rule or core-language change. The selected
owned profile remains experimental; caller migration, production qualification
and the original kernel objective remain incomplete.

The new [C component](pipe-cases.c) runs four times in the enclosing process
fixture. Each execution grows the heap through the ordinary `brk` interface,
issues 36 calls, restores the original heap bytes and break, and returns through
the existing source/backend ABI adapter. No finished application binary is
patched, no debugger assigns guest state, and no signal delivery is requested.

The [final native run](native-test.log) passes **144 source-issued calls**,
including 132 FIFO requests: 76 `EFAULT` buffer refusals, four nonblocking empty
reads, sixteen positive byte transfers, four pair creations and eight closes.
The remaining requests query the available count or observe EOF. Cases cover
partial/unmapped, wrapped, read-only and supervisor-only outputs; invalid input
extents; read-only source data; page/directory crossings beyond the predecessor
heap; short reads limited to available bytes; and unused unmapped destinations
on empty/EOF reads.

The [observer source](inputs/linux-kernel/labs/qemu/x86_64/user_extent_pipe_observer.py.gz)
compares all 8192 process-table bytes and all 0x5000 shared FIFO bytes with the
operation's permitted transition on every FIFO request. Creation requires
previously free description storage and the lowest free descriptor pair.
Refusals preserve table, FIFO payload, cursors, count and generation history.
Successful reads verify actual output, FIFO order and consumed-ring erasure.
The observer checks readable portions of the surrounding user-buffer window on
124 requests; unmapped portions are not read or claimed observed. Successful
transfers permit only their designated timestamp fields to change, requiring
positive observations; this is not a new independent RTC qualification.

Each source execution closes both endpoints and verifies exact restoration of
the process table and shared FIFO storage, with only the generation counter
advancing by one. The enclosing observer also verifies heap, command, register,
stack and terminal resource retirement. These are selected nonblocking cases,
not complete pipe capacity, alias, fork/wait, signal, wrap or every buffer/error
qualification.

[Readback](readback.json) independently verifies all 9009 linked text bytes in
the executable user load, source identity, the native syscall adapter and a
conservative **1352-of-2048-byte** user-stack bound. All 17 selected kernel
objects/images are byte-identical to the
[preceding census checkpoint](../stage-3-call-census/README.md); the unchanged
7606-byte serial output remains part of the enclosing fixture. The application
image intentionally changes to include the new source program. The default ABI
fixture still passes its exact committed digest check.

The [observations](guest/observations.json) reach 79 of the same 91 emitted
extent-helper call sites. Four are new relative to the preceding 77-site
cohort: pipe creation, read, write and FIONREAD. Matching kernel/census identities
establish an **81-site union**, leaving ten unobserved sites in these two
cohorts: three bounded fallback inputs, positional/serial descriptor reads,
named-directory publication and four volatile-file transfers. First reach
remains separate from complete buffer/access qualification.

The final native command takes 144.53 seconds, with peak 370196480 bytes,
zero validation swap and zero limit/OOM/kill events. All
[six post-check commands](post-validation.json) pass, with peak 554610688 bytes
and the same zero swap/events. They cover source-fixture build/default-digest
tests, workspace and included-file formatting, locked workspace/all-target
checking, Rust file sizes and whitespace. These Rust checks preceded the final
Python observer refinement; the final native run executes that exact refined
observer. Every command used the ordinary bounded wrapper and host reserve.

Earlier attempts retain their distinct status:

- [Compilation](earlier-attempts/compile/test.log) rejected a string initializer
  that omitted room for its terminator. The corrected array retains the tested
  sixteen-byte payload and its separate terminator.
- [The timestamp observer](earlier-attempts/timestamp-observer/test.log) wrongly
  expected the fourth creation timestamp to be zero. Its original source and
  failed guest observations are retained; the kernel did not change.
- [The intermediate passing run](earlier-attempts/before-window-review/test.log)
  passed 144 calls before review added previously-free-description and user
  window checks. It is superseded by the final run, not presented as evidence
  of the later observer checks.

Risk remains high for the underlying memory/ownership contract. This increment
changes only test construction and observation; separate artifact comparison
confirms unchanged kernel bytes. Review was single-agent authority/source review
and independent artifact/state readback, with the two additional checks added
during review. No independent human security review is claimed.

[Selected source snapshots](source-identity.json), [scope](scope.json) and the
[retained-file manifest](manifest.json) distinguish this checkpoint from a
complete producer census or committed cold reconstruction. All remaining direct
user-access, error/lifetime, ownership/mode/stability and production gates remain.

To rerun from the repository root with LLVM tools, a C compiler, QEMU and GDB:

```sh
rtk proxy env RUST_MIN_STACK=16777216 scripts/run_bounded.py -- cargo test -p topal-compiler --lib --locked -j1 systems_frame_target::root_tests::owned_source_fixture_checks_pipe_transfer_boundaries -- --ignored --exact --test-threads=1 --nocapture
```

The test prints its build path under `linux-kernel/.cache/x86_64`. Retained
runner scripts additionally capture source/resource identities; their original
cache-relative helper paths must be restored for replay.
