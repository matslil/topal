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

The checked frontend resolves each admitted source-ordered overload before IR
generation and gives every selected input signature a distinct call-graph node
and private LLVM symbol. Argument expressions are modeled once before candidate
filtering; only the selected candidate's canonical scalar conversions reach
code generation. Staticness is retained as semantic availability rather than a
different machine convention: a static body may select only another static
declaration, while root and ordinary runtime contexts may select either form.
This increment covers statically decidable scalar headers. Dynamic structural
classifier dispatch, function values, closures, and recursive call graphs remain
later frontend work and do not leak into the private ABI prematurely.

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
descriptors containing UTF-8 bytes and a cached canonical source spelling.
Only relocation-free byte arrays reside in the static image; descriptors are
constructed through the Topal allocator at run time because their pointers
otherwise require loader-applied absolute relocations. The compiler computes
the display spelling for the literal-only admitted slice; future dynamic String
constructors must populate the same invariant. Functions, decision joins,
Result payloads, DWARF, and GDB all use the same private pointer representation.
The syscall runtime writes canonical ordinary or tagged Topal literals without
relying on a C locale or string library. This increment admits literal transport
and display, not the remaining Unicode String operations in roadmap increment
4.

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

Ordered comparison decisions lower directly to LLVM conditional branches in
source order. Each matcher operand is emitted in its reached test block, each
action in its selected block, and compatible machine-scalar results merge with
an SSA `phi`. Closed decisions over `Comparison` lower through `switch` to the
three nominal alternatives. The subject is emitted once before either control
flow graph, so LLVM optimization may simplify the graph later without changing
Topal evaluation order at O0.

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
| `br`, `switch`, and `phi` | used | once-evaluated Boolean, exact-matcher, Comparison, and fallible arithmetic control flow with typed result joins |
| DWARF debug metadata and frame pointers | used | GDB source debugging at the reference level, with bundled renderers for private Int, Rational, and finite exact Range objects |
| `llvm.ctlz` | used | target-independent significant-bit count for finite exact exponentiation |
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
