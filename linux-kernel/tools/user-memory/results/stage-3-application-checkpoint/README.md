# Stage 3 ordinary-application checkpoint

This is selected experimental execution for `TOPAL-SYSTEMS-USER-EXTENT-001`,
not completed Stage 3 or production qualification. The default kernel selection
remains unchanged.

[The application run](application-run.json) builds a fresh owned-profile kernel
and runs the unchanged pinned BusyBox ELF. The observer records the actual
prelude-to-process-table signal transfer, successful serial delivery, zero exit
status at selection, and terminal erasure of the 8192-byte process table,
512-byte command state, thread registrations, heap state and heap tokens.
The exact application-output comparison also passes. See
[the observations](guest/observations.json) and [test log](application-test.log).

[Post-validation](post-validation.json) passes formatting, explicit included-file
formatting, locked workspace/all-target checking, seventeen user-memory model
tests, the unchanged source-size check and whitespace checking. This is not a
full-workspace test execution. [Object readback](unchanged-objects-readback.json)
compares fourteen exact object pairs with the preceding private caller
experiment, including both predecessor objects. It does not make that earlier
experiment a production qualification.

The selected run peaked at 1199562752 bytes, with no limit/OOM/kill events or
swap use. The wrapper and host reserve were unchanged. A preceding experiment
failed with a disk quota error while writing under `/tmp`; application artifacts
now use ignored `linux-kernel/.cache/x86_64` storage. No filesystem quota changed.
Earlier observer failures retain their original failed status and logs.
The local Git attribute exempts verbatim logs from whitespace formatting;
their exact bytes remain covered by the manifest.

[Source identity](source-identity.json) records the forty-four modified or
untracked inputs. Their executable bytes stayed unchanged during and after the
passing run. The only recorded post-run input edit removes an extra blank line
at the end of the lab README. This input list is not a complete producer census
or clean-checkout execution claim. [Artifact digests](artifacts.json) identify
the uncommitted experiment; the external application and its containing archive
remain outside Git under `TK-BINARY-001`.

To reproduce, supply the pinned external dependency and run the explicit
ignored interoperability test in [the lab instructions](../../../../labs/qemu/x86_64/README.md#owned-first-image-exit-regression).
Historical patched fixtures were not rerun for this checkpoint. Source-built
native boundary/error cases, complete all-caller qualification and canonical
production binding/cold reconstruction remain acceptance blockers, as recorded
in [the scope](scope.json). The original Linux-compatible kernel goal remains
unfinished.
