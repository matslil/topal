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
use an exact compiler-private signature keyed by `topal-native/1`, and future
foreign adapters may use target `ccc` only with fixed-width scalars or opaque
handles. Aggregate classification is adapter work and will be checked against
the target's reference C frontend before a foreign interface is admitted.

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
call 1) and `exit` (system call 60), as assigned by the kernel's authoritative
[x86-64 syscall table](https://github.com/torvalds/linux/blob/master/arch/x86/entry/syscalls/syscall_64.tbl).
Partial writes and interruption remain platform results handled by the Topal
support routine; they are not replaced with libc behavior.

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
| DWARF debug metadata and frame pointers | used | GDB source debugging at the reference level |
| `llvm-readobj` / `llvm-objdump` | test and qualification use | object, dependency, symbol, and line-table inspection |
| `llvm-link`, `llvm-dis`, `llvm-extract`, `llvm-diff`, `llvm-reduce` | qualification and failure reduction only | production linking occurs from verified modules; these tools remain useful for backend diagnosis but do not improve emitted semantics merely by being invoked |
| `llvm-ar`, `llvm-ranlib`, `llvm-nm`, `llvm-size` | archive packaging deferred; inspection as needed | compiled-library container and installation rules must precede a public native archive format |
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
