# Generated Linux 7.2.9 interface inventory

The `7.2.9/x86_64` directory is a deterministic coverage index generated from
the verified upstream archive and an x86-64 `headers_install` result. It stores
interface identities, paths, hashes, and source locations rather than copies of
Linux headers or implementation source.

Generate the installed headers through the repository resource wrapper:

```console
scripts/run_bounded.py -- make -C /path/to/linux-7.2.9 \
  O=/tmp/topal-linux-build ARCH=x86_64 headers_install \
  INSTALL_HDR_PATH=/tmp/topal-linux-headers
```

Then reproduce the inventory:

```console
python3 linux-kernel/tools/inventory_linux.py \
  --source /path/to/linux-7.2.9 \
  --headers /tmp/topal-linux-headers \
  --archive /path/to/linux-7.2.9.tar.xz \
  --output linux-kernel/inventory/7.2.9/x86_64
```

Pass `--check` to compare a clean regeneration with the committed files. The
tool verifies the source release, x86-64 installed-header boundary, and optional
archive digest before producing output.

See the [coverage report](coverage.md) for counts, interpretation, and the
reconciliation work that source indexing deliberately leaves open.
Entry arrays use one compact JSON object per line so baseline changes remain
reviewable without sacrificing standard JSON parsing.

## Coverage and limitations

The generated sets are exhaustive for their stated source boundaries:

- native `common` and `64` rows in the x86-64 syscall table;
- every installed UAPI header and its content digest;
- every `What:` entry in `Documentation/ABI` with stability and source;
- every native x86-64 vDSO export in its version script, including conditions;
- every direct installed-header `_IO`-family macro definition found by the
  documented lexical extractor; and
- every YAML or legacy text Device Tree binding in the baseline source.

They are not a claim that lexical extraction completely describes semantics.
In particular, ioctl aliases and nonstandard encodings, runtime text formats,
Netlink policies, socket options, BPF commands, structure validity, state
machines, privilege, configuration, and device conditions still require the
reviewed knowledge records and later semantic inventories. File digests ensure
that such work cannot silently ignore an upstream source change.
