# Stage 3: ordinary metadata and named-inode lifetimes

`TK-DEC-283`, `TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-INODE-MODE-STATE-001`,
`TOPAL-SYSTEMS-NAMESPACE-TRANSACTION-001`.

The [ordinary C program](../../../../labs/qemu/x86_64/user_extent_metadata_cases.c)
executes named creation, positional write, descriptor truncation, shared modes,
hard link, rename, unlink, truncating reopen and open-unlinked lifetime through
normal syscall arguments. It checks public identities, modes, link counts,
sizes and positions. Four iterations also supply five ordinary refusal cases
each: unreadable chmod/rename names, invalid fchmod fd, existing hard-link
destination and missing unlink target. No kernel-private state is supplied.

The [read-only observer](../../../../labs/qemu/x86_64/user_extent_metadata_observer.py)
joins every mutating operation to its actual clock return after the verified
caller cleanup block. It compares the complete 36 KiB filesystem and 8 KiB
descriptor table against the [independent byte model](../../metadata_model.py),
preserving all unlisted bytes. It checks stat output and adjacent user bytes,
protected process context and complete final name/inode/page reclamation.
The retained identity counter and parent timestamps follow their explicit
lifetime rules. Numeric model images do not grant kernel owners or loans.

The [native run](native/metadata/run.json) and
[independent saved-record replay](native-readback.json) establish:

| Evidence | Count |
| --- | ---: |
| Ordinary syscalls | 136 |
| Complete filesystem/table transitions | 124 |
| Complete stat outputs | 44 |
| Actual admitted clock returns | 44 |
| Private-state-preserving refusals | 20 |
| Cleanup activations / observed zero bytes | 146 / 5208 |

The descriptor shrink removes two nonzero payload bytes and checks the entire
4094-byte page tail. Positional writes preserve description positions. Two
independent descriptions retain shared identity and mode through alias changes,
truncation and unlink; closing the final description reclaims all released
inode/page/name state. Stat queries preserve private state and match all 144
output bytes. Clock seconds come from the observed caller result, rather than
being inferred from stored timestamps.

Saved replay independently applies every recorded complete delta, reconstructs
all stat records, binds immutable path/payload bytes to the loaded source text,
checks the ordinary syscall adapter, and verifies actual compiler stack-usage
records against the unchanged 2048-byte user reserve. It is not another VM run,
cryptographic authentication or proof of actual kernel memory ownership.

The fresh run observes **12/13 cleanup blocks**. It adds the previously
unobserved mode, truncating-open, unlink, link, rename and descriptor-truncation
callers; ordinary write was reached by the preceding storage cohort. The
[exact matching-kernel comparison and union](artifact-comparison.json) therefore
cover **13/13 selected cleanup blocks**. This is block reach and boundary-state
evidence, not every branch, length, failure or exception path.
Helper first reach is **76/91 fresh** and **83/91 joined**. The
[conditional index](caller-access-index.json) remains **90/91 helper sites and
325 user accesses**; the named-root fallback outside the graph still lacks an
interval. All three fallback obligations retain their independent gates.

Seventeen core objects/images, thirteen audits and four other metadata artifacts
match the preceding checkpoint byte-for-byte. The kernel remains
`47b785aca3794d171d508a3fdbb616057a613016c1783ca9ec95236dbba2199f`.
The call-disassembly listing differs only in its checked second-line diagnostic
filename. Callback fields match except the separately versioned baseline
provenance digest. Actual baseline source/module/object inputs are equal;
retained intermediate text/hash differences remain recorded without an
equivalence claim. This increment runs no new GNU decoder and changes no kernel
behavior, source-controller bounds, core language meaning or production selection.

All **141 models/readers**, [fourteen offline record controls](controls/controls.json),
two [fixture tests](native/fixtures/run.json), and
[seven affected checks](checks/commands.json) pass. The new metadata cohort has
a finite 600-second debugger deadline and finishes its full test in about
291 seconds. It retains all cleanup observations, the bounded wrapper, zero
swap and zero memory-event counters, guest isolation and host protections.
The earlier 128-call cohort passed before descriptor truncation was added;
its [record](history/pre-ftruncate-run.json) is historical, not the current
seven input versions. A readback control exposed an unlisted-clock lookup
raising KeyError; explicit membership now rejects it with an assertion.
Initial byte-comparison assumptions also required separating diagnostic and
baseline provenance identities. These are validation findings, not reported
model/provider safeguard notices.

The [source record](source-identity.json) binds **172 selected inputs**, seven
local and 165 inherited. Both final native/build commands retain the same seven
changed input versions without changes during execution. The
[83 artifact identities](artifact-identity.json) include explicit current and
preceding provenance/listing records and actual baseline inputs. First-party
source ELF and fixture bytes remain separate from the immutable external BusyBox
input, which is not copied here.

[Frozen replay](replay.json) restores selected sources/artifacts and exactly
reproduces the source join, thirteen reports, all 141 models and saved complete
metadata/stat/clock/source-image readback:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-source-metadata/replay-source-metadata.py linux-kernel/.cache/x86_64/source-metadata-replay-new
```

Use a new output directory. Replay executes no ELF, compiler, GNU decoder, GDB
or VM. [Archive validation](validate-archive.py) checks selected current-source,
snapshot, artifact, report, native, control, replay and link bindings.
[Review](review.json) records high memory/lifetime/evidence risk and single-agent
authority/source/full-diff review with separate model/record interpretation;
no independent human security review is claimed.

Actual owners/nonaliasing, continuous stable lifetime, complete helper/entry/
callee/return/fatal/exception and whole-syscall effects, unrun native lengths and
failure paths, remaining caller/fallback coverage, canonical clean production
reconstruction and full Linux distribution/container/VM-host acceptance remain
required. Stage 3, initial Step 4 and the original kernel goal remain unfinished.
