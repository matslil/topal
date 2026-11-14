# Calendar data placement and current-kernel regressions

Provider revision **235** places the private Gregorian tables in allocated
read-only ELF data sections instead of the process instruction body. This
implements the existing `TK-DEC-243`,
`TOPAL-SYSTEMS-STATIC-PROCESS-REALTIME-SECONDS-001` and
`TOPAL-SYSTEMS-ARTIFACT-001` contracts. RTC acquisition, admitted years, arithmetic,
native outputs, user validation and cleanup retain their existing behavior.
No language syntax, core semantics or public activation policy changes.

The year object contains **4,672 bytes** for 1970 through 2261; the month object
contains **192 bytes**. Both are data symbols in non-writable, non-executable,
non-TLS ELF sections and initialized `PT_LOAD` segments with flags **4**.
Two existing signed RIP-relative relocation forms bind the consumers to their
objects. Linker flags describe the artifact; hardware permissions remain a
separate obligation.

The [placement reader](../inspect_calendar_placement.py) reconstructs the old
process object with exactly those **4,864 bytes** removed. All retained labels,
**6,237 local fixups**, **24 preceding external bindings** and the **two new
calendar references** account for the complete new body. Every remaining
instruction/literal byte must match that transformation, and the whole new
object must reconstruct the linked function. An independent Gregorian oracle
checks both object and linked table bytes. Four byte-model tests reject
unplanned instruction/data changes, missing fixups, changed labels and incomplete
or mistyped external bindings. These are comparisons of host data, not patched
native applications or a complete control-flow proof.

## Executed validation

The [closed native archive](stage-3-calendar-placement/report.json) and
[frozen review](stage-3-calendar-placement-review.json) retain the actual builds,
observer scripts, commands, logs, source closure and saved-record replay.

| Check | Result and scope |
| --- | --- |
| Compiler/artifact checks | Formatting, locked workspace/all-target checking, three exact provider-object tests, unchanged default ABI bytes and Rust file ceiling pass |
| Host models | **259 user-memory + 82 process-memory = 341** tests pass |
| Ordinary child exec | Four generations and sixteen ordinary rejected exec calls pass independent readback |
| Unchanged pinned BusyBox | All **51 workloads** pass, including namespace/file/mode operations, command execution, prompts, child state and pipe substitution |
| Kernel identity | Child and application builds both produce `d8f937bdb2a4ad17cf4c20bca8c83c691af4fd3204b7316cdec1c713cffaf997` |
| Tested source stability | All **759 watched inputs** match before and after the **6,607.49-second** driver |
| Native resources | Peak **2,068,377,600** of **3,383,169,024 bytes**; zero swap and all recorded memory-event counters zero |
| Frozen replay | Both layouts, child readback, 51 application records, **859 altered-record controls** and 341 models reproduce without another guest execution |

The native archive has **840 entries**, sharing **749** exact source snapshots
and retaining **10** overlays. Its manifest SHA-256 is
`4f1c2497b4eac3b7765ce4e7967083b225c3dfdbed46af266fd0dc337e1718bc`.
Replay peaks at **1,680,486,400 / 3,345,719,296 bytes**, with zero swap and
zero recorded events. The source snapshots are the actual tested inputs;
subsequent descriptive reports are not part of that native source-stability
claim. The existing nonfatal GDB `.debug_names` warning remains retained.

The [driver](../run_calendar_placement.py) selects exact static-object tests and
ordinary guest workloads. It does not run the full compiler suite or historical
host-RWX/debugger-injected-state cases. The interrupted full compiler command
remains incomplete. These results establish no reason for an AI safeguard
interruption.

## Fresh listings and observed mappings

The [companion audit](stage-3-calendar-placement-audit/report.json) and
[review](stage-3-calendar-placement-audit-review.json) preserve fresh LLVM/GNU
listings, actual mapping samples, exact methods and frozen replay. Both listings
contain **221,624 instruction rows** and agree on all **848 selected privileged
controls**. The new selected process graph has **31,210 instructions**; this
does not recreate the predecessor's broader nineteen-root graph or establish a
complete entry set.

One embedded opcode candidate remains: `0f 30` occurs inside the signed rel32
field of `0f 85 0f 30 00 00` at **46,105,563**. Its containing program-header
function is byte-identical to the predecessor. The relocated `CF` calendar byte
at **44,979,984** is outside the executable ELF section. Neither placement nor
decoder agreement proves that alternate entry is impossible.

The fresh 51-workload observations reach **66 of 91** declared extent-helper
sites. All **25 unsampled sites** are listed in the audit. This is helper-entry
coverage for this kernel and corpus, not normal-return or continuous-lifetime
coverage. The older **89/91** combined observation belongs to its predecessor
kernel and is not relabeled as fresh coverage here.

The saved child-exec diagnostic examines **234 before/after event samples** at
the ELF load/calendar endpoints: **3,712 of 3,744 point checks** resolve through
captured tables. Every resolved route is an identity mapping with write bits
set at every level, execute-disable bits clear and supervisor-only access.
The remaining **32 checks** are the sixteen endpoints on both sides of final
retirement event 116 (operation 6): actual CR3 is **46,292,992**, while the
captured context names root **48,377,856**. The actual CR3 hierarchy is absent
from that sample; the reader reports it unresolved rather than substituting
the context root. EFER is not independently sampled by this diagnostic.

Thus the ELF change does **not** establish hardware read-only/NX protection or
runtime immutability. Current bootstrap and static-process builders construct
separate supervisor large-page mappings. Two mixed 2 MiB regions contain
different kernel load roles; protecting them needs an explicit qualified
permission/ownership refinement across the relevant producers, validators and
root transitions. Existing CR0.WP initialization and all user-backing,
ownership, publication and invalidation checks remain required.

The companion archive has **21 entries**; its manifest SHA-256 is
`003fb3d31e452816969dee8de401f0f5275cb83250a0249cd7e866bb978b83a6`.
Its replay repeats mapping interpretation, listing inventory and helper union
from frozen readers and archived data, with no guest or decoder rerun. Recorded
peak/limit are **867,328,000 / 3,406,233,600 bytes**, with zero swap and events.

## Review and remaining work

Risk is high for silent native relocation errors and incorrect memory assumptions.
Review is single-agent self-review against the approved authority, independent
Gregorian/layout comparison, negative models, fresh ordinary execution and
frozen saved-record replay. No independent human security review is claimed.

Complete entries/effects, all caller/error/fatal paths, continuous exclusion,
mapping writers, AP/DMA behavior, full compiler completion and cold production
reconstruction remain open. The owned/process-MM selection stays private.
Complete Linux UAPI, DTS, distribution, Docker/Podman and VM-host acceptance
remain unfinished.

In a fresh checkout, the two retained replay methods can be run in order through
the standard wrapper. The first output directory must not already exist; the
companion intentionally uses that frozen source directory.

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/results/stage-3-calendar-placement/methods/replay.py linux-kernel/tools/user-memory/results/stage-3-calendar-placement linux-kernel/.cache/x86_64/calendar-placement-frozen
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/results/stage-3-calendar-placement-audit/methods/replay.py linux-kernel/tools/user-memory/results/stage-3-calendar-placement-audit
```
