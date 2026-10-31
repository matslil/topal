# Step 4: ordinary Topal kernel policy library

`TK-DEC-252` implements the first source-bound freestanding kernel library.
The immutable boot-root access decision now executes ordinary Topal Boolean/Enum
functions in [boot-open-access.t](kernel/policy/boot-open-access.t), checked and
lowered by the shared frontend/backend. Native flags, lookup, errno translation,
description publication and resource-right checks remain at the machine boundary.
Core-language meaning is unchanged.

This is an incremental Step 4 result. It does not establish general kernel
modules, allocating libraries, general processes, a distribution boot, containers
or VM hosting. The [remaining acceptance work](step-4-status.md) still applies.

## Checked publication and ownership

The [kernel root](kernel/arch/x86_64/toolchain-gate.t) declares its source unit,
public export, semantic interface and 1024-byte stack limit. `topalc` loads that
unit under the explicit `--library-root`; the in-memory API requires a matching
snapshot. Missing/private/invalid source cannot select a fallback. Imports,
nonfinite values, effects, foreign calls, recursion and other unsupported forms
cannot enter the admitted pure call graph.

Publication emits and inspects a third object, separate from root and machine
provider. It retains ordinary helper calls and source debug information. It
rejects undefined/unexpected symbols, writable allocated data, TLS, hosted runtime
dependencies, unsupported instructions and missing/inconsistent/excessive stack
evidence. LLVM uses the x86-64 freestanding target, no red zone, no implicit
floating point and no tail-call conversion. The generated adapter translates
actual source Enum tags to semantic choices; invalid input/result representations
fail closed.

The artifact owns immutable executable code. Static-process entry borrows its
binding through a private thirteenth native argument, using the previous clock
argument's padding word. It retains the binding across exec, aligns each native
call, checks the supervisor stack reserve and erases/reads back the private
binding before terminal return. A source result cannot grant a writable
description in the immutable namespace. No context, resource or pointer enters
the source policy.

Provenance binds source/client/LLVM/object digests, selected export/interface,
Enum adaptation, LLVM identity and actual per-function stack sizes/call paths.
The source snapshot, LLVM and library object are published atomically with the
kernel. The boot adapter also checks that linked code and library binding agree.

## Executed evidence

The primary kernel is:

```text
e00325855cc2d214e573b28f7158cdb469af09aeba0462059257572291f8898a
```

Artifact/root/provider/object/boot/gate revisions are
`228 / 217 / 218 / 210 / 128 / 228`.

The [library harness](labs/qemu/x86_64/library_gate.py) and
[retained evidence](labs/qemu/x86_64/results/topal-kernel-source-library-gate.json)
record fifteen actual kernel-to-policy calls covering all eight Boolean inputs.
GDB observes ordinary Topal helper frames, the generated bridge and the native
process caller. Each call preserves the native callee-saved registers and all
sixteen XMM registers, returns the expected semantic choice and uses the retained
code binding. Terminal restoration observes its erased witness.

LLVM metadata plus the acyclic call graph prove a maximum of **180 bytes**,
including helper/adapter frames, return addresses and the 32-byte native alignment
reserve, within the owned 1024-byte reserve. A separate shallow, wide-parameter
function is rejected for excessive actual stack use. Native host execution checks
all eight valid inputs and the invalid `8`/maximum-word representations.

Two source variants use separate kernels and records:

- Reversing the Enum declaration order preserves Linux behavior through tag
  adaptation and passes full clean-fixture execution.
- Changing the regular-file write denial to `NotDirectory` changes actual library
  behavior. GDB observes the independent user witness's exact failing comparison:
  received `ENOTDIR` (20), expected `EROFS` (30), unequal flags and its intentional
  failure loop. The normal runtime gate rejects that kernel. Timeout alone is
  insufficient to qualify this counterexample.

Both variants leave root/provider object digests unchanged while changing the
library object. The counterexample is excluded from successful qualification.
It does not claim process completion or reclamation.

The primary kernel passes **39 physical runs**: three clean environment modes,
26 unchanged pinned BusyBox workloads, five controlled RTC cases, three controlled
relative-time cases, completed-deadline notification injection and the library
debugger run. The reordered-Enum kernel passes one additional run. The historical
deadline predecessor and the wrong-denial kernel remain separate expected
failures. Hardware time advancement retains its explicit test-input status.

Sixteen lab contracts pass. Compiler tests cover ordinary lowering, finite input
execution, representation independence, unsupported values/cycles/imports,
contained source loading, missing/private/mismatched snapshots, atomic failure
cleanup, excessive stack and source-bound boot packaging. The bounded full-workspace suite passes, including 203 compiler library tests
and source-bound boot-packager coverage. Formatting, relative links, whitespace
and the inventory of 242 unique normative systems rules also pass.

## Review and limits

Risk is high at the purity, stack, ABI and code-ownership boundary. Self-review
followed the authority chain, ordinary frontend/function lowering, complete call
closure, strict object/mapping checks, stack arithmetic, argument alignment,
rights enforcement, exec retention and terminal erasure. Executed source variants
and the independent userspace failure observation check that compiler-native
policy has not replaced the Topal function's meaning.

The current profile supports one finite source unit and semantic interface. It
does not qualify arbitrary types, allocating code or cross-module imports.
Substantial kernel policy still resides in bounded native adapters. AArch64 and
RISC-V inform the semantic facts/choices and ownership design, but require their
own native ABI, frame, mapping and stack-reserve evidence before implementation.
See the [design](design/freestanding-kernel-policy-library.md) and
[Step 2 architecture](design/topal-kernel-architecture.md).
