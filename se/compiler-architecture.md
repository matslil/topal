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
Persistent aggregate storage and public interoperation remain separate
representation decisions.

An admitted Tuple parameter uses that same recursive LLVM literal-struct type
as one exact private `fastcc` argument. The caller constructs the aggregate
with `insertvalue`; the callee recursively decomposes it with `extractvalue`
into the existing field-value representation. LLVM owns the x86-64 physical
register/stack classification under the module triple and data layout, so the
frontend neither duplicates a target ABI algorithm nor promises this private
shape at a library or foreign boundary. A named Tuple parameter uses the same
target-aligned debug-only shadow as a named Tuple local because LLVM 22 does not
retain the aggregate SSA argument as a reliably inspectable O0 value. A
discarded Tuple remains in the checked and LLVM signature but requires neither
decomposition nor a debug binding.

When an admitted decision produces one of these Tuples, lowering keeps the
existing decomposed representation through the merge: it recursively creates
one LLVM `phi` for each machine-represented leaf and no node for Unit leaves.
This gives every predecessor an exact scalar edge value, preserves nesting, and
avoids both an aggregate `phi` and a temporary materialization. The frontend
still proves a single complete action type and retains the decision family's
once-only subject, ordered matching, and single delayed action semantics; LLVM
only lowers the already explicit control flow and typed leaf joins.

An admitted structural Record uses a related private literal struct without
conflating type identity and display order. Field values occupy canonical label
order so independently written but structurally identical classifiers have one
LLVM type. A trailing `i32` permutation maps each display position to a
canonical field, preserving the construction order required by the source
value across calls, returns, nesting, and decision joins. The frontend forms and
decomposes this semantic carrier with `insertvalue` and `extractvalue`; LLVM
owns only its target-physical `fastcc` lowering. A Record-valued decision joins
each canonical field recursively and each permutation entry with a scalar
`phi`, and dynamic display selects fields through that checked permutation.

Record DWARF exposes only the canonical semantic named fields at their actual
offsets while the composite size accounts for the trailing private order data.
A named Record local or parameter uses the same target-aligned debug-only stack
shadow strategy as a Tuple. The order data is not fabricated as a source member,
and neither the private carrier nor its debug shadow is a public ABI, persistent
semantic storage, heap allocation, or runtime object.

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
statically decidable scalar headers. A closed named-root or symbolic `Function`
result reuses the private i32 observation tag while its specialization retains
the callable identity separately, so a later application is still a direct
call or operation rather than tag dispatch. A non-capturing inferred anonymous
result retains its body and arity through the same specialization side table,
while the machine result remains the private observation tag. Target-aligned
debug-only shadows keep otherwise-dead Function result bindings observable at
O0. Aggregate-contained, dynamic, nested, and published Function results,
dynamic structural classifier dispatch, escaping or dynamically stored
capturing closures outside the private anonymous-result boundary below, and
remaining recursive call graphs remain later frontend work and do not leak into
the private ABI prematurely.

Within a single compiled source application, the executable `root` namespace
is also a frontend identity rather than a runtime lookup table. The checked
model recognizes the live root value and resolves a directly qualified
function against the collected root declaration set before ordinary overload
selection. This lookup bypasses same-named lexical bindings by construction.
The resulting IR is the same direct private call used by unqualified selection;
a sealed constant represents `root` only when the source observes the Scope
value itself. Namespace aliases require declaration snapshots, packages require
published interfaces, and compiled libraries require canonical artifact
metadata, so those remain explicit later representation decisions rather than
being approximated by this direct-root path.

The first alias increment retains root function declarations as immutable
checked-model snapshots. An alias therefore carries namespace identity and a
source-ordered map of already-visible function headers only during compilation;
the native value remains the same sealed zero-data identity used for observable
Scope display. Alias chains clone those facts, and qualified selection produces
the normal direct private call. This deliberately avoids committing namespace
lookup tables, callable pointers, or public Scope layout before root data
storage, generators, packages, and compiled-library interfaces have defined
their distinct representation and artifact requirements.

Source-root `use` composes with that snapshot representation rather than adding
a second namespace mechanism. The frontend requires the single operand to be
the live root Scope or an already retained root alias and returns the same
checked namespace facts; an optional binding captures them at its ordinary
source position. Member resolution therefore continues through the existing
direct private function calls and stable data storage identities. `use` itself
is erased before LLVM, while an observed binding retains only the existing
sealed Scope tag and DWARF identity. Multi-component published paths and
package/library lookup wait for canonical interface metadata and never consult
ambient host filesystem or process state as a substitute.

Source-root data members use a parallel immutable snapshot map. Each checked
binding receives an internal storage key derived from its declaration location,
while its source spelling remains the DWARF variable name. Qualified data
selection carries that key into the checked expression, so LLVM reuses the
already-emitted SSA value even through a caller lexical shadow; it neither
replays the initializer nor consults a namespace object. Entry-frame SSA is not
valid in an independently callable function; direct source-root selection keeps
using this storage identity, while the first function-body increment below
provides an explicit private cross-frame representation.

The first general Scope-parameter increment crosses that frame boundary by
specialization rather than by inventing a public namespace object. At each
admitted root or root-alias call, the checked function environment receives the
concrete immutable declaration snapshot. Function members remain frontend
metadata and still select direct private callees. For each non-discarded Scope
parameter, every snapshot data member whose value already has a complete
private representation is appended as an exact hidden `fastcc` parameter; the
callee rewrites its namespace data map to those parameter storage identities.
Passing the Scope onward repeats that
typed environment threading, so forwarded selection never reaches back into a
caller frame. Unsupported members remain static facts and are rejected if
selected. The explicit sealed `i32` Scope parameter and material hidden values
receive DWARF entries, with target-aligned debug shadows where needed. This
finite closure conversion deliberately over-captures represented immutable data
to make forwarding correct without analysis-order mutation. LLVM owns physical
AMD64 parameter classification, and no namespace table, allocation, indirect
dispatch, foreign runtime, public Scope ABI, or ABI revision results. Scope
escape/results, function-local live-root formation, nested/non-root namespaces,
and compiled-library environments remain later representation decisions.

Explicit defining-context selection has a narrower first representation. For a
root function called from the source entry frame, the frontend finds scalar
root members referenced as `@ member`, filters them by the function declaration
position, and appends their already-evaluated compiler values as private capture
arguments in root declaration order. The callee receives ordinary exact LLVM
parameters under `fastcc`; its body binds them under distinct `@ member` storage
keys, so neither caller locals nor ordinary same-named parameters can intercept
selection. DWARF uses the source spelling `@ member`. This is private closure
conversion without an environment object. Aggregate/callable capture, escape,
qualified root access outside the exact direct-entry data case below, and
public/library contexts remain deferred to the unified closure and
compiled-library ABI.

The first defining-context forwarding extension computes transitive scalar
requirements for a finite acyclic chain of statically named functions before
instantiating its outer frame. The value remains the immutable snapshot captured
by the function containing the direct `@ member` selection; declaration
filtering is therefore applied at that selecting function, never at the caller.
Each intermediate private signature receives the exact value in root
declaration order and forwards its caller parameter at the direct `fastcc` edge.
No callee reaches into an ancestor frame or global context object. Target-aligned
debug-only stack shadows keep both active and suspended-frame values observable
when LLVM uses call-clobbered locations at O0. Overload-dependent, recursive,
function-value, nested, and anonymous chains remain fail-closed except for the
proof-backed scalar recursion case described below.

A future compiled-library context boundary must encode canonical context
instance and source-session identity, every selection and call edge, stable
callee/member identity and overload, captured declaration position,
visibility/order, canonical classifier and representation,
capture order/lifetime/effects, and a versioned target adapter independently of
private parameter names, LLVM types or symbols, debug shadows, and physical
placement.

Direct function-body `root member` data selection uses a related but
deliberately different private capture. Resolution observes the live
source-session root at the direct entry-frame call, so a member initialized
after the function declaration but before invocation is eligible. The frontend
scans exact root data selections, orders their already-evaluated storage values
by root declaration position, and appends exact `fastcc` parameters bound under
distinct `root member` names. A same-named explicit parameter cannot intercept
the selection, and DWARF exposes both values independently. This is
call-position root closure conversion, not defining-context capture: it creates
no global, namespace table, lookup, allocation, or initializer replay, and LLVM
retains authority over AMD64 physical placement. Members without a complete
private machine representation remain rejected.

The first cross-function extension computes transitive root-data requirements
for a finite acyclic chain of statically named functions before instantiating
its outer frame. Each intermediate signature receives the same represented
root values, ordered by root declaration position within the existing hidden
capture group, and each direct call forwards its caller parameters. The live
root snapshot is still chosen once at the entry-frame call position; no callee
reaches into an ancestor frame or process-global object. Debug-only aligned
stack shadows preserve the forwarded parameters when LLVM assigns their
machine values to call-clobbered locations, so GDB can recover both the active
and suspended frames at O0 without making those shadows semantic storage.
Overload-dependent chains and function-value/nested/anonymous edges remain
rejected; recursion is admitted only through the proof-backed scalar case below.

Proof-backed recursive scalar environments reuse the existing recursion graph
and private symbol reservation rather than introducing a recursive-closure
runtime. The frontend first computes the transitive union of exact root and
defining-context captures for every required graph member. The entry edge
supplies the immutable defining-context snapshot and live root call-position
snapshot; every direct or mutual back-edge then passes its current hidden
parameters unchanged to the already-reserved symbol. This cannot admit a cycle:
the independent direct, mutual, or explicit-measure termination proof remains a
prerequisite. Matching `fastcc` prototypes leave physical AMD64 placement to
LLVM, retain `noinline` without the false `norecurse` attribute, and use the
same aligned debug shadows to expose each capture in every suspended recursive
frame at O0. There is no cycle/environment table, lookup, initializer replay,
allocation, dispatcher, indirect call, or public ABI.

A future compiled-library boundary must encode source-session namespace
identity, every selection and call edge, stable callee/member identity,
visibility/declaration order, recursion graph/member identity and proof
evidence, canonical classifier and representation, capture
order/lifetime/effects, and a versioned target adapter independently of private
parameter names, LLVM types or symbols, debug shadows, and physical placement.

Named function values similarly split observable identity from call lowering.
The checked binding retains the original declaration vector and application
specializes from that vector, so the backend emits the same direct private call
as an unaliased source name. A deterministic module-local integer identifies
the name only when the Function value is displayed or inspected in DWARF. It is
not a code pointer or dispatch-table index. This preserves a future choice of
public callable/closure representation without burdening current private calls
with a provisional runtime ABI.

Symbolic Function values use the same separation. The initial `+`, `-`, and
`<=>` tags remain first for stable private observation, followed by every
equality, ordering, arithmetic, and range callable in deterministic canonical
order. Checked application rewrites the retained identity to the ordinary
operation expression before backend lowering, preserving existing conversion,
fallibility, result, and endpoint semantics. Binary operands remain a
source-level positional product and are decomposed by the frontend; LLVM sees
only the already-selected operation and its normal machine values. The tag
consequently cannot introduce indirect control flow or constrain a later
general callable ABI.

Private Function inputs extend the same scheme across one specialization
boundary. The caller passes the observation tag in the source parameter slot,
while checked callable metadata is propagated into the specialized callee model
and determines its direct operations. Different callable identities may create
different private instances of the same source function. LLVM receives exact
i32 prototypes and owns physical register/stack placement for the target. When
specialization erases every computational use of the tag, a debug-only aligned
stack shadow preserves the source parameter for DWARF/GDB without turning it
into runtime dispatch or a public callable representation.

Inferred anonymous functions extend specialization without choosing a closure
ABI. The checked binding retains the parameter patterns, body, construction
identity, and lexical capture snapshot. A direct application in the defining
invocation passes represented immutable captures after the explicit operands
as deterministic exact private parameters, so the anonymous frame never reads
another native frame. A non-capturing value may additionally pass through a
specialized private Function parameter or result. Multi-parameter calls
decompose their positional product in source order. A flat product parameter
pattern first materializes its complete Tuple operand as compiler-private SSA,
then projects every field exactly once in lexical order. The generated private
signature flattens those fields among ordinary source parameters and appends
captures afterward, while source arity and callable identity continue to count
patterns rather than machine parameters. Flat mixed symbolic applications are
explicitly regrouped left-to-right before ordinary operation checking. The
observation tag exists only for `<anonymous fn/N>` display and DWARF. Capturing
Function parameters extend this private specialization when the caller retains
the exact anonymous or nested callable facts: its already-evaluated immutable
captures are appended as deterministic hidden arguments at every ordinary
Function boundary and remapped to that callee's storage identities. The same
facts and values may be forwarded through another specialized call. Hidden
forwarding arguments are excluded from source DWARF; only the source Function
parameter and the source-named captures in the eventually invoked callable are
shown. A nested callable receives a module-private observation tag only when it
enters this path. No tag dispatch, environment object, capture allocation, or
cross-frame lookup is introduced, and LLVM continues to lower every exact
private prototype for the target. Recursive patterns compose with this path as
described below. Capturing Function results outside the admitted private
anonymous and exact nested-result boundaries below, unsupported captured state,
other escaping environments, and a public/library closure representation
remain coordinated later design with canonical callable, ordered-capture,
lifetime/effect, representation, and target-adapter metadata.

