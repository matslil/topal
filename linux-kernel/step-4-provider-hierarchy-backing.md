# Provider-selected static hierarchy backing

Status: systems-only implementation of `TK-DEC-286`, under the standing chat
authorization. The [design](design/kernel-image-protection.md#provider-selected-static-hierarchy-request),
`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001`,
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-SYSTEMS-STATIC-ELF-PROVIDER-BACKING-001` define its scope. Core language
semantics and the bootstrap-equivalent mapping contract remain unchanged.

The static-image hierarchy request now accepts its exact sealed owner and
`backing is provider-selected`. A distinct checked operation, model transition,
semantic trace and provider identity preserve that purpose. Parsing no longer
turns this form into a source-selected numeric allocation. The old explicit
five-frame form retains its original operation and trace. Mixed forms,
duplicates, native selectors and wrong owners are refused.

The current x86 provider derives its existing exact five-frame, unit-aligned
requirement. Lowering reuses the complete allocator and actual owner lifecycle:
sealed construction, reservation, attach, activation/restoration, detached
reclaim and whole release. Count/alignment, issuer/generation, pins, disjointness,
exclusions and user-table admission are not widened. Unsupported protected
templates remain refused. New image-derived geometry still requires complete
qualified image/owner binding across every producer and usable root.

## Validation and review

The [committed result](tools/user-memory/results/stage-4-provider-hierarchy-backing.json)
records all **eleven commands passing** over **948 stable inputs** in
**365.77 seconds**. All **651 frontend** and **255 semantics** tests pass,
including the three new source checks and two new model checks. Both focused
lowering tests, five canonical image-layout checks, the default ABI fixture,
workspace/all-target checking, formatting, the Rust file ceiling, **271 user**
and **83 process-memory** models pass.

The fresh VM completes the unchanged serial contract and all four ordinary
program executions (initial execution plus three exec replacements), with
**16 valid and 92 refused** application buffer requests. Its actual kernel and
the separately rebuilt original-profile kernel both retain SHA-256
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.
The private binding reconstructs **23 objects and 862 contributions** and
keeps production/protected qualification false. No 51-workload corpus rerun
is claimed for this source snapshot. Peak/limit are
**2,204,266,496 / 3,274,539,008 bytes**, with zero swap and all sampled memory
events zero. The full local record is
`.cache/x86_64/provider-hierarchy-backing-stage-1/run.json`; the actual VM build
is `.cache/x86_64/process-mm-interface-regression-2629628-1794634910047855526`.

[The reproducible driver](tools/user-memory/run_hierarchy_backing.py) runs the
frontend and semantics suites, focused native-lowering checks, an ordinary
source-built VM case, canonical image-layout/default-ABI checks, workspace
checking, formatting, file-size checks and both existing Python model suites.
Its report retains exact commands, source hashes, actual images and bounded
resource observations. The legacy private image is freshly rebuilt separately.
The native case follows the existing serial-only workload through initial
process execution and three exec replacements. Its inactive initializer output
remains application data. It grants no page-table authority or effective image
protection. Historical full-compiler host-RWX/injected-state tests are not run;
that full suite remains incomplete.

The selected review treats affine backing and compatibility as high risk.
Single-agent review compares both source forms' complete root-object bytes and
compiler-derived owner descriptors, checks state/refusal behavior at source and
IR boundaries, and follows every use through attach, restoration, reclaim and
release. Existing native validation mechanisms remain in place. A separate
review pass checks the complete diff against the higher-level contracts and
confirms that no existing user-admission or five-page validator was removed.

Authoring failures included test-only use of unavailable `Clone`/`Debug`, an
imprecise fixture substring matching `503` instead of `5`, and a planning test
correctly refused by the complete frame-source binding gate. The tests were
corrected, including a missing entry-revocation step in a lifecycle fixture;
the binding and lifecycle gates were retained. These are ordinary development
failures and establish no explanation of any earlier AI safeguard interruption.

## Remaining work

This establishes a runnable purpose-preserving source interface for the
existing provider. It does not yet allocate image-derived extra table storage,
bind that storage to the final protected image, change hardware permissions or
qualify a protected context. Bootstrap/process builders, all aliases and roots,
effective controls, publication, asynchronous writers and lifetime/fatal paths
remain required. Production publication and the original complete Linux UAPI,
DTS, distribution, Docker/Podman and VM-host objectives remain unfinished.
