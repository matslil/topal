# Stage 1 model qualification

The [record](qualification.json) retains 49 exact source inputs, the runner and
five complete command logs. Each input/log is losslessly compressed with both
stored-byte and uncompressed-byte SHA-256 identities. This stage changes design,
requirements, specification and the independent model. It changes no production
Rust, LLVM or Topal kernel source and claims no new native behavior.

[Independent readback](independent-readback.json) checks the retained identities,
production source equality to the recorded preceding commit, complete test
outcomes and isolated archived-model replay. Its reader and replay log are
retained. The failed initial harness setup is recorded separately in
[its failure scope](../stage-1-harness-path-failure/failure.json); it stopped
before the runner/input archival step.

Run the model from a checkout containing this stage:

```sh
scripts/run_bounded.py -- python3 -m unittest discover -s linux-kernel/tools/user-memory -v
```

For archived replay, find `model.py` and `test_model.py` in `sources` and
`snapshots`, verify the stored and uncompressed hashes, and decompress them into
a fresh ignored directory beneath `linux-kernel/.cache`. Run unittest discovery
there through the unchanged resource wrapper. No private VM cache, guest binary
or original absolute source path is needed. The retained readback program does
this exact replay as well as input/log checks; it assumes the qualified source
inputs in the containing checkout still match the record.

The retained qualification runner refuses to overwrite its output. A repeat may
decompress it into ignored scratch and change only its output-directory literal
to another empty ignored destination. Preserve and compare the original inputs
and command arguments; report the destination-only change explicitly.

[Supporting tools](supporting-tools.json) retains the primary-research auditor.
Reproducing that audit requires the exact external Linux archive and expanded
files named in the lab manifest. Those upstream bytes remain outside Git; the
committed research record contains hashes, names and interface facts only.
