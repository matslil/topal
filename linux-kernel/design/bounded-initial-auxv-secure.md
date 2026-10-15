# Bounded initial `AT_SECURE` auxiliary-vector fact

This record resolves `TK-DEC-086` as a systems-profile refinement. It tells
libc that the sealed PID-one image did not cross a privilege-changing execution
boundary, without introducing mutable credentials or a general ELF loader.

## Decision

The provider places one native `AT_SECURE` entry with value zero after the
existing program-header facts and before `AT_NULL` on the bounded initial user
stack. The value must agree with the sealed real/effective root credentials and
the absence of set-user-ID, set-group-ID, file-capability, or security-module
transition processing in this profile.

Constructed code validates the exact ordered tag and value, validates the moved
`AT_NULL` terminator, consumes the enlarged bounded block, and continues with
the existing application proof. The executable-name string remains the same
bounded stack object and `argv[0]`/`AT_EXECFN` alias.

Omitting the fact was rejected because libc otherwise lacks an explicit secure-
execution result. Publishing one was rejected because no qualifying privilege
transition exists and doing so would impose secure-mode environment behavior
without implementing the transition. Deriving the value in machine-provider
policy was rejected because executable security state belongs to the kernel
loader/process contract.

The portable meaning is a non-secure initial execution fact. X86-64, AArch64,
and RISC-V use the same Linux auxiliary-vector meaning even though their entry
lowerings differ. This increment does not add environment entries, privilege
transitions, capabilities, security modules, `AT_RANDOM`, or a general
auxiliary-vector builder.
