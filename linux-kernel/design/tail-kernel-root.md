# Kernel root tail adapter

This continues TK-DEC-282 under standing systems-only authorization. The new
borrowed-byte-region-tail-action-v1 interface selects a distinct root profile.
It keeps the existing owned 64-byte plain bootstrap-region call and supplies the
immutable relative facts (64, 0, 1) to the selected region/three-Nat Boolean source.
No source address, instruction or new core grammar is introduced.

The root callsite must remain a typed live exclusive byte-owner call, without an
atomic alias. Its x86 adapter requires CPL0, masked interrupts, a bounded aligned
root stack and the exact bootstrap storage extent/owner-offset relation. These
checks complement the actual typed caller/linked-object proof; geometry alone
does not create ownership. Existing acyclic interfaces retain their behavior.

Source, checkpoint and completion use the existing private stack sequentially.
The outer invocation controller runs on the kernel caller stack. Its actual
compiled frames, argument/alignment area and the adapter's preserved state must
fit the combined reserve. The adapter retains an independent recovery record
before calling LLVM, constructs the caller-owned immutable facts, and owns the
private/history/output domains. A private normal status plus actual Boolean true
is required before owner resumption.

After return it checks the inner domains, erases the output and retired controller
stack area, restores the original caller, erases popped spill copies and revokes
recovery. A running-source/controller failure recovers the outer stack through
the independent record, preserves stores/issuance, poisons an admitted history,
reclaims all private/output/controller domains and halts without owner resumption.
A pre-entry kernel refusal grants no arbitrary-address cleanup authority. A late
failure after popped spills uses still-restored registers and never reloads erased
saved values. Terminal root finalization erases the remaining scalar history.

Initial VM experiments may link exact source/native components with a checked
typed root and use a test-only boot fixture. They must be labelled unpublished
experiments, retain all exact inputs and actual observations, and must not forge
canonical provenance. Canonical publication remains refused until source/module,
native code/data/stack/caller/ownership/context binding and boot validation are
complete. VM component observations do not establish general allocation, userspace,
container or VM-host acceptance. AArch64/RISC-V require their own target adapters
under the same ownership, value and retirement obligations.

The initial experiment retains the full checked boot/handler shape from the
current toolchain gate. Its optional source slots are unselected except for the
explicit tail slot; its observed-byte guard expects the actual traversal's zero.
It executes the boot setup and clean-room static-ELF ABI fixture before the root loan,
then reaches the normal root continuation and final fatal disposition. This
separate fixture does not replace the canonical kernel source or certify its
other source-library slots or general Linux compatibility.

The target debugger metadata marks private-stack boundaries with GNU CIE `S`.
GDB uses that augmentation to admit discontinuous stack order and displays such
frames as signal trampolines; this target presentation introduces no Topal signal
event. Saved-register and CFA rules still describe the actual bridge at each
instruction. Ordinary bridges retain their existing CIE bytes. See the primary
[GDB DWARF unwinder](https://gnu.googlesource.com/binutils-gdb/+/d01e823438c7dc264d6885fbbfeace4d8955dcb7/gdb/dwarf2/frame.c).
Native tests identify these frames by their actual PC and symbol as well as
checking the recovered caller. The kernel test walks entry, normal retirement
and failure recovery; repeated string instructions are observed at both ends,
with separate whole-domain readback.

The outer frame budget includes the adapter return word (8 bytes), original
callee register saves (48), argument/alignment area (at most 80), and the measured
compiled controller/inner path. Its erased caller area is the 80-byte allowance
plus the compiled controller's owning-stack path, rounded to a word. The private
history (40), result (56), recovery (32) and immutable input (24) retain distinct
domains; none is hidden in the 65536-byte temporary reserve. A pre-admission
counter/generation refusal preserves the prior history; an admitted source or
result failure poisons the advanced history while preserving performed stores.
