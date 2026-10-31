# Step 4 initial empty robust-list registration

## Outcome

The initial thread now implements bounded Linux-compatible `set_robust_list`
for one native 24-byte head. Registration requires a complete readable user
extent and exact length 24, retains the candidate privately without access,
and returns zero.

The constructed application creates a self-linked head with zero futex offset
and no pending operation on its writable stack. Terminal exit copies the
complete head into private kernel staging under fault recovery, proves the
list empty, clears private registration state, and then performs the existing
clear-child-TID action. No core-language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-ROBUST-EMPTY-001` requires live-context and
executable-return validation, exact native length, complete readable-candidate
checking, no registration-time dereference, terminal complete copy, self-link
and null-pending checks, and state clearing before terminal completion.
Structural tests prove application head construction and registration, exact
dispatch and range checks, private state, the 24-byte terminal loop, both
empty-state checks, state clear, and teardown rejection.

Artifact schema `/61`, provider `/58`, provider object `/58`, artifact root
`/59`, boot adapter `/21`, and gate `/63` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Terminal completion and teardown are
reachable only after complete robust-head validation. The evidence records:

- linked kernel: `5cd65b6e2f723c58c53289c79b575842a92dbb7ac1fe146f8e25dc6bb37e1fce`;
- boot image: `1a2efab3c03970b3b6dff3bbcbfe2fa07e09a17da114679aae661cadfdc30b84`;
- boot resource: `995c6ecb43767f871db3871d0945253ffee543d72bee304245443b9910fab6ec`;
- generated default image: `c14fb8023777709a75cedc80156169cd93bd267163e47152c192915772dc5f4d`;
- supplied image: `ccecaab9e4bef708eee74b0548ebdf11e4e5f67afa3a11cea4f8210c18882cab`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `4b09ecc07833b9d1ef52ba3bf53777ab12d9ef138f11e2967c647af7ba685167`;
- boot provenance: `0b0d84d11c8b644b545235db43f012bb7435ed437cd9b9f151320cf05f71bce0`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `432593f4c5793c61b19425f7e35b513b6ae0501fe1f5a9b832e1215b626a3c7a`.

## Risk and review

Risk is high because a user pointer persists until terminal processing and a
native linked structure is interpreted. Review covered exact size and extent,
no early access, complete fault-contained staging, self-link identity, null
pending state, opaque unused futex offset, copy-before-clear ordering, state
lifetime, teardown rejection, structural inspection, full workspace
validation, and two independent physical runs.

## Remaining boundary

Only an empty head is admitted. There is no list traversal, pending operation,
owner-death mutation, futex wake, multiple threads, clone, shared
synchronization, signals, exec, or general robust-list service.
