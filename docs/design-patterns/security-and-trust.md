# Security and trust patterns

These patterns protect authority, confidentiality, integrity, or physical
assets. “Resource” includes devices and actuators, not only files and network
services, and the limitations explicitly include physical side channels and
hardware trust.

## ST-01 — Object capability and least authority

**Good fit.** Components should receive only the ability to perform the exact
external actions needed for their role.

**References.** Saltzer and Schroeder state least privilege among the central
protection principles.[^st1] Pony demonstrates derived authority passed as
rights-bearing values rather than ambient access.[^st2]

**Hardware assumptions.** None; resources may be files, network endpoints,
devices, actuators, secure elements, or deployment services.

**Problem.** Ambient global authority lets an unrelated bug or compromised
component affect every resource available to the process.

**Structure.** Represent authority as unforgeable values with narrow rights;
derive weaker capabilities; pass them explicitly; revoke through a mediator or
lifetime end.

**Limitations.** Capability leakage is authority leakage; revocation and
enumeration are nontrivial; the OS/hardware boundary must enforce the promise.

**Core-language support required.** Unforgeable opaque capabilities, explicit
dependencies instead of globals, rights attenuation, linear/affine use when
needed, resource-parameterized effects, endpoint identity, and sandboxed host
binding.

## ST-02 — Complete mediation with fail-safe default

**Good fit.** Every access to a protected object or hazardous operation must be
authorized under current policy, and uncertainty must deny rather than grant.

**References.** Saltzer and Schroeder define fail-safe defaults and complete
mediation as protection design principles.[^st1]

**Hardware assumptions.** A trustworthy enforcement boundary: language
runtime, OS, hypervisor, device, or hardware protection domain.

**Problem.** Cached or scattered checks become stale or miss alternate paths;
error fallbacks accidentally permit access.

**Structure.** Route every operation through one non-bypassable reference
monitor, decide from positive authority, and make invalid/unknown states fail
closed.

**Limitations.** Mediation adds latency and can become a bottleneck or single
failure point. Correct enforcement does not make the policy correct.

**Core-language support required.** Encapsulation that cannot be bypassed,
explicit effects/resources, opaque handles, exhaustive result/error handling,
revocable policy state, atomic policy updates, and trustworthy foreign/device
boundaries.

## ST-03 — Separation of privilege / two-authority action

**Good fit.** One mistake, compromised principal, or software action must not
be sufficient to initiate a high-consequence operation.

**References.** Saltzer and Schroeder recommend separation of privilege.[^st1]
NASA requires at least two independent operator actions for safety-critical
overrides and forbids one software event from initiating a hazard.[^st3]

**Hardware assumptions.** Independence may require separate input devices,
principals, channels, or hardware roots rather than two values on one machine.

**Problem.** A single credential, event, or code path is a single point of
catastrophic authorization failure.

**Structure.** Require distinct capabilities or approvals, bind both to the
same operation/session, validate freshness and independence, then consume them
atomically.

**Limitations.** Adds operational latency and availability dependencies;
nominally distinct approvals may share a failure or coercion domain.

**Core-language support required.** Distinct authority identities and roles,
linear one-shot tokens, freshness/session types, atomic multi-capability
consumption, auditable effects, timeout/cancel, and non-forgeable provenance.

## ST-04 — Lattice information-flow control

**Good fit.** Confidentiality or integrity policy should constrain all
computed flows, not merely explicit access to storage.

**References.** Denning models security classes as a lattice and derives
mechanisms for automatically certifying permitted program flows.[^st4]

**Hardware assumptions.** None for explicit/implicit software flows; physical
side channels need target-specific models and mitigation.

**Problem.** Data can leak through derived values, branches, errors, messages,
termination, and timing even when access checks are correct.

**Structure.** Label data and effects; join labels as information combines;
permit flows only along policy order; make declassification explicit and
authorized.

