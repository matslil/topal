# Stage 3: bounded serial input and partial output

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The [serial prefix reader](../../audit_serial_prefix.py) checks the actual
descriptor receiver's zero-length exit, unsigned 256-byte cap, output admission,
staging wipe, UART receive loop, prefix publication and staging cleanup. The
zero guard's fallthrough and admission success edge dominate the selected copy;
the exact local instructions have no calls or alternate incoming entries.

At each staging byte store, `RSI = R12+1408+received` and
`0 <= received < admitted <= 256`. A byte increments received once. The receiver
publishes at the cap or when the UART has no next byte, so publication has
`1 <= received <= admitted`. It copies exactly that prefix using the admitted
R8 candidate and R15 count, with DF clear. UART waiting performs no memory store
and has no eventual-progress guarantee. Numeric staging expressions and closed
register effects do not establish private owners or nonaliasing.

The [copy observer](../../../../labs/qemu/x86_64/user_extent_copy_observer.py)
accepts a positive prefix only for a report explicitly carrying that contract.
Ordinary exact copies retain their count-equality rule. Actual R15/RCX, helper
registers, captured arguments, copied bytes, translation permissions and
nonoverlapping selected physical ranges are checked at observed boundaries.
Equal boundary observations are not continuous mapping-lifetime evidence.

The unchanged [source-built serial fixture](../../../../labs/qemu/x86_64/user_extent_serial_cases.c)
passes four executions and **64 syscalls** in the [fresh VM run](native/serial/run.json).
The selected helper has 28 attempts, twelve completed copies, sixteen refused
admissions and 64 copied bytes. Each 512-byte request admits 256 bytes and
returns eight bytes; all four short outputs preserve the untouched suffix.
Existing source observations also verify actual UART bytes, syscall arguments,
the full selected process table, protected context, phase and retirement.

[Separate saved-record readback](native-readback.json) reconstructs all 28
admission records and the source/UART payload sequence, and reconciles aggregate
REP attempts/completions/refusals/bytes. The existing serial observer persists
per-syscall admission and UART records; the generic REP observer retains counts
and a first sample, not a complete saved per-copy event transcript. This is not
a second native execution or an independent reconstruction of every translation.

[Independent GNU readback](gnu/readback.json) agrees on 56 instruction boundaries
in two windows, selected zero/cap/receive/publication operands and reverse zero/
admission cuts. [Twelve offline ELF controls](controls/controls.json) reject
changes to cap/zero/count/candidate/staging/increment/DF/carry instructions after
outer artifact identities are rebound. Changed control images are never
executed, booted or loaded into a VM.

The [unchanged pinned BusyBox regression](native/pinned/run.json) also passes.
Both final native commands use the same five frozen changed inputs; validation
swap and all recorded memory-event counters are zero. Seventeen kernel objects
and images per cohort match the preceding stat checkpoint byte for byte.
[Artifact comparisons](artifact-comparison.json) retain those identities.
No pinned third-party application binary is copied into this archive or changed.

The [joined conditional index](caller-access-index.json) contains **84/91 helper
sites and 311 distinct user-access instructions**, leaving seven sites without
intervals. [Fresh first-reach observations](native-coverage.json) are subsets of
the preceding matching-kernel 88/91 union. Kernel executable bytes, canonical
selection and authoritative language/system meaning are unchanged.

[Source identities](source-identity.json) freeze 149 selected inputs: five local
and 144 inherited snapshots. [Seventeen selected artifacts](artifact-identity.json)
bind the shared executable/graph/origins and eight local-contract reports.
[Frozen replay](replay.json) reproduces the source join, all eight reports,
**88 model/reader tests** and saved serial-record correspondence without
executing an ELF, compiler, GNU decoder, debugger or VM:

```sh
rtk proxy env RUST_MIN_STACK=16777216 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-serial-prefix/replay-serial-prefix.py linux-kernel/.cache/x86_64/serial-prefix-replay-new
```

Use a new output directory. This is selected evidence reconstruction, not a
complete clean producer. [Archive validation](validate-archive.py) checks current
selected sources, snapshots, report/index bindings, native identities and links.

The [review](review.json) retains three failed verification attempts: the first
control runner stopped at an incorrectly selected MOV byte; a misspelled test
selector ran zero tests; and the correct pinned test initially lacked its
required image-path environment variable and failed before VM execution. The
corrected commands pass. The focused runner now rejects zero selected tests.
The zero-test command's nominal exit zero is explicitly not validation.
No model/provider safeguard notice occurred.

Risk is high if these conditional intervals are overstated. Review is by one
agent, with authority/source/full-diff review, counterexamples, independent GNU
operands/cuts, ordinary native execution and saved-record reconstruction.
Actual owners/nonaliasing, continuous check-to-copy lifetime, complete helper/
entry/callee/return/fatal/exception contracts, seven remaining sites, all three
fallback obligations and canonical production reconstruction remain required.
Stage 3, initial Step 4 and the original Linux-compatible kernel goal remain
unfinished.
