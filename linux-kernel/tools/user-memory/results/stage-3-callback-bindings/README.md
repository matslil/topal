# Stage 3 source-controller registration and callback observations

`TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-POLICY-LINKED-INPUTS-001`,
`TOPAL-SYSTEMS-SOURCE-LIBRARY-001`, `TOPAL-SYSTEMS-NUMERIC-RUNTIME-001`.

The [binding record](source/callback-bindings.json.gz) follows the five actual
source-controller code relocations into the bootstrap's thirteen stack pushes,
their native argument positions and the process's initial private-slot stores.
The six indirect-call slices load those slots into RAX, test for null and align
the call stack; decoded direct branches do not enter their interiors. This
binds the selected registered entries, without creating authority from metadata.

| Private slot | Bootstrap argument | Existing source controller |
| --- | --- | --- |
| 952 | 13 | Eight-Boolean file-open plan; two call sites |
| 960 | 14 | Relative sleep deadline over ordinary numeric facts |
| 976 | 16 | Interpretation of copied immutable executable-header bytes |
| 984 | 17 | Normalized syscall-number to semantic-operation selection |
| 992 | 18 | Structured process-identity proposal with owned retained state |

The inspector independently resolves 978 allocated-section relocations in all
five actual source objects and compares every relocated byte to the linked
kernel: 389 functions in 357 allocated code/read-only-data sections, totaling
38541 bytes. It also verifies source/module/object digests and actual root
argument addresses. Existing recorded stack/call certificates remain identified;
their full call/resource/effect proofs are not independently repeated here.
ELF section flags describe artifacts, not hardware page permissions.

The final [source-storage run](native-run.json) passes its existing 208 ordinary
source-issued syscalls and all six callback sites. The [observations](source/guest/observations.json.gz)
record 1168 completed calls: 1080 dispatch, three header, 53 named-open-plan,
24 deadline, four identity and four anonymous-create-plan calls. Every observed
call checks the actual target against the registered linked entry, CPL0,
CR0.WP, aligned stack, supervisor-executable translation and unchanged CR3,
callee-saved registers, captured syscall arguments, callback registry and all
8192 process-table bytes on return. These checks allow the controllers' private
arena/retained-store work; they do not inventory every memory effect.

First use of each controller checks its complete live code/data section bytes
against the linked artifact. [Readback](readback.json) independently folds the
recorded three-level supervisor huge-page translations and compares all 357
live-section digests to actual ELF payloads. Effective user access is denied and
execution permitted. Supervisor write permission is present: this evidence is
not hardware read-only text or a proof of immutability over every execution path.
The header input is the actual immutable archive prefix; scalar/controller
results still confer no resource rights.

The final [unchanged pinned BusyBox exit](application-run.json) also passes,
with 77 completed dispatch calls and its existing signal transfer/terminal
retirement checks. Its other callback sites remain unobserved in that cohort.
All 17 compared kernel objects/images, both cohorts' application images and boot
resources match their respective preceding baselines. All 7606 source-console
bytes and 6999 pinned-application console bytes are unchanged. The pinned digest
remains `7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.

[Nine offline controls](controls.json) reject changed instructions in each of the
five controller slots, a literal, a relocated table entry, a bootstrap code
argument and an initial registration slot. Each corrects the outer kernel/hash
association before testing the stronger byte/argument binding. These are copies
of first-party artifacts only: none is booted or executed, and no pinned
third-party binary is modified. This is consistency evidence, not authentication
against replacing all source and inner provenance together.

The final source command takes 161.10 seconds, with peak 399228928 bytes;
the selected test takes 160.97 seconds. The final pinned command's selected test
takes 71.09 seconds, with command peak 383361024 bytes.
[Six final post-checks](post-validation.json) pass, including two fixture-build
tests, locked workspace/all-target checking, formatting, size and whitespace;
their peak is 55246848 bytes. All recorded native/post runs retain the normal
bounded wrapper, zero swap and zero limit/OOM/kill events. No host protection,
guest registers, mappings or private guest state is changed by the observers.

Earlier passing source-poll/pinned runs and their exact initial observer remain
under [initial scope](initial/scope.json). The first source cohort reaches five
sites. Source review found its unused five-bit policy bound incomplete: the
approved open-plan interface has eight Boolean facts, and anonymous creation
passes 41. The final observer checks that full domain and the exact creation
combination, and the final storage cohort reaches the sixth site. Earlier bytes
are not claimed as final-observer qualification. Host reader/readback corrections
are recorded [separately](exploration.json); no native failure or AI safeguard
denial occurred in these cohorts.

The [source record](source-identity.json) freezes 93 selected inputs, including
the actual Topal controllers and module bytes. It is not the complete producer.
[Artifact identities](artifact-identity.json) and [manifest](manifest.json) bind
the retained source/object/linked/native evidence. [Offline replay](replay.json)
reproduces the exact binding report from frozen first-party artifacts and tools:

```sh
rtk proxy python3 linux-kernel/tools/user-memory/results/stage-3-callback-bindings/replay-bindings.py linux-kernel/.cache/callback-binding-replay /path/to/llvm-objdump
```

Risk remains high around the memory boundary. This increment changes inspection,
observation and test integration; compared kernel bytes are unchanged. Review is
single-agent authority/source review, exact relocation/ELF reading, offline
counterexamples and native observations, not independent human security review.
Complete source/runtime/callee effects, alternate/exception entries, pointer
authority and admission dominance, stable mapping lifetime, every caller's
length/error/staging/cleanup obligations and native ownership/mode/stale-state
qualification remain required. The three fallback obligations, canonical
activation, exact committed production reconstruction and full original kernel
acceptance remain unfinished.
