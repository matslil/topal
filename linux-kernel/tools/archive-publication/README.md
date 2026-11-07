# Publishing qualification archives

The VM runners retain complete raw artifacts locally. Some contain the pinned
external BusyBox ELF directly or inside a generated initramfs. Those dependencies
stay outside Git under [TK-BINARY-001](../../design/license-and-provenance.md).

`archive_publication.py publish` preserves the producer's original record and
file index verbatim as `raw-qualification.json` (or `raw-archive.json`) and
`raw-files.json`. It verifies every indexed raw blob, then moves blobs containing
the exact pinned external ELF to ignored `linux-kernel/.cache/qualification-external/`.
The published index still names every original artifact and retains its exact
length and SHA-256. First-party blobs remain in the archive.

`publication-projection.json` records the external input's identity, the original
record/index hashes, the publication tool hash and exact reconstruction recipes.
A recipe contains the external input by reference and compressed literal bytes
for first-party wrapper material. It preserves repeated occurrences and complete
initramfs headers, padding and trailers. The tool verifies reconstructed bytes
against the original artifact before publishing. The current record binds the
published index and projection separately from the preserved producer record.
This storage change neither replaces observations nor changes their result or
qualification scope.

Post-run publication-check corrections use `verification_revision.py` to bind
the revised verifier source separately. The original experiment snapshots,
outcomes and failed verification logs remain intact. Only the two named
publication-contract modules and post-run archive verifier can differ; runtime
drivers, inspectors, callers and compiler/kernel inputs must still match their
original hashes. The validator rejects altered runtime inputs, damaged original
snapshots and unrecorded verifier edits. These corrections need no repeated VM
execution when all actual experiment inputs remain unchanged.

Run publication after all raw qualification producers have finished, passing a
namespace archive before its corresponding full archive:

```console
scripts/run_bounded.py -- python3 linux-kernel/tools/archive-publication/archive_publication.py publish \
  --payload /path/to/the/pinned/busybox \
  --archive linux-kernel/labs/qemu/x86_64/results/tk281/namespace-consumer \
  --archive linux-kernel/labs/qemu/x86_64/results/tk281/final
```

The publish command accepts only the BusyBox digest declared by the current lab.
It refuses an already projected archive. Original VM scratch artifacts remain
untouched. No third-party binary is imported or assigned a first-party license.

On a checkout without the external blob cache, obtain the exact
[pinned qualification dependency](../../knowledge/x86_64/static-busybox-launch.md)
and hydrate each published archive before running its publication contracts:

```console
scripts/run_bounded.py -- python3 linux-kernel/tools/archive-publication/archive_publication.py hydrate \
  --payload /path/to/the/pinned/busybox \
  --archive linux-kernel/labs/qemu/x86_64/results/tk281/namespace-consumer \
  --archive linux-kernel/labs/qemu/x86_64/results/tk281/final
scripts/run_bounded.py -- python3 -m unittest discover \
  -s linux-kernel/labs/qemu/x86_64 -p 'test_*.py' -v
```

Hydration verifies the supplied dependency, original metadata, projection,
reconstructed artifact hashes and cache destinations. It performs no download
or execution of the dependency. A missing or mismatched input remains a failed
precondition; no artifact is skipped. Historical projected archives use their
recorded external input identity.

The publication tool's own tests cover complete byte recovery after cache loss,
duplicate inputs and repeated embedded payloads, preservation of producer
records, corrupted raw evidence, incorrect dependencies, altered recipes and
invalid cache destinations:

```console
scripts/run_bounded.py -- python3 -m unittest discover \
  -s linux-kernel/tools/archive-publication -p 'test_*.py' -v
```
