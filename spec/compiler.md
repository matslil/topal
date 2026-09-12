# Native compiler and artifact conformance

## Formal text

### TOPAL-COMPILER-TARGET-001 — Qualified initial target

Compiler target identity SHALL include architecture, vendor, operating system,
environment, data layout, object format, CPU baseline, and features. Compiler
revision 1 SHALL accept only `x86_64-unknown-linux-gnu` with the qualified
layout and x86-64 baseline recorded in `se/compiler-architecture.md`; it SHALL
reject every other target before lowering. Host CPU discovery SHALL NOT silently
strengthen the emitted feature set.

### TOPAL-COMPILER-PLATFORM-001 — Freestanding Linux execution

The initial executable SHALL define its own process entry and SHALL link with
no foreign-language startup object, standard library, runtime, default library,
or ELF interpreter. Linux interactions SHALL occur only through target support
operations whose exact syscall number, register convention, memory effects,
partial completion, interruption, and error behavior are explicit. The first
support operations SHALL implement complete standard-output writes, private
anonymous mappings for compiler-owned runtime storage, and process termination.

### TOPAL-COMPILER-O0-001 — Unoptimized semantic reference

`-O0` SHALL perform every mandatory parse, type, evidence, representation,
cleanup, and target-validity lowering, but SHALL perform no optional
source-semantic rewrite or specialization beyond that required to represent an
accepted program. LLVM verification and correctness-preserving backend lowering
remain mandatory. The resulting executable's observable value and trace SHALL
equal the interpreter's for every source in their shared implemented subset.

### TOPAL-COMPILER-INT-001 — Arbitrary finite Int representation

Every admitted finite `Int` SHALL be represented without a fixed machine-word
bound. Its private native object SHALL use one canonical zero encoding, a
normalized sign, no redundant high limbs, and enough dynamically allocated
limbs to hold the exact magnitude. Integer negation, absolute value, addition,
subtraction, multiplication, equality, ordering, function passage,
control-flow joins, and decimal observation SHALL preserve the corresponding
`TOPAL-NUM-*` value for all operand sizes that available target storage can
hold.

Runtime operations SHALL NOT call an undeclared foreign allocator, arithmetic
helper, unwinder, or standard library. Storage failure SHALL follow an explicit
platform-failure path rather than producing a truncated or invalid `Int`.

### TOPAL-COMPILER-EXACT-001 — Finite exact-number runtime

Every admitted finite `Rational` SHALL retain canonical coprime arbitrary-
precision `Int` numerator and positive denominator values, including canonical
zero. Closed construction, exact decimal literals, Int embedding and division,
Rational negation, absolute value, addition, subtraction, multiplication,
division, natural and negative powers, equality, ordering, three-way
comparison, decimal observation, function passage, and control-flow joins
SHALL preserve the corresponding `TOPAL-NUM-*` value without approximation.

Finite Int power, Euclidean modulo, and quotient/modulo SHALL use the same
unbounded representation and normative sign rules. A path requiring a dynamic
arithmetic `Result` SHALL remain outside the admitted subset until its complete
value and error representation is implemented; the compiler SHALL NOT replace
that Result with process termination. A statically evident zero divisor SHALL
remain a source diagnostic.

Position-independent output without an ELF interpreter SHALL contain no
load-time pointer relocation. Runtime construction SHALL be used when a private
aggregate would otherwise require such a relocation.

### TOPAL-COMPILER-RANGE-001 — Finite exact ranges

Every admitted finite explicitly bounded `Range Int` and `Range Rational`
SHALL retain its exact lower and upper endpoints and both inclusivity states.
Construction, mixed Int-to-Rational endpoint conversion, classification,
function passage, control-flow joins, membership by either operand order,
intersection, emptiness, bound observation, inclusivity observation, textual
output, and debugging SHALL preserve the corresponding `TOPAL-RANGE-*`
semantics without enumeration or endpoint adjustment.

The private representation SHALL remain opaque outside its versioned native
ABI and SHALL contain no load-time pointer relocation in output without an ELF
interpreter. Unbounded ranges, infinite endpoints, and range-based collection
selection SHALL remain outside the admitted subset until their prerequisite
semantics and value representations are implemented.

### TOPAL-COMPILER-LLVM-001 — LLVM module and tool qualification

Every LLVM module SHALL carry the exact qualified target triple and data layout,
use only supported standard LLVM semantics, and pass LLVM verification before
object emission. The compiler SHALL reject an incompatible LLVM major. The
invoked LLVM version, CPU baseline, optimization level, relocation model, and
debug mode SHALL be reproducible artifact inputs.

### TOPAL-COMPILER-ABI-001 — Sealed native ABI

Every private machine signature SHALL belong to an exact Topal native-ABI,
compiler, language, and target revision. It SHALL NOT be treated as a public
foreign ABI. A future foreign adapter SHALL use the target's qualified calling
convention and SHALL expose only explicitly described fixed-width scalars,
opaque handles, or independently verified aggregate coercions.

### TOPAL-COMPILER-ARTIFACT-001 — Complete native manifest

A native artifact manifest SHALL identify its schema, language and GEIR
revisions, target and data layout, object and platform ABIs, CPU/features,
compiler and LLVM versions, optimization/debug modes, source/interface and
dependency digests, exports, platform requirements, evidence, provenance, and
native slices. A consumer SHALL validate all required fields and digests before
using machine code or semantic metadata. LLVM IR and bitcode SHALL be
LLVM-major-qualified rebuildable payloads rather than the compatibility
boundary.

### TOPAL-COMPILER-DEBUG-001 — GDB-observable unoptimized code

Debug-enabled `-O0` output SHALL emit DWARF source files, line mappings,
subprograms, machine-represented parameters and immutable locals, their
supported value types, and unwindable frames sufficient for tested GDB source
breakpoints, stepping, backtraces, and value inspection. Compiler-generated
platform frames SHALL be distinguishable from source functions. A source value
without a truthful machine representation SHALL be omitted until that
representation is implemented; it SHALL NOT be represented by an unrelated
machine value. A debugger renderer MAY decode a documented private object into
its source value, but SHALL reject corrupt representation metadata safely.

### TOPAL-COMPILER-TEST-001 — Shared executable regressions

Compiler conformance tests SHALL consume the same Topal source identity as the
interpreter test, compile it at `-O0`, execute the resulting binary, and compare
its observable result with the interpreter. Compiler build and executable run
resources SHALL have identities and baselines distinct from interpreter
execution.

### TOPAL-COMPILER-SUBSET-001 — Explicit incremental boundary

Until whole-core closure, the compiler SHALL publish its implemented rule set
and SHALL reject any program requiring another rule with stable
`E-COMPILER-UNSUPPORTED` diagnostics. A representation limit SHALL reject only
when the compiler cannot prove conformance; it SHALL NOT wrap, truncate, use
foreign undefined behavior, or change Topal's unbounded semantic domain.

### TOPAL-COMPILER-TOOL-001 — LLVM facility accounting

Every applicable LLVM facility considered for code generation, optimization,
debugging, qualification, profiling, or binary inspection SHALL be recorded as
used, deferred, or not applicable with its correctness and integration reason.
Changing a disposition SHALL update validation evidence in the same increment.
