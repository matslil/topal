# Stage 3 source-built user-buffer checkpoint

This selected experiment exercises `TOPAL-SYSTEMS-USER-EXTENT-001` through a
source-compiled C function and a normal backend syscall adapter. The linked
component is included while constructing a fresh first-party ABI witness;
no completed application binary is patched. The source variant checks actual
delivered command bytes instead of assuming padding outside a read's result.

[The native run](native-run.json) passes four source-function invocations and
56 ordinary user syscalls. [The runtime observations](guest/observations.json)
check actual arguments/results, preserved System V callee registers and stack
alignment, restored original heap bytes, and whole-process heap, token, command
and thread-registration retirement. The source cases cover successful `uname`
output across page/directory boundaries; refused full output without a partial
write; unmapped, wrapping, read-only and supervisor candidates; console
zero-count/error priority; and heap growth/restoration.

[Component readback](source-component-readback.json) independently parses the
program headers and checks that all 895 linked code bytes appear unchanged in
the final application's executable, read-only load. It checks the actual ABI
adapter and the conservative 528-byte source-call stack cost. This is a user
stack cost, separate from the existing kernel caller reserve.

[Kernel readback](unchanged-kernel-readback.json) verifies seventeen exact
object/image comparisons with the preceding application checkpoint, including
the complete kernel ELF and boot image. The 7606-byte serial transcript exactly
matches the earlier clean ABI run. [The committed-generator record](committed-generator.json)
rebuilds the default fixture from the exact `81ae694` source closure. Both
build-only tests pass, including default-constructor byte preservation.
The [five post-checks](post-validation.json) pass formatting, included-file
formatting, locked workspace/all-target compilation, source size and whitespace.
No full-workspace test execution is claimed.

Selected native peak memory was 348213248 bytes, with no limit/OOM/kill events
or validation swap under the unchanged wrapper and host reserve. The earlier
readback timeout keeps its original failed status: the old prelude expected
zero padding beyond a short read before reaching the source function.
Only the source variant's generated assertions changed; default fixture bytes
and the kernel/boot images remain unchanged.
An earlier passing run is retained before adding the explicit physical
command-byte preservation/erasure checks. The current run includes those checks.

[Source identity](source-identity.json) freezes seven changed executable/test
inputs. It is not a complete producer-input census or clean-checkout execution
claim. Binary artifacts remain in ignored state; [digests](artifacts.json)
and first-party logs/metadata are retained here. The local Git attribute
preserves verbatim log spacing, with exact bytes covered by the manifest.

Reproduce with [the explicit native test](../../../../labs/qemu/x86_64/README.md#source-built-user-buffer-cases).
Direct startup/stat/directory/command native boundary cases, complete caller
qualification, canonical publication/cold reconstruction and the full original
kernel goal remain unfinished. See [scope](scope.json). This is not completed
Stage 3 and does not activate a new production profile.
