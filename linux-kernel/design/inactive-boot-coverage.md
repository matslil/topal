# Retained boot coverage in inactive banks

`TK-DEC-291` is a systems-only refinement under the standing authorization.
It discharges the non-image reachability part of preparation, while the
[protected-context gates](kernel-image-protection.md) remain mandatory.
No core-language or source syntax change is introduced.

The existing adapter defines seven retained regions: retired handoff storage,
entry tables, GDT, IDT, IDT pointer, MMIO table and bootstrap stack. The actual
boot binding supplies the boot-parameter page and boot-resource interval. Each
region must be identity-mapped, readable/writable, supervisor-only and NX in
both constructed banks, using the entry direct view's cache index zero. Handoff
code has already jumped into the actual executable image; marking retained
handoff storage NX is valid only after that transition. This does not authorize
execution of that code again or release any boot reservation. Descriptors keep
write availability for their declared subsequent updates and hardware effects.
The existing two MMIO windows remain supervisor RW/NX at cache index three;
reachability supplies no device access or DMA authority.

The provider derives fixed extents from the adapter source, rather than copying
numbers into a second native contract. Dynamic extents come from the retained
validated boot binding. Each exact extent is checked through its complete
outward-rounded pages. Bounds, overflow and direct/MMIO-domain distinctions
precede walking. Every table read stays inside the exact whole bank; an absent
entry, unexpected granule, external pointer or invalid effective rights fails
fatally. This check follows complete word-for-word template/owner validation;
it does not replace it, widen USER-table admission or permit arbitrary tables.
The separate empty USER leaf cannot supply missing kernel coverage.

A private journal retains all eleven exact extents, page counts and cache
indices. Construction success records one completed verification; discard
repeats the check and records two. Earlier phases have an empty journal and
later phases retain the completed facts. No role is inferred merely from a
counter. An independent reader reconstructs roles from the actual adapter,
boot journal and captured complete table bytes, walks every covered page in
both banks, and compares the exact native journal.

Alternatives: rechecking the template alone does not expose a missing declared
boot role; sampling endpoints misses holes and ancestor restrictions; making
all non-image mappings executable would defeat the selected protection. The
bounded complete walk retains actual role coverage without activating tables.

Arm and RISC-V providers must use their own supported granules, table formats,
effective permissions and memory attributes. Neither four-KiB pages nor x86
cache-index numbers enter source. Active/root/alias integration, CPU and
asynchronous writer exclusion, effective WP/NXE, device attributes, publication,
continuous lifetime, native failure paths and full Linux acceptance remain
independent qualification requirements.
