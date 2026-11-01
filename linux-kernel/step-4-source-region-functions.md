# Step 4: ordinary source functions over borrowed kernel regions

`TK-DEC-270`; source and standalone x86-64 object qualification. This stage
implements the [checked source boundary](design/source-kernel-region-functions.md)
over the [borrowed-region model](step-4-borrowed-region-model.md). Kernel-root
binding, invocation and physical reclamation remain required. Step 4 remains
in progress.

## Source and object boundary

The qualified compiler context selects one published ordinary
`(KernelByteRegion) -> Unit` entry by its declared signature. It instantiates
symbolic typed inputs without an executable source client or a source region
constructor. The normal application compiler does not admit the vocabulary.
Imported modules retain their own feature selection, visibility, frozen bytes
and debug source identities. Public module specializations are independently
checked with symbolic declared input types and no private caller declarations.

Count, byte load/store and contained read-only/read-write window operations use
ordinary Nat/Boolean/Optional meanings. Arbitrarily large native indices refuse
without truncation. Direct helpers execute through shared ordinary function
lowering. All views derive from one entry owner; the closed nonrecursive graph
propagates its Read/Write effects and rejects false empty `Effects ()` bounds,
captures and escaping views. Named argument helpers execute once in source
field order, including when that differs from canonical native slots.

The private 64-byte control and 64-byte view representation validates code,
owner, monotonic invocation generation, lifetime, extent/overflow and rights.
The caller must already own live mapped, disjoint owner/control/temporary
extents. Addresses and numeric owner labels confer no authority. The standalone
object does not establish kernel ownership or physical cleanup merely because
a C test caller supplies correct storage.

The builder binds source/modules, type/effects, runtime, target/data layout,
private ABI, emitted/retained LLVM, object/disassembly and complete proofs in
`topal.kernel-region-source-object.x86_64/1`. Inspection checks all code,
GPR-only instructions, relocations/calls, stack metadata, exports/imports and
absence of writable allocated globals/TLS. Only the two qualified temporary
allocation/fatal hooks remain undefined. The recorded complete stack bound
assumes each imported hook meets its declared eight-byte stack contract. The
C client's hosted failure exit is a refusal oracle; it does not qualify that
kernel hook or the physical kernel stack. A verifier reproduces the entire
source proof and inspected object in fresh work space; a record cannot qualify
an unchecked supplied object. Tool work paths are excluded from replay identity.

The reviewed shared numeric/collection bounds charge every argument/helper
evaluation, temporary and derived view. Primitive runtime bytes are pinned to
the cost review. Limits remain a 65536-byte arena, 4096-byte complete stack,
256-bit numeric proof budget and 256 issued view nodes, including retired
nodes awaiting arena erasure. These are conservative proofs, not measured peaks.

## Independent native evidence

The real [byte helper](kernel/support/bytes.t) reads and stores actual owner
bytes through an imported ordinary function. The qualification-only
[C caller](labs/qemu/x86_64/region_source_client.c) is never linked into a kernel.
Each primary/read-only/boundary/argument-order variant runs 18 positive calls
over nine extents with both rights settings, plus 12 pre-entry refusals. Owner
bytes and controls use shared mappings so fork cannot hide mutation. Empty
owners are inaccessible and read-only owners have actual read-only mappings;
complete payload/canary comparison checks every byte. The allocator enforces
the generated conservative bound, then clears and reads back the entire arena.

| Variant | Proved arena bytes | Observed peak | Proved stack bytes | Issued-view bound |
| --- | ---: | ---: | ---: | ---: |
| Imported byte copy | 240 | 240 | 512 | 2 |
| Read-only derived window | 240 | 240 | 512 | 2 |
| Boundary refusals | 2248 | 656 | 640 | 6 |
| Argument-helper effect order | 136 | 136 | 464 | 1 |

Boundary cases include an extent-end byte, 2^64 index, oversized byte value,
wrapped window, empty end window, rights widening and read-only store. The
effect-order oracle reads a byte before a later named argument changes it;
reordering produces the wrong payload. Pre-entry cases cover code/owner,
generation/exhaustion, active/issued state, extent, rights, null control,
control/owner overlap and base overflow, with complete unchanged shared-state
comparison. A changed imported helper deliberately fails the independent byte
oracle. Missing/private module exports and private caller-scope dependencies
also reject.

The [GDB oracle](labs/qemu/x86_64/region_source_debug.py) observes all 18 native
returns and three actual imported helper calls, their source/parameter debug
information and caller chain. It checks restoration of RBX, RBP, R12–R15 and
RSP. Certificate replay rejects changed allocation/view/stack bounds, ABI,
effects, runtime digest, source snapshot, LLVM and object bytes. Capacity cases
reject excessive temporary bytes, issued views and numeric bounds before object
emission. Malformed record fields reject.

## Validation and remaining integration

The full Cargo workspace test suite passes, including 617 language tests
(seven new region-source tests), 249 compiler-library tests (five new native/
certificate/boundary tests), 261 compiler CLI tests and 168 semantic-model tests.
Cargo/format validations use `scripts/run_bounded.py`; the successful workspace run uses
serial compilation/test execution, a 16 MiB Rust stack and disabled host test
debug information. Generated Topal object debug information remains enabled and
is checked by GDB. An initial focused build without serial compilation was killed
by its automatically selected 875 MiB cgroup limit. The limit was not raised or
bypassed; serial compilation resolves that failure. The older model-stage failed
workspace attempt retains its historical result in that report.
The final adapter/hook cost-certificate pins pass all five focused compiler
tests again. The all-target workspace check, workspace formatting, explicit
format checks for included Rust files, whitespace, relative links and all 283
unique normative systems rules also pass.

Risk is high at authority/provenance, lifetime, evaluation order, shared lowering
and complete bounds. The implementing agent reviewed authoritative resource/
effect/layout meaning, the borrowed protocol, source isolation, every primitive
access and cost, the full object/call closure and independent native oracles.
Human PR review remains required; no independent code-review result is claimed.

The next work binds this source interface into the actual kernel root, lends an
existing owned region and temporary storage, executes helper frames under QEMU,
and verifies complete arena/retired-stack/code-binding reclamation before owner
resumption. No new QEMU cohort is attributed to this standalone stage. The last
physically qualified kernel remains the
[positional-transfer kernel](step-4-positional-file-transfers.md). Distribution
replacement, general execution/signals, persistent VFS, networking/isolation,
Docker/Podman and VM hosting remain required by the [status ledger](step-4-status.md).
