# Step 4 initial thread signal mask

## Outcome

The initial thread now implements bounded Linux-compatible
`rt_sigprocmask`. One private native 64-bit word starts empty and supports
query, block, unblock, and replacement. Supplied sets have the `SIGKILL` and
`SIGSTOP` bits removed before installation, and optional output receives the
word from before the update.

The constructed application replaces the mask with an ordinary signal plus
both unblockable signals, proves the returned empty word, queries and proves
that only the ordinary signal remains, unblocks it while proving the prior
word, and queries the final empty word. No core-language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-SIGNAL-MASK-001` requires live-context and
executable-return validation, exact native set size, complete readable input
and writable output extents, fault-contained staging and publication, all
three update operations, unblockable-bit removal, update-before-output
ordering, and terminal state clearing. Structural tests prove the four
application calls and values, dispatch and size checks, both mapping ranges,
both transfer directions, update operations, private state, output ordering,
terminal clearing, and teardown rejection.

Artifact schema `/63`, provider `/60`, provider object `/60`, artifact root
`/61`, boot adapter `/23`, and gate `/65` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Signal-mask replacement, query,
unblock, and final query all complete before terminal state clearing. The
evidence records:

- linked kernel: `b9e9c6b6c0e52755d3454f2122d57e8e0f79710ba675828711d9b430ee941e3b`;
- boot image: `7d005afadf12d59c2616cb1575974a9bbe4b2b7a91ab01ffd3852f173298348b`;
- boot resource: `018c99c68dd27362faf2935561cfaa7f870ad67da97bd360f15b7cca6be1192f`;
- generated default image: `330cb07ec001bb0981d0f1b7d11f0a308082f1534fc714b5f3e4fb37c6c6bb01`;
- supplied image: `fb4a838390eba7db9a4d6d468857c03b8272835119a7d24adf1285e06d58f640`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `1c9aaf245b011efe00229cb53454642d79acba3dd52904ce6aacd9cfacc9e134`;
- boot provenance: `dc2b4979a7a1187510f1fbf5bc1a4057494c529afe8041b72415c039f197bd76`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `917b03f1ec91cf1ac30c5f56bf428b5fe806f057b7f7deb62926025fc0da2ef5`.

## Risk and review

Risk is high because this introduces mutable thread state, two user pointers,
Linux-specific update ordering, and future signal-subsystem coupling. Review
covered exact native size, null-input query behavior, all update operations,
unblockable-bit removal, complete range validation, direction-specific fault
containment, commit-before-output behavior, terminal clearing, teardown
rejection, structural inspection, full workspace validation, and two
independent physical runs.

## Remaining boundary

There is no signal generation, pending queue, action registration, delivery,
alternate stack, interrupted-syscall restart, `rt_sigreturn`, multiple threads,
or general signal subsystem.
