# Step 4: ordinary source syscall selection

`TK-DEC-261` implements the [source selection design](design/source-linux-syscall-selection.md),
`TOPAL-SYSTEMS-NUMERIC-ENUMS-001` and
`TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001`. An ordinary Topal module now selects
the existing static Linux operations from an ABI-normalized selector. The native
adapter routes the actual named source result after cleanup. This moves selection
policy into Topal; argument interpretation, retained subsystem state and substantial
Linux handler policy still need migration. Core-language meaning is unchanged.
[Step 4](step-4-status.md) remains unfinished.

The primary kernel SHA-256 is:

```text
be55695f826efdb6d796668172565b267471376c5c40947d468513a74bba71ad
```

Artifact/root/provider/object/boot/gate revisions are
`237 / 223 / 224 / 216 / 137 / 237`. Numeric metadata, allocation certificate
and owned numeric machine profile advance to version five, recording the
nominal source-tag adaptation. Finite-library and module-snapshot formats retain
their existing versions.

## Source and native responsibilities

The [x86-64 source ABI module](kernel/arch/x86_64/policy/static-linux-dispatch.t)
exports `Nat -> StaticLinuxOperation`. Its payload-free nominal Enum contains
55 named operations and `Unsupported`. A balanced tree of ordinary comparisons
and acyclic helper calls selects the operation; the imported
[comparison helper](kernel/support/syscall.t) receives semantic numbers only.
The source declaration deliberately orders its alternatives differently from the
private handler encoding. The ordinary frontend retains declared input/result
types, while the adapter checks the complete named vocabulary and binds actual
tags by name. Changing source declaration order does not change operation identity.

The source input is an untrusted normalized selector in 0..2^64-1. Native x86-64
low-32-bit normalization precedes invocation, preserving the existing ABI. Source
sees no user arguments, pointer, context, resource or execution capability.
Selecting a named operation does not bypass the existing handler's resource,
startup, argument, mapping, process, error or lifecycle checks. Missing syscall
implementations remain missing; `Unsupported` retains native ENOSYS behavior.

The private bridge boxes the selector through the shared arbitrary-precision
runtime, calls ordinary source and adapts its actual nominal result. Native
routing consumes only that checked operation, with no redispatch of the original
selector and no silent fallback for a declared source unit. Existing private
handler codes are retained behind an explicit named adapter; their number-shaped
encoding is neither the source Enum representation nor an external interface.

Selection follows complete user-context capture and the switch to the owned
kernel stack. It finishes before argument inspection or handler entry. The
invocation preserves native callee-saved and SIMD state and restores all six
captured argument words plus saved return-IP/flags facts. Those words remain
untrusted. The complete source arena and exactly the retired 4096-byte stack
reserve are erased before routing. The first integrated run exposed a missing
return-IP restoration; restoring both return facts fixed it, and the physical
boundary oracle now checks those facts on every observed invocation.

Dispatch, header and deadline policies work independently or share one exclusive
arena with distinct active-policy identities and private code/runtime bindings.
Dispatch cleanup precedes any header/deadline invocation by the selected handler.
Exec retains the selected bindings; termination erases and reads back all of them.
Exact source/module bytes, generated bridges, retained runtimes, native objects
and resource proofs remain linked through publication and boot adaptation.

## Bounded nominal values and resource proof

The numeric subset now admits payload-free root-module Enum construction,
bindings, direct helper transport, equality and decisions. Bounds retain the
checked kind and possible tags; joins require the same kind. Decisions validate
every emitted action, retain reachable result alternatives and charge the maximum
reachable action plus the subject. A known constructor can therefore bound one
action without admitting an unqualified dead form. Imported nominal interfaces,
Enum container/Record payloads, general variants/effects, callbacks, recursion and
persistent source allocation remain outside this qualification.

The complete dispatch allocation bound is **56 bytes** within **65536**:
**24** bytes for the maximum-size boxed selector, **32** owner-header bytes and
zero source/runtime allocation for this comparison/tag policy. The maximum
operand width is 64 bits; ordinary numeric semantics remain arbitrary precision.
The complete actual stack bound is **632 bytes** within **4096**, including native
saves (**64**), alignment (**32**), all actual source/runtime frames and returns.
The largest observed arena use is **56 bytes**. Header and deadline bounds remain
**1224/9648** and **1352/41624** stack/allocation bytes respectively.

Native protocol qualification executes **65651 cases per Enum order**, **131302**
in total, through two actual independently inspected objects. Each order covers
all selectors 0..65535, high-bit variants of every mapped selector and extreme
64-bit facts. The independent expected set includes holes and bounded/error arms.
Reversing source alternatives changes every tag mapping while preserving results,
allocation bounds and native behavior. Additional tests cover exact nominal
vocabulary/signature, reachable branch costs, joins, equality, rejected dead forms,
all independent/shared policy combinations and altered boot metadata. The semantic
model independently checks the normative selector relation and normalization
boundaries without assigning authority or private handler encodings.

