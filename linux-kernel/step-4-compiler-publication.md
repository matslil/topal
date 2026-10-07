# Step 4 ordinary compiler systems publication

## Outcome

The qualified x86-64 systems provider is now available through the ordinary
`topalc` target interface rather than only through a lab-specific Rust caller.
The existing target, CPU, board, emission, LLVM-tool, source, and output options
select the path:

```console
topalc \
  --target x86_64-unknown-none \
  --cpu generic \
  --board topal-qemu-pc-q35-10.2 \
  --emit executable \
  -o ARTIFACT-DIRECTORY \
  SOURCE.t
```

`ARTIFACT-DIRECTORY` is published atomically and contains exactly
`kernel.elf`, `kernel.debug`, `kernel.map`, and `provenance.json`. The same
source checker, sealed provider, linker, and structural inspector used by the
pinned-QEMU gate own the publication. No Linux process frontend, runtime,
startup object, syscall, libc, or dynamic loader participates.

The target registry therefore reports `topal-x86-64-qemu-kernel` as
executable-qualified. The qualification is artifact-specific: it does not make
ordinary source a freestanding kernel or route a systems program through the
native Linux-process pipeline.

## Fail-closed controls

The initial systems publication accepts only:

- the exact `x86_64-unknown-none` target and pinned QEMU board;
- the generic qualified CPU with no custom target model;
- the canonical executable artifact set;
- the default unoptimized systems profile; and
- no hosted standard-library dependency.

Unsupported emission, optimization, board, CPU, model, source feature, or
provider fails before the output directory exists. Boot-protocol packaging
remains a separate qualified adapter and is not implied by `--emit executable`.

## Evidence and remaining boundary

Compiler library tests retain the complete provider/artifact structural
coverage. CLI integration tests execute `topalc`, inspect the exact four-file
publication set, and prove unsupported controls leave no output. The pinned
QEMU harness consumes this ordinary publication before invoking the separate
packaging adapter; its version-two evidence record names
`topalc-target-interface/1` and retains the same deterministic artifact and
serial digests.
