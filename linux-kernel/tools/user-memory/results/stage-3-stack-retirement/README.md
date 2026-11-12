# Stage 3: selected private stack retirement

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The experimental owned selection now explicitly erases the clock's twelve
popped register words, its returned RTC word and each direct caller's returned
clock word. Random extraction likewise erases its returned word and four popped
caller words before restoring the suspended stack. The existing extractor's
128-byte scratch retirement remains separately checked. Predecessor selections
emit no new cleanup instructions.

| Selected layout | Retired bytes | Maximum call-tree stack |
| --- | ---: | ---: |
| Clock | 96 saved + 8 RTC return + 8 caller return | 112 |
| Reusable random extraction | 32 saved + 8 return + existing 128 scratch | 168 |

The [closed reader](../../audit_stack_retirement.py) checks thirteen blocks and
27 qword immediate-zero stores, including all nine direct process calls to the
clock wrapper. Exact negative offsets keep each store within its previously
used, already-popped slots. MOV preserves all GPRs, RSP and flags. Every block
has a closed predecessor chain through its continuation. The
[file-input reader](../../audit_file_inputs.py) retains the complete clock/RTC
normal-return layout and effects; the
[random reader](../../audit_random_outputs.py) retains the switched-stack,
extractor and output-loop contracts. These remain conditional on actual private
owners, nonaliasing and stable access lifetime; numeric stack bounds do not
construct authority.

The [read-only observer](../../../../labs/qemu/x86_64/user_extent_stack_observer.py)
checks every reached cleanup activation: all sixteen GPRs, RSP and flags agree
at its boundaries, the exact retired bytes are zero, and adjacent words remain
unchanged. Adjacent-word reads belong to the debugger, not to the kernel's
write footprint. It records unreached blocks explicitly.

[Saved native readback](native-readback.json) binds the actual observations,
observer programs and audit/kernel identities:

| Cohort | Cleanup blocks reached | Completed activations | Zero bytes observed |
| --- | ---: | ---: | ---: |
| source | 2 | 14 | 280 |
| storage | 7 | 74 | 2520 |
| pinned | 0 | 0 | 0 |

The fresh union reaches **7/13 cleanup blocks** and
**83/91 helper sites**. No preceding kernel's native coverage is
reused. Six clock caller blocks remain unobserved; their exact offsets are
listed in the saved native readback. Static closure and actual
boundary observations do not establish continuous lifetime, actual owners,
unreached paths or exception/terminal cleanup.

[Complete emission comparison](emission-comparison.json.gz) reconstructs all
232003 process bytes from the preceding 231679-byte function using exactly
thirteen cleanup insertions (324 bytes) and 4262 derived relative-displacement
updates. Eighty-five updates lie outside the selected graph; they use a fresh
GNU linear decode of the complete preceding byte span. The 34192-row linear
listing contributes 3160 nonoverlapping instructions beyond the selected graph;
it grants no extra reachability or ownership claim. All other 162 provider
functions and fourteen component objects, including predecessor provider and
gateway, remain byte-identical. Shifted relocation identities also agree.

Fresh [GNU decoding](gnu-identity.json) independently agrees with all 31051
selected instruction lengths and 36977 graph edges.
[Operand readback](gnu-operands.json) checks all 27 exact zero-store operands
and their complete predecessor chains. The
[sixteen offline ELF controls](controls/controls.json) reject a wrong slot in
every block, nonzero immediate, wrong base and flag-changing opcode. Modified
first-party ELF images are parsed only, never executed, booted or loaded into
a VM. All **132 model/reader tests**, three VM regressions and two fixture tests
pass, together with [six affected checks](checks/commands.json).

The first long ordinary-buffer run hit its 300-second debugger deadline after
51 of the preceding passing workload's 87 source-case markers. A 420-second
attempt reached 84 markers before its deadline. Both the
[300-second run](history/deadline-300/run.json) and
[420-second run](history/deadline-420/run.json) remain failed, with their original
logs and resource counters. The long
ordinary-buffer cohort with the additional stack observer now has a finite
480-second deadline. Other cohorts retain their previous deadlines. No cleanup
observations were dropped; memory/swap limits, guest boundaries and host
protections remain. No model/provider safeguard notice occurred. Earlier Rust
borrowing, report tuple/list equality, omitted clock-caller and incomplete
comparison-inventory findings are development failures, not safety notices.

The [source record](source-identity.json) binds **168 selected inputs**,
**17 local** and the remainder inherited under exact hashes.
All four final native/build commands use the same **17 changed input versions**
without changes during execution. [Artifact identities](artifact-identity.json)
retain the changed kernel/reports and first-party inputs, with explicit preceding
comparison inputs. Pinned BusyBox retains its exact external digest; its ELF and
boot resource are not copied here.

The [fresh conditional index](caller-access-index.json) remains **90/91 helper
sites and 325 user-access instructions**. The named-root fallback outside the
listed graph still lacks an interval. All three fallback obligations remain
separate. This increment addresses the selected normal-return clock/random
saved-word gap, not every enclosing caller or syscall's retirement.

[Frozen replay](replay.json) restores selected sources/artifacts and exactly
reproduces the source join, thirteen reports, 132 models, GNU operand readback,
complete emission comparison and saved native readback:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-stack-retirement/replay-stack-retirement.py linux-kernel/.cache/x86_64/stack-retirement-replay-new
```

Use a new output directory. Replay executes no ELF, compiler, GNU decoder, GDB
or VM and is not a complete clean producer.
[Archive validation](validate-archive.py) checks selected current-source,
snapshot, artifact, report, native, replay and link identities.
[Review](review.json) records high memory/lifetime/evidence risk and single-agent
authority/source/full-diff review with separate decoder/model/record checks;
no independent human security review is claimed.

Actual owners/nonaliasing, continuous lifetime, complete helper/entry/callee/
return/fatal/exception and whole-syscall effects, unrun native paths and random
lengths, remaining caller/fallback coverage, canonical production selection and
clean reconstruction remain required. Stage 3, initial Step 4 and the original
Linux-compatible kernel goal remain unfinished.
