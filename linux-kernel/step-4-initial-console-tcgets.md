# Step 4 initial console `TCGETS`

## Outcome

The Topal kernel now implements Linux x86-64 `ioctl(TCGETS)` for every live
initial console description. It publishes the native 36-byte `termios` with
raw input, output, and local flags, `B38400|CS8|CREAD|CLOCAL`, line discipline
zero, and only `VMIN=1` among the 19 control bytes. These attributes describe
the current exact-fill serial path without fabricating canonical processing,
echo, signals, editing, or output conversion.

Descriptor lookup precedes request decoding, which precedes pointer access.
Invalid or closed slots return `-EBADF`; unsupported live-console requests
return `-ENOTTY`; invalid accepted-request candidates return `-EFAULT`. A valid
query requires one complete overflow-free writable extent, privately stages
the full value, publishes it under page-fault recovery, and retains no pointer.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-TCGETS-001` adds no core-language element. Language,
semantic, artifact, and provider-object tests cover syscall 16, request
`0x5401`, all 36 bytes, four ring-3 calls, input/output equality, exact errors,
three-stage precedence, mapping bounds, private staging, exact copy length,
fault recovery, and no retained pointer. AArch64/RISC-V pressure testing keeps
the portable terminal object and ordering while selecting syscall 29.

The checked root contains 144 fatal edges, a 317-event semantic trace, 3,090
relocations, and 2,706 UART calls. The provider plan retains 84 operations and
160 entropy bytes. Artifact `/105`, provider/provider-object `/97`, root `/98`,
boot adapter `/65`, and gate `/107` identify this contract.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 83
ordered observations. `TOPAL_KERNEL_USER_TCGETS_OK` is reachable only after
both complete structures, live unknown-request `ENOTTY`, and closed-slot
`EBADF` return exactly. Recorded hashes are:

- linked kernel: `b7500d5eb32a913cf3edf4a80adec1ffdebe6382fb1a46d57381036a1b5e1f53`;
- boot image: `b5e307e3f6f4081c7471ee8c872a0971e2e1fbf5e6f5a8dae7240d28a9123dc8`;
- boot resource: `7ad699f1980595eca9b25adf7773a5898763d1dc6ad57ee09ff3c23f0971fa27`;
- generated image: `c2525a7051d401511aa3d94f709798eb510313c9aff47b48d9ed2d6bde93771b`;
- supplied image: `0d20097302f253d1581e55739171af96429572e4713df0f42cac125788f3c132`;
- artifact provenance: `29bc821fcea134282a082ff098b439484c4754f3ca18a21703007dd2fdccc899`;
- boot provenance: `a6b6d8a7cc23a0900d93f4557c724a1a862b5ea996487d6cd19953243398ae88`;
- serial output: `be19f4d05a8501813b870a3234b420779b66d181c34b3dd9ce72c6237ed56926`;
- evidence: `4a824ed97738ce362abd080073df6a68fe5bd0dba2c9ec7a0225cb40c6d558d4`.

## Risk and remaining boundary

Risk is high because ioctl classification, native aggregate layout, reported
terminal behavior, error precedence, and fault handling are externally
observable. Review covered every byte, exact constants, lookup/request/pointer
ordering, bounds, staging, recovery balance, description equality, unknown and
closed requests, architecture separation, structural ordering, executable-page
capacity, and duplicate physical execution.

The profile remains one immutable raw serial terminal. It does not implement
`TCSETS*`, `termios2`, window size, sessions, process groups, job control,
canonical input, echo, signal generation, flow control, ptys, polling,
dynamically opened ttys, or a general ioctl registry.
