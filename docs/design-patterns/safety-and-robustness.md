# Safety and robustness patterns

## SR-01 — Simplex runtime assurance

**Good fit.** A high-performance or adaptive controller cannot be fully assured,
but a simpler fallback controller and a safe operating region can be assured.

**References.** SEI's Simplex architecture separates a complex controller from
a high-assurance subsystem and switching logic.[^sr1]

**Hardware assumptions.** Control system with trustworthy sensing/actuation and
enough timing margin to detect danger and switch before leaving the safe region.

**Problem.** Requiring the whole advanced controller to meet the highest
assurance level can be infeasible, while letting it act unchecked is unsafe.

**Structure.** Run complex and safety controllers behind a decision module;
monitor invariant/region conditions and transfer authority to the verified
fallback before safety can be violated.

**Limitations.** The monitor, switch, model, sensors, and fallback become the
trusted base. Conservative switching reduces performance; correlated hardware
faults may defeat both controllers.

**Core-language support required.** Isolated components and capabilities,
explicit actuator authority transfer, state invariants, bounded-time monitors,
deterministic switching, redundant inputs, and independently verifiable modules.

## SR-02 — Watchdog and heartbeat monitor

**Good fit.** A critical component or processor must be detected and recovered
when it hangs, stalls, or misses progress deadlines.

**References.** NASA/JPL F Prime documents periodic ping/response health checks,
warning/fatal thresholds, and an optional independent hardware watchdog.[^sr2]

**Hardware assumptions.** A clock and monitor; strong recovery uses a watchdog
independent of the monitored software and often a reset line.

**Problem.** Functional error handling cannot run when the responsible task or
processor no longer makes progress.

**Structure.** Send keyed periodic challenges or require progress heartbeats;
escalate missed thresholds and stroke the hardware watchdog only while health
conditions hold.

**Limitations.** A responsive but incorrect component passes; bad thresholds
cause false reset or slow detection; common scheduler/clock failure can fool a
software-only monitor.

**Core-language support required.** Timed protocols, unique correlation keys,
task liveness observation without shared internals, escalation state machines,
MMIO/device authority, restart-safe resource cleanup, and explicit assumptions.

## SR-03 — Recovery block with acceptance test

**Good fit.** Residual software design faults are plausible and independently
developed alternate implementations can be checked against a useful acceptance
condition.

**References.** Randell introduces recovery blocks, conversations, and fault-
tolerant interfaces for error detection and recovery.[^sr3]

**Hardware assumptions.** None; persistence or external effects require a
rollback/compensation mechanism.

**Problem.** One implementation may produce an undetected wrong result even
when ordinary exceptions are handled.

**Structure.** Establish a recovery point, run a primary, test its result, and
restore state and try alternates until one passes or the block fails.

**Limitations.** Acceptance tests can share specification faults; alternates
may not be independent; rollback is expensive or impossible for irrevocable
physical effects.

**Core-language support required.** Result/exception handling, transactional or
persistent snapshots, effect rollback/compensation classification, scoped
resources, pure acceptance predicates, and structured aggregation of failures.

## SR-04 — Diverse modular redundancy with voter

**Good fit.** A single random hardware fault or independently caused
implementation fault must not immediately corrupt a critical output.

**References.** Lyons and Vanderkulk analyze triple-modular redundancy for
computer reliability.[^sr4] IEC 61508 guidance calls for fault-tolerance
strategies including suitable redundancy and diversity.[^sr5]

**Hardware assumptions.** Strong hardware-fault tolerance needs separated
channels, power/clock/failure domains, and a trustworthy voter.

**Problem.** One faulty computation or component otherwise controls the result.

**Structure.** Execute equivalent channels, compare or majority-vote outputs,
diagnose disagreement, and isolate or repair a failing channel.

**Limitations.** Costs at least multiple execution/memory; the voter and common
requirements are single points; correlated faults and equivalent wrong answers
defeat voting.