A private capturing anonymous Function result uses the same facts without
turning them into a public closure representation. The specialized callee
returns one exact LLVM aggregate containing the observation tag followed by the
already-evaluated immutable captures in retained order. The caller immediately
decomposes that aggregate into compiler-only SSA storage, so later bindings,
private Function parameters, and another admitted private Function result can
reuse the exact callable and values without a heap object, environment pointer,
or tag dispatch. LLVM derives the physical x86-64 aggregate-return convention
from the target data layout. Source DWARF continues to describe a `Function`
result; the returned transport fields remain hidden, while an eventual direct
anonymous call exposes the captures under their source names. Exact nested
Function escape is admitted by
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`. Function-valued or otherwise
unsupported captures, dynamic selection, capture-bearing aggregate containment outside
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, publication, and the canonical
library closure ABI remain deferred.

Left-associative application may consume any exact admitted private Function
result without first creating a source binding. The checked frontend folds one
operand at a time, retaining the named, symbolic, or anonymous callable facts
separately from the observation tag and placing each intermediate Function in
compiler-only once-only storage. For a captured result, that storage also owns
the aggregate fields already extracted by the caller. LLVM therefore sees the
same sequence of exact direct `fastcc` calls that a source binding would have
produced; there is no result-tag dispatch, function pointer, closure object, or
re-evaluation of the factory. The private chain storage has no source DWARF
variable, while each invoked source function and anonymous capture remains
visible in its ordinary frame.

Exact Function values may also inhabit recursively nested private Tuple and
Record values. The checked frontend retains a structural fact tree beside the
ordinary aggregate expression; each Function leaf records its named, symbolic,
or anonymous callable identity independently of the i32 observation field that
LLVM transports. Bindings, Record selection, and anonymous product projection
recover the appropriate subtree, so eventual application remains a direct
specialization. A local aggregate may retain a capturing callable while its
lexical values remain alive. The capture-free base boundary carries the same
structural facts without an environment transport. Exact `Optional Function`
containment extends that tree under `TOPAL-COMPILER-OPTIONAL-FUNCTION-001`.
Exact selected nominal Sum payloads extend it under
`TOPAL-COMPILER-SUM-FUNCTION-001`, retaining the semantic alternative
separately from the runtime tag. Exact arithmetic Result success payloads extend
it under `TOPAL-COMPILER-RESULT-FUNCTION-001`, retaining conditional success
facts independently of the runtime success/Error tag. Exact finite Lists extend
it under `TOPAL-COMPILER-LIST-FUNCTION-001`, retaining length and one ordered
fact subtree per entry. Exact fixed-size Arrays collected from those Lists
extend it under
`TOPAL-COMPILER-ARRAY-FUNCTION-001`, retaining the declared extent and the same
ordered entry subtrees. Exact nonempty String-keyed Maps extend it under
`TOPAL-COMPILER-MAP-FUNCTION-001`, resolving duplicate keys according to the
source collision policy and retaining one value subtree per surviving exact
key. Other containers and opaque or branch-selected identities still fail
before LLVM.

Private definitions and calls use recursively exact LLVM aggregates under
`fastcc`; LLVM owns their target register, stack, and return coercion. DWARF
describes the source aggregate recursively and renders Function members through
the existing private enumeration. The tags never dispatch, and no closure
object, environment pointer, allocation, or public ABI results. Canonical
compiled-library metadata must eventually record each Function field's
structural path, callable and capture identities/classifiers, lifetime, effects,
representation identity, and target adapter rather than publish this LLVM
aggregate or its observation tags.

The capture-bearing extension assigns every Function leaf a canonical path of
zero-based Tuple indexes, Record labels, admitted `Optional` payload edges,
admitted nominal Sum alternative-name payload edges, and admitted `Result`
success edges, plus zero-based finite List-entry edges, visited depth-first from
left to right. Exact Arrays add zero-based Array-entry edges, and exact Maps add
semantic String-keyed Map-value edges in first-key occurrence order after
collision resolution. Parameter specialization carries the ordinary source
aggregate followed by each leaf's ordered capture operands.
Result lowering returns a private aggregate whose first field is the unchanged
source aggregate and whose remaining fields are the captures in that same path
order. The caller extracts the source value once, attaches each capture to its
checked callable facts, and can forward, select, or recursively destructure it
without replaying construction.
Capturing anonymous results, nested Function parameters, and exact nested
Function results under `TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001` are therefore
direct specializations; Function-containing capture state remains rejected.

LLVM still owns AMD64 register, stack, and aggregate-return placement for every
matching `fastcc` prototype. DWARF exposes only the source
Tuple/Record/Optional/Sum/Result/List/Array/Map and the eventual callable's
source-named captures, never the extended result fields or hidden parameter
names. This transport
remains module-private and creates no closure object, environment pointer,
allocation, callback, function
pointer, indirect dispatch, foreign dependency, or native ABI change. A future
compiled library must serialize the canonical aggregate path, callable/capture
identities, capture classifiers and order, lifetime, effects, representation
identity, and target adapter; it must not serialize the current private LLVM
aggregate, observation tags, or hidden-operand layout.

The exact nested-result extension reuses these scalar and aggregate transports
for one nonrecursive, nonoverloaded nested declaration whose identity remains
known throughout the private path. A factory result contains the nested
observation tag followed by its already-evaluated lexical, defining-context,
and live-root values; a Tuple, Record, Optional, Sum, Result, finite List,
fixed-size Array, or exact nonempty String-keyed Map result associates the same
values with the Function leaf's canonical path. The caller immediately owns
those SSA snapshots, so the factory frame may return before a later direct
application.
Separate factory calls may share the declaration tag but retain independent
capture values. Private forwarding remaps storage identities without replaying
the factory or consulting a dead frame. O0 DWARF exposes the factory result as
`Function`, the source aggregate without hidden fields, and the captures as
source-named arguments in the eventual nested frame. Recursive or overloaded
nested identities, Function- or Generator-containing capture state, opaque or
dynamic selection, persistent storage, and public/library escape remain
rejected pending canonical lifetime/effect, callable/capture-schema,
representation, and target-adapter metadata.

The exact Optional extension boxes only the existing private i32 Function
observation value beneath the Topal-owned `Some` header; `None Function` has no
payload. The checked fact tree separately records exact absence or the present
callable identity and ordered captures. A `Some` decision recovers those facts
for its payload binding, so application is one direct specialization rather
than Optional-tag or Function-tag dispatch. Optional parameters and results may
forward the source pointer followed by path-ordered captures, including an
escaping exact nested Function environment. Repeated anonymous-pattern
identity first compares the ordinary Optional tag/payload representation and,
when both present callables have the same identity, compares both admitted
capture snapshots. Opaque or branch-selected present identities and a
Function-valued capture remain rejected. Allocation is limited to the existing
Topal Optional representation and does not create a closure/environment object.
DWARF and the GDB printer retain `Optional Function` and render the boxed
Function observation value without exposing the hidden capture transport.
Future compiled-library metadata must identify the Optional payload path,
presence requirements, callable/capture schema, representation, lifetime,
effects, and target adapter independently of the private header, observation
tag, or hidden operands.

The exact nominal Sum extension preserves the existing private i32 tag plus
declaration-ordered statically typed payload slots. The checked fact tree
separately records the exact selected alternative and its active payload facts;
a complete decision recovers those facts for the payload binding so eventual
Function application remains a direct specialization. Captures follow a
semantic alternative-name path and the ordinary source Sum aggregate travels
before the hidden capture operands or result fields. Repeated anonymous-pattern
identity compares tags and the active payload observation first, then matching
capture snapshots only for the same retained callable. Inactive slots are never
observed. No allocation, closure object, tag dispatcher, or public Sum ABI is
introduced. Future compiled-library metadata must preserve nominal identity,
ordered alternatives and payload schemas, active-selection proof, callable and
capture paths, representation, lifetime, effects, and target adapters
independently of private tags, inactive LLVM layout, hidden operands, and
physical placement.

The exact Result extension preserves the existing Topal-owned success/Error
pointer representation. A success boxes the private i32 Function observation;
an Error remains the original structured Error, including provenance. The
checked fact tree records the callable and capture schema that applies
conditionally to the success payload. A complete Result decision attaches
those facts only to the `Ok` binding, so eventual application is a direct
specialization. Captures follow a semantic Result-success path, and private
parameters/results carry the ordinary source pointer before hidden captures.
An early Error return fills otherwise-unobservable hidden capture result fields
with representation-valid zero carriers, never evaluates skipped capture
initializers, and exposes neither those carriers nor a callable on the Error
path. Allocation remains limited to the existing Result object and successful
i32 payload box. Future compiled-library metadata must preserve the Result
success classifier and Error-code vocabulary, conditional-success proof,
propagation semantics, callable/capture paths, representation, lifetime,
effects, and target adapters independently of private headers, zero carriers,
LLVM symbols, and physical placement.

The exact finite List extension preserves Topal-owned singly linked nodes and
null `Empty`. Each Function node contains the private i32 observation followed
by the aligned next pointer; callable facts and immutable captures remain
compiler-only and do not enlarge the source node. The checked fact vector
records exact length, entry order, and one callable/capture schema per entry. A
complete List decision transfers the head facts to its first binding and the
remaining vector to its rest binding, so eventual application remains a direct
specialization. Captures use zero-based source entry paths and travel after the
ordinary List pointer across private parameters/results; a returning caller
owns and remaps them once. Distinct positions and factory invocations retain
independent snapshots. Allocation remains limited to existing process-lifetime
List nodes. Future compiled-library metadata must preserve exact length/order,
entry paths, callable/capture schemas, List representation/ownership, lifetime,
effects, and target adapters independently of node offsets, observation tags,
private names, LLVM types/symbols, and physical placement.

The exact fixed-size Array extension materializes an exact finite
`List Function` with the existing 16-byte Topal-owned Array header: an i64
entry count followed by a pointer to the immutable List nodes. It does not copy
or enlarge those nodes. The checked fact vector retains the declared extent,
entry order, and callable/capture schema. Exact checked indexing produces the
ordinary `Optional Function` representation and transfers the indexed facts
only on the `Some` path; an out-of-bounds exact index produces `None`. Captures
use zero-based Array-entry paths and travel after the ordinary Array pointer
across private parameters/results. Distinct entries and factory invocations
retain independent snapshots, and eventual application remains a direct
specialization. Allocation is limited to the existing Array header and the
ordinary successful Optional Function observation box. Future compiled-library
metadata must preserve the exact extent/order, source collection relationship,
Array and List representation/ownership, entry paths, callable/capture schemas,
lifetime, effects, and target adapters independently of headers, node offsets,
observation tags, private names, LLVM types/symbols, and physical placement.

The exact Map extension collects an exact nonempty
`List (String, Function)` with the existing 16-byte Topal-owned Map header and
24-byte linked nodes. Each node retains a String pointer, the ordinary i32
Function observation in its value slot, and a next pointer; captures do not
enlarge the header or nodes. The checked fact vector requires exact String keys,
resolves `reject`, `keep-first`, or `keep-last` before LLVM, and retains one
callable/capture schema per surviving key in first-key occurrence order. An
exact-key lookup produces the ordinary `Optional Function` representation and
transfers only that key's facts on `Some`; a missing exact key produces `None`.
Captures use semantic String-keyed Map-value paths and travel after the ordinary
Map pointer across private parameters/results. Distinct keys and factory
invocations retain independent snapshots, and eventual application remains a
direct specialization. Allocation is limited to existing source List nodes,
Map nodes and header, and the ordinary successful Optional Function observation
box. Until the interpreter has a typed empty generic Map construction, empty
Map Function collection remains rejected to preserve shared source parity.
Future compiled-library metadata must preserve collision policy, exact key set,
first-occurrence/collision-resolution relationship, Map/List
representation/ownership, semantic key paths, callable/capture schemas,
lifetime, effects, and target adapters independently of headers, node offsets,
observation tags, private names, LLVM types/symbols, debug shadows, and physical
placement.

An inferred anonymous Function may recursively destructure positional products.
The checked frontend materializes the complete call operand once, then walks
the pattern and exact Tuple types depth-first from left to right, representing
each nested field as a projection from that private value. The source arity and
anonymous Function identity count top-level patterns; the private LLVM
prototype instead lists every admitted leaf in lexical order, followed by
captures. This composes with direct, bound, capturing, and returned anonymous
Functions while keeping all calls exact and direct. DWARF exposes the source
leaf names and values but not the projection carrier, and no pattern object,
allocation, runtime descriptor, or public ABI is introduced.

A repeated non-discard name across ordinary anonymous parameters or recursively
destructured product leaves retains every consumed private machine operand but
creates only the first source binding. Later same-classifier scalar occurrences
become lexically ordered function-entry identity guards. They reuse direct
exact integer, Rational, String/Character, or enum-like comparison. A
capture-free Function compares its observation tag. When both operands retain
the same captured anonymous source or captured named declaration identity, the
boundary additionally forwards both already-evaluated ordered capture
snapshots. Anonymous sources receive one canonical tag across their private
specializations; captured named identity uses the exact source name and
declaration set, initially for a non-escaping nested Function within its
defining invocation. The guard compares callable identity first and then every
capture having admitted exact compiler equality. A different callable identity
mismatches without transporting the later capture state. Missing, inconsistent,
or required non-equality capture facts fail before LLVM. Neither case
invokes user Equality, conversion, canonical equivalence, or tag dispatch. A
mismatch calls a Topal-owned diagnostic helper which writes through the Linux
syscall service and exits 65 before the body. Repeated operands are
omitted from DWARF variable metadata, leaving one unambiguous source parameter.
The same metadata and control-flow shape extends to repeated aggregate values
only when the private boundary and exact comparison already exist: recursive
Tuple/Record fields are decomposed in semantic order, while admitted Optional
and List pointers reuse their Topal-owned tag/payload or ordered-entry
comparisons. A recursive Tuple/Record may also contain Function leaves when the
frontend retains one exact callable fact at every path. Those leaves compare
their deterministic private i32 observation fields; the facts continue to
select direct specializations, and the fields never dispatch control flow. A
captured Function leaf uses the same canonical aggregate path and ordered
capture transport as other private Function aggregate boundaries. When every
corresponding leaf has the same stable callable identity, both capture
snapshots follow the source operands and guards compare them after all ordinary
aggregate fields. If any callable identity differs, its observation field
makes the aggregate unequal and those captures need not cross the boundary.
Missing or opaque callable facts, inconsistent required capture schemas, and
captures without admitted exact equality fail before LLVM. LLVM still owns
scalar/aggregate register and stack coercion, and the first occurrence alone
receives the existing target-derived debug shadow. No generic aggregate
matcher, closure/environment object, or allocation identity is introduced.
Canonical library metadata must record canonical aggregate paths, stable
callable identities, ordered capture schemas/classifiers, semantic equality
requirements, representation identity, lifetime/effects, and target adapters
independently of the module-private tag, hidden-parameter layout, and LLVM
types. Result repeated identity, Range, Generator, refined, authority-bearing, unsupported
capture classifiers, recursive/overloaded or otherwise unsupported escaping
nested callable identity, Function containment outside admitted
Tuple/Record/Optional/Sum/Result/List/Array/Map paths, and ordinary named-header repetition remain
deferred with their broader representation and overload consequences.

Named nested lexical functions declared directly in an ordinary function body
establish the first private capture boundary without choosing that general
closure design. At the declaration point, the frontend snapshots the finite
visible immutable environment and retains every value with an admitted private
representation, except names shadowed by the nested function's explicit
parameters. Direct application specializes the nested declaration and passes
source parameters followed by those original SSA values as deterministic exact
hidden `fastcc` parameters. Under
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`, an exact private result may instead
materialize the nested observation tag and those same immutable values; it
still introduces no function pointer, indirect dispatch, environment
allocation, or caller-frame reference. LLVM owns the physical AMD64 parameter
and result classification, while DWARF presents the nested source frame and
both explicit and captured arguments under their source names. Static,
effectful, recursive, overloaded, sibling-referencing, opaque/dynamically
selected, persistently stored, or published nested closures; name collisions
with visible or active callables; and captures of callable, Scope, or
constraint/evidence state remain deferred to the unified closure and
library-interface design.

