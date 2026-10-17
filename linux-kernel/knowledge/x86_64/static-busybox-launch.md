# Pinned static BusyBox launch facts

This record captures independently observed loader and startup facts for the
Step 4 command-prompt target. It describes Debian package
`busybox-static` version `1:1.37.0-6+b9` from the pinned Step 3 image; it does
not import or redistribute the binary.

The installed `/usr/bin/busybox` is 2,024,544 bytes with SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.
It is an ELF64 little-endian x86-64 `ET_EXEC` image with entry `0x4a6580`, 11
program headers, no interpreter, and four ordered `PT_LOAD` segments:

| Role | File interval | Virtual interval | Flags | Alignment |
|---|---:|---:|---|---:|
| headers | `0..0x6e0` | `0x400000..0x4006e0` | R | `0x1000` |
| text | `0x1000..0x18c285` | `0x401000..0x58c285` | R/X | `0x1000` |
| read-only data | `0x18d000..0x1e4040` | `0x58d000..0x5e4040` | R | `0x1000` |
| writable data/BSS | `0x1e4660..0x1edd88` | file bytes at `0x5e5660`, memory through `0x5f61c8` | R/W | `0x1000` |

The page-rounded load span is `0x400000..0x5f7000`, or 503 base pages, and
fits within one aligned 2 MiB x86-64 leaf-table window. The image also carries
`PT_TLS` (`0x40` file bytes, `0x82` memory bytes), non-executable GNU stack,
and GNU RELRO covering the page-rounded `0x5e5000..0x5ed000` interval.

A root `busybox true` observation on Linux 7.2.9 performs, in order,
`execve`, `brk`, `arch_prctl(ARCH_SET_FS)`, `set_tid_address`,
`set_robust_list`, `rseq`, `prlimit64`, `readlinkat`, `getrandom`, further
`brk`, RELRO `mprotect`, `prctl(PR_GET_NAME)`, `getuid`, and `exit_group`.
Earlier differential injection established that `rseq`, `readlinkat`, and
`prctl` may return `ENOSYS` without preventing the selected builtin command.

These facts size and shape the first real application gate. They do not make
this one binary the definition of Linux compatibility.
