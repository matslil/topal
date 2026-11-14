# Production frame-profile qualification

`qualification.json` retains the original frozen-worktree experiment. It records
2,140 input identities and snapshots, 60 commands, full Rust regressions, the
clean ABI fixture and all 51 unchanged pinned BusyBox workloads. The application
observer checks actual syscall context, source CFI, heap bytes/mappings, affine
extent identity, whole release, caller retirement and complete firmware topology.
`independent-readback.json` checks the archived observations and exact debugger
programs against the frozen sources, as well as separate ELF/load/boot parsing.

This qualifies the current frame profile. General virtual memory, broader
process/VFS state, distribution replacement, containers and VM hosting remain
unfinished. The current 519-leaf, 1 GiB physical-access and legacy user-copy/
serialized-process limits remain explicit.

The experiment ran from a frozen worktree. Its committed-source auditor compares
every qualified source blob with a supplied commit; it does not assert that the
original execution began from a clean checkout. From the repository root:

```console
gzip -dc linux-kernel/tools/tail-traversal/results/stage-4-frame-production/committed-source-audit.py.txt.gz > linux-kernel/.cache/audit-frame-inputs.py
scripts/run_bounded.py -- python3 linux-kernel/.cache/audit-frame-inputs.py HEAD
```

Use the commit containing this qualification when later source changes exist.
The four factual progress/traceability files are original verification snapshots,
rather than executable producer inputs; subsequent factual updates are separate
from the frozen experiment.

`production-binaries.json` is an additional lossless storage projection of ten
closed first-party kernel, boot-image and initial-user outputs. Its bytes must
match the original experiment's binary identities. The projection does not
modify the experiment record or import third-party material. External BusyBox
and initramfs files containing it remain outside Git under
[TK-BINARY-001](../../../../design/license-and-provenance.md).

To restore the production images without the original private VM cache, obtain
the exact [pinned BusyBox dependency](../../../../knowledge/x86_64/static-busybox-launch.md),
then run from a checkout whose producer inputs match this qualification:

```console
scripts/run_bounded.py -- python3 linux-kernel/tools/tail-traversal/results/stage-4-frame-production/reconstruct_production.py \
  --kind busybox --busybox /path/to/pinned/busybox \
  --destination linux-kernel/.cache/restored-frame-busybox
scripts/run_bounded.py -- python3 linux-kernel/tools/tail-traversal/results/stage-4-frame-production/reconstruct_production.py \
  --kind abi --destination linux-kernel/.cache/restored-frame-abi
```

Each destination must be new. Reconstruction restores first-party bytes and
provenance, checks every published output, then invokes the normal boot adapter.
The adapter freshly reconstructs the complete source/native binding from the
embedded immutable snapshots, preserving original diagnostic source names without
requiring those original absolute source paths. The rebuilt boot image, resource
and boot provenance must match the original experiment exactly.

The original validation runner is retained as `validation-runner.py.txt.gz`.
To repeat the complete experiment, decompress a copy under `linux-kernel/.cache`
and change only its `out=root/.../stage-4-frame-production` destination to a new
empty results directory. Run that copy through `scripts/run_bounded.py` from the
repository root. Preserve all source checks, commands, limits and dependency
checks. This storage-only derivation avoids overwriting the original cohort;
it is a new experiment, not a replacement for this evidence. Full historical raw
component readback may additionally require regenerating its cache-only binaries
using the retained source/LLVM/native objects and command records.
