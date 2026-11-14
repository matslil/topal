# Scoped ownership of inactive image tables

`TK-DEC-292` is a systems-only refinement under standing authorization. The
current preparation erases/releases its storage before ordinary boot. A live
consumer instead needs exact retained owners across allocations, userspace and
root restoration. This increment supplies that lifetime without activation.

After the prepared observation, source consumes the exact table resource with
`memory kernel image retain tables image-tables during { ... }`. The block is
the independently checked ordinary boot continuation. This element owns a
`RetainedKernelImageTables` resource internally; it exposes no address, table
format or mutation capability. It is a scoped systems protocol, not a general
core-language destructor or exception-handling feature.

Before entering the scope, the provider performs the same whole-owner, initial
boot/context, complete template and retained-role comparisons required by the
immediate-discard path. The latter keeps its existing meaning. Retention then
ends the initial context comparison window. Original boot snapshots stay
historical: they do not assert that subsequent WP/NXE, CR3 or descriptor state
is unchanged. Future activation requires complete fresh qualification.

Both image banks and the separate low USER leaf remain pinned and inactive.
Issuer, IDs, slots, generations, exact physical extents and complete stored
words remain bound. Ordinary allocations must exclude these actual owners.
The USER walker keeps its exact legacy table admission. The ordinary bootstrap
and process banks remain separate owners; the prepared banks cannot substitute
for them. Current child/pair paths retain one process hierarchy and exchange
USER leaves/continuations; retention does not invent a cloned root bank.

The source checker places the consuming retirement after all checked body
operations and before its designated terminal disposition. It verifies owner
identity/generation, inactivity, complete unchanged banks/zero low leaf and
retained-role coverage, then erases, reads back, unpins and releases all three
whole extents. The issuer's next ID may have advanced through ordinary work;
it must not regress and is not required to equal its initial value. Earlier
fatal branches retain/poison authority rather than attempting partial cleanup.

Alternatives: leaving ordinary untracked pointers after preparation loses
ownership; holding a fixed count without generation/pin checks loses identity;
reusing the old exact boot-state comparison after userspace falsely assumes an
unchanged context. A distinct affine scoped owner preserves storage while
making the context boundary explicit.

Other architectures retain the same resource/lifetime contract with their own
granules and table formats. This stage must observe ordinary user/exec and root
restoration with live owners, complete retirement and predecessor behavior.
Boundary/model evidence is not continuous effective-protection evidence.
All-root integration, CPU/asynchronous exclusion, effective modes, publication,
native failure paths, continuous protection and full Linux acceptance remain
independent requirements. Host protections and bounded VM limits stay intact.
