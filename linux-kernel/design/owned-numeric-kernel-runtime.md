# Owned runtime for pure numeric kernel libraries

Status: systems-only direction adopted under standing authorization;
implemented and qualified for the initial numeric unit (`TK-DEC-253`);
[execution report](../step-4-owned-numeric-runtime.md).

Step 2 requires ordinary Topal kernel policy and a freestanding runtime. The
first finite source library does not allocate, whereas ordinary `Nat`/`Int`
lowering uses the existing arbitrary-precision runtime and its hosted allocator.
Reusing that hosted allocation path inside the kernel is invalid. The next
profile supplies bounded, kernel-owned temporary storage for ordinary numeric
values; it preserves the existing arithmetic, types and error semantics.

## Source policy and selected interface

The first numeric unit implements relative-sleep deadline arithmetic. Its public
ordinary function accepts seconds, nanoseconds and raw clock ticks as `Nat`, plus
a Boolean indicating an extended clock epoch, and returns an `Int`. The private
adapter validates a nonnegative result in the native deadline range. Source
computes the existing Linux-compatible conversion saturation, saturating addition
and upward tick rounding. A positive extended epoch selects the saturated target.
User copy, interval validation, clock ownership/sampling and waiting remain native
mechanisms around that source decision. No source assembly or new numeric type
is introduced.

An optional artifact `time-library` declares a contained source unit, public
export, `relative-sleep-deadline-v1` interface, 4096-byte stack limit and
65536-byte temporary-runtime limit. The existing optional `kernel-library` and
its allocation-free profile retain their contracts. A declared numeric import
cannot fall back to native arithmetic when loading/checking/qualification fails.

The ordinary shared frontend selects the export at its declared parameter types,
without narrowing those parameters to a generated client's literal values.
Its complete ordinary helper closure is checked. The initial numeric profile
admits local values, decisions and acyclic calls over Boolean, `Nat`/`Int`, and
the ordinary Euclidean integer pair. Arithmetic is restricted to the qualified
addition, multiplication, comparison and nonzero-divisor Euclidean operations.
Effects, foreign calls, resources/addresses, dynamic callables, recursion, imports
and other unsupported forms remain profile errors. The interface uses ordinary
type/overload selection; it does not alter core-language rules.

## Runtime storage and ownership

The artifact binds checked source code and the selected shared numeric runtime
closure. The root owns an explicitly reserved temporary arena and lends it with
the code binding to the prepared process. The library object owns no writable
global or TLS state. Allocation is a target-private runtime service backed by
that borrowed arena, with no hosted syscall, libc or host allocator dependency.
Source functions receive numeric values, never the arena, pointers or contexts.

A pure numeric invocation exclusively activates the borrowed arena, constructs
the normal private numeric representations from validated native facts, executes
ordinary source/runtime functions and copies out the validated scalar result.
It then erases and reads back the entire used storage and allocation state before
returning to the waiting mechanism. No allocated value escapes the invocation.
Nested/reentrant activation, wrong owner, exhaustion, invalid representation,
out-of-range result or incomplete erasure fails closed. The binding remains live
across exec and is erased/read back at terminal return.

The initial provider uses its already reserved supervisor identity mappings and
single active process invocation. It must validate storage extent, alignment,
capacity, disjointness and exclusive activity. Source cannot select privileged
code, create allocation authority or reinterpret equal numeric values as owner
identity. Later concurrency requires a separately qualified per-invocation or
per-context lending arrangement, rather than a shared ambient allocator.

## Required bounds and machine evidence

Publication must prove the complete stack requirement from actual LLVM frame
metadata and a closed call graph including retained runtime helpers, return
addresses and native alignment. It must also prove a conservative allocation
bound from admitted input ranges, source/helper operations and the actual shared
runtime algorithms. Euclidean runtime iteration and allocation counts must be
bounded by the checked operand bit widths. A measured high-water mark alone does
not discharge this requirement. Missing, cyclic, dynamic or excessive evidence
prevents publication.

The numeric runtime keeps arbitrary-precision meaning. A bound describes this
admitted interface/call graph and its owned resources, never silent truncation or
a machine-sized reinterpretation of `Nat`/`Int`. The source function may change
within that profile only when the checked graph still fits its declared limits.

The x86-64 closure must retain ordinary lowering and numeric algorithms while
binding only qualified allocation/fatal mechanisms. It disables red zone,
implicit floating point and tail-call conversion, checks actual code/dependencies
and preserves caller ABI/extended state. The library bridge and arena lending
remain target-private. AArch64 and RISC-V need their own ABI, alignment, frame,
mapping and owner evidence; the source arithmetic and storage-lifetime contract
remain portable.
The first interface uses the existing qualified ten-nanosecond clock view; it
is not a claim that other hardware counters have that period. Another provider
shall normalize its counter to that semantic view or qualify an interface with
explicit scale facts, preserving saturation and rounding.

## Qualification and review

Risk is high: arithmetic representation, hidden runtime allocation, privileged
code ownership and bounded resource lending interact. Review must trace the
ordinary frontend, complete source/runtime closure, range/bit-width/iteration
bounds, stack costs, storage activation, result conversion, ABI preservation,
exec retention and complete erasure.

Qualification must compare source results with independent arithmetic across
zero, rounding, ordinary and saturation boundaries; observe real source/runtime
frames and allocation high-water marks; exercise invalid/missing source, bounds,
storage and representations; reject a deliberately wrong source result; and run
the full shared-services corpus on one kernel. The existing controlled HPET
cases retain explicit hardware test-input status. Native root/provider objects
must remain independent of policy bodies.

A temporary hosted prototype already compiles the ordinary deadline source and
returns `922337203685477581` for the conversion-saturated example. Replacing only
its test allocator with a 65536-byte arena observes 9768 bytes in 380 allocations,
including hosted result printing. This is preliminary feasibility evidence,
not a kernel result, a generic-input proof or the required publication bound.

General allocation, arbitrary containers/modules, scheduling, multiple user
processes, writable filesystems, distribution boot, containers and VM hosting
remain unfinished. See the [first source library](freestanding-kernel-policy-library.md),
[architecture](topal-kernel-architecture.md), [gap analysis](topal-gap-analysis.md)
and [Step 4 status](../step-4-status.md).
