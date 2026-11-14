# Boot participant arrival protocol

`TK-DEC-293` defines the systems-only CPU-entry lifecycle needed before live
image protection can account for every processor. It distinguishes startup
dispatch, target-local entry, acknowledgment and dispatch completion. A CPU may
acknowledge before its startup call returns; failed or missing dispatch
completion still prevents success. Every expected CPU, including the bootstrap
processor, must acknowledge the exact provider/domain/image/session binding.

The [design](design/boot-participant-arrival.md) traces
`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` through
`TOPAL-SYSTEMS-BOOT-PARTICIPANT-ARRIVAL-001` to the
[portable model](tools/user-memory/boot_participants.py) and
[independent ordering checks](tools/user-memory/test_boot_participants.py).
The standing systems-feature authorization covers this design increment;
core language meaning is unchanged.

Whole entry/root/stack/mailbox/exception owners retain their issuer, allocation
identity, generation, extent, role and pins. Shared entry code/roots require an
explicit sharing declaration; other entry storage stays participant-private.
Supporting trampoline code has its own whole owner. Cancellation returns
ownership only before arming. Thereafter timeout, partial startup, stale entry,
membership change or fault retains/poisons the complete domain. A delayed CPU
cannot revive that session or justify reusing its storage.

## Validation and review

The [receipt](tools/user-memory/results/stage-4-boot-participant-arrival.json)
records **318 user-memory and 83 process-memory tests passing**, including
**22 new arrival tests**, over **345 unchanged watched inputs** in **8.30
seconds**. The three-CPU prefix oracle checks all **40,320** dispatch/entry/
acknowledgment orderings, accepting exactly **560** under immediate successful
dispatch returns. A separate asynchronous-return oracle checks all **720**
two-CPU event orderings, accepting exactly **45**, including target arrival
before startup return. Each invalid prefix retains all owners and enters the
modeled fatal state.

Other controls exercise unsolicited/duplicate/foreign/stale events; each local
entry binding; changed issuer, identity, generation, extent, role, pins and
liveness; equal-looking Boolean fields; ambiguous/incomplete inventory;
unqualified online-capable admission; shared private storage; timeout and faults
at each entry boundary; late results; post-arrival failure; and prohibited
cancellation/rearming. Input records describe provider facts and create no
native or language authority. The model does not prove epoch issuance,
inventory completeness, actual memory ordering or clock/exception behavior.

Both suites execute inside the existing no-swap bounded scope. Peak/limit are
**42,909,696 / 3,296,378,880 bytes**, with zero memory events. Host protections
and the two-CPU VM configuration remain unchanged. No VM, compiler lowering,
native startup or protected activation ran for this stage; existing native
evidence remains historical. Only model code and its documentation change.

Reproduce with a fresh lab output directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_boot_participants.py linux-kernel/.cache/x86_64/NEW-OUTPUT
```

Risk is high because asynchronous entry and resource lifetime interact. A
separate single-agent review pass checked the design against image publication,
ownership, USER admission and fatal requirements, then checked each model
transition and independent event oracle. This is protocol/model evidence;
native security review and execution remain mandatory before selection.

## Remaining work

Implement the explicit systems operation and typed entry through source, checked
IR, semantics and the compiler. Qualify actual x86 startup storage, target-local
capability/mode/exception setup, dispatch/arrival deadlines and ordered native
acknowledgments in the pinned two-CPU VM. Native partial-failure, asynchronous
entry and continuous-lifetime gaps remain acceptance blockers.

Arrival alone cannot authorize effective image protection. All active and
restorable roots, aliases, later writers, firmware/device assumptions and
publication/maintenance obligations still need integration. Step 4 and the full
Linux-interface, distribution, Docker/Podman and VM-host goal remain unfinished.
