# Source ELF64 header policy over an owned snapshot

`TK-DEC-260`; systems-only refinement adopted under standing authorization.
Design adopted; implementation and physical qualification remain required.

The next boundary moves actual executable-header interpretation into an
ordinary source module. Add an optional `image-library` declaration with the
versioned `static-elf64-header-v1` interface and the existing exact 4096-byte
stack and 65536-byte temporary-storage limits. Its public ordinary export has
the declared type `(Array (64, Int), Nat, Nat) -> Boolean`: copied bytes,
expected ELF machine identity and maximum admitted program-header count.
Every byte value is an ordinary integer in 0..255; the two caller facts are
in 1..65535. Source receives no address, resource, context or memory view.

The private native adapter receives the owned immutable header address, exact
length 64 and the two facts. It checks private input shape before access and
copies all bytes into invocation-owned ordinary List/Array representation,
using shared integer boxing and collection algorithms. Byte order is preserved.
Caller resource validation must establish the complete readable immutable
extent and lifetime before invocation. The snapshot is not a grant to inspect
arbitrary addresses, user memory or MMIO. The original file remains unchanged;
no source pointer or aggregate result can escape the Boolean result adapter.

The source policy preserves the currently admitted header checks: ELF magic,
ELF64 class, little-endian encoding, current identification/version, ET_EXEC,
the supplied machine identity, 64-byte header, 56-byte program-header entries,
and a program-header count in 1..the supplied maximum. It does not strengthen
checks on otherwise unexamined identification padding/ABI bytes or claim full
file/segment validation. Complete load-plan, bounds, mapping and artifact
ownership checks remain with the existing preparation path.

At same-image exec, invoke the policy after successful pathname/vector/
environment staging and before close-on-exec filtering or any destructive image,
signal or generation transition. False produces ENOEXEC with staging erased and
all retained process state/resources preserved. Invalid private bindings,
ownership, reentry, exhausted resources or malformed native results fail closed.
The new declaration must never silently use a native fallback if source loading,
checking or publication fails. Roots without it retain their previous contracts.

Header and deadline policies must work independently or together. They share
one root-owned temporary arena through exclusive non-overlapping invocations;
they do not exchange retained source values. Ownership/initialization, active
policy identity, code bindings and terminal erasure cover the complete selected
policy set. Header-only numeric operation must not require a deadline policy.
Each compiled unit receives a distinct private code/runtime symbol namespace
so duplicate local helpers cannot confuse linked-code or frame inspection.
Shared runtime algorithms remain unchanged; namespace adaptation and all
generated bridge work are included in digested publication and proof.

The allocation proof includes every copied input box, List node, Array header,
native-fact box, source evaluation and owner header, including invalid headers.
Complete actual stack/call evidence includes the snapshot bridge, source/module
helpers, runtime callees, returns and alignment. Both existing limits remain.
All copied values, retired stack and active invocation state are erased on
acceptance and refusal; every selected binding is erased at terminal return.

Qualification must compare actual source Array bytes and caller facts against
the owned file and independent protocol decoding; inspect imported source
frames; execute independent valid/mutated-header cases and private-input guards;
prove source refusal before commit with unchanged file/retained process state;
exercise a header-only numeric configuration; and reject a changed source
helper at the independent header oracle. A controlled copied-byte fault may
exercise refusal without modifying the immutable file. It must be reported as
controlled input, not as support for arbitrary executable files. Retain the
complete previous runtime corpus, source snapshots, isolation and erasure tests.

Keeping header parsing in native policy would leave source unable to consume
owned bytes. Passing a raw address into Topal would create a new authority and
lifetime boundary. A copied semantic Array reuses approved collection meaning
and makes that boundary explicit at the existing native adapter. The ELF64
format is an external protocol, not an x86 storage primitive: AArch64 and RISC-V
can use their machine identities with the same semantic snapshot. Other ELF
classes/encodings require their own qualified interfaces, without changing core
Array semantics or claiming their implementation here.

Risk is high at input-resource lifetime, copying/representation, simultaneous
policy publication, namespace uniqueness, complete resource bounds, refusal
atomicity and teardown. Review must follow source semantics through generated
bridge/runtime code, native ownership checks, independent fixture encoders and
actual storage/state reclamation. General ELF mapping, persistent subsystem
state/effects, writable storage and distribution/container/VM-host acceptance
remain required by the [Step 4 status](../step-4-status.md).
