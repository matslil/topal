# Repeated replacement of the retained static executable

Status: systems-only refinement adopted under standing chat authorization;
implementation and qualification pending (`TK-DEC-246`).

The retained immutable executable and pristine writable snapshot belong to the
process lifetime. Successful exec consumes an image generation, not that retained
execution authority. A later live generation may request the same replacement
after its admitted startup completes. The original shell command's phase is
not authority for that later request.

This refinement keeps the bounded executable, argv/envp, descriptor, signal,
identity, stack and mapping contracts. The initial native qualification still
requires the declared complete 34-page anonymous backing in its qualified
state; sparse/arbitrary memory layouts and general executable selection remain
later work. The adapter validates current address-space/backing identity before
copying, copies all inputs before commit, filters current close-on-exec slots,
resets caught dispositions and image/thread state, and retains shared offsets,
ignored dispositions, blocked signals, cwd and prepared credentials.

Every generation restores pristine bytes, clears BSS/old anonymous bytes,
revokes old mappings, constructs fresh value-derived startup vectors and random
bytes, erases staging and enters the new image. Failure before commit preserves
the current generation; failure after commit is terminal. Repetition must not
reuse native pointers or stack/auxiliary identity from an earlier generation.
Terminal cleanup releases retained authority once, regardless of successful
replacement count. No fixed replacement count or application identity grants
permission. A private qualification marker may record each committed entry;
it supplies evidence and no runtime authority.

Executed qualification shall cover at least three successful replacements,
failed-prefix retries after the first replacement, continuing shared offsets,
new close-on-exec filtering on each transition, retained cwd/ignored signal and
mask, fresh random bytes, final nonzero status and complete outer rollback.
External programs shall perform multiple exec calls through ordinary native
interfaces with one unchanged kernel. Portable meaning is image-generation
ownership; each architecture supplies its own entry, register reset, mapping
completion and native ABI adapter. Core language meaning is unchanged.

Risk is high because destructive transitions recur on one process owner.
Review follows live owner validation, complete pre-commit copies, retained
snapshot lifetime, repeated filtering/reset, generation-specific stack/random
state and exactly-once terminal reclamation. The pinned Linux `fs/exec.c`
process/image lifetime and exec commit rules are the primary comparison input.
