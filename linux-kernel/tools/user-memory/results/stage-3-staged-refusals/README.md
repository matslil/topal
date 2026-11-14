# Stage 3 ordinary refusals at staged transfers

`TOPAL-SYSTEMS-USER-EXTENT-001`, `TK-DEC-283`.

The existing source-built application now exercises the preceding
[sixteen staged regions](../stage-3-staged-transfers/README.md) with paired
native admission/transfer observation. No kernel, source-built application,
pinned BusyBox or boot-resource executable bytes change. This closes the
specific missing ordinary-buffer refusal coverage for those selected regions;
it does not complete their ownership/lifetime or whole-syscall qualification.

| Evidence | Result |
| --- | --- |
| [Source run](native-run.json), [readback](readback.json) | 796 syscalls, including 393 `EFAULT` results |
| Selected staged calls | 111 attempts: 91 refusals and 20 completed transfers, 1910 bytes |
| Selected regions | All sixteen have both refused and completed calls |
| [Interface readback](interface-readback.json) | Exact 7200 linked text bytes; 1720-of-2048-byte source stack bound; full stat/directory/command reconstruction |
| [Pinned exit](application/native-run.json), [readback](application/readback.json) | Exact preceding native observations and 6999 console bytes |
| [Post-checks](post-validation.json) | 39 model/reader/observer tests, two fixture build tests and all seven check commands pass |

The source application and its ordinary syscall adapter are unchanged. It
passes unmapped, noncanonical, supervisor-only, read-only-output and partial
page-end candidates, then performs successful transfers across page/directory
boundaries beyond the predecessor's heap bound. The existing native observer
continues checking every syscall sequence, guard bytes, protected controls,
file/signal/command state, register preservation and terminal retirement.
Its ordinary request/error-order rules remain in force.

For this cohort, staged breakpoints are initially disabled and armed only
between entry and return at the source-built syscall adapter, for the selected
syscall family. They cover the entire admission/transfer interval. Heap calls
have no staged breakpoints armed. The helper observation checks actual candidate
and length, carry result, eleven preserved registers, erased return word,
captured arguments and control state. A refused helper additionally leaves its
selected staging or linked-source counterpart bytes unchanged. Its retained
event is joined to the actual source syscall number/arguments, matching pointer,
`EFAULT` result and unchanged phase. The debugger never assigns guest registers,
mappings, ownership records or application bytes.

Successful transfers retain the preceding full payload, exact index/bounds,
GS/context and boundary translation/permission/nonoverlap checks. First reach
is recorded at these sixteen selected calls only. This cohort does not install
the broad census, controller or REP-copy observers and makes no new claims from
them. Other cohorts retain their default complete observer selection. The
matching-kernel historical census union stays 88 of 91; all three fallback
obligations remain open. The [existing caller/access index](../stage-3-staged-transfers/caller-access-index.json)
still covers forty selected local intervals and 249 user-access instructions,
with 51 helper sites lacking such intervals. Other accesses may remain even
after the listed intervals.

Readback compares the existing source observation sections against the preceding
run. It identifies eight independently produced `getrandom` words and their
exact propagation into unused arguments/registers. Only those values and the
private 32-byte random-key field differ across boots: 436 retained leaf values.
The key is the extract input at GS+192..224, within the protected-control
snapshot beginning at GS+152. This cross-boot comparison excludes that field
from equality; each run's native `EFAULT` assertions still compare **all 424
protected bytes**, including the key, without masking. This makes no claim of
cryptographic validation. All other compared observation values agree.

The source fixture's loaded image and boot archive agree exactly with their
preceding bytes. Its diagnostic linked-ELF identity differs while all 7200
loaded text bytes agree; three unused mode-source fields have since been added
as null. Callback reports agree except their revalidated baseline-provenance
identity. Seventeen kernel objects/images, original census/graph/source metadata
and access-audit reports agree. Source console output remains 7606 bytes.
The unchanged pinned cohort's entire native observation object agrees exactly
with its predecessor; its executable artifacts, original metadata and output
also agree. Fresh callback/provenance diagnostic records are retained separately.

[Six offline diagnostic controls](staged-refusal-readback-controls.json) reject
changed protected bytes before/after the random-key field, an altered syscall
result, wrong refusal result/pointer and a false counterpart-preservation claim.
No modified diagnostic is loaded into a VM. New observer tests exercise default
versus selected completion, invalid selections, breakpoint lifetime across heap
and transfer calls, and pointer/error/phase association. These scheduling models
stub the guest and provide no native permission evidence.

Two preceding runs exceeded the unchanged 300-second VM deadline:
[all observers](initial-timeout/run.json) and
[staged observers with the broad first-reach census](second-timeout/run.json).
Their logs, generated debugger scripts and exact changed-input snapshots retain
failed status. Scoped observation then passes: 294.01 seconds for the selected
source test and 105.95 seconds for pinned exit. Native peaks are 441843712 and
428797952 bytes; post-check peak is 55832576 bytes. All recorded bounded commands
have zero validation swap and zero limit/OOM/kill events. Source inputs stay
unchanged during both successful native runs.

A separate [pinned configuration failure](pinned-configuration-failure/run.json)
omitted the required image environment variable and failed before boot. The
corrected invocation passes with the original image. The initial
[strict readback failure](initial-staged-refusal-readback.log) incorrectly
required entropy equality across boots; its original reader and command remain
retained. These are validation/configuration failures, not reported AI safeguard
denials. No new safeguard notice was received or inferred.

The [123-input identity](source-identity.json) stores five changed/new snapshots
and explicitly inherits 118 unchanged inputs. [Artifacts](artifact-identity.json)
and [earlier evidence](inherited-evidence.json) are inherited under exact
identities. The [manifest](manifest.json) separates local files and inherited
Git-equal references. [Python-only replay](replay.json) restores the selected
source/artifact bytes and reproduces the source join and unchanged staged audit:

```sh
rtk python3 linux-kernel/tools/user-memory/results/stage-3-staged-refusals/replay-staged-refusals.py linux-kernel/.cache/staged-refusal-replay
```

Replay does not rebuild or run a kernel, GNU/LLVM, GDB or a VM. Risk remains
high at the memory boundary. Review is single-agent source/authority/full-diff
review, separate native observations, interface reconstruction and offline
controls; no independent human security review is claimed. Complete mapping,
counterpart/table/backing ownership, continuous lifetime, helper/entry/callee/
exception contracts, native mode/stale-state evidence, other user accesses,
canonical activation and exact committed production reconstruction remain
required. Stage 3, initial Step 4 and the original distribution/device/driver/
container/VM-host kernel objective remain incomplete.