One or both syntactic operands may contain a closed package at the same checked
boundary, and either package may be mixed with an admitted ordinary scalar
operand. Package fields may use admitted machine scalars, exact capture-free
Tuple/Record classifiers, exact nominal Sum classifiers, or exact direct
Function values already supported by the private function ABI. Exact Tuple and
Record fields may also contain admitted Function leaves and their represented
immutable capture snapshots. Exact List, Optional, Result, and Range fields may
also reuse the pointer-carrier representation already admitted at private
function boundaries when their contained classifiers are supported and contain
no Function. Exact `Array (N, Int)`, `Set Int`, `Bag Int`, and
`Map (String, Int)` fields likewise reuse their existing private pointer
carriers while the checked model retains the constructor, Array extent, and
element or key/value classifiers independently of that coincident machine
representation. An exact root or root-alias `Scope` field reuses the existing
specialized private Scope boundary: the checked model retains namespace
identity, snapshot declarations, overload order, and represented data facts
beside the sealed observation value, while the call appends the exact material
data values already required by that boundary. A full positional product
already has declaration order. A labeled product is instead associated by
stable field identity. For a
compound call, the frontend retains every explicit operand and field once in
global source order through compiler-private SSA bindings, then evaluates
omitted defaults in operand/field declaration order and permutes retained values
into that same declaration order. A direct Function binding retains its exact
callable identity and capture facts beside the observation tag; a
Function-containing aggregate binding retains the complete recursive structural
fact tree, canonical Function paths, and callable/capture facts beside the
runtime aggregate; and a represented container binding retains its exact generic
classifier beside the pointer carrier. An Array/Set/Bag/Map binding additionally
retains its exact constructor and extent/element/key/value facts beside the
pointer carrier. A Scope binding retains its exact namespace snapshot and
represented data facts beside the observation tag. Reordering therefore cannot
make any admitted field opaque or replay its initializer.
The private callee receives one ordinary LLVM parameter per unpackaged operand
or source field; a structured field remains one exact aggregate parameter
rather than being decomposed into more package fields. This makes every
parameter classifier and DWARF binding explicit while LLVM retains
responsibility for target register/stack placement. It also avoids `byval`,
`sret`, `inalloca`, and `preallocated`: those attributes encode specific
memory/ABI obligations and are reserved for a deliberate public aggregate
interface rather than being inferred from source packaging syntax. The private
bindings and hidden callable-capture transport are deliberately absent from
source-level debugging. Material Scope data parameters remain visible under
qualified names as required by the existing Scope boundary. Function
containment outside admitted Tuple/Record/Optional/Sum/Result/List/Array/Map paths,
opaque/computed/nested/non-root Scope values, unsupported container or
collection payloads and other non-scalar fields, nested package declarations,
opaque whole-package values, context-dependent
defaults, recursive compound signatures, and public package adapters remain
checked-frontend and library-interface work.
Published metadata will need syntactic-operand partition/order, stable field
identities and declaration order, complete canonical structural or nominal
classifiers, Sum alternatives/payloads, collection constructors and Array
extents, element/key/value classifiers, collection ordering/uniqueness/
multiplicity/collision semantics, namespace identity and snapshot position,
member visibility/declaration order, complete namespace function overload,
generator, represented-data, and hidden-data ordering schemas, canonical
Function-leaf paths, callable
source/declaration identity and overload sets, capture schemas, default
semantics and dependencies, generic container constructors and contained/error-
domain/endpoint classifiers, evaluation effects, representation/lifetime
identity, and target adapters without exposing compiler-private binding names,
LLVM types, or physical placement.

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

The initial infinity increment keeps that finite representation and every
private function signature unchanged. A contextual root-local exact infinity
is retained in the checked model as an internal `InfiniteInt` or `InfiniteNat`
proof type, then lowered to one immutable executable-private Int-shaped
sentinel: sign tag 2 denotes positive infinity, sign tag 3 denotes negative
infinity, and both require zero limbs. No sentinel is admitted at a function,
library, persistence, or serialization boundary. Runtime comparison and output
test these tags before entering finite limb logic; the existing opaque Range
header can therefore retain them as exact endpoints. LLVM still owns pointer
placement and instruction selection, while the frontend owns the closed
semantic admission boundary. Because the tags cannot appear in a
`topal-native/6` signature and change no finite object, this executable-local
representation does not revise that ABI. Future boundary admission requires
canonical metadata for the numeric domain and infinity semantics rather than
publishing these tags.

The next infinity increment retains a Rational infinity as a normal private
Rational header whose numerator is the corresponding executable-private Int
sentinel and whose denominator is canonical one. The compiler constructs that
pointer-bearing header at run time, so the static PIE still requires no loader
relocation. Rational comparison and display recognize the numerator sentinel
before finite cross-multiplication, while the opaque Range header continues to
retain endpoint pointers unchanged. The checked model uses an internal
`InfiniteRational` proof type and rejects cross-domain Int/Rational infinity
conversion, functions, publication, persistence, serialization, and libraries
before lowering. Consequently neither the finite Rational layout nor the
`topal-native/6` machine ABI changes.

The arithmetic increment carries a direction fact beside each closed checked
infinity binding. It admits unary operations and `+`, `-`, or `*` only when the
result direction or a statically indeterminate form is provable. LLVM O0 still
calls the runtime operation: Int negation, absolute value, addition,
subtraction, multiplication, and zero testing recognize the sentinel before
finite storage logic. Rational arithmetic composes those operations and its
canonical constructor returns a sentinel numerator over denominator one before
finite greatest-common-divisor reduction. Thus correctness does not depend on
constant folding. Opposite sums, equal-direction infinity subtraction, and
statically proven zero products are source diagnostics. A runtime-dependent
zero product is an ordinary arithmetic `Result`, with zero producing the
`indeterminate` code and nonzero returning the signed infinity. This path is
restricted to closed root expressions: it reuses the private Result/Error
representation and source provenance without admitting an infinity at a Topal
function or external boundary. Dedicated Int and Rational helpers validate
that one operand is the expected sentinel, test zero before finite arithmetic,
and fail closed if checked code violates that invariant. At O0 the generator
appends this runtime fragment only when the checked program contains the
fallible operation; this is dependency selection rather than optional semantic
optimization, and unrelated programs do not compile or carry the helpers.

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

Nominal modular values reuse the canonical arbitrary-precision Int pointer as
their private carrier rather than acquiring a machine-width representation.
The checked model retains the declaration identity and canonical inclusive
bounds; generated wrapping arithmetic performs the exact Int operation and
then lowers `lower + ((value - lower) modulo modulus)` through existing runtime
calls. This gives `-O0` the specified result for positive and negative ranges
without relying on LLVM overflow behavior. Private `fastcc` functions carry the
pointer directly and leave physical AMD64 lowering to LLVM. Distinct DWARF
storage identities and typedefs restore the nominal source type for GDB even
though the runtime layout is shared. This is not a public, foreign,
serialization, or compiled-library numeric ABI, and it does not revise
`topal-native/6`.

A modular declaration may resolve an earlier immutable root binding whose
initializer is a closed finite inclusive Int range; the compiler substitutes
that source expression while collecting declarations, before ordinary value
lowering. Checked construction uses static interval evidence when conclusive.
Otherwise generated control flow compares the once-evaluated arbitrary-
precision Int with both inclusive bounds and joins either the original Int
pointer in the existing Result success header or a source-located
`root.Name(Int)`/`out-of-range` Error. A modular-success Result uses the same
private two-word runtime header as every arithmetic Result. Its specialized
DWARF header changes only the debug payload type from opaque pointer to the
nominal modular typedef, preserving layout while keeping that type reachable
and renderable in GDB. This extension therefore needs neither a platform ABI
choice nor a `topal-native/6` revision.

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

Closed native serialization reuses the canonical `topal-serialization` codec
only while checking and lowering. The emitted executable contains the resulting
relocation-free protocol bytes, not the Rust codec or any Rust/C/C++ runtime.
At source evaluation, the Topal runtime copies those embedded expected bytes
into private stream storage, and a private 16-byte Topal-owned descriptor records
the copy's address and count. The published stream copy is thereafter immutable.
The separately retained source value is still evaluated exactly once; `lang
deserialize` checks the descriptor count and compares every byte in the copy
with the compiler-validated expected stream before it may return that retained
value. This is a fused compiler-created producer/consumer representation, not a
general runtime protocol parser, and malformed private state takes the
corruption exit. Arbitrary or external streams remain outside this increment.

The closed external-location increment retains layout, range, offset, and
location semantics in frontend metadata rather than encoding them as LLVM
types. Its source graph names an MMIO medium but grants no Linux device mapping,
resource capability, or native-adapter authority. The executable consequently
must not convert `0x40000000 + 32` to a host pointer. It uses a Topal-owned
process-private header containing arbitrary-precision range-start and offset
evidence, an initialization tag, and the immutable semantic Nat snapshot. The
write and read remain separate noinline calls in source order at `-O0`; this is
the interpreter-equivalent authority-free model, not a real-MMIO optimization
or integration claim. An adapter-backed implementation will require explicit
platform-package authority, volatile/atomic ordering appropriate to the device,
fallible error translation, and target-qualified address rules.

