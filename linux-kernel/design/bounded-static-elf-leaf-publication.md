# Bounded static ELF leaf publication

This record resolves `TK-DEC-129` as the first mapping-authority transition for
the pinned static ELF. It is a systems-profile refinement only: portable source
names an opaque construction and semantic page roles, while the provider owns
translation-table representation and instructions.

## Decision

The exact live 503-frame payload extent and disjoint one-frame x86-64 metadata
extent shall be consumed together into one affine `StaticImageConstruction`.
The consumed resources cannot be released or reused while that owner is live.
Construction preparation is fallible and shall publish, in an inactive opaque
translation object, exactly 503 adjacent initial mappings:

- virtual pages are the checked static-ELF page plan's ordered
  `0x0040_0000..0x005f_7000` interval;
- payload frame ordinal `i` backs virtual page ordinal `i` for every
  `0 <= i < 503`;
- every initial leaf is present, user-accessible, writable, and
  non-executable (RW/NX); and
- the remaining nine entries in the qualified 512-entry leaf table are absent.

The checked source emits exactly one
`TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_OK` witness while the construction owns
both inputs. It then revokes the inactive mappings, requires target publication
completion, returns and releases the exact metadata extent, reclaims and
releases the exact payload extent, and only then continues. Failure is fatal;
no partial owner or mapping may escape.

## Provider boundary

Source does not name a page-table level, entry width, bit encoding, physical or
virtual integer, invalidation instruction, or register. The x86-64 provider
may realize the opaque object as one aligned 4 KiB, 512-entry leaf table because
the checked span occupies one two-MiB window. It shall validate both input
intervals again, zero the complete table before construction, install the 503
ordered payload leaves with the architecture NX facility enabled by the
qualified translation path, leave nine entries absent, verify the complete
result, and publish it with the target-required ordering operation.

The object is inactive: no parent entry makes it reachable from a running user
address space in this increment. Revocation shall clear all 512 entries and
complete target-required ordering before metadata ownership returns. Therefore
the witness proves exact leaf construction without granting user access or
requiring per-leaf invalidation.

## Affine return protocol

Preparation consumes both `PhysicalFrameExtent` identities. Revocation refines
the live construction and returns the original metadata extent while retaining
exclusive payload ownership. After source releases metadata, payload reclaim
consumes the revoked construction and returns the original indivisible
503-frame extent. Wrong construction identity, release before return, payload
reclaim before revocation or metadata release, duplicate revoke/reclaim,
reversed release, and disposition with any construction state live are invalid.

## Architecture pressure test

The common contract is exact page-to-frame order, semantic initial RW/NX
rights, inactive atomic publication, complete revocation, and affine return.
AArch64 may use several levels, break-before-make rules, and a different
execute-never encoding; RISC-V may use Sv39/Sv48 leaf conventions and `sfence`;
capability systems may retain sealed mapping capabilities. None of those
representations changes the source operation or exposes addresses.

## Alternatives rejected

Writing entries through ordinary byte mappings was rejected because it exposes
target representation and weakens typed mapping authority. Publishing leaves
one at a time was rejected because partial failure would require 503 visible
rollback states. Attaching the table to the existing probe address space was
rejected because that owner has a different layout and lifetime. Skipping
revocation because the object is inactive was rejected because metadata reuse
must not preserve stale authority. Applying final R/RX/R/RW/RELRO rights here
was rejected because zero/copy must occur while every payload page is RW/NX.

## Deliberate bounds

This increment constructs and revokes the exact initial leaves but does not
attach them to a runnable address space, touch payload bytes, copy the ELF,
apply final permissions, initialize TLS or stack state, select an entry, or
enter BusyBox. General page counts, sparse backing, shared pages, demand paging,
huge pages, and recoverable publication failure remain outside scope.
