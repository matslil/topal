# Bounded initial credential auxiliary-vector facts

This record resolves `TK-DEC-057` as a systems-profile refinement. It extends
the provider-sealed Linux-compatible initial stack with the coherent
`AT_UID`/`AT_EUID`/`AT_GID`/`AT_EGID` fact group for the sealed PID-one root
profile. Constructed code must validate all four facts before `AT_NULL`. It
changes no core-language rule and exposes neither credential encodings nor
auxiliary-vector tags to source.

## Decision

The opaque address-space preparation and user-entry operations retain their
architecture-neutral meaning. The provider must append four credential facts
after `AT_EXECFN` and before `AT_NULL`: real and effective user identity, then
real and effective group identity. Each must agree with the credential
identity of the prepared process; the initial bounded profile admits only the
provider-sealed privileged identity used by PID one.

The constructed entry must validate the exact ordered fact group, validate the
existing terminator and shared executable-name string, consume the complete
bounded block, and restore the prior stack top before continuing. After exact
kernel return, source publishes the prior process-entry witnesses, then
`TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK`, followed by the existing image
and teardown witnesses.

```text
prepare sealed PID-one root profile -> construct process-entry stack
  -> validate prior facts -> validate real/effective user/group identity
  -> validate AT_NULL and shared string -> consume block
  -> exact kernel return -> ordered credential witness -> remaining witnesses
```

Source names neither numeric credentials nor tag encodings, pointer width,
word order, alignment, addresses, or instruction encodings. Structural
evidence must prove exact initialization, agreement of all four facts with the
prepared process profile, bounded consumption, stack balance, and fail-closed
branches.

## Isolation and failure contract

No source operation can select or modify credentials. Missing, reordered,
duplicated, unequal, or non-profile values; malformed termination; stack
imbalance; an early or missing credential witness; or teardown before all
witnesses is invalid. Fatal-only completion remains in force after permission
publication begins.

## Cross-architecture pressure

The portable contract is a coherent real/effective user/group identity fact
group matching the prepared process. It does not prescribe numeric tags,
scalar width, byte order, stack direction, alignment, or machine encoding.
The initial x86-64 provider uses Linux x86-64 auxiliary-vector tags and four
zero-valued root identities. AArch64 and RISC-V providers must use their Linux
process-entry conventions. Capability architectures gain no additional memory
authority from scalar credential facts.

This increment does not admit source-selected credentials, transitions,
namespaces, capabilities, secure-execution state, ELF program headers, random
state, platform strings, vDSO/vvar, dynamic linking, general ELF loading,
demand paging, or a general process model.
