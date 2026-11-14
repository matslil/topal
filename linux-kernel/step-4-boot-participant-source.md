# Checked boot participant initialization

`TK-DEC-294` implements the source/IR boundary of the
[arrival contract](design/boot-participant-arrival.md). The retained-table scope
can explicitly request `memory boot participants initialize (entry is
provider-selected)`, consume its exact affine `BootParticipantArrivals` resource
into kernel lifetime, observe arrival and enter the existing checked boot
continuation. The [source design](design/boot-participant-source.md) preserves
the unborrowed initial boot binding and makes all native state provider-private.

`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` trace through
`TOPAL-SYSTEMS-BOOT-PARTICIPANT-SOURCE-001` to the
[source checker](../src/topal-language/src/systems_boot_participants.rs),
[semantic ownership protocol](../src/topal-semantics/src/systems_boot_participants_model.rs),
typed entry descriptor, distinct operations/transitions and compiler refusal
boundaries. The decision register also indexes the preceding `TK-DEC-293`
arrival contract. Systems-only design follows the standing chat authorization;
core language semantics and existing handler roles remain unchanged.

## Source, ownership and native boundaries

Initialization is admitted immediately after the table-retention observation.
Its success branch consumes the exact allocator/arrival binding once, before
observation or ordinary work. Entry policy, extra native fields, foreign or
ambiguous names, missing/duplicate/interleaved consumers and late initialization
are rejected. Ordinary work closes admission even when the allocator later
returns to an otherwise matching ownership state. Semantic checking separately
rejects malformed manually assembled IR and use in other entry contexts.

`BootParticipantEntry::InitialProviderSelected` records the typed entry purpose
`topal.systems.entry.boot-participant/1`. It supplies no native address or evidence
of execution. `RequireBootParticipantArrivals` records a runtime obligation;
the trace predicts no participant identities or successful CPU execution.
`RetainBootParticipantDomain` preserves the supporting domain for kernel
lifetime and releases no backing. The future participant roots cannot refer to
the inactive banks discarded by the existing table-retention scope.

Both native provider-planning routes reject this selection until actual entry
qualification, and direct operation emission rejects it before writing any
instructions or relocations. Existing roots never initialize CPUs implicitly.
The failure branch must enter fatal; native partial effects still require
retaining/poisoning all affected authority under the arrival contract.

## Validation and review

The [receipt](tools/user-memory/results/stage-4-boot-participant-source.json)
records **14 commands passing over 1,002 unchanged watched inputs** in
**400.37 seconds**. Checks pass **661 frontend and 263 semantics tests**,
three participant compiler-boundary tests, four provider tests, six backing
tests, four legacy-owner tests, five canonical-layout tests, the default ABI
fixture and the legacy private build. Three backing tests remain ignored;
they are not counted as executed native evidence. Workspace/all-target checking,
Cargo and explicit included-file formatting, Rust file ceilings and **318
user-memory plus 83 process-memory models** also pass.

The increment adds five frontend, four semantic and three compiler tests.
They cover renamed affine bindings, native-policy/geometry substitution,
failure disposition, consumption/order, forged success IR, reclaimed admission,
foreign entry contexts, honest trace requirements, and native planning/emission
refusal. The complete-root fixture is shared between frontend and compiler
checks. No VM or participant startup executes in this stage.

The fresh legacy kernel is **1,767,200 bytes**, with unchanged SHA-256
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.
The build-only receipt confirms no VM execution. Peak/limit are
**2,514,309,120 / 3,323,703,296 bytes**, with zero memory events or validation
swap. Host protections and the pinned two-CPU VM configuration remain intact.

Reproduce with a fresh lab directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_boot_participant_source.py linux-kernel/.cache/x86_64/NEW-OUTPUT
```

Risk is high for affine lifetime and cross-layer IR meaning. A separate
single-agent review checked higher-level publication/ownership/fatal intent,
source branch/resource binding, independent semantic ordering, complete effects,
native refusal boundaries and unchanged legacy output. The independent arrival
oracles remain part of the memory-model regression; they do not establish
native clocks, memory ordering, actual entry or continuous exclusion.

## Remaining work

Implement and qualify owned x86 startup storage, actual per-CPU typed entry,
target-local capability/mode/exception setup, ordered acknowledgments, bounded
dispatch/arrival and retained failure/lifetime behavior. The compiler refusal
must remain until those obligations are met. Native failure/asynchronous and
continuous-lifetime gaps remain acceptance blockers.

Then integrate protected mappings across every active/restorable root, alias,
writer and participant. Step 4 and the original Linux interfaces, distribution,
Docker/Podman and VM-host acceptance remain unfinished.
