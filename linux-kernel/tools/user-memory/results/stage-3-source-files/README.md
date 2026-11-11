# Stage 3 source-built bounded file transfers

This selected checkpoint exercises `TOPAL-SYSTEMS-USER-EXTENT-001` through
ordinary application arguments and linked C functions. Twenty bounded file
callbacks issue 176 observed syscalls. Four initial/replacement images also
retain the preceding 200 startup and 56 shared-runtime syscalls. No completed
application is patched; kernel and pinned third-party binaries are unchanged.

The [native run](native-run.json) and [observations](guest/observations.json) cover:

- Four complete `fstat` records for stdout, profile, root and bin descriptors,
  with cross-directory/page output beyond the predecessor heap geometry.
- Five pathname-stat queries with complete page-split pathname admission,
  rejected input and output candidates, and absent lookup without unused
  output admission or publication.
- Six pathname opens with rejected input preserving state before a page-split
  valid pathname performs the expected absence, allocation or reuse.
- Root/bin `getdents64` records and EOF. Nonempty results write 72/80 bytes
  into an 80-byte mapping ending at an unmapped page despite a requested
  count of 512. EOF uses an unmapped destination without touching user bytes.
- Profile EOF with numeric range refusals before a successful unmapped-output
  EOF, plus preserved user output, private staging and lifecycle flags.

Every refused transfer checks phase preservation; file `EFAULT` additionally
checks exact private staging/lifecycle controls. The observer verifies restored
heap contents, live credential vectors, source and original syscall registers,
and whole-process heap/command/thread-registration retirement. It observes
guest state without assigning registers, memory, translations or ownership.

[Independent readback](independent-readback.json) reconstructs all stat and
directory bytes, including padding, separately from the emitter and C program.
It parses ELF sections/loads and all 4685 linked source bytes, both syscall
adapters, the resolved C bridge and all 25 generated call sites with disjoint
scratch. It checks seventeen exact object/image comparisons against the prior
startup checkpoint, including kernel ELF and boot image, and all 7606 serial
bytes. The native outputs and existing enclosing witness both pass.

The compiler stack records sum to 496 bytes for runtime C functions, 168 for
startup functions and 400 for file functions, each including conservative return
words. Only startup can delegate to file cases; runtime cannot call either.
Taking the larger complete path, then adding the existing 1024-byte scratch
and 128-byte adapter/alignment/return margin, gives 1720 of 2048 bytes. This
preserves the fixture bound and remains separate from the kernel caller reserve.

The [six post-check commands](post-validation.json) pass two build-only tests,
including default-fixture byte preservation, formatting, locked workspace and
all-target compilation, source size and whitespace. No full-workspace test
execution or clean-checkout reconstruction is claimed. Native peak memory was
348737536 bytes, with no limit/OOM/kill events or validation swap under the
unchanged wrapper and host reserve.

[Earlier failures](earlier-failures) retain their nonzero statuses. The first
observer attempted a zero-byte GDB read after the first stat result; empty
observations now perform no read. The stronger directory fixture initially
triggered the C compiler's negative-index bounds warning. Expressing the guard
as the first byte of the complete area fixes that source warning without
disabling warnings. The compile failure never reached native execution.
Neither failure is classified as an AI safeguard denial or resource exhaustion.

[Source identity](source-identity.json) records nine selected executable/test
inputs. It is not a complete producer census. Documentation and evidence were
recorded after execution. [Artifact identities](artifacts.json), first-party
source and original logs are retained; executable payloads remain ignored.
The local Git attribute preserves log whitespace covered by the manifest.

Reproduce with the [explicit native test](../../../../labs/qemu/x86_64/README.md#source-built-user-buffer-cases).
The artifact comparisons require retained local baseline artifacts and are not
cold reconstruction. Remaining direct signal/path/output/header/command cases,
complete ownership/mode/stability qualification, canonical binding and exact
production qualification remain required. Stage 3, initial Step 4 and the full
original kernel goal are unfinished. See [scope](scope.json).
