# Static-process realtime seconds

Status: systems-only design adopted under the standing authorization;
implemented and qualified (`TK-DEC-243`); see the [Step 4 report](../step-4-reusable-readonly-namespace.md).

Directory-consuming `ls` queries native `time` during setup. A monotonic uptime
value or constant calendar date would not implement that interface. The process
adapter shall observe a platform-qualified realtime source and return signed
Unix-epoch seconds. A nonnull native output receives that same complete value
after writable user validation; a null output performs a pure query. A bad
output returns `EFAULT` without partial publication. Private acquisition and
conversion staging is erased on every return and at terminal reclamation.
This adds no setting, timezone, timer, subsecond or clock-synchronization claim.

The portable resource is a qualified realtime observation, separate from the
monotonic clock and without source access to native ports or calendar fields.
Initial x86-64 realization reads the pinned Q35 RTC under exclusive indexed-I/O
ownership and the existing serialized interrupt-masked syscall context. It
checks update-in-progress, compares complete consecutive samples and validates
calendar fields and representation before converting Gregorian UTC to epoch
seconds. Binary and BCD 24-hour samples are admitted for years 1970 through
2261; malformed, unavailable or persistently incoherent samples cannot produce
a fabricated success. The adapter does not write RTC date/control data.

The QEMU model's index read is not a saved index/NMI-state value; it must not
be treated as one. This backend is qualified for that pinned virtual platform,
not an assertion about every physical CMOS/NMI implementation. AArch64/RISC-V
may obtain realtime from a different qualified clock or firmware resource while
retaining the same epoch observation/copy contract. No core-language change or
source assembly is introduced.

The qualified last whole year precedes this QEMU implementation's signed
nanosecond horizon in 2262. A requested RTC base beyond that horizon is not
evidence of a valid hardware date: a 2399 probe wrapped to a different sample.
The backend must observe actual qualified clock data, rather than substitute
the requested test date. Wider realtime ranges require a suitable provider.

Qualification shall distinguish null and nonnull queries, returned/output
equality, complete user-copy faults and register preservation. Calendar checks
shall cover leap years, non-leap centuries and epoch/date boundaries; physical
cases shall observe declared QEMU RTC dates and complete process rollback.
External `ls` must continue through the same service. Risk is high because a
coherent calendar sample, conversion and native output must agree. Review shall
check exclusive acquisition, field bounds, calendar arithmetic, actual platform
provenance and erased staging. Realtime is not required to inherit monotonic
clock guarantees or accept user-provided timestamps as clock authority.

Primary sources are pinned Linux `kernel/time/time.c` (`time`) and
[QEMU 10.2 RTC implementation](https://raw.githubusercontent.com/qemu/qemu/v10.2.0/hw/rtc/mc146818rtc.c).
