# Calendar data behind the linear decoder discrepancy

The [closed composable archive](stage-3-calendar-data/report.json) and
[frozen replay review](stage-3-calendar-data-review.json) explain the previously
unidentified context of the selected LLVM/GNU discrepancy under
`TOPAL-SYSTEMS-USER-EXTENT-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-REALTIME-SECONDS-001`.

The historical site `0x2bbaa5d` and current site `0x2bbc8f2` (45,861,106) contain
the same calendar record: **38,351 days** from 1970-01-01 to 2075-01-01,
followed by a zero leap-year field. Its sixteen bytes are
`cf950000000000000000000000000000`. The `CF` byte is the start of that day field.
LLVM's linear listing interprets it as `iretd`. GNU's preceding instruction
consumes it in `00 cf`, after the two tools use different recovery lengths for
an earlier unknown/bad row. Four fresh explicit-start disassembler commands,
two tools on each unchanged saved kernel, interpret `CF` as an interrupt return
when instructed to start there. None of these commands executes the bytes.

[GNU documents](https://sourceware.org/binutils/docs/binutils/objdump.html) how
data in code sections can affect disassembly. Both its address options and
[LLVM's explicit-start option](https://llvm.org/docs/CommandGuide/llvm-objdump.html#cmdoption-llvm-objdump-start-address)
select where interpretation starts. Such a listing does not establish a CPU
entry or that execution reaches the selected byte.

The new reader independently reconstructs all **292 year records** for
1970..2261 and all **twelve month records** using Python's Gregorian date
arithmetic. Both functions contain the exact **4,864-byte** payload. In the
current kernel it occupies `[45,859,426, 45,864,290)`. Encoded RIP-relative
references bind the table addresses. The selected year consumer compares 1970
and 2261, rejects outside that range, subtracts 1970, shifts by four and reads
both qwords. Its maximum row offset is 4,656 bytes, inside the 4,672-byte year
table. This recognizes a specific encoded consumer; it is not a full RTC or
month-consumer verification.

All **206,533 instructions** in the existing nineteen-root conditional graph
match the current ELF and avoid the calendar payload. The year-consumer's ten
instructions and nine inner fallthrough edges are present, with no declared
root, callback or edge bypassing its guards. The graph's four existing
entry/callback/code-table-stack/return assumptions remain necessary. These
checks do not establish that the declared entry set is complete.

The current provider's complete **234,325-byte** process function reconstructs
the linked function through **24 relocations**. Test-only byte-origin metadata
binds the selected eight-byte field to `systems_process_realtime.rs:192`, where
the emitter writes the day count; its labels also agree with the independently
identified year and month payloads. Metadata describes a producer, not native
execution or memory authority.

Seven new unit tests exercise corrupt/duplicate/missing tables, century leap
behavior, damaged bounds/index/load/reference/rejection encodings, wrong field
starts, graph-byte mismatch and graph overlap/interior-entry/callback/edge
bypasses. All **255 user-memory** and **82 process-memory** tests pass. Four
additional controls alter actual saved function bytes or a graph row in Python
only: changed calendar data, changed year maximum, redirected reference and an
entry into the data must fail. They never change or execute an ELF file.

The first driver attempt rejected the historical paging archive because it
assumed the newer flat-manifest schema. The corrected reader checks the pinned
legacy `files` member with the same exact file closure and byte/hash obligations.
A subsequent strengthened check failed because it treated callbacks as a list;
the saved graph uses a mapping. The corrected implementation and tests use that
actual shape. Exact failed sources and tool-notice summaries are retained;
original complete stderr and resource counters for these attempts were not
saved. Frozen replay reproduces both original reader errors. An earlier
successful audit without the additional guard-entry check is superseded by
the final strengthened result. No VM or kernel failure occurred.

The 35-entry archive pins five predecessor manifests and records every consumed
packed/raw artifact identity. It reuses **749 exact sources**, adds **seven
overlays**, and restores **756 inputs** for replay. Both case reports, explicit
decoder rows, source reconstruction, conditional graph checks, four controls,
both original errors and all **337 tests** reproduce without invoking another
disassembler, compiler or VM. Its manifest SHA-256 is
`8508c497126423f0c99dbc71e4c1093fbc7bc67ffa21106b2bdca0a4f3e21ec3`.
Qualification peak/limit is **655,634,432 / 3,368,960,000 bytes**; collection/replay
is **720,965,632 / 3,386,380,288 bytes**. Both recorded scopes have zero swap and
zero sampled memory events under the normal resource wrapper.

Risk is high if data identification were mistaken for complete control-flow
exclusion. Single-agent review therefore combines independent calendar values,
actual bytes/consumer encodings, both decoders, complete object-to-linked
reconstruction, conditional graph checks, counterexamples and frozen replay.
There is no independent human security review. The raw inventory disagreement
and candidate remain retained; existing checks and activation gates are unchanged.
These constants remain in an executable ELF section. Hardware NX, runtime
immutability, alternate/exception/AP/DMA entries, complete effects/lifetime,
native coverage and production reconstruction remain unproved. Both case reports
and the archive summary retain `qualification_complete=false`. The kernel, boot image, language design,
production selection and host protections are unchanged. Step 4 and the complete
Linux distribution/container/VM-host goal remain unfinished.
