# Owned inactive image-table construction

`TK-DEC-288` is a systems-only refinement under the standing authorization.
It realizes actual inactive table construction after
[backing preparation](kernel-image-backing-preparation.md), preserving core
language semantics and the separate protected-activation gates.

Boot source requests `kernel image prepare tables` for `current-kernel` with
provider-selected page policy. One affine `InactiveKernelImageTables` resource
owns all storage. Its only initial consumer is `kernel image discard tables`;
the exact preparation/discard observations delimit the protocol. No other work
interleaves, and backing-only resources cannot substitute for table resources.

The x86 provider reserves distinct image-sized bootstrap/process banks and a
separately owned one-page low USER leaf. The process bank's reserved high USER
leaf remains within its whole bank; additional kernel subdivision tables remain
outside USER admission. The independent low leaf is zero and referenced only
from the inactive process topology. Its ownership cannot come from a numeric
address or a spare page in another extent.

All whole owners are validated before initialization. Both banks are constructed
from the actual final image's role partition and read back word for word; the
low USER leaf remains zero. The existing draft non-image RW/NX and MMIO template
is descriptive construction only, not qualified active coverage or attributes.
Discard validates the same owners and constructed bytes, erases/verifies all
three extents, then unpins/releases complete owners with issuer history retained.
Failure is fatal/poisoned, with no ordinary partial-success path.

The constructor's own constant tables affect image layout. Fresh complete
reconstruction must converge on the exact image partition/template and storage
used by the emitted constructor, not only a matching page count. Identity is
bound externally to the fresh image, avoiding an embedded self-hash. Public
unbound lowering and active protected selection remain unavailable.

Validation uses ordinary allocator calls, source-built user programs and
read-only native observations. An independent byte oracle checks both complete
banks and the zero low leaf, followed by complete discard/release. Existing
five-page active paths and USER validators stay unchanged. Full root/alias/
writer/mode/publication/lifetime integration and Linux compatibility remain open.
