# Supplied static ELF artifact

This record resolves `TK-DEC-131` as the publication boundary between the
approved static-ELF plan and later runtime construction. It is a systems
toolchain refinement only. It neither changes portable Topal syntax nor makes
the selected image executable by the current kernel.

## Decision

Qualified x86-64 systems publication may select one caller-owned static ELF
with `--static-user-image PATH`. The compiler shall read and validate the
complete file before creating publication state. Acceptance requires the exact
approved 503-page image shape: the pinned ELF64 identity and entry, eleven
program headers, four adjacent R/RX/R/RW load roles and their file and memory
intervals, the contained TLS interval, the contained page-rounded GNU RELRO
interval, one non-executable GNU stack declaration, and the 2,024,544-byte file
extent. Payload bytes not used as checked ELF metadata remain caller authority.

On success, the selected bytes become the canonical `static-user.elf` output
and its canonical name and SHA-256 digest enter artifact provenance. The host
path shall not enter any output. A publication with no static-image option
retains the existing output set; a publication with the option has exactly one
additional output named and bound by provenance.

```text
caller-owned static ELF
  -> complete ELF and exact-shape validation
  -> immutable 503-page load/page plans
  -> atomic exact-byte static-user.elf publication
  -> canonical-name and digest provenance binding
```

The separately selected `--initial-user-image` remains the small executable
which the current boot resource and runtime consume. The two options may be
used together, but neither may substitute for the other. This increment does
not add `static-user.elf` to the boot resource and the kernel does not yet copy
its bytes into the zeroed construction.

## Failure and ownership

An unreadable file, duplicate option, use on a nonsystems target, malformed
ELF, noncanonical shape, publication collision, failed write, or provenance
encoding failure shall fail before the destination directory becomes visible.
The publisher must validate the bytes that it publishes, not metadata from a
different file or a caller assertion. Exact-byte publication and content
provenance preserve caller ownership without importing the image into this
repository.

The repository shall not contain or redistribute the pinned BusyBox binary.
Qualification may generate a deterministic clean-room fixture with the same
checked metadata shape and independently chosen payload bytes. That fixture
proves the selection, validation, publication, and digest boundary; it does not
claim application compatibility or substitute for later execution of the
pinned external binary.

## Architecture pressure test

Caller-owned content, validation before publication, canonical naming, and
content provenance are architecture-independent policy. AArch64 and RISC-V
would use their own ELF machine, address, relocation, permission, and page-plan
profiles. Capability architectures may additionally bind authority metadata.
Those differences belong to architecture profiles and do not justify exposing
host paths or target ELF parsing as portable source operations.

## Alternatives rejected

Committing the BusyBox file was rejected because it would redistribute an
external GPL binary and erase the explicit caller-input boundary. Pinning only
its digest was rejected because the compiler must still validate every layout
fact used by later construction. Reusing `--initial-user-image` was rejected
because that option currently names a different 12 KiB runtime contract.
Publishing an unbound side file was rejected because downstream tooling could
silently consume bytes that were not part of the reviewed artifact. Packaging
or copying the image in this increment was rejected because the current boot
resource and zeroed construction do not yet have an exact-byte transfer
lifecycle.

## Deliberate bounds

This increment selects, validates, publishes, and binds one exact-shape static
ELF. It does not redistribute BusyBox, package the image for boot, expose file
bytes to checked source, copy or verify load intervals, apply final W^X/RELRO
permissions, initialize TLS or a process stack, attach an address space,
publish an entry, or enter the image. PIE, dynamic linking, arbitrary static
ELF layouts, multiple images, and mutable post-publication inputs remain out of
scope.
