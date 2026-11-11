# Source-built serial descriptor transfers

This checkpoint exercises `TOPAL-SYSTEMS-USER-EXTENT-001` and the serial
description retained by `TOPAL-SYSTEMS-STATIC-PROCESS-FD-TABLE-001`. It adds test
construction and observations, changing no kernel protection rule, production
selection or core-language meaning. Stage 3 and the original kernel objective
remain incomplete.

The [C component](serial-cases.c) executes four times in the enclosing process
fixture. It grows its heap through `brk`, announces readiness through ordinary
stdout writes and receives known bytes through ordinary stdin reads. The host
sends each eight-byte payload only after receiving its application marker over
the existing VM serial connection. The [eight-message record](guest/serial-dialogue.json)
retains exact markers, payloads and serial-output positions. Bootstrap input and
the original command continue through their existing protocol.

Each source execution receives a first byte before issuing five invalid-buffer
reads, two invalid-description reads and a zero-count read. It then receives
the remaining seven bytes unchanged, including a directory-crossing destination
beyond the predecessor heap. A second payload exercises a 512-byte request
whose existing serial quantum admits 256 mapped bytes, while returning only
the eight available bytes. The program preserves surrounding bytes, accepts
positive short progress in its receive loops, and restores its original heap
prefix and break. No completed application is patched or host result substituted.

The [native run](native-test.log) passes **64 source-issued syscalls**: twelve
heap operations and 52 I/O requests. There are twenty `EFAULT` refusals, eight
`EBADF` refusals, four zero-count reads and twelve positive reads. The kernel
consumes **64 UART data bytes** and returns those same 64 bytes to the application.
No UART data byte is consumed by a refused or zero-count request.

The [native-site binding](serial-native-sites.json) locates the selected serial
extent-helper call and the actual `inb`/following store in the linked kernel.
It checks their instruction boundaries against LLVM's provider disassembly,
not a source-supplied instruction address. Ordinary debugger breakpoints
observe the call's candidate/length and the byte returned by the UART instruction
before it is staged. They require kernel privilege, CR0.WP, the UART data port
and the expected kernel staging address. They never assign guest registers,
mappings, ownership or application data.

The observer independently matches consumed bytes against the host dialogue and
actual user output. It verifies complete process-table preservation, clean
staging, unchanged historical control state and readable user copy windows on
all 52 I/O requests. Unmapped portions of windows are not read or claimed
observed. Four requests record the exact 512-to-256 admission; surrounding and
unreturned bytes remain unchanged. The enclosing fixture separately checks
register/stack preservation, heap prefix, command state and terminal retirement.
This does not qualify complete tty, termios, interrupt, nonblocking or device
driver behavior.

[Independent readback](readback.json) verifies all **8343 linked user text
bytes**, the syscall adapter, source identity and a conservative
**1304-of-2048-byte** source-stack bound. All 17 selected kernel objects/images
are byte-identical to the
[preceding storage checkpoint](../stage-3-source-storage/README.md). The application
image intentionally changes. The serial result grows from 7606 to 7782 bytes
solely through the eight application markers; removing those exact markers
recovers the preceding serial result byte for byte. The default ABI fixture
still passes its exact committed digest check.

The [observations](guest/observations.json) reach 76 sites in the unchanged
91-call emitted census. The serial descriptor site is new relative to the
preceding three cohorts, producing an **88-of-91 matching-kernel union**.
Three bounded fallback inputs remain unobserved. First reach cannot establish
load/store dominance or complete length, direction, error and lifetime coverage.
No old check is removed or production gate discharged on that basis.

The native test passes in **144.64 seconds**, with peak 988024832 bytes including
compilation. All [six post-checks](post-validation.json) pass: default/source
fixture build tests, workspace and included-file formatting, locked workspace/
all-target checking, Rust source sizes and whitespace. Their peak is 547930112
bytes. Both runs retain the ordinary bounded wrapper and host reserve, with
zero validation swap and zero limit/OOM/kill events. The first native attempt
and post-checks pass without a failed retry in this checkpoint.

Risk remains high for the underlying user-access contract. This increment is
test-only, and separate artifact comparison confirms unchanged kernel bytes.
Review was single-agent authority/source review plus independent instruction,
state, serial and ELF readback; no independent human security review is claimed.
The harness's optional dialogue is empty for previous modes, preserving their
existing protocol. The serial test itself also exercises that protocol before
its additional markers.

[Selected snapshots](source-identity.json), [scope](scope.json) and the
[manifest](manifest.json) retain exact evidence without claiming the complete
producer or committed cold reconstruction. Remaining direct/shared-access,
ownership/mode/stability and canonical production gates remain required.

To repeat from the repository root with LLVM tools, a C compiler, QEMU and GDB:

```sh
rtk proxy env RUST_MIN_STACK=16777216 scripts/run_bounded.py -- cargo test -p topal-compiler --lib --locked -j1 systems_frame_target::root_tests::owned_source_fixture_checks_serial_transfer_boundaries -- --ignored --exact --test-threads=1 --nocapture
```

The test prints its build path under `linux-kernel/.cache/x86_64`. Retained
runners additionally capture selected source/resource identities; restore their
original cache-relative helper paths when replaying them.
