# Stage 3: bounded random output paths

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-STATIC-ELF-REUSABLE-GETRANDOM-001`.

The [conditional reader](../../audit_random_outputs.py) connects the bounded
random service's writable admission to its initial and reusable output loop.
Exact guards require `1 <= RSI <= 16`; the candidate is retained in `RDI`.
Admission and both count guards dominate the selected byte store/readback.
The loop starts at index zero, checks completion before accessing memory,
advances once and uses only `candidate + completed` with
`0 <= completed < admitted <= 16`. Private source-buffer expressions remain
owner/nonalias obligations.

The reusable path saves the current stack pointer in `GS:272`, switches to the
retained privileged stack, saves four words, calls the actual extractor,
restores the words/stack and clears the suspended-stack field. The selected
maximum borrowed stack cost is **168 bytes**: 32 saved bytes, an eight-byte
callee return word and 128 bytes of scratch. The initial path uses the retained
initial output without invoking extraction.

The [extractor effect reader](../../random_extract_effects.py) covers all
**1843 instructions** in the stack-local leaf. Closed CFG/frame dataflow checks
that `RSI`/`RDI` remain unmodified, explicit private accesses fit the selected
32-byte key/output expressions, every return restores entry RSP and both
allocated paths erase all 128 scratch bytes before releasing them. The scratch
starts unqualified; only proved zero stores clear its dirty-byte mask.
This is a conditional static effect/retirement result, not a cryptographic
proof or native observation of every failure path. Actual key/output/stack
owners and nonaliasing remain required. **Erasure of the caller's 32 retired
saved bytes and return words is unverified**, as is the preceding clock's
96-byte spill cleanup.

The [read-only observer](../../../../labs/qemu/x86_64/user_extent_random_observer.py)
checks helper/captured-register preservation, preservation of the observed
private bytes, marker and service reference on refusal, actual output source,
restored stack/suspended-stack field, every
indexed store and readback, exact opaque payload and physical source/destination
exclusion. Complete table observations preserve all entries and bits except
recorded monotonic Accessed/Dirty sets on the actual admitted writable user
destination leaves. The approved design treats those hardware status updates
separately from permission changes. Permission, backing, NX, page-size,
bit-clearing, unrelated-entry and stale-cache changes remain rejected.
Equal boundary observations do not prove continuous stability or actual owners.

The [ordinary buffer VM](native/source/run.json) and
[independent saved-record join](native-readback.json) establish:

| Result | Calls | Bytes copied |
| --- | ---: | ---: |
| Refused admission | 40 | 0 |
| Initial output | 1 | 8 |
| Reusable output | 7 | 56 |
| Early zero/invalid-flag/excessive-count results | 24 | 0 |

All eight completed requests use length eight. Their 64 stores and 64 readbacks
match exact index sequences and hashes of the bytes actually returned through
the ordinary source program. Five invalid candidates per startup request
include unmapped, noncanonical, read-only, numeric-limit and page-crossing
failures. The early results perform no admission. Other native lengths remain
an acceptance gap; models and exact loop bounds cover one through sixteen.
Protected private-state and mapping assertions retain their live observer
evidence; the saved-record join is not another native run.

The fresh [storage](native/storage/run.json) and unchanged
[pinned BusyBox](native/pinned/run.json) regressions and two
[fixture build tests](native/fixtures/run.json) also pass. Each of these four
commands retains the same eight recorded input versions without changes during
execution, zero validation swap and zero memory-event counters. After those
commands, one **nonexecuted Python model-test file** receives a one-line mock
namespace repair. [Source identities](source-identity.json) retain its exact
old version and prove the isolated change; all other recorded inputs still
match. The Rust native targets run explicit audit/observer scripts and do not
discover that Python unittest module. All **121 final model/reader tests** use
the repaired namespace and pass.

[GNU readback](gnu-operands.json) independently agrees on register/memory/stack
effects for all 1843 extractor instructions and **1900 selected instructions**
overall, including switched-stack/count/candidate/loop operands and three reverse
guard cuts. It uses the preceding identical-kernel full transcript of 31,024
instructions and 36,950 edges; this increment runs no new GNU decoder and no
second frame-dataflow algorithm. [Twenty-three offline ELF controls](controls/controls.json)
reject changed bounds, candidate/count/source, stack fields/saves, loop index,
store/readback, carry, allocation, zero register, final-word erasure and a
coherently redirected wipe-bypass edge. Changed images are never executed,
booted or loaded into a VM.

[Actual artifact comparisons](artifact-comparison.json) preserve seventeen core
objects/images and fifteen preceding metadata/reports per fresh cohort and 33
other inherited storage artifacts. [Intermediate ELF comparison](source-intermediate-comparison.json)
preserves all headers, allocated sections and load bytes while retaining the
three differing nonallocated DWARF sections under their own identities. The
installed storage fixture/boot bytes and pinned third-party bytes are unchanged.
No third-party ELF/boot archive is copied here. Executable kernel bytes, core
language, authoritative design meaning and canonical production selection are
unchanged.

The [conditional index](caller-access-index.json) now covers **90/91 helper
sites and 325 distinct user-access instructions**. The named-root fallback,
outside the listed direct graph, still lacks a selected interval. All three
fallback obligations remain separate. [First reach](native-coverage.json)
remains 83/91 fresh and 87/91 with explicitly bound preceding same-kernel cohorts.

The archive binds 164 selected sources, eight local and 156 inherited, plus the
separately recorded old model-test version. [Thirty-six artifacts](artifact-identity.json)
include the executable/graph/origins, twelve reports, components, first-party
fixture and prior intermediate comparison input. [Frozen replay](replay.json)
exactly reproduces the source join, twelve reports, 121 models, saved native
and GNU readbacks, intermediate comparison and isolated model-version repair:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-random-outputs/replay-random-outputs.py linux-kernel/.cache/x86_64/random-output-replay-new
```

Use a new output directory. Replay executes no ELF, compiler, GNU decoder, GDB
or VM; it is not a complete clean producer.
[Archive validation](validate-archive.py) checks source/version/artifact/report/
native/index and local-link bindings. [Review](review.json) is by one agent with
higher-authority/source/full-diff review and separate effect/record interpretation.
Risk remains high for memory, lifetime and evidence errors.

History retains a synthetic control that initially failed at an earlier graph
check, the overly strict raw-table equality failure, the unchanged-kernel
diagnostic identifying the exact `0x60` destination-leaf change and the isolated
model mock-namespace failure. These are build/observer/model results; no
model/provider safeguard notice occurred. The strict table comparison was
replaced only after reviewing its complete preservation obligations and adding
the status-transition counterexamples.

Actual owners/nonaliasing, continuous lifetime, complete helper/entry/callee/
return/fatal/exception and whole-syscall retirement, unrun native lengths and
extractor failure paths, saved-word cleanup, the remaining site, fallback
obligations, further profiles and canonical clean production qualification remain
open. Stage 3, initial Step 4 and the original Linux-compatible kernel goal
remain unfinished.
