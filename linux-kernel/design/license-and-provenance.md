# Kernel licensing and provenance policy

## Decision

`TK-DEC-011` adopts a clean-room implementation boundary for the Topal kernel.
Independently authored Topal kernel source, kernel-specific build tools, and
tests are first-party repository work under the Unlicense. The repository does
not copy or adapt Linux implementation code merely because Linux defines the
compatibility target.

This policy was approved in the project discussion before kernel-foundation
implementation. It aligns Rust workspace metadata with the repository's root
license; it does not relicense third-party material.

## TK-LICENSE-001 — First-party kernel work

New first-party work below `linux-kernel/` is covered by the root Unlicense
unless a file or deliberately isolated subtree carries a different explicit
SPDX expression. Contributors shall not label third-party or derived material
as Unlicense.

The implementation may use interface specifications, architecture manuals,
reviewed knowledge records, generated inventories, and independently observed
Linux behavior. It shall be expressed independently in the Topal-native design
and shall not reproduce Linux internal source structure or implementation
expression.

## TK-PROVENANCE-001 — Required import record

Before any third-party source or generated copy is committed, its review record
shall identify:

- upstream project and immutable version or revision;
- authoritative source URL and path;
- cryptographic digest of the exact input;
- original SPDX expression and required copyright or notice text;
- whether the committed result is verbatim, generated, translated, or modified;
- generator and reproducible command when generation is involved;
- destination path and compatibility with the surrounding artifact; and
- reviewer disposition.

Missing, ambiguous, or incompatible provenance fails closed. A generator does
not change the license of its input or output merely by transforming syntax.

## TK-UAPI-LICENSE-001 — Linux UAPI exception boundary

An exact Linux UAPI import, if later required, shall be isolated from
first-party kernel source and retain the actual upstream per-file SPDX
expression. Linux UAPI headers commonly use
`GPL-2.0 WITH Linux-syscall-note`; the importer shall preserve the expression
found in the pinned Linux 7.2.9 input rather than assume that every file is
identical. Required license and exception texts, notices, source paths, input
digests, and generator identity shall accompany the import.

Compatibility facts recorded from UAPI, documentation, or observation are not
silently treated as copied source. Conversely, calling a copied definition an
interface fact does not remove its provenance or license obligations.

## TK-BINARY-001 — Firmware and binary material

Firmware, prebuilt objects, archives, VM images, and other third-party binaries
shall not be committed without a separate redistribution review recording the
same provenance fields and any delivery restrictions. A system-installed or
downloaded qualification dependency may instead be pinned by identity and
digest while remaining outside version control, as the QEMU lab already does.

## Enforcement boundary

Every implementation PR shall state whether it adds third-party material. A PR
that does shall include the import record and applicable license texts in the
same review unit. Automated inventory and SPDX checks may enforce this policy,
but passing a tool does not replace the required human disposition.

The authoritative Linux licensing reference is
<https://kernel.org/doc/html/next/process/license-rules.html>. This record is a
project contribution policy, not legal advice or a claim that every interface
fact has the same copyright status.
