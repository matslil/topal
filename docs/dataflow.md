# Clocked static-rate dataflow

`Flow` in revision `v0.2` refines a typed stream with a logical clock, exact
static consume/produce rates, and an initial delay:

```topal
DecimatedSamples is Flow (
  value is Sample,
  clock is audio-clock,
  consumes is 2,
  produces is 1,
  initial-delay is 0
)
```

Rates are nonnegative static `Nat` values per logical tick. A mode-dependent
flow is a closed sum of individually static modes and changes mode only at a
tick boundary. `delay ( initial is value, input is flow )` is the ordinary
stateful construction on a feedback edge.

For a closed graph the checker solves balance equations, rejects inconsistent
rates and zero-delay cycles, derives a deterministic periodic sequential
schedule, and computes finite minimum buffer capacities for that schedule. The
schedule, capacities, and assumptions are compiler-owned implementation
evidence; source and graph translators cannot assert them.

The interpreter executes one derived schedule in actor order for each logical
period. A compiler may fuse actors, statically allocate buffers, or vectorize
periods only when the observable sequence is unchanged. Physical placement,
DSP/FPGA resources, systolic mapping, and pipeline timing remain deferred to
the architecture model.
