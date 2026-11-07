# Step 4 bounded static ELF process entry

## Outcome

The qualified x86-64 path now consumes the persistent randomized static ELF
process, enters its sealed ELF entry in CPL3 with the sealed Linux initial
stack, accepts a native `exit_group(0)` syscall, and returns through the exact
kernel continuation. The provider restores CR3, GS/FS and syscall MSRs before
returning one affine completed-process owner.

The clean-room fixture contains only a fourteen-byte terminal probe at the
already validated entry. It is not BusyBox and makes no application-
compatibility claim. A qualification-only inverse clears the private context
and restores the randomized owner, after which the existing initial-random and
construction rollback completes unchanged.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-PROCESS-ENTRY-001` adds no core-language
element. Negative tests reject substituted owner, ABI, terminal result, policy
and marker. Structural tests bind the sealed entry and stack, `IRETQ`, the
clean-room `SYSCALL`, exactly two CR3 switches, terminal syscall selection,
complete context erasure, and normalization of only the hardware-managed
Accessed/Dirty bits set on the entry and stack leaves.

Artifact `/157`, provider `/147`, provider object `/139`, root `/146`, gate
`/159`, boot adapter `/69`, and boot resource `/2` identify this contract. The
149-rule systems profile admits 404 checked-source operations; the provider
plan contains 152; the root contains 1,096 fatal edges, a 465-event semantic
trace, 8,616 typed relocations, 5,995 UART calls, 114 required text symbols,
59 bootstrap-storage references, 395 opaque-storage derivations, and 155
ordered physical observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced an
identical evidence file (`dbbd63a8f45a704f2c3372fbae9a13f56741b712502206539b6883e99c90ea30`)
and all 155 observations in order:

- static image: `94219c5d1ef75c0a0315d1a59c9fd7e951b2f40fb4363af3a1eeef1e385eb9b7`;
- linked kernel: `5a7fdc65310d4dbab2fca508b11ed714184fc54cfaba8ba5c49a4b38a6acf93c`;
- boot image: `acb5c745ce0bce5bf03d4fc94157b6f97b3917eacd31533dc21fc59f5cba88c4`;
- boot resource: `873e764db4facba6782e7fad21477debea8d42ffecbe715c8e5754bc3e129617`;
- artifact provenance: `1d76a85a9979f2dfabf3142d5bd27bb13832be55ca6d0761a1131a34f8c5e579`;
- boot provenance: `eb3eaa2a2a59511813906f9f3e1d31c2fb77a6dd503d4218b71a974cb30e2fe3`;
- serial: `054e7fcc1448fb04f5e08503470eab99ce76b355a89ad9043e6c803537582719`.

The complete workspace suite and the three Python gate tests pass.

## Risk and remaining boundary

Risk is high because this increment crosses privilege and address-space
boundaries and returns through a machine syscall entry. Review covered affine
state, native entry/return, complete privileged-state restoration, hardware
page-status normalization, clean-room provenance, structural evidence,
negative cases, full rollback, and duplicate physical execution.

This increment does not initialize static TLS, accept a nonterminal syscall,
execute BusyBox, start a command prompt, run containers, or expose a
virtualization-host interface. Those remain separate userspace gates.
