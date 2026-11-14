# Explicit boot participant source protocol

`TK-DEC-294` applies the systems-only authorization to the source/IR boundary of
[boot participant arrival](boot-participant-arrival.md). Core language semantics
and the eight existing source handler roles keep their meaning. The initial
entry is a typed provider entry descriptor, selected by purpose rather than
source-specified machine state.

Inside the retained-image-table scope, immediately after its retention
observation and before translation construction, a boot allocator may request:

```topal
memory boot participants initialize (entry is provider-selected)
  Ok arrivals then {
    memory boot participants retain arrivals for-kernel-lifetime
    memory console write "TOPAL_BOOT_PARTICIPANTS_ARRIVED"
    // The existing checked translation/boot continuation follows here.
  }
  Error failure then {
    failure fatal "boot participant initialization failed"
  }
```

Initialization requires the unborrowed initial allocator and a complete
retained boot/image binding. Success returns one affine `BootParticipantArrivals`
resource. Its typed `BootParticipantEntry::InitialProviderSelected` descriptor
names `topal.systems.entry.boot-participant/1`. The qualified provider must bind
that entry to each actual participant's local state and supporting whole owners
under `TOPAL-SYSTEMS-BOOT-PARTICIPANT-ARRIVAL-001`. Source receives no CPU count,
native identity, entry address, stack pointer, controller or register interface.
The entry descriptor supplies no evidence that any CPU has executed.

The exact allocator and returned resource must be consumed once into a retained
boot domain before the arrival observation and ordinary continuation. The
domain's entry/root/stack/mailbox/exception owners remain pinned until its
terminal kernel lifetime ends. This consumer releases no backing. There is no
source retry, post-arm cancellation, reclamation, scheduler or hotplug operation.
The failure branch must enter fatal; after possible entry effects, the provider
retains/poisons all affected resources even if an initialization result failed.

The source checker rejects altered entry policy, geometry fields, foreign or
ambiguous resource/context names, missing/duplicate/interleaved consumers and
placement outside that initial retained scope. Semantic validation independently
checks the same ordering and pending consumption, including manually assembled
IR. An ordinary operation closes the initial admission window; initialization
cannot be moved later merely because the allocator becomes unborrowed again.
No existing source root requests initialization implicitly.

Checked IR keeps a distinct initialization operation carrying the typed entry
descriptor and failure disposition, followed by a kernel-lifetime retention
operation. The transition trace records a *requirement* for actual arrivals and
retained ownership. It predicts no participant identities, successful CPU
execution or protection. The separate portable arrival model continues to
cover dispatch/entry/acknowledgment/completion and partial-failure ordering.

The native compiler must refuse this selection until whole startup storage,
target-local typed entry, per-CPU capabilities/mode/exception paths, ordered
acknowledgments, bounded deadlines and fatal/lifetime behavior are qualified.
Neither a planning entry nor an ignored operation may stand in for native work.
This increment therefore checks source/model meaning while native publication
remains explicitly unavailable. Existing profiles retain their provider maps.

Future x86 implementation must preserve the inactive table scope's existing
contract: its banks remain inactive and unreferenced by participant entries,
so their ordinary terminal discard remains valid. Participant roots and other
supporting storage need their own retained domain binding. Protected activation
later requires explicit integration of every active/restorable root, alias,
writer, CPU and asynchronous path; arrival alone grants none of that authority.
Other architectures realize the same entry/domain/lifetime purpose using their
own qualified firmware and translation mechanisms.
