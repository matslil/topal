# Design-pattern research library

This library inventories recurring program and system structures that are
recommended in at least one documented context and that can place requirements
on a programming language. It is an input to Topal design review, not a claim
that every pattern is universally good or that Topal already supports it.

The catalog deliberately treats security broadly. It includes authority,
information flow, timing leakage, physical-device control, fault containment,
platform roots of trust, and safety-related recovery rather than equating
security with network attack prevention.

## Method

The search combined primary research with official language, platform,
standards, and engineering guidance. A candidate was retained when all of the
following held:

1. a primary or responsible-source reference describes the structure or
   recommends it in a stated situation;
2. the structure changes what a useful core language may need to express or
   prove; and
3. it adds a materially different requirement from entries already present.

Search proceeded by concern (performance, latency and jitter, memory, safety,
security, robustness, distribution), paradigm (object-oriented, functional,
concurrent, synchronous, dataflow, and DSL), and target (CPU, GPU, NPU,
DSP/MCU, device I/O, and distributed nodes). It stopped after each cell in the
coverage matrix had both research and practitioner/official coverage where
available, and additional searches produced only variants of an existing
language requirement. This is *thematic saturation*, not mathematical proof of
completeness. New patterns should be added when they expose a new core-language
requirement or a meaningfully different tradeoff.

The sources were checked on 2026-09-10. Each entry follows [the record
schema](schema.md), includes counter-indications, and identifies only language
support needed to *express or safely optimize* the pattern. Library/runtime,
operating-system, certification-process, and hardware obligations are called
out rather than misrepresented as syntax features.

## Coverage matrix

| Concern or setting | CPU | GPU | NPU | DSP / MCU | Distributed / external |
| --- | --- | --- | --- | --- | --- |
| Abstraction and reuse | AP, FD | FD, HA | FD, HA | AP, FD | AP, DS |
| Throughput / total speed | MM, CC | HA | HA | HA | CC, DS |
| Tail latency / jitter | MM, RT | HA | HA | RT, HA | RT, DS |
| Memory / data movement | MM | MM, HA | HA | MM, HA | MM, DS |
| Safety / fault tolerance | AP, SR | SR | SR | RT, SR | SR, DS |
| Security / trust | AP, ST | ST | ST | ST | ST, DS |
| Determinism / analyzability | FD, RT | HA | HA | RT, HA | CC, DS |

## Catalog

| Family | IDs | Patterns |
| --- | --- | --- |
| [Abstraction and lifecycle](abstraction-and-lifecycle.md) | AP-01..AP-06 | Strategy/capability, adapter, algebraic state, typestate/session, scoped resource, smart constructor/contract |
| [Functional composition and DSLs](functional-and-dsl.md) | FD-01..FD-06 | Bulk combinators, parser combinators, effect handlers, EDSL, staging, persistent update |
| [Memory and CPU data paths](memory-and-cpu.md) | MM-01..MM-06 | Uniqueness update, regions, preallocation, zero-copy views, SoA/AoSoA, cache tiling |
| [Concurrency](concurrency.md) | CC-01..CC-06 | Structured fork/join, actors, CSP, SPSC ring, RCU/hazard reclamation, STM |
| [Real-time and low latency](real-time-and-low-latency.md) | RT-01..RT-06 | Time-triggered execution, rate-monotonic scheduling, priority ceiling, busy polling, split interrupt work, bounded work/WCET |
| [Safety and robustness](safety-and-robustness.md) | SR-01..SR-06 | Simplex, watchdog, recovery block, voting redundancy, safe state, supervision tree |
| [Security and trust](security-and-trust.md) | ST-01..ST-06 | Least authority, complete mediation, separation of privilege, information flow, constant time, hardware compartments |
| [Distributed reliability](distributed-reliability.md) | DS-01..DS-06 | Bounded retry, circuit breaker, bulkhead, idempotent consumer, saga, transactional outbox |
| [Accelerators and DSP](accelerators-and-dsp.md) | HA-01..HA-12 | SIMT, tiling, layout, fusion, DMA pipeline, schedules, quantization, mixed precision, sparsity, systolic flow, synchronous dataflow, saturating fixed point |

## Deliberate boundaries

- The library does not copy every Gang-of-Four pattern. Patterns such as
  Factory Method, Decorator, Facade, Proxy, Command, Observer, and Visitor are
  representable combinations of first-class functions, products/sums,
  interfaces, modules, and messaging for the core-language questions at issue.
  AP-01 through AP-04 capture the distinct dispatch, boundary, and state needs.
- Algorithms (sorting, consensus, cryptographic primitives, neural-network
  architectures) are included only when their recurring *program structure*
  creates a language requirement.
- Development-process practices such as code review, fuzzing, qualification,
  supply-chain due diligence, and certification are essential but are not
  program design patterns. Language features can provide evidence for them but
  cannot implement the process.
- Deployment-only patterns such as geographic redundancy and blue/green
  rollout are excluded unless their protocol must be expressed in program code.
- Hardware mechanisms alone are excluded. A hardware entry appears only when
  source programs or compilers need shape, layout, scheduling, numerical, or
  authority information to use the mechanism well.

## Reading a requirement

“Core-language support required” is intentionally implementation-neutral. For
example, a pattern may require observable ownership transfer but not a
particular borrow-checker syntax; it may require a way to state associative
reduction without requiring one GPU API. This makes the library suitable for
comparing interpreted and compiled Topal while still asking whether an
optimizing compiler could generate code as efficient as the best comparator.