The private header is expressed as an ordinary LLVM structure with opaque
pointers. LLVM therefore continues to own physical layout, instruction
selection, registers, and scheduling, while the frontend owns schema
validation, representability, fit, authority rejection, and access order. The
runtime allocates only through the existing Linux `mmap` syscall boundary and
adds no foreign symbol or language runtime. DWARF describes the nominal
layout-backed Nat and Location header; the bundled renderer bounds all reads and
validates pointers, initialization state, and the stored semantic value.

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

Closed direct `characters` foreach keeps that segmentation decision in the
checked frontend. For a compile-time-known preserved String, the model retains
the ordered complete grapheme clusters and a capture-free Character-to-Unit
action. The backend materializes each Character through the existing immutable
String descriptor, evaluates the source expression once, and emits the action
inline in preserved order; empty input emits no action. This is required O0
semantic expansion rather than LLVM loop
discovery or Unicode interpretation. A debug-only pointer stack shadow keeps
the current Character inspectable even when its action is otherwise erased.
There is no semantic traversal allocation, Generator object/token, dispatcher,
callback, indirect call, foreign Unicode table, or other-language runtime.
Dynamic traversal stays closed until Topal owns the generated pinned-Unicode
path, while compiled-library boundaries additionally require canonical
Generator, segmentation, action-evidence, ownership, and target-adapter
metadata.

A named closed Character generator retains the same ordered cluster evidence
beside its root binding in compiler-session provenance. Runtime construction
evaluates the String once and exposes only the existing private semantic
Generator token for observation and DWARF; that token never carries a cursor or
drives traversal. Consuming the local binding transfers its retained evidence
to the direct expansion, then the checked model rejects every later source use.
This establishes local linearity without a copyable continuation object.
Root abandonment, general/dynamic function results, and nonspecialized
parameter traversal remain closed until a general owned state and close path
exist. Canonical library metadata must identify the Generator classifier,
pinned segmentation, evidence, ownership, and target adapter independently of
the private token.

Closed unchanged Character collection also keeps segmentation in the checked
frontend, but needs no runtime traversal representation. The checked model
retains the ordered clusters as proof that `characters text collect String`
has the exact preserved source sequence. The backend evaluates `text` once and
forwards that immutable String descriptor as the result. This is explicit O0
semantic lowering, not LLVM identity discovery, and does not generalize to a
selected, mapped, stored, or otherwise transformed traversal. It therefore
adds no intermediate List, concatenation loop, Generator state, Unicode
runtime, foreign dependency, or ABI surface; library use remains gated on
canonical traversal evidence and target-adapter metadata.

A narrow built-in close boundary transfers that private token into an ordinary
single-parameter Unit function whose statement-free Unit body leaves it
untraversed. The checked model appends an explicit close at scope exit. Since
the closed Character generator owns no runtime object or live state, generated
close is a no-op; the private call still uses LLVM `fastcc` so the backend, not
Topal source semantics, selects AMD64 register placement. A debug-only `i32`
stack shadow keeps the otherwise unused semantic parameter inspectable. This
does not establish a public Generator ABI: general traversal, return, state,
close dispatch, and library metadata remain closed.

A second narrow boundary specializes a single-parameter Unit function whose
only executable body is capture-free Character foreach. Before checking each
top-level call instance, the compiler transfers that argument's retained
cluster sequence under the parameter's local name, then restores the
compiler-session map and linearity set afterward. Every call has a distinct
private symbol, so different closed Strings cannot share provenance. The caller
evaluates the String once and passes the same `fastcc i32` ownership token; the
callee expands the exact Characters and action inline, exhausts the traversal,
and returns without close. Nested calls stay closed. This is compile-session
specialization, not serialized state or a public ABI, and adds no callback,
indirect call, or Generator runtime.

Fresh function results use the same private representation only for a
statement-free ordinary function specialized from one exact top-level String
argument. The checked body must be exactly `characters` of its String
parameter. Each private symbol keys a compiler-session side table containing
that call's pinned Character sequence; binding the call result transfers the
entry into the caller's ordinary linear Generator tracking. Generated code
passes the existing immutable String descriptor to the private function with
LLVM `fastcc`, returns the `i32` token without close, then expands traversal in
the caller. A debug-only source-parameter shadow preserves GDB inspection even
though no runtime segmentation occurs. Dynamic, recursive, nested, static,
anonymous, unbound, or compound result paths remain closed, and the side table
is neither artifact metadata nor a public Generator ABI.

The first custom-continuation result boundary applies the same private transfer
pattern to a statement-free ordinary Character function whose body directly
constructs the exact single-yield custom generator from its parameter. Each
call-specialized symbol keys the retained declaration, exact Character,
suspension/final graph, and ownership edge; the callee returns only the private
`i32` observation token through LLVM `fastcc`, and the caller expands its own
retained traversal after taking ownership. A debug-only Character pointer
shadow keeps the factory parameter live through the return instruction. The
side table remains compiler-session evidence rather than a serialized or public
continuation representation; compiled-library support still requires canonical
declaration, direction, suspension, capture/effect, ownership/close, and target
adapter metadata.

The first custom-continuation parameter boundary reverses that private mapping
for one root-owned exact single-yield instance. Before checking each ordinary
consumer specialization, the frontend maps the caller's retained declaration,
Character, suspension/final graph, and ownership edge to the sole Generator
parameter; it restores the compiler-session map afterward while leaving the
caller binding consumed. The caller passes only the compiler-private `i32`
observation token through LLVM `fastcc`, and the callee expands the retained
Character action, Unit resume, and final Unit directly. Generator and Character
debug shadows preserve source inspection across both frames. This adds no
continuation representation or public ABI; compiled-library transfer will need
canonical construction, parameter-site, action, capture/effect,
ownership/consumption/close, and target-adapter metadata instead of this private
specialization map.

An unconsumed exact custom parameter keeps the same mapping through an explicit
checked close node that pairs the callee-local Generator with its transferred
construction provenance. The node preserves declaration identity, exact
Character, suspension/final graph, ownership edge, and the lexical root close
domain even though none is materialized as native continuation state. Because
this first close boundary has a discarded yield result and no handler, locals,
cleanup, effects, or post-suspension work, O0 lowering proves the close and final
Unit before erasing both. LLVM still owns private-token placement through
`fastcc`, while Generator DWARF remains live in the callee. Compiled-library
support must serialize canonical transfer/close sites, domain, construction,
suspension, capture/effect/cleanup, ownership, and target-adapter evidence rather
than depend on the checked-program node layout.

The same private parameter mapping now preserves a distinct final Character
when the complete Generator classifier is
`Generator Character Unit Character`. The caller still transfers only the
compiler-private `i32` ownership token with LLVM `fastcc`; the callee expands
the retained yielded Character action, Unit resumption, and final Character in
that semantic order, then returns the ordinary immutable Character descriptor.
LLVM's target lowering selects both argument and return placement, so the
frontend embeds no AMD64 register convention. Generator, yielded-Character,
and function-result DWARF remain truthful even though no continuation object or
state machine exists. A compiled library must carry the canonical full
classifier, declaration, separate yield/final graph, construction/transfer
sites, action, capture/effect and ownership/close evidence, plus target-adapter
requirements; neither the private token nor the compile-session provenance map
is a serializable ABI.

The matching custom-continuation result boundary carries that complete
`Generator Character Unit Character` identity out of a statement-free factory.
The factory receives its exact Character as an ordinary immutable descriptor,
returns only the private `i32` ownership token with LLVM `fastcc`, and keys the
yield/final/suspension evidence by its private symbol. After binding the token,
the caller expands the retained yielded Character action and Unit resumption,
then materializes the distinct final Character. A Character parameter shadow,
Generator return type, and caller yield shadow keep both frames and all value
directions inspectable in GDB. The per-symbol map remains compile-session
specialization evidence: compiled-library support must serialize canonical
classifier, declaration, construction/result-transfer sites, yield/final graph,
action, capture/effect, ownership/close, and target-adapter metadata instead of
the token or map representation.

The next custom-continuation slice makes the initial direction independent by
accepting a String for `Generator Character Unit Unit`. Its checked provenance
stores the classified initial parameter and a small ordered pre-suspension
block separately from the Character yield and Unit resume/final directions.
At application, O0 lowering evaluates the existing immutable String descriptor
once, calls the Topal-owned emptiness operation, retains Boolean debug evidence,
and only then exposes the private `i32` suspension token. Later traversal
materializes the exact Character action and Unit resume without a continuation
object or semantic state allocation. LLVM owns target data layout and machine
instruction selection; neither the frontend nor the Topal syscall runtime
embeds a System V register rule. Compiled libraries must eventually serialize
the independent initial classifier, ordered phase/binding and suspension graph,
construction/action sites, captures/effects, ownership/close state, and target
adapter requirements instead of the checked-node or private-token encoding.

The following custom-continuation slice makes the yield direction independent
by accepting `Generator String Unit Unit` with a String initial value. Checked
provenance records each suspension as either the once-evaluated initial
descriptor or an exact String literal, independently of the Generator's initial,
resume, and final classifiers. Root traversal expands those values, actions, and
Unit resumptions in source order at O0; it reuses the captured initial SSA value
instead of evaluating the application operand again and allocates no semantic
continuation state. The `i32` Generator token remains private debug/ownership
evidence, while yielded Strings use the existing `topal-native/6` descriptor.
LLVM continues to own target layout and machine calling-convention placement;
the frontend embeds no System V register rule, and the Linux runtime remains
Topal-owned allocator/syscall code with no C/C++ standard library. Compiled
libraries must eventually serialize independent initial and direction
classifiers, ordered yield-value provenance, suspension/final graph,
construction/action sites, captures/effects, ownership/close state, and target
adapter requirements rather than either compiler-session representation.

The next value-continuation slice carries an exact distinct final String through
`Generator String Unit String`. The checked graph keeps that final expression
and classifier separate from the once-evaluated input, ordered yields, and Unit
resume edges. O0 traversal expands each yield/action/resume before creating the
final immutable String descriptor, so source semantics do not depend on an LLVM
optimization or a semantic continuation allocation. The full Generator
classifier, private `i32` ownership token, yielded value, final-expression source
location, and entry frame remain represented in DWARF. LLVM retains control of
target layout and register placement, and the Linux runtime remains Topal-owned
allocator/syscall code with no C/C++ standard library. Compiled libraries must
eventually serialize independent direction classifiers, ordered yields, the
distinct final-value expression and provenance, graph sites, captures/effects,
ownership/close state, and target adapters instead of compile-session nodes or
tokens.

The next resumption-continuation slice records one ordinary discarded String
emptiness computation between suspensions. The checked graph stores a typed
continuation block and its successful-resumption ordinal separately from the
once-evaluated initial descriptor, ordered yields, action, and final Unit. At O0
the expanded traversal invokes the first action, resumes with Unit, enters a
generator-lexical block that observes the captured initial String, and only
then materializes the following yield. The initial String, both action values,
full Generator classifier, continuation source site, private `i32` ownership
token, and entry frame remain available to DWARF/GDB without allocating a
semantic continuation object. LLVM still controls target layout and register
placement, while Topal-owned allocator/syscall code provides Linux integration
without a C/C++ standard library. Future compiled-library metadata must encode
ordered body phases, successful-resumption ordinals, expression provenance,
initial and direction classifiers, suspension/final sites, captures/effects,
ownership/close state, and target adapters rather than compiler-session blocks
or tokens.

The next completion slice distinguishes an explicit `return "..."` reached
before any suspension from an implicit generator final expression. Checked
provenance carries the return keyword/site, exact String result, independent
directions, and empty yield/continuation evidence. Application still evaluates
its initial String once; root traversal observes the explicit completion,
invokes no action, and materializes the result descriptor directly at O0. A
debug-only stack shadow keeps the otherwise unused initial parameter live at
the return site without creating semantic Generator state. The full classifier,
private `i32` ownership token, initial value, return location, and entry frame
remain available to DWARF/GDB, while no yielded action binding is fabricated.
LLVM owns target layout and register placement, and Linux integration continues
through Topal-owned allocator/syscall code without a C/C++ standard library.
Future compiled-library metadata must distinguish explicit/implicit completion,
return provenance, reachability and empty-yield evidence, direction
classifiers, graph sites, captures/effects, ownership/close state, and target
adapters rather than checked nodes or debug storage.

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