**Limitations.** Conservative labels overtaint; covert/physical channels and
availability are not automatically covered; declassification policy is hard.

**Core-language support required.** Provenance-carrying labels, implicit-flow
analysis, effect/message propagation, policy lattices, scoped declassification
authority, model identity, and preserved labels/evidence through compilation.

## ST-05 — Constant-time secret processing

**Good fit.** Cryptographic or authentication code must prevent secret-
dependent timing and memory-access observations under a stated target model.

**References.** Barthe et al. formalize constant-time simulation and
compiler-pass preservation of the countermeasure.[^st5]

**Hardware assumptions.** A specified ISA/microarchitecture and observation
model; caches, speculation, power, EM, interrupts, and frequency scaling can
extend that model.

**Problem.** A source algorithm with no obvious output leak may reveal secrets
through branches, instruction counts, addresses, compiler transformations, or
microarchitecture.

**Structure.** Keep control flow and accessed addresses independent of secrets,
use constant-time primitives/selects, and verify preservation through every
lowering stage against the chosen model.

**Limitations.** Constant time covers only named observations; it can be slower;
compilers can break source-level intent; physical power/EM attacks need other
countermeasures.

**Core-language support required.** Secret provenance and leakage contracts,
branch/address dependence analysis, constant-time primitives, target evidence,
secure-compilation obligations, optimization barriers where proved necessary,
and model/version metadata.

## ST-06 — Hardware-enforced compartment

**Good fit.** Components in one process need low-overhead isolation and precise
memory/authority bounds so a memory-safety fault has a limited blast radius.

**References.** CHERI combines bounded unforgeable hardware capabilities,
sealed invocation, and compiler/OS support for scalable compartments.[^st6]

**Hardware assumptions.** Capability ISA such as CHERI, or a less efficient
MMU/MPU/process equivalent with defined domain transitions.

**Problem.** Language memory safety alone does not isolate unsafe/foreign code,
logic faults, or over-authorized components; process isolation can be too
coarse and expensive.

**Structure.** Place components in protection domains, give each bounded
capabilities, cross through typed sealed gates, and share only explicitly
authorized memory/resources.

**Limitations.** Hardware and OS enter the trusted base; shared services remain
confused-deputy risks; transitions and marshaling cost time; it does not prove
functional correctness.

**Core-language support required.** Compartment/module identity, explicit
authority manifests, bounded layouts, opaque sealed entry capabilities,
typed cross-domain ABI, controlled shared-memory ownership, and a backend able
to preserve capabilities rather than lower them to forgeable integers.

## Sources

[^st1]: J. Saltzer and M. Schroeder, [“The Protection of Information in Computer Systems”](https://www.mit.edu/~Saltzer/publications/protection/index.html), *Proceedings of the IEEE*, 1975.
[^st2]: Pony, [Derived Authority](https://tutorial.ponylang.io/object-capabilities/derived-authority.html).
[^st3]: NASA Software Engineering Handbook, [SWE-134 — Safety Critical Software Requirements](https://swehb.nasa.gov/spaces/7150/pages/16449641/SWE-134%2B-%2BSafety%2BCritical%2BSoftware%2BRequirements?desktop=true&macroName=div).
[^st4]: D. Denning, [“A Lattice Model of Secure Information Flow”](https://doi.org/10.1145/360051.360056), *Communications of the ACM*, 1976.
[^st5]: G. Barthe et al., [“Secure Compilation of Side-Channel Countermeasures: The Case of Cryptographic Constant-Time”](https://doi.org/10.1109/CSF.2018.00031), IEEE CSF, 2018.
[^st6]: R. Watson et al., [“CHERI: A Hybrid Capability-System Architecture for Scalable Software Compartmentalization”](https://www.cl.cam.ac.uk/research/security/ctsrd/pdfs/201505-oakland2015-cheri-compartmentalization.pdf), IEEE S&P, 2015.
