# Step 4 writable initial user data

## Outcome

The qualified systems profile now distinguishes verified immutable data from
verified writable non-executable data. The initial user ELF declares its data
segment `PF_R|PF_W`; after the provider has copied and verified initialized
bytes and zeroed BSS while the mapping is RO/NX, checked source explicitly
promotes that mapping to the writable-data role. The x86-64 provider adds write
authority, retains NX, performs the required fence and target invalidation, and
returns a distinct affine owner.

The constructed application proves the permission is useful rather than merely
encoded: it observes a zero BSS byte, writes `0xa5`, reads the value back, and
only then proceeds through the existing Linux process-entry and syscall probe.
No core-language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-WRITABLE-DATA-001` requires writable data to begin from a
verified user data mapping, preserve user and read authority, add write
authority, keep execute absent, invalidate the affected translation, and
consume the prior owner. The portable semantic operation names that transition;
the x86-64 provider alone owns the PTE bit manipulation, `SFENCE`, and `INVLPG`.

Artifact schema `/52`, provider `/49`, provider object `/49`, artifact root
`/50`, boot adapter `/12`, and gate `/54` carry the new transition and evidence.
The boot-resource schema remains `/1`. The artifact trace contains 197 semantic
events, 63 placements, and 1,794 relocation applications; the generated adapter
contains 1,608 UART writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 49 ordered markers. The supplied ELF's writable segment passed
the independent envelope validator, exact publication and packaging, runtime
segment validation, complete initialized-data/BSS preparation, permission
promotion, ring-3 write/read proof, and final ownership reclamation. The
evidence records:

- linked kernel: `7e924bcfb716f0114203f79c6f5c5a53c5523335abd900ffbe41c91901ef666d`;
- boot image: `2aebc24466d8fa3bba70415525a0c9aa25c5fbb505d55cf25ce24b63e0e46667`;
- boot resource: `75ddbda3d5dfec735672e808ad868d31e96c9a848ab2a82be2a68fab7c25aa12`;
- generated default image: `ee8ddc48f0c469a11626998b119ef9db3df35057c2bcfc6126b24f6482d32cf6`;
- supplied image: `85bdcfe5aeb9567517dd6e4bcaa268ef0d6860fd9aaaa04b8be9bdba31c72840`;
- artifact provenance: `0567c628c2dc2ff1e3b06d9c92f1b0a3be985ea99ea88efce768a4d8d851a159`;
- boot provenance: `e1edb7148da28f519a92b937fe00d80561186ae9279ec940748962b23fc799fd`;
- serial: `ca23537a4a4e6d354e0e39a708d5c46895929c41cf79328d84c907e956ad4630`;
- evidence record: `c39c63163fe66cfaa4866cf5cb163cfcf7e995a89ca63cd081920264c34905e9`.

## Risk and review

Risk is high because the change grants ring-3 write authority and changes an
ELF load-segment contract. Review covered authority direction, affine state
consumption, W^X preservation, target invalidation, ELF flags, BSS initialization
before promotion, a positive ring-3 mutation, fail-closed mismatches, complete
workspace validation, and two independent physical runs. The semantic operation
is architecture-neutral; only its x86-64 realization is implemented.

## Remaining boundary

The initial process can mutate one bounded data page, but it is still a sealed
static ELF with no general loader, dynamic linker, heap growth, TLS, files,
signals, threads, or process creation. The syscall set remains a qualification
subset rather than the complete pinned Linux ABI.