The remaining Error observations preserve the already allocated Error object.
`detail` and `cause` load their reserved nullable pointer slots and translate
null only into the matching nominal Optional `None`. `source` first checks the
stored source descriptor; when present, it copies the stored one-based line and
column into canonical Int objects referenced by a private 16-byte
SourceLocation header. The existing Optional header then carries the String,
Error, or SourceLocation pointer with its statically retained classifier.
SourceLocation display loads the two Int pointers and uses the Topal integer
writer, while DWARF describes the same two-field layout and GDB validates and
decodes it. This allocation is semantic representation work required at O0,
not an optimization. LLVM remains responsible for ordinary branch and private
calling-convention lowering. Because Error's reserved fields and Optional's
opaque payload layout do not change, the new previously unavailable private
type does not revise `topal-native/6` or create a public/foreign interface.

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

The first immutable container representation specializes `List Effect` without
claiming a generic or public List layout. `Empty` is a null pointer. Each
`Entry` is a naturally aligned 16-byte process-lifetime node with the sealed
one-byte Effect carrier at offset zero and its remaining-node pointer at offset
eight; padding is not source state. Construction evaluates the value before the
remaining List, then allocates through the existing Linux `mmap` boundary.
Canonical output walks the chain iteratively and emits closing constructors
without recursion, while the GDB renderer bounds traversal and rejects cycles
or unreadable nodes. Private `fastcc` signatures carry only the pointer and let
LLVM select physical AMD64 placement. DWARF describes the semantic
`List Effect` typedef and private node shape, but neither that description nor
the storage becomes a foreign ABI, serialized identity, compiled-library
contract, or layout commitment for other element types. General reachability
reclamation remains deferred; process-lifetime retention is safe for this
immutable executable-only slice and does not revise `topal-native/6`.

The completed empty-Effect specialization adds a conditional finite equality/
count fragment, exact i8 decision loads, direct null emptiness, and private
Tuple/Record/package passage. Since every currently admitted payload is the
canonical empty row, equality is exact length equality; this is not a layout or
identity rule for future nonempty effect rows. DWARF and GDB retain `List Effect`,
and future library metadata must carry effect-row identity/evidence separately
from node representation, ownership, lifetime, effects, and target adapters.

`List Comparison` uses immutable 16-byte private nodes whose first four bytes
retain the existing closed i32 carrier (`Less = -1`, `Equal = 0`,
`Greater = 1`) and whose remaining pointer begins at byte eight. A conditional
finite fragment compares exact carriers and counts nodes; decisions use i32
loads and emptiness remains a null test. LLVM owns private AMD64 placement, and
DWARF/GDB preserve both `List Comparison` and each alternative. The node is not
a public enum or container ABI; future library metadata carries the semantic
alternative mapping, representation/ownership/lifetime/effect facts, and a
versioned target adapter independently of offsets, LLVM types, and helpers.

`List ErrorCode` uses the same private physical class while retaining the
distinct `lang arithmetic ArithmeticErrorCode` vocabulary and its four closed
qualified alternatives. Its conditional finite fragment compares exact i32
tags and counts nodes; decisions, display, DWARF, and GDB map those tags only
through that vocabulary. The mapping is not a generic ErrorCode ABI and must
not be reused for a future vocabulary by tag coincidence. Library metadata
therefore carries canonical vocabulary/alternative identities alongside
representation, ownership, lifetime, effects, and the target adapter.

`List Unit` uses immutable 16-byte private nodes with a validated zero i8
carrier and remaining pointer at byte eight. Because every entry is `()`, its
conditional finite equality fragment compares length; counting, decisions, and
emptiness remain finite at O0. Unit-returning private functions still use their
existing void result convention, while Unit fields inside aggregates retain an
i8 carrier and LLVM selects physical AMD64 placement. DWARF/GDB preserve `Unit`
and `List Unit`; neither carrier supplies `Completed` evidence or publishes a
node ABI. Future metadata records Unit identity, representation/ownership,
lifetime, effects, and target adapters independently of private details.

`List Completed` uses a distinct compiler-selected specialization with an
immutable validated-zero i8/pointer node. Its conditional finite fragment also
compares length and counts nodes, but decisions yield explicit `Completed`
evidence and private function results retain the existing typed i8 convention.
LLVM owns physical AMD64 placement; DWARF/GDB preserve the distinct Completed
and List identities. Similar private layouts never make Completed equivalent to
Unit or Effect, and future metadata carries completion semantics, representation
and ownership, lifetime, effects, and target adapters independently of offsets.

`List Type` uses immutable private 16-byte nodes with the existing closed i32
fundamental-Type carrier and remaining pointer at byte eight. Its conditional
finite fragment compares exact canonical identities and counts nodes; decisions
recover the existing typed enum value. LLVM owns physical AMD64 placement and
DWARF/GDB retain both `List Type` and all seven fundamental identities. The i32
mapping is neither a host/LLVM descriptor nor a public reflection ABI. Future
metadata carries the canonical identity set, representation/ownership/lifetime/
effect facts, and a versioned target adapter independently of private numbers,
offsets, types, and helpers.

Lists of payload-free nominal Enums use the same private physical class while
retaining the exact declaration identity and ordered alternatives in the checked
model and debug metadata. A conditional shared finite fragment compares i32 tags
only for operands already proven to have the same nominal List classifier and
counts nodes at O0; decisions recover that exact nominal enum. LLVM owns physical
AMD64 placement. The declaration-local tag mapping is neither portable nor
interchangeable with another enum. Future metadata carries canonical declaration
and alternative identities, representation/ownership/lifetime/effects, and a
versioned target adapter independently of private tags, offsets, and helpers.

Lists of nominal modular values use immutable private 16-byte nodes containing
the existing canonical Int pointer and remaining pointer. A conditional common
finite fragment compares representatives with exact Int comparison only after
same-modular-declaration checking and counts nodes at O0; decisions recover the
exact modular type and pointer. LLVM owns physical AMD64 placement, and
private Tuple/Record passage keeps exact pointer fields. DWARF/GDB retain the
declaration name and value. No List operation narrows or re-reduces a
representative. Future metadata carries the canonical declaration, signedness,
bounds, representation/ownership/lifetime/effects, and a versioned target
adapter independently of private Int/List layouts, offsets, and helpers.

`List Optional Int` uses immutable private 16-byte nodes containing an existing
Optional header pointer and remaining pointer. Its conditional finite fragment
delegates entry comparison to Optional-Int equality and counts at O0. Decisions,
private aggregates, DWARF, and GDB preserve both container layers; metadata must
describe them independently of headers, node offsets, and helper names.

`List Optional Rational` composes the same private pointer-node strategy with
the canonical Rational payload already retained by each Optional header. Its
separate conditional fragment delegates exact entry comparison to
Optional-Rational equality. LLVM owns placement; DWARF/GDB and future metadata
retain all three classifier layers without exposing headers, offsets, or helpers.

`List Optional String` similarly stores each existing Optional header pointer in
an immutable private node and delegates element equality through Optional-String
to exact descriptor/byte comparison. Topal retains allocation and output; LLVM
owns placement, while DWARF/GDB and future metadata retain List, Optional, and
String identity without publishing either private representation.

`List Boolean` uses a separately selected 16-byte private node with the source
i1 value at offset zero, target padding that is never source state, and the
remaining-node pointer at offset eight. Construction and complete List
decomposition use checked element facts to select i1 loads and stores. A
conditional Boolean-List runtime fragment provides finite structural equality
and entry counting; emptiness is a direct null test. Private pointer passage,
Tuple/Record/package containment, canonical display, target-derived DWARF, and
the bounded GDB renderer retain the complete `List Boolean` classifier. LLVM
owns physical AMD64 argument and result placement. The fragment introduces no
type tag, generic node ABI, foreign allocator, C/C++ runtime, other-language
standard library, or native-ABI revision. Future compiled-library metadata must
record element classification, node representation and ownership, lifetime,
effects, and a versioned target adapter independently of private offsets,
helper names, LLVM types, and physical placement.

`List String` retains immutable 16-byte nodes containing the existing String
descriptor pointer followed by the remaining-node pointer. A separately
selected finite runtime fragment performs structural equality by delegating
each entry to canonical preserved-sequence String equality and counts nodes;
complete decomposition loads the exact descriptor pointer and emptiness remains
a null test. Private parameter/result/package and Tuple/Record passage uses the
same exact source pointer with LLVM-owned AMD64 placement. Canonical display,
target-derived DWARF, and GDB preserve String delimiters and the full List
classifier. This adds no String copy, host text API, foreign allocator, C/C++
runtime, other-language standard library, public/generic node ABI, or ABI
revision. Future compiled-library metadata remains independent of private node
offsets, helper names, LLVM types, and physical placement.

`List Character` is a distinct checked and debug classifier but uses the same
private immutable descriptor-pointer node shape as `List String`, because a
Character retains its complete String representation and derives exact String
equality. Construction proves the one-user-perceived-character constraint
before storage; decisions reload that unchanged descriptor. Equality and count
reuse the conditional finite String-List fragment, canonical display preserves
the complete scalar sequence and delimiters, and GDB selects the Character
element interpretation from DWARF. No code-point representation, normalization,
host text API, foreign runtime, public layout, or ABI revision is introduced.
Published-library metadata must preserve the Character classifier and constraint
evidence separately from the private physical reuse, along with representation,
ownership, lifetime, effects, and a versioned target adapter.

`List Nat` is another checked/debug specialization over an existing private
payload shape. Each node stores the exact Int-compatible pointer followed by
the remaining-node pointer; finite entries are validated as nonnegative before
publication and the existing positive-infinity sentinel remains a valid Nat
entry. Structural equality and counting reuse the finite Int-List fragment,
while decisions, display, DWARF, and GDB retain the Nat classifier. No unsigned
machine width, truncation, wrapping, second numeric representation, foreign
runtime, public layout, or ABI revision is introduced. Published-library
metadata must carry Nat constraint evidence, infinity capability,
representation, ownership, lifetime, effects, and a versioned target adapter
independently of this private physical reuse.

`List Rational` uses immutable 16-byte nodes containing the existing exact
Rational descriptor pointer and remaining pointer. Its conditional finite
runtime fragment compares entries through canonical Rational comparison and
counts nodes. Decisions, display, DWARF, and GDB retain exact reduced fractions
and infinities. LLVM owns private AMD64 placement; no floating-point, foreign
numeric runtime, public layout, or ABI revision is introduced.

`List Int` reuses only that node's private size and next-pointer position: its
first word is the existing canonical arbitrary-precision Int pointer rather
than an Effect byte. This is a statically selected node interpretation, not a
type-erased generic layout or runtime tag. Generated construction and display
select the exact payload load/store from the checked element classifier. The
three Int containment observations use finite, allocation-free runtime loops:
one compares each entry, one applies a consecutive-prefix check at successive
source positions, and one advances the pattern only after an equal entry.
Empty sequence and subsequence patterns return true before reading a source
node. All comparison delegates to the existing exact Int comparator, leaving
inputs unchanged and semantics independent of LLVM optimization. The compiler
adds this specialized internal runtime fragment only when checked containment
expressions require it, so unrelated modules do not pay its LLVM assembly
cost. Private calls still carry an opaque List pointer with LLVM-owned physical
placement; distinct `List Int` DWARF and GDB decoding restores the semantic
payload type without turning either List layout into a public or serialized
contract.

Int List removal is split into another compiler-selected internal fragment so
containment-only, removal-only, and unrelated modules assemble only the helpers
they require; the shared private node declaration is emitted once when either
family is present. Both removal operations scan with finite nonrecursive loops
and exact Int comparison. `remove-first` returns an unmatched input directly,
shares the untouched suffix after a match, and reconstructs only the preceding
entries. `remove-all` first proves whether any match exists and counts retained
entries, then either returns the original pointer, returns Empty, or fills one
Topal-allocated contiguous block of logical 16-byte nodes in source order.
Initialization mutates only fresh inaccessible storage; every published List
remains immutable. This sharing and batching are private implementation choices,
not pointer-identity semantics or a persistent/public layout promise. Display,
DWARF, and GDB continue to observe the same `List Int` source value.

The basic Int List core is a third independently selected fragment over that
same private node declaration. Prepend directly links a fresh head, while
concat counts and copies only its left input into one contiguous Topal-owned
mapping before linking the unchanged right input. Append composes a fresh
singleton with concat, and reverse fills one contiguous mapping in reverse
index order. Count, emptiness, first/rest/uncons, structural equality, and
List-decision decomposition use finite generated or runtime control flow and
canonical Int comparison; none relies on an optimization pass. The Optional
tag, rather than payload nullness, distinguishes `Some Empty` from `None`.
`uncons` stores its private two-pointer payload in Topal-owned memory. Semantic
DWARF typedefs and validating GDB rendering recover `Optional List Int` and
`Optional (Int, List Int)` without making the Optional header, pair, or node a
public interface. Modules without these checked operations omit this fragment.

