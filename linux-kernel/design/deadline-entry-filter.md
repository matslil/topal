# Completed deadline notification filtering

Status: systems-provider correction adopted under standing authorization;
implemented and physically qualified (`TK-DEC-248`); see the
[qualification report](../step-4-deadline-entry-filter.md).

A raw target interrupt is not itself a typed deadline context. The provider
must bind accepted delivery to the current armed event and must not construct a
second affine context after that event completed. A queued hardware notification
may remain after completion and be delivered when a suspended interrupt frame
resumes with interrupts enabled. Re-entering source completion/preemption for
that stale notification violates the existing single-event contract.

The initial x86 adapter shall recognize its retained completed event before
entering the source handler, acknowledge that raw notification and return with
the interrupted register/frame image unchanged. It shall neither re-complete
the event nor preempt a context again. Armed and other invalid states retain
their existing checked paths. Completion shall close the timer route as well
as disable/clear the source before controller acknowledgement; a notification
already queued in the local controller still requires entry filtering.

This is a provider correction of `TOPAL-SYSTEMS-DEADLINE-EVENT-001` and the
existing preemption lifetime. Source ownership, operations and dispositions do
not change. Another architecture must bind raw-notification admission to its
own event/controller lifecycle rather than inherit x86 vectors or MMIO.

Qualification shall deterministically inject a completed event's raw vector
at worker resumption, establish failure on the predecessor, and prove the
corrected provider resumes and completes the original context/resource sequence
without a second typed completion or preemption. Repeated ordinary QEMU runs,
register/frame checks and the complete application/rollback corpus remain
regressions. The injection must be recorded as test setup, never kernel policy.
Risk is high because filtering, controller acknowledgement and affine context
transfer interact. No core-language change is introduced.
