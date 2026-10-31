# Caller-owned exec environments and stack payloads

Status: systems-feature direction adopted under the standing authorization;
implementation and qualification pending (`TK-DEC-245`).

## Purpose and ownership

Exec must transport the caller's environment rather than recognize four chosen
strings. This refinement retains checked owned byte aggregates for argv and
envp independently of executable identity. Argument zero is caller data; it
must not grant execution authority or be required to equal the pathname.
The selected executable remains the separately qualified immutable object.

The initial adapter keeps one through eight arguments and 256 aggregate argument
bytes, and admits zero through sixteen environment entries occupying at most
1024 bytes including terminators. Null envp is an empty environment. Duplicate
keys, empty strings and arbitrary byte contents are transported without kernel
interpretation, sorting or key/payload comparison. A later binding may raise
these resource bounds with its own backing and qualification. Null/empty argv
normalization, general executable selection and unbounded ARG_MAX coverage
remain separately tracked; this profile does not claim all exec inputs.

Every vector qword and string byte must pass checked live readable-user
validation before access. A count overflow is detected at the first additional
nonnull vector entry without reading that entry's string. Aggregate exhaustion
returns `E2BIG`; bad input returns `EFAULT`. Failed requests preserve image,
descriptors, shared offsets, cwd, signal state and prepared identity, and erase
the entire private aggregate. No old user pointer survives the copy.

## Replacement stack

The fresh image receives argc, argv, envp and complete native auxiliary vectors
derived from owned lengths and offsets. Each pointer names this image's storage.
Both vector terminators and the full `AT_NULL` pair are explicit. `AT_EXECFN`
names the admitted execution pathname independently of argv[0]. Platform and
random payloads belong to the new image and use its actual auxiliary identity.

All argument/environment/execution-name/platform/random bytes shall be disjoint
from the vector area and ordinary stack-growth region. In the initial x86-64
binding they lie above the entry stack pointer, so downward growth cannot
overwrite startup strings. The fixed bounded x86-64
binding may use a new aligned entry and reserved maximum vector/payload areas
inside the existing two-page stack, while retaining a declared free stack
prefix. This changes placement, not source semantics or the public ownership
model. Kernel calls and staging still use the private kernel stack.

The snapshot's unused interval below its rollback-stack copy can hold the
larger aggregate without new native allocation. Its exact offset/size is private;
construction must prove separation from pristine image bytes and rollback data.
The whole aggregate is consumed and verified zero before image entry. Terminal
reclamation keeps complete snapshot/stack erasure. Startup providers must use
the actual generation's stack/auxiliary identity rather than a prior fixed
environment or payload address.

## Qualification and review

The clean executed corpus must cover null/empty/multiple environments, repeated
keys and empty entries, maximum entry/byte boundaries, inaccessible vector and
string prefixes, checked overflow and a nonnull seventeenth pointer whose
string must not be read. It must retry successfully after failures, verify exact
transported bytes and pointers, separate executable name from arbitrary argv[0],
prove payloads survive deliberate writes below initial RSP, and complete normal
nonzero-status reclamation. Exec failure must preserve inherited process state.

External cases shall change cwd or export variables before exec and observe
the resulting environment/cwd through ordinary applications. Existing argument,
file/directory, clock, signal, identity and rollback cases remain regressions.
No application name, environment key or output comparison may implement the
service. The harness alone compares expected observations.

Risk is high because untrusted variable aggregates and stack placement cross
image destruction. Review must check complete copies before commit, resource
limits/error order, offset arithmetic, vector/payload separation, downward stack
growth, generation identity and staging erasure. Portable meaning stays owned
argument/environment bytes and image lifetimes. X86-64, AArch64 and RISC-V use
their own vector widths, stack alignment/layout and entry adapters. No
core-language change or native-pointer escape is introduced.

Stack direction is a binding property: AArch64 and RISC-V also grow downward;
Linux PA-RISC selects `STACK_GROWSUP` in its pinned architecture Kconfig. A
future upward-growing binding must place payloads outside its own growth region
rather than inherit x86 address ordering or a register named RSP. The portable
contract is payload isolation, not that register name or one address order.

Primary reference is pinned Linux `fs/exec.c` (vector counting, copied strings,
exec failure/commit ordering) and x86 ELF startup. Existing bounded argument
and exec-lifetime rules remain the predecessor contract for this refinement.