Contextual `List Int` map, select, and Int-state fold do not introduce a
callable runtime representation. The checked frontend binds each anonymous
parameter to an exact loop value and specializes its body directly into the
enclosing module. Generated loops advance one source node at a time. Map links
one fresh result node per entry; select allocates only after its predicate is
true and links accepted nodes; fold carries its exact state pointer in an LLVM
phi. Fresh result links may be initialized incrementally because no reference
escapes before completion, after which the List remains immutable. The source
head and fold initial value are evaluated once in source order, and every body
runs once for each reached entry in List order. This deliberately keeps LLVM in
control of SSA and target instruction lowering while avoiding callback ABI,
indirect-call, dispatcher, host-recursion, and other-language runtime choices.

An anonymous Function bound before a collection use keeps the existing private
Function tag for source identity, display, DWARF, and GDB, but its callable
meaning stays in checked compiler metadata: parameter pattern, body, static
context, and definition-time capture facts. Map, select, or fold resolves that
metadata and instantiates the same direct loop body as the contextual form.
The tag never becomes an execution key, address, callback, or published symbol.
Consequently this extension preserves lexical snapshots without adding a
closure object, indirect call, collection dispatcher, callable ABI, or runtime
dependency.

Traversal-controlled `List Int` folds add a private immutable two-word object:
an unsigned Continue/Finish tag at offset zero and the existing canonical Int
pointer at offset eight. Constructor lowering evaluates the payload and fills
both words through the Topal-owned allocator. After each specialized action,
the generated loop loads both fields and branches directly: Continue feeds the
payload into the next state phi, while Finish feeds it into the fold-result phi
and makes later nodes unreachable. Ordinary Int-result folds keep their
existing path. This is compiler-generated SSA control flow rather than a
runtime fragment, dispatcher, callback ABI, or optimization. Semantic DWARF
describes the private object so the validating GDB printer can recover the two
constructors. The carrier is deliberately rejected at ordinary function
boundaries until a versioned compiled-library interface can describe and adapt
it, so this executable-local layout does not revise `topal-native/6` or become
a public ABI.

The first product-element List specialization began with `List (Int, Int)`
consumed by an Int-result map. Its private node expands to
three words: the two canonical Int pointers occupy the ordinary target-derived
Tuple field offsets zero and eight, and the remaining-node pointer follows at
offset sixteen. No separate Tuple object or payload pointer is introduced.
The checked anonymous product pattern is flattened to two exact Int bindings;
the generated map loop loads those words directly and reuses the existing
`List Int` result construction. List DWARF derives its remaining-field offset
and node size from the element layout, and GDB selects the corresponding
three-word decoder while preserving source pair spelling. This keeps source
order, field identity, and LLVM-owned instruction lowering visible without a
generic List runtime, type tag, callback, indirect call, or foreign dependency.
The later ordinary core increment admits private pair-List function and
aggregate boundaries, total decomposition, equality, count, and emptiness
while keeping the same layout executable-local; it does not create a
compiled-library ABI or revise `topal-native/6`. The corresponding ordinary
`List (Int, String)` increment reuses the same inline product design and its
exact Int/String equality loop for both direct inner values and outer recursive
List equality. The ordinary `List (String, Int)` increment applies the same
executable-private design to the existing Map-collector input node, adds an
exact source-order String/Int equality loop, and leaves the Map representation
and native ABI unchanged. The ordinary `List (String, String)` increment reuses
that three-pointer inline node with two immutable String descriptors and an
exact source-order String/String equality loop; LLVM continues to own physical
placement and no public or foreign ABI is introduced.

The first recursive List specialization composes that inline product approach
without declaring a generic node. An inner `List (Int, String)` uses three
words for its exact Int pointer, immutable String pointer, and remaining node;
an outer `List List (Int, String)` uses two words for its inner-List pointer and
remaining node. Empty stays null at either level, including as the valid
payload of a tagged `Some Empty`. Direct private outer-List functions carry one
opaque pointer in a matching LLVM `fastcc` prototype, leaving AMD64 register
placement to LLVM. Shape-selected finite loops count and project outer nodes;
outer equality calls a finite inner loop that uses the canonical exact Int and
String comparators. Generated display nests its ordinary iterative List
control-flow regions, and target-derived DWARF plus the bounded GDB renderer
recover both semantic levels. These exact paths require no host recursion,
callback, indirect call, runtime type tag, C/C++ support, or generic List ABI at
O0. The later ordinary inner-List increment admits private boundaries, total
decomposition, equality, count, and emptiness without publishing this layout.
General recursive representation metadata remains closed until a versioned
compiled-library schema and target adapters can describe it safely.

Range-selected `List Int` values use a separately conditional private LLVM
fragment. It visits each immutable node once, asks the exact Range runtime about
either the stored arbitrary-precision value or an exact Int converted from the
physical zero-based position, and links only freshly initialized result nodes.
The traversal never mutates or publishes a partial source/result node. Closed
String index selection instead remains target-independent checked-model
evaluation using the shared pinned user-perceived-Character segmentation, then
enters ordinary String lowering. LLVM deliberately does not receive Unicode
segmentation responsibility, and dynamic String slicing remains rejected until
a freestanding Topal implementation exists. SelectionOf, RangeSelectionOf, and
SliceOf are proof facts rather than runtime descriptors in this increment.

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
so stock GDB shows source labels. Nested enum declarations remain a separate
scope increment.

The initial external-layout policy names reuse this checked nominal-enum and
DWARF path as seven fixed language-owned families. Their private `i32` tags
exist only to preserve exact values through the admitted executable boundary;
they are not external representation tags or compiled-library identities.
Resolving or displaying one constructs no layout, touches no external memory,
and grants no access authority. Apart from the existing Topal-owned Linux write
syscall used for final output, this slice introduces no platform or layout
runtime. A future library interface must publish canonical semantic policy
identities independently of these private target tags.

The qualified `lang generator generator-closed` value follows the same private
nominal-enum path as a single-alternative
`lang generator GeneratorErrorCode`. This is an ordinary immutable value only:
the frontend does not create a continuation or attach `Error.domain`, generator
identity, or yield provenance, and the backend introduces no generator storage,
state machine, allocation, or runtime call. Canonical library metadata must
identify the qualified vocabulary and alternative rather than treating private
tag zero as an interchange identity.

The first Generator value slice retains the checked `Int` initial expression,
unary next operation, and optional directly chained `take-while` predicate in
the compiler model without executing either anonymous body. Generated code
evaluates the initial expression once and represents only a live
construction-stage `Generator Int Unit Unit` binding as private `i32` zero.
That value is an observation token, not a continuation layout: it carries no
seed, capture, program counter, ownership, or stable identity. Linear checking
permits one local consumption and closes the aggregate, qualified-member,
decision, equality, discard, abandonment, function, and library escape paths
before LLVM. Semantic enum-shaped DWARF lets stock GDB inspect the token using
the canonical Generator spelling, while ordinary output is still emitted by
the Topal-owned syscall writer. Traversal will replace this token with an
executable representation; compiled-library metadata must describe the
canonical Generator classifier and captured operations/evidence, then select a
target adapter rather than exporting this private token.

Finite collection bypasses that construction-only token for one exact direct
composition. The frontend accepts only unary `collect (initial iterate next
take-while predicate)` at `Generator Int Unit Unit`, with no captured outer
binding in either anonymous body. The backend carries current Int, List head,
and previous node as LLVM SSA loop values. It evaluates the predicate before
the accepted block, allocates and tail-links a fresh immutable 16-byte
`List Int` node only there, evaluates the next body after acceptance, and feeds
the next candidate back to the loop. The false edge reaches completion without
publishing or advancing the rejected candidate. Existing arbitrary-precision
Int operations, the Topal-owned Linux allocator, List display, semantic DWARF,
and the bounded GDB renderer are reused without a Generator object, runtime
dispatcher, callback, indirect call, or host recursion. The specialized loop
is executable-local and leaves Generator function/library representation and
canonical captured-operation metadata to a later versioned boundary.

The first `unfold` slice likewise stops at lazy construction. The checked model
retains a `List Int` seed, unary
`List Int -> Optional (Int, List Int)` step, and exact
`Generator Int Unit Unit` result, preserving that the seed and yielded
classifiers differ. Generated code evaluates the seed once but emits neither
the step nor executable generator state; final observation reuses the private
`i32` Generator token and semantic DWARF. The interpreter's matching lazy value
records the Int yield classifier for the recognized List/`uncons` step without
invoking it, so shared differential execution observes the design-specified
classifier. Generator captures and all local escape routes remain checked out.
Traversal will require an owned seed/capture/state representation and cleanup;
compiled-library metadata must carry the canonical Generator classifier, seed
and yield classifiers, step signature, captured operation/evidence identities,
and target adapter rather than exporting this construction token.

Finite List/`uncons` unfold collection uses compiler-held provenance rather
than expanding that token into a runtime object. Linear bindings and local
moves retain the checked construction only inside the compilation session. At
the consuming `collect`, the frontend proves an already evaluated immutable
`List Int` seed identity and a statement-free unary step that is exactly
`uncons` of its parameter. The backend then carries the current seed, output
head, and previous output node as explicit LLVM SSA values. A null current seed
implements `None`; otherwise direct head/tail loads implement the proven
`Some (yield, next-seed)` result before a fresh output node is allocated and
linked. This is required semantic lowering at O0, not an optimization-pass
assumption. It leaves the source List unchanged and needs no Optional object,
uncons helper, Generator allocation, callback, indirect call, or generic
runtime. General steps, captures, resumable state, and library boundaries stay
closed until canonical Generator/seed/yield/operation/evidence metadata and an
owned target representation exist.

Bounded Int iterate `foreach` reuses compiler-held construction provenance to
avoid treating the observation token as continuation state. This first root
statement specialization requires an Int-literal initial value and
capture-free checked next, predicate, and Unit action blocks. The backend
carries the current immutable arbitrary-precision Int in an LLVM `phi`, emits
the predicate before the accepted edge, binds the accepted value directly into
the action, and emits next only after that action completes with Unit. The
rejected edge returns Unit immediately. This ordering is frontend-owned O0
semantics; LLVM retains responsibility for target instruction selection and
register/stack placement but is not asked to discover the traversal. The loop
allocates no collection and needs no Generator object, runtime dispatcher,
callback, indirect call, or host recursion. DWARF describes the iteration Int
and final Unit binding. General captures and traversal need canonical
operation/predicate/body evidence, an owned continuation layout, cleanup, and
target-adapted compiled-library metadata.

The first custom-generator slice proves a complete root declaration before
LLVM: one Character input is yielded once, resumed with Unit, and followed by a
final Unit. Starting the generator evaluates the initial Character once and
retains the declaration plus yielded value as compile-session provenance. Root
foreach consumes that local provenance and expands the single Character action,
erased Unit resume, and final Unit directly in source order. The existing
private `i32` observation token preserves linear ownership and semantic DWARF
without becoming a program counter or continuation layout. This is mandatory
O0 semantic lowering, leaving instruction selection and physical placement to
LLVM while adding no Generator allocation, dispatcher, callback, indirect
call, unwind dependency, foreign runtime, or public ABI. General state machines
and compiled libraries require canonical declaration/direction/suspension,
capture/effect, ownership/close, and target-adapter metadata rather than this
executable-local proof.

The adjacent repeated-suspension slice generalizes that proof to consecutive
discarded yields of the same initial Character. The checked model retains an
ordered finite yield sequence; construction still evaluates the source value
once and represents suspension at its first entry. Root foreach expands one
inline action block per retained yield and places the erased Unit resumption
between blocks before final Unit. This is frontend-owned O0 ordering rather
than LLVM loop-unrolling policy. It reuses the private ownership/debug token and
one Character debug shadow without allocating a continuation or establishing a
library ABI. Ordinary inter-yield state, dynamic yielded values, captures,
close handling, and external boundaries still require the canonical suspension
graph, environment/effect, ownership/close, and target-adapter metadata planned
for general generators.

The first local-state slice admits one leading explicitly classified Character
alias of the generator's sole initial Character and requires every retained
yield to observe that alias. Because the admitted initializer is an immutable
identity, construction still evaluates the source operand once; the checked
model records the local name, type, source span, and exact descriptor alongside
the suspension sequence without inserting the name into the caller
environment. During root traversal, lowering materializes the proven immutable
value and gives the local a debug-only pointer shadow inside a generator
lexical DWARF block before the first action. The shadow remains available
across admitted yields and leaves scope with the traversal. It is not semantic
continuation storage and establishes no public or library layout. General
initializer computation, additional state, mutation, resume values, captures,
and external boundaries still require canonical state identities and types in
the suspension graph and compiled-library metadata.

