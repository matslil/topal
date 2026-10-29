# Step 4 increment: bounded static-process missing-directory condition

## Outcome

This increment replaces the successful third condition with exact
`test -d /missing || echo missing\n`. Both profiles consume the same five
ordered command publications, issue one exact absolute metadata request, and
receive `-ENOENT` without metadata publication. The applications consume that
failure and select one exact eight-byte `missing\n` standard-output write.

The provider validates the complete pathname and native output candidate,
leaves the failed-call candidate untouched, retains only a private missing-path
fact, validates the clean stack or pinned BusyBox heap output candidate, stages
all selected bytes before comparison or publication, erases staging, and
records one private false-edge-output fact. The following `pwd` and terminal
rollback require that fact. No provider-owned confirmation, arbitrary
pathname, general shell evaluator, output stream, or general VFS is introduced.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-MISSING-DIRECTORY-CONDITION-001` and
decision `TK-DEC-225` bind the transition. Structural qualification inspects
all five input publications, the full 33-byte third command and zero tail,
exact `AT_FDCWD` selector, `/missing\0` pathname, zero flags, complete
144-byte writable candidate, exact `-ENOENT`, absence of metadata publication,
clean unchanged-candidate proof, exact selected write, clean stack and external
heap candidates, private staging and erasure, the dedicated observation
lifecycle, later `pwd`, terminal clearing, and rollback.

Artifact `/204`, provider `/194`, provider object `/186`, root object `/193`,
boot adapter `/104`, and gate `/204` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and a
2,897-byte continuation with five reads, five clean prompts, root `chdir`, the
negative lookup, selected false-edge output, root output, and terminal exit.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`4bc07a1a0ec18e483794d2babc68dc802c0a8b573adc97f32260912ff5d62599`
and
`9190b990918516b35ea5e1594c236dddfbcec37397960c268ff826810581ccbf`.
They bind boot image
`dc878575332ab52e81ee7ca2c5cb4cfa92ceecf07be2a52ea082ce88127d8759`,
distinguish the clean unchanged-output witness from the pinned BusyBox
application witness, complete every ordered marker, remain running afterward,
and remain serially quiescent. Their serial transcript hashes are
`afc0baa97fcae757039a2a12eaaa4c6cb08a0ecfa9cd55d864c2676e757248a6`
and
`50409f835835fe00dbf543acc09c7bafdba317bb47448fe6aa73435abe125ac7`.

The Python lab contract tests, compiler and language suites, semantic suite,
full Cargo workspace suite, formatting check, and repeated clean plus external
QEMU executions pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because the real application must consume a native negative errno,
select its false control-flow edge, and transfer output while pathname and
candidate authority, address-space state, staging, observation, and teardown
remain bounded. Review combines exact request bytes, unchanged failure output,
selector and result, clean and external mapping profiles, transfer extent,
affine ownership, terminal cleanup, rollback, and physical runtime evidence.

This proves one failed externally observable condition for one absent sealed
root child. It does not provide arbitrary conditions, relative paths,
traversal, successful non-root objects, directory enumeration, file contents,
mutable filesystem objects, multiple processes, containers, or VM-host support.