**Core-language support required.** Explicit independent execution/resource
domains, deterministic comparable results, voter combinators, fault evidence,
diverse backend/implementation selection, bounded timing, and degraded-mode
protocols.

## SR-05 — Known fail-safe state and guarded transition

**Good fit.** Software controls a hazard and must initialize, terminate, or
respond to off-nominal conditions without entering an unsafe configuration.

**References.** NASA SWE-134 requires known safe initialization/restart,
guarded state transitions, command sequencing, integrity checks, and timely
off-nominal response.[^sr6]

**Hardware assumptions.** Sensors and effectors capable of detecting and
reaching the safe state within the hazard-response interval.

**Problem.** Cleanup or arbitrary restart is not necessarily physically safe;
one out-of-order or unchecked command can create a hazard.

**Structure.** Model operational and safe states explicitly; validate
prerequisites and sequence; make failure/termination transition through an
authorized, time-bounded safe-state procedure.

**Limitations.** No universal safe state exists for every system; entering it
may itself be hazardous; model/sensor failures and insufficient actuation
cannot be solved by language semantics.

**Core-language support required.** Typestate, guarded total transitions,
fail-closed result handling, effect ordering, critical-data identity,
time-bounded cleanup, actuator capabilities, and non-bypassable shutdown paths.

## SR-06 — Supervision tree and restart containment

**Good fit.** Long-running services consist of components that can fail and be
restarted according to explicit dependency and escalation policy.

**References.** Erlang/OTP defines supervisors, child specifications, restart
strategies, intensity limits, and supervision trees.[^sr7]

**Hardware assumptions.** None; process isolation improves containment, while
distributed supervisors need failure detectors.

**Problem.** Ad-hoc error handling tangles normal logic with restart policy and
lets repeated faults create restart storms or corrupt dependents.

**Structure.** Separate workers from supervisors; arrange ownership as a tree;
on abnormal termination restart one child or a defined sibling group, subject
to intensity and escalation bounds.

**Limitations.** Restart loses non-durable state and cannot repair deterministic
input/configuration bugs; dependency order and idempotent initialization matter.

**Core-language support required.** Structured task ownership, observable typed
termination, restart-safe constructors, scoped resources, named restart and
escalation policies, bounded retry/intensity, monitoring capabilities, and
state recovery interfaces.

## Sources

[^sr1]: CMU SEI, [An Architectural Description of the Simplex Architecture](https://www.sei.cmu.edu/library/an-architectural-description-of-the-simplex-architecture/), CMU/SEI-96-TR-006, 1996; see also [SEI's architecture history](https://www.sei.cmu.edu/history-of-innovation/setting-a-foundation-for-software-architecture/).
[^sr2]: NASA/JPL F Prime, [Health Monitoring Functionality](https://fprime.jpl.nasa.gov/devel/docs/reference/system-functional/health-monitoring/).
[^sr3]: B. Randell, [“System Structure for Software Fault Tolerance”](https://doi.org/10.1145/390016.808467), 1975.
[^sr4]: R. Lyons and W. Vanderkulk, [“The Use of Triple-Modular Redundancy to Improve Computer Reliability”](https://doi.org/10.1147/rd.62.0200), *IBM Journal of Research and Development*, 1962.
[^sr5]: CASS, [IEC 61508-3 Software Architecture Design guidance](https://61508.org/wp-content/uploads/2023/12/CASS-GUIDE-508-SW-IEC61508-3-Software-TOES-v3.3.pdf), TOE 23.
[^sr6]: NASA Software Engineering Handbook, [SWE-134 — Safety Critical Software Requirements](https://swehb.nasa.gov/spaces/7150/pages/16449641/SWE-134%2B-%2BSafety%2BCritical%2BSoftware%2BRequirements?desktop=true&macroName=div).
[^sr7]: Erlang/OTP, [Supervisor Behaviour](https://www.erlang.org/doc/system/sup_princ.html).
