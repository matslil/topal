# Step 4 bounded initial `sched_yield`

## Outcome

The sole runnable initial user thread now implements Linux x86-64 syscall 24,
`sched_yield`. With no eligible peer, the kernel preserves the exact context
and private process state and returns zero through the ordinary executable
continuation without a provider context switch.

Constructed code checks the zero result and continues through its subsequent
unsupported-call and stdout proofs. The syscall has a dedicated arm and is not
satisfied by the `-ENOSYS` fallback.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-SCHED-YIELD-001` requires exact success, context/state
preservation, kernel-owned no-peer selection, and continued execution.
Structural tests prove syscall-24 dispatch and exactly one constructed call.

Artifact schema `/73`, provider `/70`, provider object `/70`, artifact root
`/71`, boot adapter `/33`, and gate `/75` identify the contract. Focused suites,
gate fixtures, formatting, and the complete workspace suite pass.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` runs were byte-identical and
reached all 51 observations. Hashes are:

- linked kernel: `4be0459c7c60a2703a0338aa7f6d5f53acd001c1954f812992d9465d7b8e3277`;
- boot image: `bcb969f64db949df4f51286a56438c2171b508dedd7a11bbd41736e78feb301c`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `2f3bcdb605b7e40c1d94848b75227c10d57ae5bdafba196680b4ff2fbb9c65e1`;
- boot provenance: `45483e8bcaef4da70372ae85df13e8c2d9da4a85ca9676e036cd56601944c1e2`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence: `04f30741e2c9a04064dc8a61f7581f1443bb5643699524448baae4dbee4c89ce`.

## Risk and remaining boundary

Risk is low-to-medium because the operation is state-preserving but changes
dispatch precedence. Review covered dedicated-arm selection, exact return,
absence of provider transfer and state mutation, continued execution, full
regression, and physical execution. Multiple threads, runnable collections,
priorities, affinity, preemption, migration, and SMP scheduling remain absent.