## Executed VM observations

The [dispatch harness](labs/qemu/x86_64/dispatch_gate.py),
[GDB inspector](labs/qemu/x86_64/dispatch_inspect.py) and
[retained evidence](labs/qemu/x86_64/results/topal-source-linux-dispatch-gate.json)
cover four physical runs:

- The primary kernel completes **1107 observed selections**, covering 60 distinct
  raw selectors. At every invocation the oracle checks normalized input, actual
  source tag, named adaptation, native code, complete context/backing/history
  preservation, restored argument/return facts, SIMD state, full temporary erasure
  and the boundary before handler entry. Terminal release clears all bindings and
  owned storage.
  These runs observe 54 mapped operations plus `Unsupported`; `Poll` selection
  is covered by the exhaustive native selector fixtures.
- A separately linked reversed-Enum kernel completes the same **1107** selections
  with the same semantics and independent tag expectations. Its SHA-256 is
  `dce0fac6113b0c4d8fd521986fa03e1a1258bec78df1448e192909bcee7975e0`.
  Native root/provider objects and the module snapshot remain unchanged.
- A dispatch-only numeric root, without header or deadline libraries, completes
  another **1107** selections and full reclamation. Its SHA-256 is
  `6743dd3ec8a81fa31e330ef2fd2735d6ced34debb67da52b1b120af99c81579b`.
- A changed source leaf returns `Unsupported` for the first `ProgramBreak`
  selector, 12. The actual tag is 50 and the adapted code is 2^64-1. The independent
  oracle rejects this choice after cleanup and before a handler executes. Native
  root/provider objects are unchanged, proving that routing consumes the changed
  source result. This expected rejection has no successful runtime result.

There are **3322 fully checked invocation boundaries** across these runs. The
actual imported source frame receives selector/candidate 12, agrees with captured
input and lies at stack depth **504**, within the complete bound. Heap values are
invocation-owned; static numeric constants are verified against actual immutable
ELF section bytes. A single exact helper location avoids multiplying a line
breakpoint across compiler instantiations. At most four hardware breakpoint
locations are active, and backtraces stop within three qualified source frames.
The debugger harness allows 180 seconds for the full per-invocation state checks;
this does not alter kernel behavior, allocation limits or the ordinary runtime
acceptance timeout.

Validation includes the full bounded serial Cargo workspace, **228 compiler
library tests**, **119 semantic-model tests**, **606 language tests**, **26
toolchain contracts plus four reference-VM contracts**, formatting/whitespace,
relative links and **257 unique normative systems rules**. Eight additional
altered dispatch bindings/certificates fail boot adaptation, alongside the
existing ten header-binding alterations.

The complete retained physical matrix contains **58 successful primary runs on
the same kernel**, five separately linked positive Enum/Record/header-only/
dispatch-only variants, one controlled header refusal and nine expected
rejections: **73 retained VM runs**. It includes the complete 34-workload
unmodified external corpus, clean environments, clocks/time, imported source and
runtime helpers, child/pair/FIFO state, large command substitution and complete
teardown. The refreshed header-only root explicitly removes both deadline and
dispatch libraries; child/pair/FIFO termination checks include the new binding.
The previous [header qualification](step-4-source-elf64-header.md) remains historical
evidence; current results bind the new source, metadata and primary kernel.

## Mapping, review and remaining work

For a future Linux baseline, compare the retained architecture syscall inventory
with the source ABI map and the named operation contract. Number mapping changes
belong in source; changed argument/error/resource semantics still require handler
and conformance updates. The selected operation vocabulary is separate from target
register capture and private native routing codes. Other architectures require
their own normalization, source mapping, native adapter, calling convention and
complete resource evidence.

Risk is high at entry capture, nominal representation, callback/routing boundaries,
shared storage, resource proofs and reclamation. Single-agent implementation and
review follow authoritative design/specification, independent native and semantic
oracles, actual source/native objects and complete physical state checks. This is
not an independent human code review. No third-party material is added: source and
tests are independently authored using reviewed ABI facts and the existing pinned
qualification dependencies; no Linux implementation code is imported.

General types/buffers, effectful retained Topal subsystems, scalable processes and
scheduling, signal delivery, general/dynamic ELF, writable storage, networking,
namespaces/cgroups, reference-distribution replacement, Docker/Podman and VM-host
qualification remain required by the [initial acceptance](step-4-status.md).
