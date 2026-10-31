# Ordinary source selection of Linux syscall operations

`TK-DEC-261`; systems-only refinement adopted under standing authorization.
Design adopted; implementation and physical qualification remain required.

The Step 2 boundary assigns Linux policy to Topal modules. The current static
process adapter still compares the captured selector directly in native code.
Move that choice into ordinary source before extending retained state or adding
more Linux services. Existing native handlers remain qualified mechanisms and
compatibility adapters; this increment does not claim that all their policy has
already moved to source.

## Source interface and meaning

Add an optional `dispatch-library` declaration with versioned interface
`static-linux-dispatch-v1`, the existing 4096-byte stack limit and 65536-byte
temporary-runtime limit. Its ordinary public export has declared type
`Nat -> StaticLinuxOperation`. The input is a caller-normalized untrusted ABI
selector in 0..2^64-1. The result is a nominal, payload-free Enum with exactly
the semantic alternatives below, plus `Unsupported`. Declaration order is not
an operation identity; the adapter binds names to private native operation codes.

The first source unit belongs in `kernel/arch/x86_64/policy/` because Linux
selector numbers are architecture ABI facts. The selector capture/normalization
belongs to the target's ABI adapter. In particular, existing x86-64 low-32-bit
normalization occurs before invoking source. AArch64/RISC-V require their own
qualified ABI normalization and source number mapping; they may reuse the semantic
operation vocabulary and owned invocation laws. The interface adds no assembly,
register representation, callable address or new core-language construct.

| Normalized x86-64 selector | Semantic operation |
| --- | --- |
| 0 | `Read` |
| 1 | `Write` |
| 2 | `Open` |
| 3 | `Close` |
| 5 | `FileStatus` |
| 7 | `Poll` |
| 8 | `Seek` |
| 10 | `ProtectMemory` |
| 12 | `ProgramBreak` |
| 13 | `SignalAction` |
| 14 | `SignalMask` |
| 16 | `IoControl` |
| 22 | `Pipe` |
| 24 | `Yield` |
| 32 | `Duplicate` |
| 33 | `DuplicateTo` |
| 35 | `NanoSleep` |
| 39 | `ProcessId` |
| 56 | `Clone` |
| 57 | `Fork` |
| 59 | `Execute` |
| 60 | `Exit` |
| 61 | `Wait` |
| 63 | `KernelIdentity` |
| 72 | `FileControl` |
| 79 | `WorkingDirectory` |
| 80 | `ChangeDirectory` |
| 81 | `ChangeDirectoryByFile` |
| 97 | `ResourceLimit` |
| 102 | `UserId` |
| 104 | `GroupId` |
| 105 | `SetUserId` |
| 106 | `SetGroupId` |
| 107 | `EffectiveUserId` |
| 108 | `EffectiveGroupId` |
| 110 | `ParentId` |
| 157 | `ProcessControl` |
| 158 | `ThreadLocal` |
| 186 | `ThreadId` |
| 201 | `RealtimeSeconds` |
| 217 | `DirectoryEntries` |
| 218 | `RegisterThreadId` |
| 228 | `ClockTime` |
| 229 | `ClockResolution` |
| 230 | `ClockSleep` |
| 231 | `ExitGroup` |
| 257 | `OpenAt` |
| 262 | `FileStatusAt` |
| 267 | `ReadLinkAt` |
| 273 | `RegisterRobustList` |
| 292 | `DuplicateToFlags` |
| 293 | `PipeFlags` |
| 302 | `ResourceLimits` |
| 318 | `RandomBytes` |
| 334 | `RestartableSequences` |

Every other input selects `Unsupported`. These are selections for the existing
bounded handlers, including their existing errors and unavailable-operation
responses. Selecting an operation does not promise success or expand a handler's
admitted resource, startup, argument, mapping, process or lifecycle contract.
In particular, this does not implement missing syscalls or the complete Linux ABI.

## Adaptation, ownership and lifetime

After saving the complete user context and switching to the owned kernel stack,
invoke source before inspecting syscall arguments or entering any handler.
The source sees only the semantic selector, never user arguments, pointers,
register slots, native resources or contexts. The bridge boxes that fact, calls
ordinary source and converts its exact nominal Enum result to a private operation
code. A malformed/unrecognized result fails closed. Native routing uses only the
selected operation after cleanup; it must not redispatch the original selector
or silently fall back if a declared source unit is missing or rejected.

The existing handler encoding may remain private during this migration. Its
number-shaped codes are not the source Enum representation or a second parse of
user input. The adapter must bind every semantic alternative explicitly and
qualification must show that changing source selection changes the routed choice
without changing native root/provider objects. An alternative Enum declaration
order must preserve all semantic choices.

Dispatch, header and deadline policies must work independently or together.
They share an arena only through exclusive sequential invocations with distinct
active-policy identities and code/runtime bindings. A dispatch invocation finishes
before a selected handler can invoke header or deadline policy. It preserves the
captured context, resource state and all argument facts; normal handler validation
still treats those restored words as untrusted. No source value escapes the
checked scalar operation adapter. Complete input/source/runtime/bridge allocation,
actual call frames, native saves and alignment enter the same unchanged reserves.
Acceptance, `Unsupported`, failure and terminal teardown retain complete erasure
and selected-binding readback obligations.

Ordinary nominal Enum construction, local transport, branches and acyclic root
helpers may join the bounded numeric subset with zero representation allocation
for payload-free tags. Identity and result-shape checking remain ordinary language
rules. Imported Boolean/numeric helpers retain their existing qualified meaning;
this does not admit arbitrary nominal module interchange, payload variants,
callbacks, effects, recursion or persistent source allocation.

## Qualification and review

Risk is high at syscall entry, target normalization, nominal representation,
argument preservation, source choice versus native fallback, shared-arena reentry,
complete bounds and terminal reclamation. Review follows authoritative ABI facts,
source checking, native bridges/routing, independent operation expectations and
actual saved context/storage. This is independently authored implementation using
reviewed interface facts under the existing licensing policy; it adds no imported
Linux implementation code.

Required qualification includes all mapped and unknown selectors, full-width
boundaries, native x86 normalization/unused-argument cases, wrong signatures and
nominal vocabularies, reordered Enum adaptation, exact source/module publication,
actual source facts and cleaned results, preserved argument/context/SIMD state,
dispatch-only and shared-policy operation, changed-source rejection before a
handler executes, stack/allocation proofs and complete runtime regression.

General syscall argument interpretation and retained subsystem state/effects,
scalable processes, dynamic ELF, writable storage, networking, namespaces/cgroups,
distribution replacement, Docker/Podman and VM hosting remain required by the
[Step 4 acceptance](../step-4-status.md).
