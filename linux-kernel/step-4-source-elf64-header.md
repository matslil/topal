# Step 4: source ELF64 header policy

`TK-DEC-260` implements the [owned header design](design/source-elf64-header-policy.md)
and `TOPAL-SYSTEMS-ELF64-HEADER-LIBRARY-001`. The same-image exec path now invokes
an ordinary Topal module over an immutable 64-entry integer Array copied from
the owned executable. Source interprets ELF fields and returns a Boolean;
it receives no native address or resource authority. Core-language meaning is
unchanged. [Step 4](step-4-status.md) remains unfinished.

The primary kernel SHA-256 is:

```text
dfd1e21ae85e43fecead91750980a790824d3dea127439c7e47422bab1119a7c
```

Artifact/root/provider/object/boot/gate revisions are
`236 / 222 / 223 / 215 / 136 / 236`. Numeric metadata, allocation certificates
and the owned numeric machine profile advance to version four. Complete
native invocation saves are now explicit in the stack certificate. The
finite-library and exact module-snapshot formats retain their existing versions.

## Source and native boundaries

The optional `image-library` has interface `static-elf64-header-v1` and declared
ordinary type `(Array (64, Int), Nat, Nat) -> Boolean`. Every input byte has range
0..255. The caller supplies expected machine identity and maximum program-header
count, each in 1..65535. The imported [ELF helper](kernel/support/elf.t) uses
constant-index checked Optional reads and ordinary arithmetic to interpret magic,
identification, type, machine, version, header/entry sizes and header count.
Unexamined bytes retain their previous admission behavior.

The native adapter validates exact length, nonnull/nonwrapping extent and fact
ranges before access. Caller validation proves the actual immutable file identity
and complete extent. The bridge copies all 64 bytes through shared integer boxing,
List construction and Array collection. Separate private symbol namespaces keep
each numeric unit's runtime, source and frame records unambiguous. Source/module
bytes, generated bridge, retained runtime, native object and proofs remain bound
to exact publication inputs and outputs. Declared missing/mismatched source
snapshots fail publication; boot adaptation rejects missing/cross-policy bindings,
unknown profiles, incomplete module provenance and inconsistent resource costs.

Header and time policies share one exclusive arena with distinct active-policy
identities and code bindings. Header-only operation is qualified separately.
Source values cannot escape an invocation. The native adapter preserves its
callee-saved registers and SIMD state, erases the whole arena and exactly the
retired caller stack reserve, then clears volatile pointers. Terminal return
erases every selected binding. The allocator checks the active policy's binding.

Exec invokes source after successful argv/environment staging and before
close-on-exec filtering, image mutation, signal reset or generation change.
False returns native `ENOEXEC` after staging erasure. Complete existing load-plan,
segment/mapping and resource validation remains required during preparation.
This is source interpretation of an owned header, not general executable loading.

## Resource proof and native tests

The conservative header allocation bound is **9648 bytes** within **65536**:
**6992** source/runtime bytes, **2624** bridge bytes and **32** owner-header bytes.
The bridge includes 64 maximum-size boxed bytes, 64 List nodes, one Array header
and two boxed facts. The operand bound is 32 bits. The complete actual stack bound
is **1224 bytes** within **4096**, including all native/source/runtime callees,
returns, **32** alignment bytes and **64** bytes of native invocation saves.
Stable parameter debug storage is included in actual frame inspection.
The existing deadline policy's complete stack bound becomes **1352 bytes**;
its allocation bound remains **41624 bytes**. Both stay within the same reserves.

The independent C protocol fixture executes **16390** cases: every value at
every byte position plus machine/count boundaries. An unaligned input and adjacent
canaries verify exact reads and unchanged bytes. Eight invalid private-input
guards cover null, short/long input, zero/excessive facts and wrapping extent;
each takes the qualified fatal path before reading the invalid address. Maximum
observed native arena use is **2152 bytes**, below the conservative bound.
The independent semantic model checks all **16384** single-byte mutations,
unexamined fields and extreme/invalid facts. Arena-model tests reject cross-owner,
cross-policy and overlapping invocations without mutation, then verify reuse and
complete reclamation in header-only and shared configurations.
Compiler qualification also rejects ten altered boot bindings/certificates,
including absent/swapped policies, missing module provenance, unknown profiles,
excessive allocation and understated bridge, call-path or complete stack costs.

