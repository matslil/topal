# Step 4: ordinary Topal file-open plans

`TK-DEC-266` implements the [source open-plan design](design/source-file-open-plans.md)
and `TOPAL-SYSTEMS-FILE-OPEN-PLAN-001`. The selected finite profile can now decide
existing-file/directory opens and anonymous temporary creation in ordinary Topal.
The native consumer validates its owned action before resource publication.
Named creation, reopen and unlink remain subsequent work. [Step 4](step-4-status.md)
remains unfinished.

## Qualified configurations

The default root retains the frozen `boot-open-decision-v1` compatibility profile.
Its current whole linked-kernel SHA-256 is:

```text
87cb212d57be06b35917f1653c358b9cf5b66603601824cb13310a124ed1346b
```

The explicitly selected `file-open-decision-v1` configuration has SHA-256:

```text
f41fbad2a478069ef361f7bece2d490277eefc37004f769d1b1212aedae56476
```

Artifact/root/provider/object/boot/gate revisions are
`242 / 228 / 229 / 221 / 142 / 242`. Finite-library metadata remains version two;
its selected interface determines argument count, nominal vocabulary, exact tag
permutation and private export. The old four-entry JSON tag representation stays
identical. Numeric metadata and the volatile-file layout stay at their existing
versions. The provider note includes revised identities, so the whole default
ELF digest changes even with the frozen source policy selected.

## Ordinary source and trusted ownership

[The source unit](kernel/policy/file-open-access.t) defines eight Boolean semantic
facts and nominal `FileOpenDecision` with eight named alternatives. Helpers
express existing/absent target disposition, exclusive creation, directory
requirements, immutable access and truncation. Write access and truncation are
separate facts; a future read-only description may request Linux truncation.
The source contains no Linux flag, selector, errno, object identifier or address.
Path walking still owns prefix/trailing errors and native flag admission.

The existing pure finite library checker and ordinary frontend/backend compile
this code. The generated adapter unpacks the selected fact count, calls ordinary
functions and adapts the actual nominal result by name. It does not interpret
source text or depend on Enum declaration order. The complete acyclic call path,
including wrapper, return addresses and existing 32-byte native shim reserve, is
**388 bytes**, within the unchanged **1024-byte** owned stack reserve. Code is
GPR-only with no red zone, hosted runtime, allocator, foreign dependency, writable
global or retained temporary graph. The frozen profile retains its separate
export, three facts, four choices and **180-byte** bound.

Existing opens derive facts after checked flag/name/free-slot/lookup processing.
The provider checks an accepted open against its owned directory requirement
and read-only access conditions. For `O_TMPFILE`, it first checks the resolved
mutable directory and admitted write/anonymous intent, then asks the source for
an absent regular target in that view. Only CreateDescription can continue.
The native owner subsequently reserves its existing object/description cells,
samples the clock and publishes its own resources. O_EXCL on an anonymous file
remains no-link intent; it is not misreported as exclusive named creation.
The source result cannot grant a code, memory, file or write capability.

Root relocation, provider selection and boot validation bind exactly the chosen
private export. Mismatched interface/export, short or duplicated tag sets and
missing binding reject before boot publication. Both version exports cannot
coexist. The binding follows existing exec/fork lifetime and complete terminal
witness erasure. No core language semantics or source assembly changes.

## Independent qualification

The [portable model](../src/topal-semantics/src/systems_file_open_plan.rs) distinguishes
Linux error precedence, absent/existing views, create/exclusive, directory and
truncate decisions without native authority. Import tests retain exact stack-only
requirements. Independent compiled C clients execute all **256** valid fact masks,
**65280** other 16-bit masks and two wide masks per source order. Normal and
reversed Enum orders both pass: **131076** native bridge calls. These clients
are test oracles and are never kernel-linked. Existing frozen native clients
remain regressions.

The [physical gate](labs/qemu/x86_64/file_open_gate.py) records normal/reversed
source selections, lifetime execution, one wrong-plan rejection and a frozen
compatibility run. Both source selections execute **81** actual policy calls,
including ordinary helper frames through missing-view/missing-kind, and preserve
callee-saved GPRs, SIMD values, stack alignment and the retained binding. Their
**130** caller-issued file syscall-return checks use actual sparse bytes and
owned storage. The normal kernel also passes the existing **161**-return
exec/fork/alias/private-umask lifetime witness and complete store reclamation.
Source variants retain identical root/provider inputs. Reversed tag order changes
the library representation and whole kernel while preserving caller behavior.

The wrong helper returns OpenDescription for the first create plan. The inspector
observes that result after an actual source call, then follows the owner's real
comparison branch to its fail-closed instruction sequence. The identity counter
remains seven, object/page/payload storage remains clean, and neither description
bank publishes an owned file description. The VM is stopped there: this expected
rejection claims no normal completion or terminal cleanup. Its **71** observed
policy returns are not a successful file-service qualification.

The current combined cohort contains **99 Topal VM observations**: **62**
successful default-kernel runs, **17** successful variants, **six** controlled
observations and **14** expected rejections. The pinned Linux 7.2.9 sparse-file
control remains separate. The old source dispatch, scalar/retained/structured
state, header, clock, process/pair/FIFO, command/environment and deadline suites
are regenerated for the current certificates; historical reports keep their
historical identities. [Open-plan evidence](labs/qemu/x86_64/results/topal-file-open-plan-gate.json)
and [traceability](traceability.md) bind exact current sources and harnesses.

Validation passes: the complete bounded serial workspace, including **243**
compiler library, **261** compiler CLI, **141** semantic-model and **610** language
tests; **31** toolchain and **four** reference-VM Python contracts; formatting,
whitespace, relative links and **268** unique normative systems rules. The first
workspace run exposed missing ledger/traceability maintenance; after correction,
the coverage tests and full workspace passed. The complete physical driver runs
under `scripts/run_bounded.py -- env RUST_MIN_STACK=16777216 python DRIVER`,
so Cargo, LLVM, GDB and QEMU children share the required resource bound.

## Review and remaining work

Risk is high because a source choice precedes privileged mutation. Review traced
source purity and the complete helper graph, target stack/calling convention,
nominal adaptation, unique versioned binding, native action revalidation and
failure before object/page publication. Independent finite/native and physical
oracles check those boundaries; the full regression suite checks shared owners.
No external reviewer or independent agent review is claimed.

This increment qualifies a reusable disposition boundary, not linked-file
namespace mutation or an effectful source filesystem. Names, independently
reopened descriptions, unlink lifetime, dynamic directory records and creation
transactions are the next use of this boundary. General persistent allocation,
ordinary effectful Topal subsystems, distribution replacement, networking,
containers and VM hosting remain acceptance work. ARM64 implementation remains
outside the approved x86-64 focus.
