# Step 4 initial scalar identity syscalls

## Outcome

The initial process now implements the closed Linux x86-64 scalar identity
family needed by a single-process, single-thread root environment: `getpid`,
`gettid`, `getppid`, `getuid`, `geteuid`, `getgid`, and `getegid`. PID and TID
are one; the parent and real/effective user and group identities are zero.
Every call requires a live user context and a return continuation inside the
runtime-derived finalized executable mapping.

The constructed application invokes and checks every member before
`exit_group(0)`. These calls expose no pointer arguments, allocate no state,
and do not change core-language rules.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-IDENTITY-001` closes the initial scalar family and
requires coherent single-process results, executable-return validation, and
no hidden identity mutation. Structural tests prove all seven application
invocations, their immediate result checks, the complete dispatch-number set,
the shared live-context and continuation guard, and the provider-probe
separation.

Artifact schema `/58`, provider `/55`, provider object `/55`, artifact root
`/56`, boot adapter `/18`, and gate `/60` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The constructed-image post-return
witnesses are reachable only after every scalar identity result has matched.
The evidence records:

- linked kernel: `a1ed8b1469077a27c2fa9adc8408f0c5e47850806fd2193b70c0c32c388006bc`;
- boot image: `405424efbf14a60080408694b6b06efe0ecd455bf33deb7631d5586c17cd9252`;
- boot resource: `08371611e560073011b5ed8a48d913aa8fc9b0c95a616f02d56902298371c908`;
- generated default image: `0298ebe04f4492d86eeac1ec1f4d07e74cefdad6d46e07c781718ed7ac078736`;
- supplied image: `da28c8fad920335caa130ec8d8956c9c96c30ebeaa64797b338bb3cc008c8d07`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `5dee169d115ff397b1faf12af2db4a19a99b09fcf4f72c3b88df3398ba8c0da2`;
- boot provenance: `0e32a7f9447560bff00028a8f1679ce35c4d15fda9810087da3d2b371cb4c24c`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `3ecf24a17892a1ee351cb14becacb596223050d2cf22e8d1c154df4593505527`.

## Risk and review

Risk is medium: the family broadens the ABI surface but has no pointer,
allocation, mutation, blocking, or concurrency behavior. Review covered exact
Linux x86-64 syscall numbers, coherent namespace values, provider-probe
isolation, executable continuation validation, dispatch completeness,
application-side result checks, full workspace validation, and two independent
physical runs.

## Remaining boundary

The identity model is intentionally one process and one thread in a root
namespace. It has no process creation, threads, sessions, process groups,
supplementary groups, credentials mutation, user namespaces, capabilities, or
security modules. Those require later process and security services.
