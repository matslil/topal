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
statically decidable scalar headers. Dynamic structural classifier dispatch,
Function results, escaping/capturing closures, and remaining recursive call
graphs remain later frontend work and do not leak into the private ABI
prematurely.

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
valid in an independently callable function, so direct function-body access
through qualified `root member` remains rejected until root storage receives an
explicit cross-function representation compatible with compiled-library
interfaces.

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
conversion without an environment object: cross-function forwarding,
aggregate/callable capture, escape, qualified root access, and public/library
contexts remain deferred to the unified closure and compiled-library ABI.

Named function values similarly split observable identity from call lowering.
The checked binding retains the original declaration vector and application
specializes from that vector, so the backend emits the same direct private call
as an unaliased source name. A deterministic module-local integer identifies
the name only when the Function value is displayed or inspected in DWARF. It is
not a code pointer or dispatch-table index. This preserves a future choice of
public callable/closure representation without burdening current private calls
with a provisional runtime ABI.

The initial symbolic Function values use the same separation. `+`, `-`, and
`<=>` receive deterministic observation tags, while checked application rewrites
the retained identity to the ordinary arithmetic or comparison expression
before backend lowering. Binary operands remain a source-level positional
product and are decomposed by the frontend; LLVM sees only the already-selected
operation and its normal machine values. The tag consequently cannot introduce
indirect control flow or constrain a later general callable ABI.

Private Function inputs extend the same scheme across one specialization
boundary. The caller passes the observation tag in the source parameter slot,
while checked callable metadata is propagated into the specialized callee model
and determines its direct operations. Different callable identities may create
different private instances of the same source function. LLVM receives exact
i32 prototypes and owns physical register/stack placement for the target. When
specialization erases every computational use of the tag, a debug-only aligned
stack shadow preserves the source parameter for DWARF/GDB without turning it
into runtime dispatch or a public callable representation.

Non-capturing inferred anonymous functions extend specialization without
choosing a closure ABI. The checked binding retains the parameter patterns,
body, construction identity, and detected lexical captures. A direct call, or
use through a specialized private Function parameter, supplies the parameter
classifiers; the frontend then checks the body and emits one exact private
`fastcc` function. Multi-parameter calls decompose their positional product in
source order, and flat mixed symbolic applications are explicitly regrouped
left-to-right before ordinary operation checking. The observation tag exists
only for `<anonymous fn/N>` display and DWARF. A detected data capture is
rejected rather than allowing entry-frame SSA to leak across function frames;
anonymous capture parameters, environments, escape analysis, and a public
closure representation remain one coordinated later design.

Named nested lexical functions declared directly in an ordinary function body
establish the first private capture boundary without choosing that general
closure design. At the declaration point, the frontend snapshots the finite
visible immutable environment and retains every value with an admitted private
representation, except names shadowed by the nested function's explicit
parameters. Direct application specializes the nested declaration and passes
source parameters followed by those original SSA values as deterministic exact
hidden `fastcc` parameters. The nested name has no runtime value and cannot
escape, so there is no function pointer, indirect dispatch, environment
allocation, or caller-frame reference. LLVM owns the physical AMD64 parameter
classification, while DWARF presents the nested source frame and both explicit
and captured arguments under their source names. Static, effectful, recursive,
overloaded, sibling-referencing, anonymous, or escaping closures, name
collisions with visible or active callables, and captures of callable, Scope,
constraint/evidence, or defining-context state remain deferred to the unified
closure and library-interface design.

One scalar packaged operand is normalized at the same checked boundary. A full
positional product already has declaration order; the initial labeled form is
limited to a declaration-order prefix with trailing closed defaults, so the
lowered argument sequence exactly preserves source evaluation followed by
default evaluation. The private callee receives one ordinary LLVM parameter per
source field. This makes every parameter classifier and DWARF binding explicit
while LLVM retains responsibility for target register/stack placement. It also
avoids `byval`, `sret`, `inalloca`, and `preallocated`: those attributes encode
specific memory/ABI obligations and are reserved for a deliberate public
aggregate interface rather than being inferred from source packaging syntax.
Broader labeled-map ordering, invocation-dependent defaults, and multiple or
mixed packages remain checked-frontend work.

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

The first product-element List specialization is deliberately limited to
`List (Int, Int)` consumed by an Int-result map. Its private node expands to
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
Pair-List function boundaries remain closed, so the executable-local storage
does not become a compiled-library ABI or revise `topal-native/6`.

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
O0. Inner pair-List boundaries and general recursive representation metadata
remain closed until a versioned compiled-library schema and target adapters can
describe them safely.

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
| `br`, `switch`, and `phi` | used | once-evaluated Boolean, exact-matcher, Comparison, nominal Enum/sum, modular bound validation, and fallible arithmetic control flow with typed result joins |
| `insertvalue` and `extractvalue` | used | target-independent construction and decomposition of exact private Tuple, Record, Union, and Variant aggregate signatures |
| DWARF debug metadata and frame pointers | used | GDB source debugging at the reference level, including explicit Scope/environment parameters, native enum/sum alternatives, nominal modular and modular-success Result values, and bundled renderers for private Int, Rational, finite exact Range, modular, and active sum values |
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
