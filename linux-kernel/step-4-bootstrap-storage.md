# Step 4 bounded bootstrap storage

## Outcome

This is the second implementation increment of Step 4. The checked kernel root
now declares exactly one artifact-provided bounded bootstrap pool:

```topal
bootstrap-storage is lang systems bounded-bootstrap-storage (
  capacity-bytes is 65536,
  alignment-bytes is 4096
)
```

The declaration is retained in the checked systems program and provisioned in
the abstract trace before bootstrap entry. Missing, duplicate, open,
non-literal, zero-sized, non-power-of-two, and over-aligned declarations fail
before lowering.

The architecture-independent reference model provides deterministic aligned
monotonic allocation. Requests carry a byte count, alignment, and semantic
placement and return either one affine region or the sealed `invalid-request`
or `exhausted` code. A release consumes its execution-bound region but does not
make its bytes reusable. Bootstrap completion requires every region to have
been released and then closes the entire pool for reclamation.

## Abstraction boundary

The model offset is useful for deterministic capacity and alignment evidence;
it is not a physical, virtual, DMA, or device address. Source does not select a
section, linker directive, register, or page-table format. This leaves an
x86-64 provider free to choose concrete image placement while AArch64 and
RISC-V can refine the same bounded ownership contract through their own boot
and translation mechanisms.

No source allocation operation, LLVM lowering, backing object, boot image, or
QEMU result is introduced here. The target therefore remains model-only and
continues to fail before publication. The next increment is the x86-64
provider/placement plan and generated entry/runtime boundary; it must remain
non-executable until structural inspection and physical boot evidence pass.

## Validation

- all 53 `topal-semantics` library tests pass;
- all six focused systems source-checker tests pass;
- the four core-language coverage and traceability tests pass;
- both hosted-feature and model-only-target compiler CLI isolation tests pass;
  and
- strict Clippy passes for `topal-semantics` and `topal-language`, with
  repository formatting checks clean.

The full `topal-language` library still has the unrelated baseline failure in
`models_direct_bounded_int_iterate_collection` recorded by the preceding Step
4 PR. This increment does not alter that compiler-model path or its test.

## Risk and review

Risk is high because bootstrap storage becomes an authority and lifetime root
for later kernel memory management. The implementation uses a closed artifact
shape, checked bounds, checked arithmetic, execution provenance, affine region
consumption, explicit phase completion, and no host allocation behavior. The
change was self-reviewed against `TOPAL-SYSTEMS-VOCABULARY-001`,
`TOPAL-SYSTEMS-AUTHORITY-001`, `TOPAL-SYSTEMS-STORAGE-001`,
`TOPAL-SYSTEMS-ARTIFACT-001`, and `TOPAL-SYSTEMS-QUALIFY-001`.
