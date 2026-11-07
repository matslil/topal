# Step 4 bounded general standard-output write

## Outcome

The initial-process personality now accepts caller-selected
`write(1, candidate, length)` requests for one through 256 bytes from a
complete provider-known readable user extent. The handler no longer requires
one exact return address, stack candidate, length, or application payload.
It validates the complete half-open extent without dereferencing an invalid
candidate, copies the whole request into private kernel staging before the
first console effect, publishes exactly the staged prefix, and returns the
requested length.

Invalid descriptors, lengths, and candidates return Linux-compatible
`-EBADF`, `-EINVAL`, and `-EFAULT`, respectively. The existing supplied ELF
still constructs and writes `TOPAL_KERNEL_USER_WRITE_OK`, preserving a stable
physical witness while leaving the provider payload-neutral. No core-language
rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-WRITE-001` records the bounded descriptor, live-state,
executable-return, length, overflow, readable-mapping, staging, output-order,
and result requirements. Structural tests prove the 256-byte cap, both known
mapping ranges, dynamic private-copy and output loops, argument-derived return,
distinct error results, and absence of the application marker bytes from the
provider syscall symbol.

Artifact schema `/55`, provider `/52`, provider object `/52`, artifact root
`/53`, boot adapter `/15`, and gate `/57` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The application-owned write marker
occurs before the input byte is consumed and before every post-return
constructed-image witness. The evidence records:

- linked kernel: `f154bb6b82704874e4412d19623b978a877dbac7bcb5046782b4ca987ed831cc`;
- boot image: `1cf03c207fafe3ec87e1ae835cce934ca042ea85fb478e7aa076db96df083e89`;
- boot resource: `605e482d6c03a69f22617d547aa6cdaef22402cba77b6ebc31e0b6a4b95e3000`;
- generated default image: `8779eb5f532abf434626b6ff48338beb5dc347e82bb7a0269996c0f6f0410f33`;
- supplied image: `de55df80b6263ae77afa3f7896772cf1ff5f35fba1a8db69800c49387bc55d11`;
- serial input: `8e35c2cd3bf6641bdb0e2050b76932cbb2e6034a0ddacc1d9bea82a6ba57f7cf`;
- artifact provenance: `19a4fd7ee449957154a80fd4b31e4ff04f33db2d861a9593b15ae6728cb08ece`;
- boot provenance: `406b7bca00e40e68794d5d51a316d2612e19a5f185d1423a14f555bbc49f7cff`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `70b562f8d9f5fc942c7fbac23a1ee6f4bb446e296f9e772dd2e2ff57c583c5a4`.

## Risk and review

Risk is high because a fixed qualification oracle becomes a reusable
pointer-bearing syscall path. Review covered arithmetic wrap, half-open range
containment, descriptor and size errors, pre-dereference failure, copy-fault
recovery, complete staging before output, dynamic loop termination, stack
restoration, pointer disposal, payload-oracle removal, structural x86 checks,
full workspace validation, and two independent physical runs.

## Remaining boundary

This is still a serial-console-backed initial-process adapter, not general
Linux file output. It has no descriptor table, partial writes, blocking,
signals, restart behavior, pipes, files, sockets, vectored I/O, asynchronous
I/O, or requests above 256 bytes. Those remain subsequent userspace-gate work.
