# Bounded static ELF post-random `brk(0)` query

## Decision

After the first successful random request, the pinned static executable issues
another Linux `brk(0)`. The bounded process profile shall answer this read-only
query with the current byte-granular program break established by the earlier
growth, `0x005f7dc0`. The mapped anonymous reserve, current-break state, random
service, and every other process resource remain unchanged.

The query is admitted only while the exact initial startup sequence is live,
the earlier query and growth are complete, and the return continuation lies in
the sealed executable mapping. A nonzero request continues to use the existing
bounded growth contract; this refinement adds no new mutation. Terminal
teardown remains responsible for revoking the heap leaf, resetting break state,
and returning the zeroed reserve.

The clean-room x86-64 payload issues `brk(0)` after the repeated-random witness,
compares the complete result with `0x005f7dc0`, and only then invokes terminal
dispatch. Source names a post-random current-break query service, not the
address, syscall number, registers, page-table state, or instruction sequence.

This abstraction is architecture-neutral: every Linux ABI provider derives
the reported current break from its private process state, while target-specific
entry and return mechanics remain below the systems boundary. The refinement
changes no core-language rule.

## Alternatives rejected

Returning the original rounded image end was rejected because the live break
has already grown. Treating the query as another growth was rejected because a
zero request is observational. Returning `-ENOSYS` or terminating the process
would break the observed static userspace startup ABI. Introducing general
heap growth, shrink, VMA collections, accounting, or concurrency was rejected
as unrelated to this read-only boundary.
