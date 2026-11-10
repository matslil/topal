# Initial workspace failure

The [original qualification attempt](qualification.json) is incomplete and
failed. Its four-thread full workspace command returned 101: the compiler target
reported 370 passing tests and one failure in
`frame_gateway_retires_actual_bootstrap_caller_storage_and_preserves_lifetimes`.
The interrupt-negative guest's debugger asserted that IF was clear at its entry
breakpoint, before its intended IF fault injection. The cause is not established.

The resource record has zero OOM, OOM-kill and group-kill events and zero
validation swap. It retains cache-pressure events; this failure is not classified
as an out-of-memory kill. No memory limit or host security control was changed.

[Failure evidence](failure-evidence.json) retains the exact failing gateway
image, debugger program and logs; the successful guarded/native target and
324-call/80-MM-operation extent VM with its 21 negative controls; and four
successful isolated repeats using the exact same gateway image. Those repeats
do not establish the cause or turn this workspace run into a passing result.

The next full workspace attempt uses one test thread and exactly the same 2157
frozen source identities. Its acceptance record is kept separately in
[Stage 2](../stage-2/qualification.json). Source snapshots, command logs and
retained component files have both compressed and uncompressed SHA-256 hashes.
The host C executable that may link external CRT code is excluded; its own
source, native object and rebuild commands are retained. No external BusyBox
executable is committed.
