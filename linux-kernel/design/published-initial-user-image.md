# Published initial user-image artifact contract

This record resolves `TK-DEC-066` as a systems toolchain refinement. It makes
the initial ELF image a canonical compiler output consumed by the boot
packager, rather than regenerating the image privately in two publication
stages. It changes no core-language rule.

## Decision

The qualified systems publication must atomically emit the initial user ELF
beside the linked kernel, debug file, link map, and provenance. The provenance
must bind the image by filename and digest. The boot packager must read that
published file, verify its digest against the artifact provenance, and package
those exact bytes as the deterministic `init` resource.

```text
checked Topal root + provider
  -> atomic linked artifact {kernel, debug, map, initial-user.elf, provenance}
  -> digest verification at the packaging boundary
  -> deterministic newc member using the exact published ELF bytes
  -> boot-image provenance binds the same image digest
```

Publication fails before rename if image generation, hashing, provenance
encoding, or any other canonical output fails. Packaging fails before its own
rename if the image is absent, changed, or not bound by provenance.

## Authority and alternatives

The compiler remains the initial image producer in this bounded increment,
but the image crosses an explicit file-and-digest boundary. The boot packager
is no longer an independent payload authority. Regenerating in both stages was
rejected because equal code paths are not a provenance relationship. Passing
unbound bytes to the packager was rejected because it would allow kernel and
boot artifacts to disagree silently.

The published filename and digest are tooling facts, not portable Topal
semantics. Other architecture providers may use different boot envelopes, but
must preserve the one-produced-image/one-consumed-image relationship.

This increment does not yet accept an arbitrary external application image,
admit writable data, generalize ELF layout, add a filesystem loader, or change
the process ABI.