The pre-yield completion slice represents a proved empty suspension sequence,
not a dormant action or a synthetic yield. Construction still evaluates the
initial Character once and creates the private linear observation token, while
the checked provenance records that execution reached final Unit before a
suspension. Root foreach consumes that token and returns Unit without emitting
the statically checked action or its debug binding. This behavior is explicit
in O0 IR and does not rely on LLVM dead-code elimination. No continuation
storage is needed; a compiled-library form will nevertheless need a canonical
terminal-before-suspension graph and final-value metadata so another compiler
can distinguish completion from a suspended state safely.

The first distinct-result slice keeps suspension provenance and final-value
provenance separate for an exact `Generator Character Unit Character`. The
checked plan records the initial Character as the single yielded value and a
closed Character literal as the final result. Construction evaluates only the
initial value and stops at the yield. Root foreach expands the action and erased
Unit resumption before materializing the final Character, then returns that
value through the ordinary native Character descriptor path. This sequencing
is frontend-owned at O0; LLVM handles instruction selection and physical
placement but is not asked to infer coroutine control flow. The private
ownership token and yielded-value debug shadow remain independent of the final
value, and no continuation or public Generator layout is introduced. A future
compiled-library form must identify result classifiers and final-value graph
nodes canonically alongside suspension and ownership metadata.

The post-resume-local slice replaces the single undifferentiated local list
with checked activation-stage evidence. A local records how many successful
Unit resumptions precede its introduction, so construction stops at the first
yield without evaluating later state and root traversal can expand prefix
actions, resumptions, local activation, and suffix yields in source order. For
the admitted immutable Character alias, the executable still needs no semantic
continuation storage: lowering introduces its lexical DWARF shadow only at the
recorded stage, after the preceding action/resumption and before the following
yield action. The stage number is compiler evidence rather than a runtime
program counter or ABI field. General body computation, mutable or multiple
locals, captures, close paths, and external boundaries still require canonical
suspension-graph transitions, environment ownership, cleanup, and target
adapters in compiled-library metadata.

The exact resume-binding slice uses the same activation evidence for an erased
Unit local. Its checked stage follows the sole yielded Character's successful
action/resumption edge; construction therefore has no binding yet, and root
traversal introduces the name only after the action returns Unit. Because the
final expression is that immutable Unit, no semantic value storage is needed.
The backend emits a debug-only `i8` shadow at the activation point so GDB can
observe source lifetime while the generated behavior remains direct O0 control
flow. This success edge stays distinct from the `generator-closed` abandonment
edge. A compiled-library form must encode both edge kinds and resume-local
activation canonically rather than expose the debug shadow or a target-specific
continuation layout.

The first custom-close slice relies on existing call specialization to retain
the root caller's exact Character in a function-local single-yield generator.
The checked function block appends an explicit close after the unconsumed
Generator binding and before final Unit. Because this exact generator has no
handler, cleanup, effect, or post-yield work, its intrinsic close error is
consumed at the generator boundary and both close value and final Unit erase in
native code. The compiler-private Generator observation token still carries
ownership and DWARF identity, not continuation state. General close lowering
requires canonical success/close edges, lexical close domain, declaration
provenance, cleanup/effect ordering, environment ownership, and target adapters
in compiled-library metadata.

The first handled-close slice extends that descriptor with the bound yield
Result, complete Error/Ok Unit actions, their binding activation and source
spans, and the nominal generator Error-code set. An exact function-scope close
materializes `Error(domain = root, code = generator-closed)` through the
Topal-owned Result/Error allocator path, selects the Error payload directly,
and runs its Unit action before function completion. The success action remains
checked metadata but is unreachable on this statically known close edge. DWARF
uses generator-specific Result/Error types so debugger rendering cannot confuse
the overlapping private numeric tag with an arithmetic Error code. General
handler lowering still requires a canonical suspension/branch graph, cleanup
and effect ordering, code-set identity, environment ownership, and target
adapters in compiled-library metadata.

The first qualified close-code slice adds an ordered nominal code matcher to
that checked handler descriptor. Because the admitted abandonment edge is known
to deliver `lang generator GeneratorErrorCode.generator-closed`, O0 lowering
selects the qualified Unit action directly after materializing and observing the
Topal-owned Error; it emits no decision switch and does not activate the generic
Error fallback binding. This selection depends on code-set identity rather than
the overlapping private numeric tag, lexical Error domain, or generator
declaration provenance. General compiled-library handling therefore needs
canonical ordered code matchers, fallbacks, code-set identities, branch
activation, and the existing suspension, effect, ownership, and target-adapter
metadata.

An admitted root-scope labeled `Union` or positional `Variant` retains its
nominal identity and declaration-ordered payload classifiers in the checked
model. Its private LLVM carrier is one non-packed literal struct containing an
`i32` tag followed by one statically typed field for every payload-bearing
alternative. Construction initializes every field with either the active
payload or an inert zero bit pattern; generated control flow never observes an
inactive field as a Topal value. This deliberately larger SSA carrier avoids a
type-erased payload, allocation, and target-specific union coercion in the
frontend. The same exact aggregate type appears at every private `fastcc`
definition, call, and return, leaving x86-64 register/stack classification to
LLVM under the qualified triple and data layout.

A sum decision emits its subject once, switches on the tag, and introduces the
complete active payload only in the selected branch environment. Invalid tags
take the compiler-runtime corruption exit. Compatible action results use the
existing typed leaf `phi` machinery. Display switches to the interpreter's
source spelling and prints only the active payload through existing Topal value
printers. DWARF describes a nominal typedef over the exact aggregate, an enum
tag carrying source alternative names, and the real payload offsets. A
target-aligned debug-only shadow compensates for LLVM 22's O0 SSA-aggregate
visibility limits, while the bundled GDB renderer hides inactive storage and
recursively renders the active payload. This carrier is neither a public or C
sum ABI nor a serialization or library-metadata identity. Recursive nominal
sums, persistent/public storage, and foreign adapters remain separate
representation increments.

