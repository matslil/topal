# Step 4 bounded static ELF initial empty robust list

The bounded static process now performs exact
`set_robust_list(0x005f76e0, 24)` registration after its clear-child-TID hook.
The clean-room payload builds the observed self-linked head with futex offset
`-32` and null pending pointer. Registration retains the candidate without
dereference and returns zero. Terminal handling validates all 24 bytes while
the process address space is active, clears registration, performs no entry
walk or futex action, then completes the prior exit-word, FS, and heap teardown.

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-EMPTY-ROBUST-LIST-001` and negative and
structural tests enforce the exact startup profile, 195-byte clean-room probe,
native selector 273, candidate, length, terminal ordering, twenty-one-qword
inverse, and complete rollback. Artifact `/162`, provider `/152`, provider
object `/144`, root `/151`, and gate `/164` identify the contract. The root has
409 operations, 470 semantic events, 8,848 relocations, and 6,226 console-writer
relocations. The systems profile contains 154 rules.

Two independent pinned-QEMU runs reached all 160 observations with identical
evidence SHA-256
`21167ea2820ac88600252f64e8347b5b4d1c6adba94e2e80440b5da307161d66`.
Key digests are linked kernel
`9ec1d752fee5c658d9d6379dfd64cba7d637cce87f2a068033f6ed033a8a33a6`,
boot image `bf53d411a75b7fa54dc7ca1d6b646a59f6ded33b18b90a66d82d1a5353af7e3b`,
boot resource `cf34de8c0dfda8c3ebb1f41a74f7c70d26075d7eae68525e6b1e4e9fdd450fe7`,
static image `5afedafc1a5b44ef9643ca281477bc6e5331923f6bc9dcd1d548968bffd0f7d8`,
artifact provenance `e62a44235b915882516ed5ec1853694a7e19dd23d2329b2088f71d912d59db2a`,
boot provenance `bff5f0361d810b514ab5d307479c24f6d42c06aeed8577c2abc63d903c0ad7db`,
source `e0e2fcfea3e8eab86cb7cb7aef7af7dafc5aefc6778913b04962b28b35889037`,
and serial output `8ddc79e27674a69ebfb800eadeee3dc98a796fc6a0f115d190b34b0954e2ec5b`.

Risk is high because terminal handling reads user memory during a privilege
transition. Review covers complete readable authority, copy-before-CR3 order,
exact empty-head validation, state clearing, absence of futex effects, earlier
teardown preservation, full workspace validation, and duplicate VM execution.
This does not add nonempty recovery, multiple threads, cloning, futex service,
signals, BusyBox redistribution, or a prompt. The next observed startup
dependency is `rseq`, whose `ENOSYS` fallback is known to be acceptable.
