# LLVM compiler and native platform architecture

This architecture realizes `TOPAL-GOAL-TOOLCHAIN-001`,
`TOPAL-GOAL-NATIVE-001`, and the `TOPAL-REQ-NATIVE-*` requirements. It records
the approved first target and the boundary between Topal semantics, LLVM, and
Linux. The [compiler conformance roadmap](compiler-conformance.md) tracks the
increments needed for complete core-language support.

## Research conclusions

LLVM modules need both an exact target triple and data layout for reliable
target-specific optimization. `DataLayout` describes pointer sizes, alignment,
endianness, and aggregate layout, while `TargetMachine` supplies the target
backend. The first qualified pair is:

```text
triple:      x86_64-unknown-linux-gnu
data layout: e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128
CPU:         x86-64 baseline (never the compiler host's native CPU)
object:      ELF64 little-endian, static PIE
```

References: LLVM's [frontend performance
guidance](https://llvm.org/docs/Frontend/PerformanceTips.html), [language
reference](https://llvm.org/docs/LangRef.html), and [code-generator
architecture](https://llvm.org/docs/CodeGenerator.html).

LLVM lowers an already selected LLVM calling convention to physical registers
and stack locations, but a source-language frontend still owns semantic value
representation and any coercion needed to form that LLVM signature. The C
calling convention is therefore not a portable Topal ABI. Internal functions
use an exact compiler-private signature keyed by `topal-native/6`, and future
foreign adapters may use target `ccc` only with fixed-width scalars or opaque
handles. Aggregate classification is adapter work and will be checked against
the target's reference C frontend before a foreign interface is admitted.

An admitted positional-product result whose fields already have exact private
machine values uses a recursively nested, non-packed LLVM literal struct. Unit
occupies a sealed `i8` field inside such a value even though a standalone Unit
function returns `void`; every other field retains its existing private scalar
carrier. Construction and observation use LLVM `insertvalue` and
`extractvalue`, and both the internal definition and every call use one exact
`fastcc` prototype. LLVM therefore owns the physical x86-64 register/stack
lowering, while the Topal frontend continues to own field order, source type
identity, and conversion. This convention is module-private and qualified by
the recorded target, LLVM version, and `topal-native/6`; it is neither a stable
library boundary nor a foreign aggregate ABI.

Tuple DWARF uses the same field order and the size, alignment, and padding
derived by the qualified x86-64 target implementation from the module data
layout. LLVM 22 does not preserve a direct SSA aggregate `#dbg_value` as an
inspectable x86-64 local in this O0 path, so a named Tuple binding also receives
a debug-only stack shadow and `#dbg_declare`. This shadow changes neither the
semantic value representation nor call transport, requires no allocator or
runtime, and is retained deliberately for deterministic GDB inspection.
Records, aggregate parameters, persistent aggregate storage, and public
interoperation remain separate representation decisions.

The checked frontend resolves each admitted source-ordered overload before IR
generation and gives every selected input signature a distinct call-graph node
and private LLVM symbol. Argument expressions are modeled once before candidate
filtering; only the selected candidate's canonical scalar conversions reach
code generation. Staticness is retained as semantic availability rather than a
different machine convention: a static body may select only another static
declaration, while root and ordinary runtime contexts may select either form.
Complete explicitly classified headers are collected for a declaration scope
before selected bodies are checked, so an acyclic scalar call may target a
later source declaration. Depth-first instantiation emits the selected callee
before its caller while retaining source locations for both DWARF frames;
ordinary initializer bindings remain source-ordered. This increment covers
statically decidable scalar headers. Dynamic structural classifier dispatch,
function values, closures, and remaining recursive call graphs remain later
frontend work and do not leak into the private ABI prematurely.

Recursion identity uses that complete selected input header, not source-name
spelling alone. A call from an active `String` overload to a same-named `Int`
overload is therefore an ordinary acyclic edge: it receives a distinct private
symbol, is emitted before its caller, and requires no recursion proof or symbol
reservation. The two overloads retain separate source parameter types and
DWARF subprograms even when their current private machine carriers are both
`ptr`. No source classifier is inferred from that coincident machine shape.

The initial recursive closures reuse the interpreter's structural termination
proof rather than defining a compiler-only proof language. For a proven unary
`Int` overload that decreases toward a lower bound or increases toward an upper
bound, the model reserves one symbol before checking the body, generalizes its
parameter facts to the declared classifier, and directs each self-edge to that
symbol. The emitted definition and calls have the exact same LLVM [`fastcc`
calling convention and
prototype](https://llvm.org/docs/LangRef.html#calling-conventions). They retain
`noinline` for predictable O0 frame inspection and never claim the LLVM
[`norecurse` attribute](https://llvm.org/docs/LangRef.html#function-attributes),
which would be false for the admitted call graph. Tail-call formation remains
an optional later LLVM optimization, not a termination or correctness premise.
Exact Int representation, allocation, and Linux integration remain unchanged,
so this closure adds no dispatch runtime, foreign dependency, or ABI revision.

Mutual `Int` recursion extends active proof metadata with the one next-member
name established independently for each declaration. A call returning to an
active overload is admitted only when the intervening active slice contains at
least two members, every member carries the same decreasing or increasing
mutual rule, every adjacent target matches, and the final target closes the
cycle. Symbols are reserved as members are instantiated, so the closing edge
uses the already exact prototype. Multiple calls to the next member are checked
and lowered independently. This graph check is entirely in the frontend; it
introduces no dispatch table, stack protocol, tail-call premise, or runtime
cycle representation.

The unary `Nat` closure uses the same representation while preserving its
constraint boundary explicitly. Ordered matching and recursive `+` or `-`
first forget `Nat` evidence to the unchanged exact `Int` carrier. Only a shared
`TOPAL-FUNCTION-RECURSION-NAT-*` proof may attach `Nat` evidence to the
recursive argument without dynamic validation: the decreasing proof checks the
inclusive nonnegative bound against every literal step, and the increasing
proof establishes that addition preserves nonnegativity. The checked
`IntToNat` node records that proof boundary but emits no instruction and no call
to `topal.runtime.int.try.to.nat`. Function linkage, DWARF `Nat` identity, and
the private `fastcc` prototype otherwise remain identical to the `Int` path.

Mutual `Nat` recursion composes the cycle check with that evidence boundary.
Each active member records the single named next member and the unary `Nat`
parameter whose bounded subtraction or nonnegative addition the shared proof
authorizes. Only that named edge may regain `Nat` evidence; ordinary `Int`
expressions and unrelated calls remain subject to dynamic validation or
rejection. The decreasing proof also checks every edge against its own
nonnegative inclusive bound, preventing an intermediate member from
overshooting below zero. Once all adjacent names and one uniform decreasing or
increasing rule close the active cycle, the already reserved exact prototypes
form the LLVM cross-calls. The evidence conversion is erased, so neither a Nat
check nor a cycle representation reaches the runtime or native ABI.

An explicit single-parameter `Decreases` measure extends that mechanism to a
larger scalar state. The compiler retains the v0.1 effect-bound span in the
source declaration, invokes the interpreter's complete structural proof, and
records the measured parameter index in its active recursion metadata. This
index is also the only multi-parameter position allowed to regain `Nat`
evidence from the proof. Other parameters are checked and passed normally and
may change independently. The measure is compile-time evidence only: it adds no
hidden argument, runtime counter, calling-convention change, or dynamic Nat
validation to the one exact private function signature.

`topal-native/6` represents finite `Int` values as immutable pointers to a
canonical sign-and-magnitude object with little-endian base-2^32 limbs. The
private signature passes that pointer directly; it never exposes the object to
a foreign calling convention. Runtime allocation uses the qualified Linux
platform boundary and retains objects until process termination in this
increment. This process-lifetime allocation policy is safe for immutable
values but is not the final reclamation policy; ownership-aware reclamation is
admitted with the container and closure representations that make reachability
nontrivial.

Finite `Rational` values are immutable objects containing two private Int
pointers: a coprime numerator and a positive denominator. The compiler emits
each literal component as relocation-free Int data, then constructs the
pointer-bearing Rational object at run time. This is required because a static
PIE with no ELF interpreter has no loader to apply absolute pointer
relocations. `llvm-readobj --relocations` qualifies that invariant.

Explicitly bounded finite exact ranges use an immutable 32-byte header holding
opaque lower and upper endpoint pointers plus two canonical inclusivity words.
The endpoint classifier remains static, so `Range Int` and `Range Rational`
share a private storage shape without erasing their semantic type. Range
construction remains a predicate value: the runtime compares exact endpoints
for membership, emptiness, and intersection and never enumerates or adjusts an
open endpoint. Pointer-bearing range objects are also constructed at run time
to preserve relocation-free no-loader PIE output.

String values use immutable `{data, length, display, display-length}`
descriptors containing UTF-8 bytes and an optional cached canonical display.
Only relocation-free byte arrays reside in the static image; descriptors are
constructed through the Topal allocator at run time because their pointers
otherwise require loader-applied absolute relocations. The compiler computes
and caches literal display spelling. A null display pointer marks a dynamic
descriptor whose canonical spelling is emitted from preserved bytes. Functions,
decision joins, Result payloads, DWARF, and GDB all use the same private pointer
representation.
The syscall runtime writes canonical ordinary or tagged Topal literals without
relying on a C locale or string library. This descriptor foundation admits
literal transport and display; subsequent increments reuse it for byte count,
exact equality, and concatenation, while the remaining Unicode String
operations stay in roadmap increment 4b3d.

The prospective UTF-8 byte-count operation reads the preserved-byte length
already stored in that descriptor; it does not scan display spelling, attach an
encoding, normalize text, or consult a locale. A private unsigned-64-to-Int
runtime helper expands the target length into at most two base-2^32 limbs and
constructs the unique one- or two-limb canonical form, with the shared zero
object for an empty String. The resulting value therefore uses the same
unbounded Int representation and debugger behavior as all other counts,
without a foreign conversion helper or fixed source-level integer limit.

Exact String equality first compares the descriptor's preserved-byte lengths
and then compares each preserved UTF-8 byte in order. Because admitted Strings
contain valid UTF-8 and preserve their Unicode scalar sequence, equal byte
sequences are exactly equal preserved sequences. The runtime does not inspect
the cached display spelling, normalize either operand, consult locale state, or
call a foreign string routine. Derived `Optional String` equality validates both
Optional tags and invokes the same comparator only when both alternatives are
`Some`; two `None` values compare equal and different alternatives do not load
payloads.

Empty construction creates the zero-length String value through the same
descriptor path, while adjacent source literals are composed into one preserved
sequence during mandatory frontend construction. Dynamic concatenation
evaluates operands left to right, checks target-length addition, obtains exact
storage from the Linux mapping boundary, and copies both preserved UTF-8 byte
sequences without normalization. The standard
[`llvm.memcpy.inline`](https://llvm.org/docs/LangRef.html#llvm-memcpy-inline-intrinsic)
intrinsic expresses each nonoverlapping dynamic copy and guarantees that target
lowering does not introduce an external function. A zero-byte result still
receives a valid private allocation so it never relies on zero-length `mmap`
behavior.

Concatenated descriptors omit the display cache. Their print path scans for a
quote and emits either the ordinary quoted spelling or the shortest
collision-free `text` tag, extending it with underscores as required. It emits
preserved data unchanged and uses only complete syscall writes; no locale,
Unicode transformation, foreign allocator, or C/C++ string routine participates.
The GDB renderer continues to decode the preserved data and therefore observes
literal and concatenated Strings identically. String emptiness reads only the
preserved-byte length, which is zero exactly when the valid UTF-8 scalar
sequence is empty.

Static Character evidence is a checked-model refinement over that same String
descriptor. For closed expressions the frontend asks the shared, pinned
Unicode 17 language-context implementation for the extended-grapheme count;
one retains Character identity, while zero or multiple clusters are a static
diagnostic. Function signatures and DWARF keep the refined identity, but LLVM
continues to carry the identical descriptor pointer and exact equality uses the
String comparator. Forgetting the evidence is therefore an IR no-op. Dynamic
validation is rejected until a Result-producing, freestanding segmentation
path is admitted, so generated behavior never falls back to host or OS Unicode
tables.

Closed Character counting and indexing use the same shared pinned segmentation
while the checked frontend still has the complete preserved sequence. This is
mandatory constant evaluation for the admitted semantic domain, not an LLVM
optimization: counts become canonical Int constants, while a valid indexed
cluster becomes an ordinary immutable String descriptor carried by the private
Optional header as Character. A negative or out-of-range exact index becomes
the existing `None` header. `Optional Character` function passage, decision
payload extraction, display, DWARF, and GDB therefore reuse existing carriers
without a new ABI. Unknown text or index values fail at the checked boundary
until generated Topal Unicode tables and segmentation code exist.

Closed universal casing, full case folding, NFC/NFD normalization, and
canonical equivalence follow the same frontend/runtime boundary. The checked
frontend evaluates them through `topal-source`, whose Unicode data is pinned by
the selected language context, and emits only exact String descriptors or
Boolean constants. This is required semantic evaluation at O0 rather than an
LLVM optimization: LLVM has neither Topal's Unicode-version authority nor a
Unicode operation to select. Unknown values remain unsupported until the same
pinned data can be generated into a Topal-owned freestanding runtime, avoiding
ambient host tables and locale behavior.

Fallible exact operations return immutable Result headers containing a
canonical success-or-error tag and one opaque payload pointer. Successful
payloads retain their statically known Topal type. Error payloads use an
intrinsic record containing the reporting domain, arithmetic code, absent
detail and cause fields, and source file, line, and column provenance. Domain
and source text refer to the same immutable String descriptors. This
`topal-native/6` layout is private to Topal signatures and GDB renderers; it is
never classified as a C aggregate or passed through a foreign runtime.

Exact Rational-to-Int and Int-to-Nat validation calls return the same Result
shape. Contextual success projection branches on the Result tag: an Error path
returns the original Result pointer immediately from the enclosing compatible
function, while only the success path loads and reclassifies the payload. This
keeps propagation field-preserving and makes early return explicit in the O0
control-flow graph.

Result decisions branch once on the Result tag. The success and whole-Error
payload bindings exist only in their selected action blocks. Qualified
arithmetic Error-code decisions switch on the closed nominal code value; a
generic Error action is the default when present, while an exhaustive four-code
table has an unreachable invalid-runtime default. String-valued actions merge
as ordinary descriptor pointers through LLVM `phi`. Error field observation
loads the stored code or domain descriptor without reconstructing the Error.
The same four arithmetic codes are available as qualified root values using the
identical sealed `i32` tags. Direct equality, function passage, display, DWARF,
and GDB therefore cannot diverge from codes observed through an Error, and no
namespace operation or foreign runtime survives into generated code.

Optional values use their own immutable 16-byte header containing a validated
`None`/`Some` tag and one opaque payload pointer. This is a distinct native
semantic representation from Result even though the current private headers
have the same physical shape: Optional constructors, observers, type identity,
DWARF names, and GDB rendering never reuse Result semantics or its tag names.
The admitted `Optional Int`, `Optional Rational`, and `Optional String` subset
crosses Topal-private function boundaries as an opaque pointer, while a
selected `Some` action reclassifies its payload from the statically retained
classifier. Optional equality validates both tags and calls the canonical Int,
exact Rational, or exact preserved-sequence String comparator only when both
values are present; an absent payload is never loaded. Rational admission
reuses the existing pointer payload and therefore does not revise
`topal-native/6`. Headers are allocated through the same Linux `mmap` platform
boundary, and no foreign aggregate convention, allocator, or standard library
participates.

Ordered comparison decisions lower directly to LLVM conditional branches in
source order. Each matcher operand is emitted in its reached test block, each
action in its selected block, and compatible machine-scalar results merge with
an SSA `phi`. Closed decisions over `Comparison` lower through `switch` to the
three nominal alternatives. The subject is emitted once before either control
flow graph, so LLVM optimization may simplify the graph later without changing
Topal evaluation order at O0.

Each admitted root-scope payload-free source `Enum` has its own checked nominal
identity and a declaration-ordered `i32` tag space. Tags pass directly only
through sealed Topal-private signatures; they are not a public C enum ABI. The
frontend rejects cross-enum equality and classification, and exhaustive
decisions lower to LLVM `switch` plus typed `phi` joins. An impossible invalid
tag takes the compiler-runtime corruption exit rather than selecting an
arbitrary source alternative. Display switches to the declared label bytes
through the Topal syscall boundary, while DWARF describes a genuine enumeration
so stock GDB shows source labels. General `Union` layout and nested enum
declarations remain separate representation and scope increments.

A direct explicit `return` in an admitted linear function body is resolved by
the checked frontend as a control-flow boundary, not an optional optimization.
The return expression is checked once against the declared output, preceding
statements remain ordered, and the unreachable source tail never enters LLVM
IR. The backend then uses the same private signature, return instruction, and
DWARF source mapping as an implicit final result.

An admitted lexical block clones the enclosing checked binding environment,
evaluates its statements in order, and discards the child environment after
constructing the final value. This implements inner shadowing and non-escape
without mutating an outer compiler environment. The backend mirrors that rule
with a private LLVM-value environment and a nested `DILexicalBlock`, so GDB
resolves an innermost scalar binding by its source scope. An empty block is the
zero-data Unit value and allocates nothing. Returns through a nested block,
nested declarations, and scopes requiring cleanup retain their later explicit
exit-edge and lifetime lowering.

`Completed` uses a private `i8` singleton carrier at function boundaries while
Unit results remain LLVM `void`. The bit pattern is not a public integer ABI:
its purpose is to keep a typed SSA result and therefore an explicit completion
dependency for every admitted call at O0. Literal construction requires no
allocation, equality compares the sealed singleton carrier, display writes the
source name through the Topal syscall layer, and DWARF describes a singleton
enumeration so GDB does not mislabel the value as an integer or Unit.

The canonical empty `Effect` row is another zero-data value but retains a third
checked identity. Its private `i8` singleton carrier crosses scalar Topal
function boundaries and decomposed products without being interchangeable with
Unit or `Completed`. Construction performs no described interaction, equality
compares canonical row identity, and display alone uses the existing Topal
write syscall. A distinct DWARF enumeration exposes `Effect` and its empty
alternative to GDB. No effect-specific runtime entry point, allocation, public
integer ABI, or foreign standard library is introduced; nonempty rows, effect
inference, collections, and aggregate function results remain later work.

The closed v0.1 fundamental `Type` values use one compiler-private `i32` tag set
for `Boolean`, `Int`, `Nat`, `Rational`, `String`, `Unit`, and `Scope`. The
checked model retains the enclosing `Type` kind and exact constant identity;
equality compares those identities, scalar calls preserve the tag, and display
selects the canonical source name. DWARF describes the closed set as `Type`, so
GDB shows the semantic identity rather than a raw integer. This is not runtime
reflection and the tag numbering is not a public ABI or library-metadata key:
future compiled-library metadata records canonical semantic identities and lets
each target lowering choose its private representation. No registry, allocation,
foreign type-information runtime, or standard library is linked.

An ordinary prefix call with one positional product operand is flattened by the
checked frontend into the declared scalar parameter sequence before overload
selection. Evaluation and ABI argument order remain left-to-right. A typed `_`
parameter participates in the same classifier check and private machine
signature but is omitted from the function binding environment and DWARF
variables; the backend therefore neither makes the discarded input addressable
nor fabricates a debugger name for it.

Within an expression or binding, an admitted positional product remains a
compiler aggregate of its already-evaluated field values. Same-classifier
product equality evaluates the complete left operand and then the complete
right operand once, recursively applies each field's admitted canonical
equality, and joins the Boolean results with LLVM `and i1`. Comparing a product
therefore adds no allocation, native object header, or foreign aggregate ABI.
Canonical conversions between differently classified corresponding fields and
product passage through machine signatures remain separate frontend and ABI
work rather than being inferred by the backend. Because this increment has no
single machine product value, it does not yet publish product bindings as DWARF
locals; their source lines and lowered field operations remain debuggable, while
a truthful aggregate DWARF representation is retained with general product
storage and ABI work in increment 3b2-b5e8.

An anonymous labeled Record uses the same decomposed expression-local strategy.
The checked model evaluates fields in source order, rejects duplicate labels,
keeps that order for display, and separately retains a canonical label-to-type
map for selection and structural identity. LLVM lowering carries labeled field
values without allocating a record object; selection chooses the already
evaluated field and display recursively emits `label is value` through existing
Topal syscall-backed value printers. This adds no record runtime or ABI. Scalar
values projected from a record retain ordinary DWARF locals and can be inspected
in GDB. The record binding itself is deliberately absent from DWARF until
increment 3b2-b5e8 supplies a truthful aggregate storage and debug representation
rather than describing a layout that does not exist.

Structural comparison also remains over decomposed values. The frontend
recursively selects one common exact numeric representation per Tuple position
and Record label before LLVM lowering. Record equality looks fields up by their
canonical labels while retaining each operand's source evaluation order. Tuple
ordering evaluates both aggregate operands completely and then uses `br` and
`phi` to skip later comparisons after the first non-Equal field. Existing exact
Int, Rational, String, and scalar equality routines do the leaf work; LLVM
provides control flow and SSA joining but does not infer Topal evidence or field
conversions. No aggregate storage, structural runtime, or ABI revision results.

Immutable Record reconstruction reuses that decomposition. The checked model
evaluates the complete base once, then evaluates replacement expressions once
in source order and applies their admitted exact classifier conversions before
LLVM lowering. The backend replaces only the named compiler values while
retaining base construction order and all unreplaced values. The original
aggregate remains unchanged because its value vector is cloned, and neither
path acquires storage. This introduces no allocator, reconstruction runtime,
foreign dependency, or ABI revision. Scalar projections from both versions use
the existing DWARF path; aggregate layout and machine-boundary passage remain
in increment 3b2-b5e8.

Nat validation already preserves an unchanged arbitrary-precision Int object
with distinct checked constraint evidence and DWARF type identity. Comparison
lowering forgets that evidence in the checked expression only: Nat/Int operands
use the existing exact Int comparator, while a Nat paired with Rational takes
the same single Int-to-Rational conversion as its base value. This emits no
unsigned narrowing, bit reinterpretation, Nat-specific runtime symbol, or ABI
conversion, and it does not retag the source binding exposed to GDB.

The correctness-first exact runtime uses binary long division, Euclidean sign
correction, Euclid's greatest-common-divisor algorithm, and exponentiation by
squaring. LLVM's documented `llvm.ctlz.i32` intrinsic determines the last
significant exponent bit with defined zero behavior; LLVM then lowers that
target-independent operation. The algorithms intentionally remain visible at
O0 rather than relying on target helper symbols or a foreign big-number
library.

LLVM's [`iN` integer type](https://llvm.org/docs/LangRef.html#integer-type)
has an arbitrary but compile-time-fixed width, capped by the IR format, while
the [code generator](https://llvm.org/docs/CodeGenerator.html#selectiondag-instruction-selection-process)
legalizes unsupported widths into target operations. It is useful for bounded
bit vectors but cannot implement a source `Int` that grows with runtime data or
available storage. Topal therefore owns the dynamic object and exact limb
algorithms; LLVM still owns lowering their ordinary `i32`/`i64` arithmetic,
control flow, registers, and instructions. This avoids both a hidden fixed
semantic limit and target-specific wide-integer helper dependencies.

GEIR remains the semantic library boundary. LLVM documents backward reading of
older bitcode, not a permanent forward-compatible language-library contract;
LLVM bitcode is consequently a rebuildable, LLVM-major-qualified native
payload only. References: [LLVM bitcode
format](https://llvm.org/docs/BitCodeFormat.html) and [LLVM link-time
optimization](https://llvm.org/docs/LinkTimeOptimization.html).

## Compilation pipeline

```text
shared source + lossless syntax
              |
              v
shared checked Topal compiler model
              |
              +--> canonical GEIR and public interface metadata
              |
              v
Topal representation and mandatory semantic lowering
              |
              v
LLVM IR + target triple + DataLayout + debug records
              |
              v
llvm-as --> opt verifier --> llc --> ELF object
                                      |
                                      v
                              LLD, no default libraries
                                      |
                                      v
                         freestanding static PIE executable
```

The initial Rust integration deliberately uses LLVM 22's process tools rather
than linking the LLVM C API. This keeps LLVM version selection visible and
avoids adding unsafe foreign-function calls to a workspace which forbids unsafe
Rust. The compiler locates an explicit `--llvm-tools` directory,
`TOPAL_LLVM_TOOLS`, `LLVM_SYS_220_PREFIX`, or the matching Rust LLVM-tools
component, and rejects a different major version.

Every compilation assembles and verifies the IR before code generation. `llc`
owns instruction selection, register allocation, machine scheduling, and ELF
object emission. LLD owns relocation and executable layout. Topal supplies its
own `_start`; linking passes no C runtime, startup files, default libraries, or
ELF interpreter. The target startup stub aligns the kernel-provided stack to
the x86-64 call-site rule before entering LLVM-generated functions; it does not
pretend that the kernel entered `_start` through a language call convention.

## Linux platform boundary

The first platform module uses the Linux x86-64 syscall ABI. Its inline assembly
is confined to generated target-support functions and declares fixed registers,
`rcx`/`r11`, and memory effects to LLVM. Initial operations are `write` (system
call 1), `mmap` (system call 9), and `exit` (system call 60), as assigned by the
kernel's authoritative
[x86-64 syscall table](https://github.com/torvalds/linux/blob/master/arch/x86/entry/syscalls/syscall_64.tbl).
Partial writes and interruption remain platform results handled by the Topal
support routine; they are not replaced with libc behavior.

The finite-Int runtime obtains private anonymous read/write mappings directly;
the constants follow Linux's exported [`mman`
interface](https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/mman-common.h).
Mapping errors terminate through the same platform boundary rather than falling
through to foreign allocation or unwinding support.

Additional services will be exposed by ordinary typed Topal platform packages.
They must record kernel ABI version assumptions, exact structures and constants,
effects, error-code mapping, cancellation and retry behavior, and architecture
qualification. A direct syscall is an implementation of that boundary, not a
new ambient source-language operation.

## Optimization ownership

Mandatory Topal work is performed at every optimization level:

- language revision and feature checking;
- name, type, overload, capability, effect, contract, and evidence resolution;
- generic instantiation and representation selection;
- rejection of unavailable target or implementation evidence;
- explicit cleanup, failure, and platform-boundary lowering; and
- generation of well-typed, non-poisoning LLVM IR.

At `-O0`, no optional Topal specialization or semantic rewrite runs. LLVM runs
verification and its O0 code generator; emitted arithmetic is still required to
implement exact Topal behavior. Higher levels will begin from LLVM's new-pass-
manager default pipelines, with Topal-specific changes admitted only by
conformance and differential tests. LLVM's [new pass manager
documentation](https://llvm.org/docs/NewPassManager.html) recommends constructing
the standard pipeline through `PassBuilder`; the command-tool integration uses
the equivalent named default pipeline when those levels are enabled.

Topal, rather than LLVM, owns transformations needing language proof: overload
selection, evidence erasure, law-based fusion, effect reordering, mutation
introduction, representation changes, and specialization. LLVM owns ordinary
SSA simplification, inlining after exact signatures exist, scalar and loop
optimization, vectorization when emitted facts permit it, instruction
selection, register allocation, and target scheduling. Topal will communicate
proved aliasing, lifetime, alignment, memory-effect, and range facts through
standard LLVM attributes, intrinsics, and metadata rather than custom LLVM
semantics.

## Debugging contract

Unoptimized output uses LLVM debug records and DWARF 5. It describes Topal
source files, source functions, machine-scalar parameters and immutable locals,
their types, and instruction locations. Structural values acquire debug
descriptions in the increment that gives them a runtime representation; until
then the compiler omits them rather than describing an unrelated value. Frame
pointers remain enabled at `-O0`. Tests drive GDB in batch mode to set source
breakpoints, step, inspect values, and obtain backtraces; an available
`llvm-dwarfdump --verify` additionally checks DWARF structure. LLVM's [source-level debugging
documentation](https://llvm.org/docs/SourceLevelDebugging.html) explains that
the frontend supplies this source mapping and the backend emits DWARF usable by
GDB.

GDB does not yet have a native Topal expression parser. The initial compiler
uses a compatible DWARF language code and emits honest Topal source names and
types; bundled pretty-printers will cover nontrivial runtime values as those
representations appear. Native evaluation of arbitrary Topal expressions is a
separate GDB integration increment and is not claimed by the bootstrap.

## Library artifact metadata

A native artifact manifest is canonical UTF-8 JSON whose first schema is
`topal.native-artifact/1`. It records:

- language, GEIR, manifest, compiler-private ABI, and platform ABI revisions;
- target triple, exact data layout, object format, CPU, and feature baseline;
- LLVM major and exact tool version, optimization level, debug format, and
  compiler identity;
- canonical source/interface and dependency identities and digests;
- exported semantic identities and their private machine symbols;
- required platform packages and implementation evidence;
- source/debug remapping, build identity, and license provenance; and
- native slices, each rejected unless every target field matches the consumer.

Unknown required fields, duplicate identities, digest mismatches, unsupported
revisions, and incompatible target slices reject the artifact before code
generation. Machine symbols and LLVM payloads are never sufficient without the
validated semantic interface.

## LLVM facility disposition

| Facility | Bootstrap disposition | Reason or qualification |
| --- | --- | --- |
| LLVM IR, opaque pointers, target triple, `DataLayout` | used | portable optimizer/code-generator boundary |
| `llvm-as` and IR verifier | used on every native build | reject malformed or internally inconsistent IR |
| New pass manager | O0 verification only | optimized pipelines wait for differential conformance coverage |
| `llc` target backend | used | instruction selection, register allocation, scheduling, ELF object emission |
| LLD | used | deterministic no-default-library static PIE link |
| `br`, `switch`, and `phi` | used | once-evaluated Boolean, exact-matcher, Comparison, nominal Enum, and fallible arithmetic control flow with typed result joins |
| DWARF debug metadata and frame pointers | used | GDB source debugging at the reference level, including native enum labels and bundled renderers for private Int, Rational, and finite exact Range objects |
| `llvm.ctlz` | used | target-independent significant-bit count for finite exact exponentiation |
| `llvm.memcpy.inline` | used | target-qualified dynamic String copies while retaining LLVM's guarantee that lowering calls no external function |
| `llvm-readobj` / `llvm-objdump` | test and qualification use | object, dependency, symbol, and line-table inspection |
| `llvm-link`, `llvm-dis`, `llvm-extract`, `llvm-diff`, `llvm-reduce` | qualification and failure reduction only | production linking occurs from verified modules; these tools remain useful for backend diagnosis but do not improve emitted semantics merely by being invoked |
| `llvm-ar`, `llvm-ranlib`, `llvm-size` | archive packaging deferred; inspection as needed | compiled-library container and installation rules must precede a public native archive format |
| `llvm-nm` | qualification use | every freestanding runtime regression rejects undefined helper symbols |
| `llvm-objcopy`, `llvm-strip`, `llvm-dwp` | split-debug/install packaging deferred | bootstrap artifacts retain full debug information; destructive stripping would violate the O0 debug contract |
| `llvm-dwarfdump` | used when installed | structural debug-information verification |
| full LTO / ThinLTO | deferred | GEIR/native-slice packaging and reproducible cache keys come first |
| sanitizers | deferred | freestanding sanitizer runtimes and their effects must be supplied explicitly |
| source coverage | deferred | requires a Topal event/source mapping and freestanding profile writer |
| instrumentation/sample PGO | deferred | profile identity and semantic reproducibility contract not yet defined |
| `llvm-cov`, `llvm-profdata`, XRay, and symbolization tools | deferred with their instrumentation producers | the freestanding profile/event writer and Topal source-observation contract are not yet defined |
| ORC JIT | not applicable to the AOT bootstrap | interpreter and future interactive compiler have separate execution contracts |
| MLIR | not selected | the current semantic forms do not yet need a custom multi-level dialect |
| custom LLVM fork, types, intrinsics, or passes | not selected | standard IR maximizes tool compatibility and upgradeability |

The matrix is updated whenever a facility becomes applicable. “All LLVM tools”
means all relevant capabilities are considered and dispositioned; it cannot
mean invoking unrelated target, JIT, GPU, or binary-rewriting tools without a
Topal requirement.
