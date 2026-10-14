# Supplied initial user-image contract

This record resolves `TK-DEC-067` as a systems publication refinement. It lets
the qualified compiler publication use a caller-supplied initial ELF while
retaining the canonical artifact and provenance boundary established by
`TK-DEC-066`. It changes no core-language rule.

## Decision

The compiler command line may select one initial application with
`--initial-user-image PATH` when, and only when, it publishes the qualified
systems target. The compiler must read and validate the complete file before
creating publication state. The accepted x86-64 profile remains deliberately
bounded: one fixed-size ELF64 executable with the current three program-header
layout, one page-bounded RX load segment, one page-bounded read-only data/BSS
segment, and an entry inside the executable file extent.

The validator must preserve the same immutable envelope facts checked by the
runtime provider and validate all admitted variable fields relationally. The
executable payload and initialized-data contents are application authority;
the toolchain must not compare them with its generated qualification program.
On success, the supplied bytes become the canonical `initial-user.elf`, and
the existing artifact and boot provenance chain binds them without recording
the host path.

```text
caller-selected ELF file
  -> complete bounded-profile validation
  -> atomic canonical artifact publication
  -> artifact digest verification by boot packager
  -> exact-byte boot resource
  -> independent runtime envelope validation and load
```

Without the option, publication continues to generate the existing
qualification image. Generated and supplied inputs with identical bytes must
produce identical artifacts; provenance is content-based, not path-based.

## Failure and scope

An unreadable file, duplicate option, use on a nonsystems target, wrong size,
wrong immutable metadata, invalid segment-size relation, or out-of-extent
entry must fail before the output directory is published. A structurally valid
payload whose application behavior is wrong may still fail closed at runtime;
the publisher does not attempt to prove arbitrary machine code semantics.

The command-line selection is a tool interface, not portable Topal syntax or a
new source-level capability. Other architecture providers may define different
bounded executable envelopes while preserving complete prepublication
validation, content-based provenance, and exact-byte consumption.

This increment does not admit general ELF layouts, dynamic interpreters,
relocations, writable segments, TLS, PIE, ASLR, filesystems, process creation,
or additional Linux syscalls.