## Physical observations and regression

The [header harness](labs/qemu/x86_64/header_gate.py) and
[GDB inspector](labs/qemu/x86_64/header_inspect.py) retain
[four physical observations](labs/qemu/x86_64/results/topal-source-elf64-header-gate.json):

- Normal execution decodes the actual source Array header and all 64 unique List
  nodes, compares each byte with the immutable file, checks machine/count facts
  and real imported helper/root/adapter frames, then verifies complete erasure
  and terminal release. The observed source stack depth is 328 bytes and arena
  high-water mark is 2128 bytes.
- Controlled rejection changes only the first copied semantic byte from 127 to
  126. Source rejects it; native `ENOEXEC` is observed before user return. The
  original file, complete descriptor/backing table, process context and clock
  history stay unchanged; only exec staging is erased in its snapshot backing.
- A separate source-helper variant deliberately skips magic admission. The
  independent oracle rejects its wrong acceptance after policy cleanup and before
  native commit. The root/provider objects and root policy source are unchanged;
  the helper snapshot/object/kernel digests change. This is expected rejection
  evidence, without a successful runtime result.
- A header-only numeric root completes normal execution and full reclamation
  with no deadline library. Its kernel SHA-256 is
  `f6aa835f7e7db084417ea951c5e27b610346b99619e1fd10692b27f07963ce47`.

These controlled inputs prove the refusal boundary; they do not demonstrate
loading arbitrary malformed external files. Debug backtraces stop at the three
qualified frames because the enclosing native syscall loop has no DWARF unwind
contract. GDB signed register rendering and monitor/detach timing are normalized
without weakening the required exact boundary proof or debugger exit status.

The complete physical regression retains **57 successful primary runs on one
kernel**, three separate positive Enum/Record-order/header-only kernels, one
controlled refusal and eight expected rejections: **69 retained VM runs**. The
primary corpus includes 34 unmodified external workloads, three clean environments,
clocks/time, ordinary modules/runtime helpers, child/pair/FIFO state and full
teardown. The Enum and Record-order variant kernels have SHA-256 values
`d63bc452890a6694739bca80808ee1ab370649b0ae2703ba1b72dbe3c8b13894` and
`3caeebac1372b7c4f6c310d4a4703520678d0acb5145c5093d4a347c9c7fac97`.

The full bounded serial Cargo workspace passes, including **224 compiler library
tests**, **118 semantic-model tests**, **605 language tests** and the in-memory
publication/boot-adapter contracts. The lab suite comprises **25 toolchain
contracts plus four reference-VM contracts**. Formatting/whitespace checks,
relative links and **255 unique normative systems rules** are checked separately.
Previous [Array qualification](step-4-freestanding-numeric-arrays.md)
remains historical evidence; current results bind the new kernel and versions.

## Review and remaining scope

Risk is high at copied-input lifetime, native representation, multiple policy
publication, namespace isolation, full resource bounds and refusal atomicity.
Single-agent implementation/review follows authoritative design and specification,
the evaluator, shared runtime, emitted object/call graph, independent C/Rust
protocol checks, actual source frames and complete state/storage reclamation.
This is not an independent human code review. No shared numeric/collection
algorithm or core-language semantics changes.

The snapshot/copy interface is architecture-neutral; other targets must qualify
their own native calling convention, code/runtime namespace, stack and resource
adapter. Expected ELF machine identity remains a semantic caller fact. General
byte buffers, persistent subsystem state/effects, scalable processes, dynamic ELF,
writable storage, networking, namespaces/cgroups, distribution replacement,
Docker/Podman and VM hosting remain required by the [initial acceptance](step-4-status.md).
