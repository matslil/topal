# Step 4 increment: reusable prepared-process observations

Prepared processes can now query PID, TID, parent PID and system information
repeatedly after startup. Queries observe retained process/artifact facts
without advancing a shell-specific sequence. An unmodified replacement BusyBox
shell can print its PID, change cwd, run several builtins and exec another
program or exit with a nonzero status. BusyBox `uname` observes the guest view.

Systems decision `TK-DEC-247` and rules
`TOPAL-SYSTEMS-STATIC-PROCESS-IDENTITY-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-UNAME-001` define this increment. Artifact `/224`,
root `/213`, provider `/214`, provider object `/206`, boot `/124` and gate `/224`
identify it. Core-language meaning is unchanged. Step 4 distribution boot,
Docker/Podman and VM hosting remain open.

## Authority, copying and lifetime

The private process table imports the prepared PID-one/single-thread view at
header offset 56. PID and TID observe one; parent PID observes zero. This is
the existing prepared process identity, retained across exec independently of
user memory. Queries ignore unused argument registers and preserve all
non-result registers, startup phase, descriptors/positions, cwd, signals and
image state. Terminal release erases the whole table, including this witness.

Native uname uses the same immutable guest record as the predecessor adapter:
six ordered 65-byte fields for `Linux`, `topal`, `7.2.9-topal`, `#1 Topal`,
`x86_64` and `(none)`, including complete zero padding. The adapter checks all
390 destination bytes against current writable mappings before copying. A
null, overflowing, unmapped, read-only or cross-guard destination returns
`EFAULT` without output. Success returns zero. Immutable source data needs no
copied user-input staging; native candidate registers are not retained after
return. Host uname values and application names do not select the record.

The original first-image preparation keeps its predecessor adapter. Shared
queries take over after the process table exists and qualified startup has
completed. Broader process identity, PID/UTS namespaces, mutable hostname/domain
and partial fault-copy semantics remain outside this qualification.

## Executed qualification

Rust and Python independently encode a clean query witness into the user
image. It runs before the first replacement and after each of the three
maximum-environment replacements. It repeats/reorders PID, TID and parent
queries with junk unused arguments; compares every uname byte; publishes at
different destinations including one ending exactly at the mapping boundary;
and checks null, overflowing, missing, read-only and one-byte-overrun buffers.
A canary proves rejected copies do not modify the accessible suffix. User-owned
scratch preserves upper-stack startup payloads around deliberate boundary
writes, then is cleared. Later file/signal/cwd/rollback witnesses still pass.

Four external cases extend the standard corpus:

| Case | Observation | Final status | Exec commits |
| --- | --- | --- | --- |
| `replacement-shell-pid` | `1\n`, followed by exec of `true` | 0 | 2 |
| `replacement-shell-cwd` | `1\n/bin\n` across another exec | 0 | 2 |
| `replacement-shell-script` | `one\n/\n/bin\n` from successive builtins | 7 | 1 |
| `uname-kernel` | `Linux 7.2.9-topal x86_64\n` | 0 | 1 |

The full matrix has three clean variants, twenty-two standard external cases
and five RTC cases. All use linked kernel SHA-256
`13720a9dad86cb2586eb4847d252ad651510fd289c614869008fa275aa9148bb`.
The controlled completed-deadline notification regression also runs on that
combined kernel, with its predecessor rejection retained separately.
Full bounded Cargo workspace tests, including 196 compiler tests, thirteen lab
contract tests, formatting, whitespace, document links and the 236-rule
inventory all pass. Evidence binds each executed input,
raw serial transcript, actual status, replacement count and complete rollback,
including a still-running QEMU and final serial quiescence.

## Risk review and remaining work

Risk is high because identity authority and user-copy boundaries affect
compatibility and isolation. Review checks imported private identity, unchanged
process state and phase, native register preservation, complete writable
extent validation, exact immutable record provenance, upper-stack fixture
preservation, exec retention and table erasure. Independent fixture encoders
and unmodified application/shell executions supplement self-review. Human PR
review and merge remain required.

Portable meaning is retained process/system views; native selectors, record
layout and copy mechanics remain target-owned. The
[design](design/reusable-process-observations.md) keeps broader namespaces and
process creation separate. General exec selection and mappings, dynamic ELF,
fork/wait, writable filesystems, networking, distribution boot, containers and
VM-host compatibility remain required work in the initial plan.