Repeated-pattern identity reuses this carrier without comparing it as raw
storage. The frontend first emits an exact tag comparison. Equal tags enter an
LLVM `switch` whose one selected branch recursively compares only that
alternative's active payload; unequal or invalid tags contribute `false`, and
an `i1` `phi` joins the predecessors. The [LLVM language
reference](https://llvm.org/docs/LangRef.html#switch-instruction) permits the
backend to lower `switch` according to the target, while
[`phi`](https://llvm.org/docs/LangRef.html#phi-instruction) retains the explicit
source-semantic predecessor result. This control flow is mandatory at O0 and
does not rely on LLVM discovering or optimizing a whole-aggregate comparison.
Although construction fully initializes every private payload slot through
[`insertvalue`](https://llvm.org/docs/LangRef.html#insertvalue-instruction),
inactive slots are representation details and never enter the comparison.
Consequently no `memcmp`, Sum-equality runtime, or target ABI algorithm is
needed. General source Equality now reuses this exact lowering only after the
checked model proves that every declared payload has canonical Equality;
structurally identical distinct declarations and a declaration with any
unsupported payload are rejected before IR. `!=` negates the joined `i1`
without adding observations. Repeated-pattern identity remains independently
specified because it is a matcher guard rather than a public Equality
capability, even though both paths share the tag-first active-payload primitive.

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
zero-data Unit value and allocates nothing.

For one unconditional cleanup-free lexical block used directly by a function
body statement boundary, increment 3b2-b5e8a retains a distinct checked exit
outcome, omits both unreachable tails, and makes the lexical expression the
ordinary function result. The backend therefore emits the existing single
machine return while preserving the nested debug scope; it needs no control-
flow carrier, unwind edge, runtime helper, or ABI change. Embedded or
conditional return-bearing blocks, nested declarations, and scopes requiring
cleanup retain their later explicit exit-edge and lifetime lowering in
increment 3b2-b5e8.

Increment 3b2-b5e8b extends that private checked exit outcome through exactly
one additional direct boundary: a lexical block that is the whole expression
of an outer explicit return. If the block returns, its exit completes the
function before the outer statement can create another return decision; if it
completes normally, the outer statement retains ordinary return semantics.
The retained lexical result lets the existing backend emit the same single
machine return and nested `DILexicalBlock`, so the extension changes neither
runtime representation nor ABI. Other compound-expression and conditional
joins remain deferred.

Increment 3b2-b5e8c admits that checked exit in either direct operand of a
symbolic operator. A left-side block result replaces the abandoned application
before any right operand is checked or emitted. A right-side block result is
wrapped in a private exit sequence whose preceding expression retains the
complete left application prefix, so source-order calls and effects occur once
without fabricating an operand value or invoking the abandoned operator. The
backend emits that prefix in the enclosing scope, emits the returned block in
its `DILexicalBlock`, and reaches the existing single machine return. The
sequence is compiler-private structure rather than a runtime carrier and
therefore changes no ABI. Constructors, packaged or nested call arguments,
overloaded or indirect calls, conditional joins, callbacks, and cleanup-bearing
exits remain deferred.

Increment 3b2-b5e8d admits that same checked exit from a direct positional or
labeled product field. The checked model retains each preceding field
expression, in source order, in the compiler-private exit sequence and then the
returning lexical block. The backend emits those abandoned expressions without
constructing a partial aggregate, enters the block's `DILexicalBlock`, and
reaches the existing single machine return. Later fields and the function tail
are absent even at O0. This changes neither runtime representation nor ABI;
products nested in constructors or call arguments and conditional or
cleanup-bearing joins remain deferred.

Increment 3b2-b5e8e admits a return-bearing block as the sole argument of a
unary prefix call or either direct argument of a binary infix call when the
callee is an unshadowed named function with one flat declaration. A right-side
exit retains the complete evaluated left application prefix in the same private
exit sequence; a left or sole-argument exit needs no prefix. In every case the
named callee is neither selected nor invoked, the returned block keeps its
`DILexicalBlock`, and the existing single machine return completes the enclosing
function. This changes no runtime representation or ABI. Overload selection,
bound or qualified callables, packages, other constructors, conditional joins,
and cleanup-bearing exits remain deferred.

Increment 3b2-b5e8f admits a return-bearing block as the direct payload of the
built-in unary `Some` constructor. The exit is resolved before payload
classification, so neither an Optional header nor an abandoned classified
binding is generated. The returned block remains the enclosing function's
private result with its nested `DILexicalBlock`, and the existing single machine
return completes the function. This changes no runtime representation or ABI;
constructors outside the next strict-unary increment, nested payload
expressions, conditional joins, and cleanup-bearing exits remain deferred.

Increment 3b2-b5e8g extends the same pre-construction exit to the strict unary
`String`, `Int`, `Nat`, and `Rational` built-ins and to an already-declared
payload-bearing Union alternative. Constructor recognition uses the checked
source identity and declaration position rather than treating an arbitrary
callable as a constructor. Conversion, payload classification, Union tag
selection, and abandoned binding generation are all omitted. The returned
block remains the private result with nested `DILexicalBlock` and the existing
single machine return. This changes no runtime representation or ABI;
`Character` forms outside 3b2-b5e8i, positional Variant forms outside
3b2-b5e8h, constraint forms outside 3b2-b5e8j, modular forms outside
3b2-b5e8k/3b2-b5e8l, collection forms outside
3b2-b5e8m/3b2-b5e8n/3b2-b5e8o/3b2-b5e8p, decision subjects outside
3b2-b5e8q/3b2-b5e8r, qualified, nested-payload, conditional, and
cleanup-bearing forms
remain deferred.

Increment 3b2-b5e8h admits the direct payload block of an already-declared
positional Variant when its literal index selects an existing alternative.
Checked Variant identity and index validation precede the payload; the exit then
omits payload classification, tag/aggregate construction, abandoned binding,
and tails. The block remains the private result with nested `DILexicalBlock` and
the existing single machine return. This changes no runtime representation or
ABI; invalid, undeclared, qualified, nested-payload, conditional, and
cleanup-bearing forms remain fail-closed.

Increment 3b2-b5e8i admits a direct return-bearing block as the operand of the
built-in `Character` constructor. Constructor identity is selected before the
operand; the exit then precedes pinned-Unicode constraint validation and omits
Character evidence, the abandoned binding, and tails. The block remains the
private result with nested `DILexicalBlock` and the existing single machine
return. This changes no runtime representation or ABI; qualified,
nested-operand, conditional, and cleanup-bearing forms remain fail-closed.

Increment 3b2-b5e8j admits a direct return-bearing block as the operand of an
already-declared named Int constraint. Constraint identity, base, and binding
declaration order are checked before the operand; the exit then precedes
predicate evaluation and refined evidence construction and omits the abandoned
binding and tails. Constraint-binding declaration positions are retained beside
their semantic tags so functions analyzed after later root statements cannot
gain forward visibility. The block remains the private result with nested
`DILexicalBlock` and the existing single machine return. This changes no runtime
representation or ABI; forward, unknown, non-Int, dynamically selected,
qualified, nested-operand, conditional, and cleanup-bearing forms remain
fail-closed.

Increment 3b2-b5e8k admits a direct return-bearing block as the operand of an
already-declared named `ModNat` or `ModInt` type. Modular identity and
declaration order are checked before the operand; the exit then precedes Int
classification, canonical-range validation, dynamic Result or nominal-value
construction, and omits the abandoned binding and tails. The block remains the
private result with nested `DILexicalBlock` and the existing single machine
return. This changes no runtime representation or ABI; forward, unknown,
dynamically selected, qualified, explicit-reduction forms outside 3b2-b5e8l,
nested-operand, conditional, and cleanup-bearing forms remain fail-closed.

Increment 3b2-b5e8l admits a direct return-bearing block as the left operand of
`operand modulo Name` when `Name` is an already-declared named `ModNat` or
`ModInt` type. Exact target identity and declaration order are checked before
the operand; the exit then precedes Int classification and the
subtract/modulo/add reduction chain and omits nominal construction, the
abandoned binding, and tails. The block remains the private result with nested
`DILexicalBlock` and the existing single machine return. This changes no runtime
representation or ABI; forward, unknown, dynamically selected, qualified,
nested-operand, conditional, and cleanup-bearing forms remain fail-closed.

Increment 3b2-b5e8m admits a direct return-bearing block as the source of the
built-in unary `collect source` operation. Exact operation selection precedes
the source; the exit then precedes finite-traversal classification, generator
consumption, and List materialization and omits List nodes, the abandoned
binding, and tails. The block remains the private result with nested
`DILexicalBlock` and the existing single machine return. This changes no runtime
representation or ABI; except for the unordered collectors in 3b2-b5e8n,
Map collection in 3b2-b5e8o, and infix Array/String collection in 3b2-b5e8p,
qualified and nested-source forms, conditional control flow, and cleanup-bearing
forms remain fail-closed.

Increment 3b2-b5e8n admits a direct return-bearing block as the source of the
exact built-in `collect-set source` and `collect-bag source` operations. Exact
operation selection precedes the source; the exit then precedes finite-List
classification, equality-dependent duplicate coalescing, multiplicity
accumulation, and Set or Bag materialization and omits collection nodes,
comparisons, the abandoned binding, and tails. The block remains the private
result with nested `DILexicalBlock` and the existing single machine return. This
changes no runtime representation or ABI; except for Map collection in
3b2-b5e8o and infix Array/String collection in 3b2-b5e8p, qualified and
nested-source forms, conditional control flow, and cleanup-bearing forms remain
fail-closed.

Increment 3b2-b5e8o admits a direct return-bearing block as the source of the
exact built-in `collect-map source resolving policy` operation after validating
the `resolving` clause and one of the three declared collision policies. The
exit then precedes pair classification, key equality, collision resolution,
and Map materialization and omits Map nodes, comparisons, the abandoned
binding, and tails. The block remains the private result with nested
`DILexicalBlock` and the existing single machine return. This changes no runtime
representation or ABI; except for infix Array/String collection in 3b2-b5e8p,
invalid policies, qualified and nested-source forms, conditional control flow,
and cleanup-bearing forms remain fail-closed.

Increment 3b2-b5e8p admits a direct return-bearing block as the left source of
the exact built-in `source collect Array` and `source collect String`
operations. Exact operation and target selection precede the source; the exit
then precedes finite-List classification, Array extent derivation, String-entry
validation, and target materialization and omits target values, the abandoned
binding, and tails. The block remains the private result with nested
`DILexicalBlock` and the existing single machine return. This changes no runtime
representation or ABI; unknown or qualified targets, nested-source forms,
conditional control flow, and cleanup-bearing forms remain fail-closed.

Increment 3b2-b5e8q admits a direct return-bearing block as the subject of a
complete Boolean decision with one literal rule followed by `otherwise`. Exact
decision-shape recognition precedes the subject; the exit then precedes Boolean
classification, matcher consideration, action selection, and action evaluation
and omits decision branches, actions, the abandoned decision value, and tails.
The block remains the private result with nested `DILexicalBlock` and the
existing single machine return. This changes no runtime representation or ABI;
other matcher sets, nested subjects, decision-action returns, conditional
control flow, and cleanup-bearing forms remain fail-closed.

Increment 3b2-b5e8r admits the other complete Boolean matcher shape: a direct
return-bearing subject followed by exactly one `false` rule and one `true`
rule, in either order and without `otherwise`. A shared frontend predicate
recognizes the exact distinct two-rule set before either interpreter execution
or checked-model analysis enters the subject. The exit therefore precedes
Boolean classification, matcher consideration, action selection, and action
evaluation and omits decision branches, actions, the abandoned decision value,
and tails. The block remains the private result with nested `DILexicalBlock`
and the existing single machine return. This changes no runtime representation
or ABI; duplicate or additional rules, other matcher sets, nested subjects,
decision-action returns, conditional control flow, and cleanup-bearing forms
remain fail-closed.

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

An explicit v0.1 `: Effects ()` function bound is represented in the checked
model as `Some(empty semantic row)`, distinct from an absent inferred bound.
All currently admitted function expressions have the exact empty language
effect row; internal allocation and the executable entry point's final display
do not become source-level function interactions. The frontend therefore checks
empty-row containment before instantiation and rejects every other effect-row
form outside the separate `Decreases` proof syntax. A narrowly admitted
single-overload `lang view` materializes identity, signature, staticness, and
the declared row only in checked compiler memory. Its binding is marked
static-only and lowers to no instruction, local, descriptor, or DWARF type.
This models the metadata needed by future compiled-library interfaces without
stabilizing its serialization or machine representation; runtime use and every
public boundary remain closed.

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

The first general static-introspection slice reuses those canonical Type
identities but keeps the phase boundary explicit. `lang Identity`, primitive
`lang TypeView`, and the active `lang LanguageContext` are distinct checked
metadata values tagged by semantic kind; static bindings retain them in compiler
memory but never enter the runtime/root environment. Exact fundamental-Type
`same-object` and `equivalent-type` relations fold to ordinary Boolean constants
before LLVM. Only `lang version` crosses into ordinary execution: its v0.1 value
is a pointer to a target-private, frame-local header containing the four existing
immutable Nat carriers. Frame materialization avoids load-time pointer
relocations, LLVM owns the x86-64 instruction and stack lowering, and matching
Version DWARF plus the validating GDB renderer expose `major`, `minor`, `patch`,
and `build` with canonical display. The header is not a public or compiled-
library ABI. Future library metadata will serialize semantic identities and
language-context values through a separately versioned schema rather than copy
this target representation. No reflection table, Version runtime helper,
foreign runtime, or other-language standard library is introduced.

Closed v0.1 Capability composition follows the same semantic/machine split even
more strictly. The checked model stores an alternative set whose members are
canonical sets of atomic Capability identities. It computes `and` as an
alternative cross-product and `or` as a set union, sorting and deduplicating at
the frontend boundary so neither source order nor LLVM optimization can affect
identity. Root Capability bindings carry only that compiler metadata. They emit no
instruction, symbol, descriptor, debug type, or debug variable. If the complete
application result is the Capability itself, the backend sends its already
canonical spelling through the existing syscall writer as a literal; there is
still no Capability runtime value or evidence namespace.

This representation intentionally stops before capability claims, operation
roles, generic evidence, function passage, or compiled-library exchange. Those
features require canonical subject and declaration identities plus a versioned
metadata schema, not a target tag or method table. The current executable thus
adds no runtime helper, dispatch, allocation, ABI surface, foreign dependency,
or other-language standard library, while future library consumers can validate
semantic identities independently of the Linux x86-64 lowering.

The first function-interface slice applies that separation to intentional call
surface conformance. The frontend constructs a nominal `root.Name` record with
a canonical set of operation names and exact parameter/result classifiers. A
direct source-root implementation is accepted only when it supplies each role
once and no other declaration. Its checked evidence maps every role to the
overload-qualified root declaration identity and any explicit empty declared
effect bound. An absent bound remains absent until the selected function body
is checked; it is never mislabeled as inferred evidence. Shape validation occurs
before the implementation declarations are
collected for ordinary call analysis, so malformed evidence cannot influence
overload selection or code generation.

After checking, only the selected ordinary functions remain on the LLVM path.
They keep the existing module-private `fastcc` definitions and direct calls;
LLVM owns their target-specific physical argument lowering. The interface and
evidence records create no machine value, vtable, function pointer, dispatch,
descriptor, allocation, symbol, relocation, or debug entry. GDB consequently
shows the truthful implementation function, parameter, line, and caller frame,
but no invented interface object. This is mandatory frontend erasure at O0,
not dead-code elimination delegated to LLVM.

The current native artifact continues to describe a closed executable and does
not claim that its empty interface/export/evidence fields serialize these
records. A future compiled-library layer must define a separately versioned,
validated schema carrying nominal interface identities, canonical operation
shapes, semantic declaration identities, effects, dependencies, and trust
information. It must then adapt those semantics to each target ABI rather than
publishing private `fastcc` signatures, LLVM types, symbols, or Linux x86-64
layouts as the language boundary.

An admitted named `Constraint` object is likewise split between semantic
metadata and a private observation value. The checked program model retains
the root binding identity, primitive base classifier, predicate parameter, and
fully checked Boolean predicate. A separately named `Constraint`-classified
binding receives its own nominal identity while reusing the same retained base
and predicate, matching the shared interpreter observation. Generated code
carries only a deterministic module-private `i32` identity tag because this
increment does not apply the predicate. Display and DWARF enumerate the
canonical `<Constraint name>` spellings, so ordinary output and GDB agree
without reflection, allocation, lookup, or a constraint runtime. The tag is
neither predicate dispatch nor a library metadata key. Capturing predicates,
constraint application/evidence, function boundaries, persistent/public
aggregate machine boundaries, and public library identities remain deferred
and are rejected rather than assigned a premature environment or ABI
representation.

For admitted application of a closed Int constraint, the frontend evaluates
the retained predicate when the operand is structurally closed, rejects a known
failure, and attaches a distinct refined classifier to an accepted unchanged
Int pointer. Equality, ordering, and arithmetic explicitly forget only that
evidence and reuse the base Int operations. An unknown operand causes the
predicate's checked expression to be emitted once with the operand bound in a
private LLVM environment; explicit `br` paths construct the existing
`Result (Int, lang arithmetic ArithmeticErrorCode)` success or the
`root.Name(Int)`/`out-of-range` Error. No optimizer is needed for either rule.
DWARF represents a refined binding as a typedef over the same Int pointer, and
a debug-only stack shadow keeps closed constants inspectable in GDB at O0.
Other bases, dependent/capturing predicates, evidence across function or
persistent/public aggregate machine boundaries, existential selection, and
public constraint metadata remain separate increments.

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
Canonical conversions between differently classified corresponding fields
remain separate frontend work rather than being inferred by the backend.

An anonymous labeled Record begins with a decomposed expression-local strategy.
The checked model evaluates fields in source order, rejects duplicate labels,
keeps that order for display, and separately retains a canonical label-to-type
map for selection and structural identity. LLVM lowering carries labeled field
values without allocating a record object; selection chooses the already
evaluated field and display recursively emits `label is value` through existing
Topal syscall-backed value printers. When a Record reaches a function,
control-flow, or debug boundary, the private canonical-values-plus-permutation
carrier described above preserves both identities. This adds no record runtime,
foreign dependency, semantic aggregate heap storage, or public ABI.

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

Diagnostic controls remain entirely above the LLVM boundary. The shared parser
validates warning and structured-identity stack discipline before the checked
compiler model runs. The current compiler emits no configurable warning stream,
so a valid control has no event to filter and is erased from the executable
model; shared syntax errors and all language errors remain unsuppressible. When
compiler warnings are added, filtering belongs at diagnostic publication using
the retained source identity and lexical extent, never in generated runtime
state.

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
source files, source functions, admitted scalar and aggregate parameters and
immutable locals, their types, and instruction locations. Structural values
acquire debug descriptions only with a truthful private representation; until
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
- for any serialized interface, protocol and language revisions, canonical
  type identities and schemas, field order, byte-order contract, and authority
  profile independently of private descriptors and symbols;
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
| `br`, `switch`, and `phi` | used | once-evaluated Boolean, exact-matcher, Comparison, nominal Enum/sum decisions, active-payload repeated identity and derived Sum equality, modular bound validation, and fallible arithmetic control flow with typed result joins |
| `insertvalue` and `extractvalue` | used | target-independent construction and decomposition of exact private Tuple, Record, Union, and Variant aggregate signatures |
| DWARF debug metadata and frame pointers | used | GDB source debugging at the reference level, including explicit Scope/environment parameters, native enum/sum alternatives, nominal modular and modular-success Result values, SerializationStream descriptors, checked external Location headers, and bundled renderers for private finite/infinite Int, Rational, exact Range, modular, active sum, native-stream, and location values |
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
