# Bounded static ELF initial `getuid`

This record resolves `TK-DEC-191` as a static-process refinement of the
approved initial scalar-identity design. It changes no core-language rule and
introduces neither mutable credentials nor a general process service.

## Boundary

After the immutable process-name witness, the pinned static executable issues
one Linux real-user-identity query. Source shall select the refined sealed
startup service and policy but shall receive no syscall number, register,
credential integer, auxiliary-vector layout, or task representation.

The provider shall require the live process-name phase, a return continuation
inside the finalized executable mapping, the active checked address-space
hierarchy, the exact retained initial-stack leaf, and the complete coherent
initial credential group. The result shall be the sealed real UID already
represented by that group: zero for the approved PID-one root profile. The
query is pointer-free; native argument registers are ignored, and no argument,
pointer, mutable credential, or identity cache is retained.

The clean-room executable shall issue the native zero-argument query, require
zero, and continue to its existing terminal request. Terminal handling shall
require the new phase and then perform the unchanged complete process rollback.

## Alternatives considered

Returning the host UID would couple guest identity to the build or test host
and contradict the sealed auxiliary-vector credentials. Returning a constant
without revalidating those facts would permit internal disagreement. Adding
the remaining scalar-identity family in one increment would skip the observed
startup boundary and make failure attribution less precise. A mutable
credential object would prematurely select transition, namespace, lifetime,
authorization, and synchronization policy.

## Cross-architecture pressure

The portable fact is coherence between a process's sealed real identity and
its initial credential metadata. Syscall numbering, instruction and register
conventions, stack representation, and credential storage remain provider
details. A future architecture may use a different entry mechanism while
proving the same pointer-free identity relation.

This increment does not add effective or group identity queries, credential
mutation, supplementary groups, user namespaces, capabilities, permission
checks, multiple processes or threads, signals, procfs, dynamic linking,
redistributed BusyBox, containers, or a prompt.
