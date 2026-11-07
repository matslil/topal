# Step 4 initial clear-child-TID word

## Outcome

The initial thread now implements bounded Linux-compatible
`set_tid_address`. A null candidate disables registration; a nonnull candidate
must be a complete writable four-byte user extent. Dispatch retains it as
private thread state without dereference and returns TID one.

The constructed application initializes and registers a word in its RW/NX data
mapping and checks the return. Terminal `exit_group(0)` writes zero to that
word under fault recovery, clears the private candidate, and only then marks
the user context terminal. Data teardown rejects retained state. No core-
language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-CLEAR-TID-001` requires live-context and executable-
return validation, complete writable-candidate checking, TID-one return,
fault-contained terminal zeroing, private-state clearing, and terminal-before-
teardown ordering. Structural tests prove registration and result checking,
private state store/load, terminal recovery installation, the four-byte zero,
state clear, and teardown rejection.

Artifact schema `/60`, provider `/57`, provider object `/57`, artifact root
`/58`, boot adapter `/20`, and gate `/62` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Terminal completion and later mapping
teardown are reachable only after registration and the generated terminal
clear sequence succeed. The evidence records:

- linked kernel: `7bb33328e7caefea0aae6eef116d3cfa1422ade7ed93df09094cefb952577c7c`;
- boot image: `d436c4ea76d2ac7e9f78d27532650bf86550035c1c8aa68c0b6b16e316a74021`;
- boot resource: `7cb9d0b63596f4b6943c495b0b657bc7782ddae097a53ad4b92b0d31b16eabd0`;
- generated default image: `f133fa4536fb5ac92211209e78a84ed4122dde7b7a1f7b2cd5d58a0d4c8c5385`;
- supplied image: `7f8325004337d24fdafa8dfec5e6f7353fb19dd36f4bcbe23b3d52bdc06e4649`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `d718c30b6772dbc4dae3953b882d18f8a5948ca75c5abdbc0f678d5e652a13dc`;
- boot provenance: `270f1ef3c8960ff9c5393cb1b729c7a026d4ada3f432233ee6c4bc45c8c54dd6`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `05db033f2f8ac61f8b23e7a4bff9b57224237a2c8cecc82e73678df8422eb579`.

## Risk and review

Risk is high because this retains a user candidate across syscalls and writes
it during terminal lifecycle transition. Review covered complete extent and
overflow validation, null disablement, no registration-time dereference,
fault recovery identity, four-byte store size, clear-before-terminal ordering,
private-state clearing, teardown rejection, structural inspection, full
workspace validation, and two independent physical runs.

## Remaining boundary

There is one thread and no possible waiter, so no futex wake occurs. This
increment has no clone, multiple threads, futex queues, robust lists, shared
synchronization, signals, exec, or general task lifecycle service.
