#define _GNU_SOURCE
#include "tail_step_bindings.h"
#include <stddef.h>
#include "tail_step_component_runtime.c"
#include <sys/mman.h>
#include <unistd.h>

typedef struct { unsigned char *mapping, *bytes; size_t mapped, count; } Guarded;
static Guarded guarded(size_t count) {
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    size_t capacity = ((count ? count : 1) + page - 1) / page * page;
    unsigned char *mapping = mmap(NULL, capacity + 2 * page, PROT_NONE,
                                  MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    assert(mapping != MAP_FAILED);
    assert(!mprotect(mapping + page, capacity, PROT_READ | PROT_WRITE));
    return (Guarded){mapping, mapping + page + capacity - count, capacity + 2 * page, count};
}
static void release(Guarded value) { assert(!munmap(value.mapping, value.mapped)); }
static void seed_number(unsigned char *destination, uint64_t magnitude, uint64_t sign) {
    uint64_t length = magnitude > UINT32_MAX ? 2 : !!magnitude;
    if (!magnitude) sign = 0;
    memcpy(destination, &sign, 8); memcpy(destination + 8, &length, 8);
    uint32_t low = (uint32_t)magnitude, high = (uint32_t)(magnitude >> 32);
    if (length) memcpy(destination + 16, &low, 4);
    if (length == 2) memcpy(destination + 20, &high, 4);
}
static __int128 signed_result(const unsigned char *number) {
    uint64_t sign, length; memcpy(&sign, number, 8); memcpy(&length, number + 8, 8);
    assert(sign <= 1 && length <= 4);
    __uint128_t value = 0;
    for (uint64_t index = length; index; --index) {
        uint32_t limb; memcpy(&limb, number + 16 + 4 * (index - 1), 4);
        value = (value << 32) + limb;
    }
    return sign ? -(__int128)value : (__int128)value;
}
static uint64_t expected_mask(void) {
    uint64_t mask = 0;
    for (unsigned index = 0; index < TAIL_PARAMETERS; ++index)
        if (TailRoots[index].kind != 4) mask |= UINT64_C(1) << index;
    return mask;
}
static int admit(TailContext *ctx, uint64_t initial, uint64_t maximum) {
    uint64_t edges = 0;
    if (initial > TAIL_BASE_BOUND) {
        uint64_t difference = initial - TAIL_BASE_BOUND;
        edges = difference / TAIL_MIN_DECREMENT + !!(difference % TAIL_MIN_DECREMENT);
    }
    if (edges == UINT64_MAX) return 0;
    uint64_t steps = edges + 1;
    if (ctx->history->frame < 2 || ctx->history->frame > maximum ||
        steps > (maximum - ctx->history->frame) / TAIL_FRAMES_PER_STEP) return 0;
    if (TAIL_LOANS_PER_STEP && (ctx->history->loan < 2 || ctx->history->loan > maximum ||
        steps > (maximum - ctx->history->loan) / (TAIL_LOANS_PER_STEP ? TAIL_LOANS_PER_STEP : 1))) return 0;
    ctx->frame_end = ctx->history->frame + steps * TAIL_FRAMES_PER_STEP;
    ctx->loan_end = ctx->history->loan + steps * TAIL_LOANS_PER_STEP;
#ifdef TAIL_NATIVE_HOOKS
    ctx->step_limit = steps;
#endif
    return 1;
}
static void verify_owner(const Guarded *owner, uint64_t count, uint64_t offset, uint64_t stores) {
    for (uint64_t index = 0; index < owner->count; ++index) {
        unsigned char expected = 0x7b;
        if (TAIL_TEST_KIND == 1 && index >= offset && index - offset < count && index - offset < stores)
            expected = 0;
        assert(owner->bytes[index] == expected);
    }
}

static uint64_t Cases;
#ifdef TAIL_CONTROLLER_FAULT_TEST
static uint64_t ControllerFault;
static int terminal_controller_fault(void) {
    return ControllerFault == 5 || (ControllerFault >= 18 && ControllerFault <= 20);
}
#ifdef TAIL_NATIVE_HOOKS
/* Deliberately damaged returned states, after independently checking actual
 * source effects. These are controller rejection fixtures, not source output. */
static void damage_checkpoint(TailContext *ctx, uint64_t *outcome) {
    unsigned char *measure = ctx->banks[1 - ctx->bank] + TailRoots[TAIL_MEASURE_PARAMETER].offset;
    uint64_t word = 0;
    switch (ControllerFault) {
    case 6: ctx->history = (History *)(uintptr_t)1; break;
    case 7: memcpy(measure, ctx->banks[ctx->bank] + TailRoots[TAIL_MEASURE_PARAMETER].offset,
                   TailRoots[TAIL_MEASURE_PARAMETER].bytes); break;
    case 8: memset(measure, 0, TailRoots[TAIL_MEASURE_PARAMETER].bytes); break;
    case 9: word = 1; memcpy(measure, &word, 8); break;
    case 10: measure[TailRoots[TAIL_MEASURE_PARAMETER].bytes - 1] = 1; break;
    case 11: memset(measure + 16, 0, 4); break;
    case 12: word = UINT64_MAX; memcpy(measure + 8, &word, 8); break;
    case 13: assert(TAIL_DEPTH > 1); ctx->frames[1].identity = 1; break;
    case 14: ctx->frame_end = NativeHistory->frame; break;
    case 15: ctx->step_frame_end = NativeHistory->frame; break;
    case 16: ctx->loan_end = NativeHistory->loan; break;
    case 17: ctx->step_loan_end = NativeHistory->loan; break;
    case 18: word = 2; memcpy(ctx->result, &word, 8); break;
    case 19: ctx->result[47] = 1; break;
    case 20: ctx->banks[1 - ctx->bank][0] = 1; break;
    case 21: *outcome = 0; break;
    case 22: ctx->work_used = TAIL_WORK_BYTES + 8; break;
    case 23: ctx->bank = 2; break;
    case 24: ctx->result_kind = 3; break;
    case 25: *outcome = 2; break;
    case 26: ctx->frames[0].identity = 0; break;
    default: break;
    }
}
#endif
static void invalid_controller_geometry(void) {
    Guarded private = guarded(65536), stack = guarded(4096);
    PrivateExtent = private.bytes; SourceStack = stack.bytes;
    memset(PrivateExtent, 0x5d, TAIL_TEMP_BYTES);
    memset(SourceStack, 0xbc, 4096);
    InvalidController = 1;
    for (unsigned choice = 0; choice < 8; ++choice) {
        void *base = PrivateExtent, *source_stack = SourceStack;
        if (choice == 0) base = NULL;
        if (choice == 1) source_stack = NULL;
        if (choice == 2) base = PrivateExtent + 1;
        if (choice == 3) source_stack = SourceStack + 1;
        if (choice == 4) base = (void *)(UINTPTR_MAX - 15);
        if (choice == 5) source_stack = (void *)(UINTPTR_MAX - 15);
        if (choice == 6) source_stack = PrivateExtent + 16;
        topal_x86_tail_boundary_active_v1 = choice == 7 ? UINTPTR_MAX : 0;
        InvalidExpectedActive = topal_x86_tail_boundary_active_v1;
        uint64_t before = InvalidFatal;
        if (!setjmp(Fatal)) topal_x86_tail_boundary_fail_v1(base, source_stack);
        assert(InvalidFatal == before + 1 && !Calls && !FatalSeen);
    }
    topal_x86_tail_boundary_active_v1 = 0; InvalidController = 0;
    release(stack); release(private);
}
#endif
#ifdef TAIL_BOUNDARY_TEST
static uint64_t Denials;
#ifdef TAIL_NATIVE_HOOKS
static void deny_native_geometry(TailContext *ctx, int completion) {
    unsigned char private_copy[TAIL_TEMP_BYTES], stack_copy[4096];
    TailOutput output_copy = *NativeOutput;
    History history = *NativeHistory;
    uint64_t calls = Calls;
    for (unsigned choice = 0; choice < (completion ? 23u : 14u); ++choice) {
        void *base = PrivateExtent, *stack = SourceStack;
        History *provided_history = NativeHistory;
        TailOutput *output = NativeOutput;
        if (choice == 0) base = NULL;
        if (choice == 1) stack = NULL;
        if (choice == 2) base = PrivateExtent + 1;
        if (choice == 3) stack = SourceStack + 1;
        if (choice == 4) base = (void *)(UINTPTR_MAX - 15);
        if (choice == 5) stack = (void *)(UINTPTR_MAX - 15);
        if (choice == 6) stack = PrivateExtent + 16;
        if (choice == 7) PrivateExtent[0] = 1;
        if (choice == 8) topal_x86_tail_boundary_active_v1 = UINTPTR_MAX;
        if (choice == 9) provided_history = NULL;
        if (choice == 10) provided_history = (History *)((unsigned char *)NativeHistory + 1);
        if (choice == 11) provided_history = (History *)(UINTPTR_MAX - 7);
        if (choice == 12) provided_history = (History *)(PrivateExtent + 64);
        if (choice == 13) provided_history = (History *)(SourceStack + 64);
        if (choice == 14) output = NULL;
        if (choice == 15) output = (TailOutput *)((unsigned char *)NativeOutput + 1);
        if (choice == 16) output = (TailOutput *)(UINTPTR_MAX - 7);
        if (choice == 17) output = (TailOutput *)(PrivateExtent + 64);
        if (choice == 18) output = (TailOutput *)(SourceStack + 64);
        if (choice == 19) output = (TailOutput *)NativeHistory;
        if (choice == 20) provided_history = (History *)NativeOutput;
        if (choice == 21) output = (TailOutput *)((uintptr_t)NativeHistory - 48);
        if (choice == 22) output = (TailOutput *)(PrivateExtent + TAIL_TEMP_BYTES - 8);
        memcpy(private_copy, PrivateExtent, sizeof(private_copy));
        memcpy(stack_copy, SourceStack, sizeof(stack_copy));
        uintptr_t active = topal_x86_tail_boundary_active_v1;
        uint64_t outcome = completion ? topal_x86_tail_complete_call_v1(base, stack, provided_history, output)
            : topal_x86_tail_checkpoint_call_v1(base, stack, provided_history, 1);
        assert(outcome == 2);
        assert(active == topal_x86_tail_boundary_active_v1 && Calls == calls);
        assert(!memcmp(private_copy, PrivateExtent, sizeof(private_copy)));
        assert(!memcmp(stack_copy, SourceStack, sizeof(stack_copy)));
        assert(ctx->history == NativeHistory && !memcmp(&history, NativeHistory, sizeof(history)));
        assert(!memcmp(&output_copy, NativeOutput, sizeof(output_copy)));
        if (choice == 7) PrivateExtent[0] = 0;
        if (choice == 8) topal_x86_tail_boundary_active_v1 = 0;
        ++Denials;
    }
}
#endif
static void deny_geometry(TailContext *ctx) {
    unsigned char before[sizeof(TailContext)];
    memcpy(before, ctx, sizeof(before));
    uint64_t calls = Calls; History history = *ctx->history;
    for (unsigned choice = 0; choice < 8; ++choice) {
        void *base = PrivateExtent, *stack = SourceStack;
        if (choice == 0) base = NULL;
        if (choice == 1) base = PrivateExtent + 1;
        if (choice == 2) stack = SourceStack + 1;
        if (choice == 3) stack = PrivateExtent + 16;
        if (choice == 4) base = (void *)(UINTPTR_MAX - 15);
        if (choice == 5) stack = (void *)(UINTPTR_MAX - 15);
        if (choice == 6) PrivateExtent[0] = 1;
        if (choice == 7) topal_x86_tail_boundary_active_v1 = UINTPTR_MAX;
        uintptr_t active = topal_x86_tail_boundary_active_v1;
        assert(topal_x86_tail_boundary_call_v1(base, stack) == 2);
        assert(active == topal_x86_tail_boundary_active_v1 && Calls == calls);
        assert(!memcmp(before, ctx, sizeof(before)) && !memcmp(&history, ctx->history, sizeof(history)));
        if (choice == 6) { assert(PrivateExtent[0] == 1); PrivateExtent[0] = 0; }
        if (choice == 7) topal_x86_tail_boundary_active_v1 = 0;
        ++Denials;
    }
}
#endif
#ifdef TAIL_NATIVE_HOOKS
static void complete_invocation(TailContext *ctx) {
    TailOutput expected; memcpy(expected.value, ctx->result, 48); expected.kind = ctx->result_kind;
    History before = *NativeHistory;
#ifdef TAIL_NATIVE_COMPLETION_FAULT_TEST
    switch (TAIL_NATIVE_COMPLETION_FAULT_TEST) {
    case 11: ctx->phase = 1; break;
    case 12: ctx->history = (History *)(uintptr_t)1; break;
    case 13: ctx->banks[0][0] = 1; break;
    case 14: ctx->work[0] = 1; break;
    case 15: ctx->starts[0] = 1; break;
    case 16: { uint64_t bad = 2; memcpy(ctx->result, &bad, 8); break; }
    case 17: NativeOutput->value[0] = 1; break;
    case 22: assert(TAIL_DEPTH > 1); ctx->frames[1].identity = 1; break;
    case 23: ctx->steps = 0; break;
    case 24: ctx->steps = ctx->step_limit + 1; break;
    default: break;
    }
#endif
    memset(SourceStack, 0xbe, 4096);
    require(topal_x86_tail_complete_call_v1(PrivateExtent, SourceStack, NativeHistory, NativeOutput) == 0);
    assert(zero(PrivateExtent, TAIL_TEMP_BYTES) && zero(SourceStack, 4096));
    assert(!topal_x86_tail_boundary_active_v1);
    assert(!memcmp(&expected, NativeOutput, sizeof(expected)));
    before.active = 0;
    assert(!memcmp(&before, NativeHistory, sizeof(before)));
    ++Completions;
}
#endif
static void run_case(uint64_t count, uint64_t offset, uint64_t owner_size,
                     uint64_t magnitude, uint64_t sign, uint64_t fail_copy,
                     uint64_t fail_allocation, uint64_t counter_maximum, uint64_t next) {
    Guarded owner = guarded((size_t)owner_size), private = guarded(65536);
    memset(owner.bytes, 0x7b, owner.count); memset(private.bytes, 0, private.count);
    if (TAIL_TEST_KIND == 2) {
        size_t page = (size_t)sysconf(_SC_PAGESIZE);
        assert(!mprotect(owner.mapping + page, owner.mapped - 2 * page, PROT_READ));
    }
    PrivateExtent = private.bytes;
    TailContext *ctx = (TailContext *)(private.bytes + TAIL_PRIVATE_OFFSET);
#ifdef TAIL_BOUNDARY_TEST
    Guarded source_stack = guarded(4096);
    SourceStack = source_stack.bytes;
    assert(!topal_x86_tail_boundary_active_v1);
    memset(private.bytes + TAIL_TEMP_BYTES, 0xd7, 65536 - TAIL_TEMP_BYTES);
#endif
#ifdef TAIL_NATIVE_HOOKS
    Guarded history_mapping, output_mapping;
    if (sign) {
        output_mapping = guarded(sizeof(TailOutput));
        history_mapping = guarded(sizeof(History));
    } else {
        history_mapping = guarded(sizeof(History));
        output_mapping = guarded(sizeof(TailOutput));
    }
    History *history = (History *)history_mapping.bytes;
#else
    History *history = malloc(sizeof(*history)); assert(history);
#endif
    *history = (History){.frame=next, .loan=next};
#ifdef TAIL_NATIVE_HOOKS
    NativeHistory = history;
    history->generation = 7;
    NativeOutput = (TailOutput *)output_mapping.bytes;
    memset(NativeOutput, 0, sizeof(*NativeOutput));
#endif
    History before = *history;
    /* Caller words live outside all erased extents and remain immutable. */
    const uint64_t caller_copy[] = {count, offset, magnitude, sign, 0x9876543210abcdef};
    Guarded caller_frame = guarded((size_t)sysconf(_SC_PAGESIZE));
    memcpy(caller_frame.bytes, caller_copy, sizeof(caller_copy));
    assert(!mprotect(caller_frame.bytes, caller_frame.count, PROT_READ));
    const uint64_t *caller = (const uint64_t *)caller_frame.bytes;
    ctx->history = history; ctx->depth = 1; ctx->bank = 0;
    ctx->frames[0].function = TAIL_ENTRY_ID; ctx->frames[0].identity = 1;
    ctx->frames[0].loan = TailHasRegion[TAIL_ENTRY_ID] ? 1 : 0;
    uint64_t writable = TAIL_TEST_KIND != 2;
    ctx->control = (Control){(void *)topal_tail_step_component_v1, 41, 7, 1, owner.bytes, owner.count, writable, 1};
#ifdef TAIL_NATIVE_HOOKS
    ctx->control.owner = (uintptr_t)history;
#endif
    ctx->frames[0].view = (View){&ctx->control, 0, owner.count, writable, 7, 41, (void *)topal_tail_step_component_v1, 0};
#ifdef TAIL_NATIVE_HOOKS
    ctx->frames[0].view.owner = (uintptr_t)history;
#endif
#ifndef TAIL_NATIVE_HOOKS
    ctx->fail_copy = fail_copy; ctx->fail_allocation = fail_allocation;
#else
    assert(!fail_copy && !fail_allocation);
#endif
    for (unsigned index = 0; index < TAIL_PARAMETERS; ++index) {
        unsigned char *root = ctx->banks[0] + TailRoots[index].offset;
        if (TailRoots[index].kind == 4) continue;
        if (TailRoots[index].kind == 3) { uint64_t value = 1; memcpy(root, &value, 8); continue; }
        uint64_t value = index == TAIL_MEASURE_PARAMETER ? count : magnitude;
        uint64_t negative = index == TAIL_MEASURE_PARAMETER ? 0 : sign;
        if (TailRoots[index].kind == 1) negative = 0;
        if (TAIL_TEST_KIND == 1 || TAIL_TEST_KIND == 2 || TAIL_TEST_KIND == 6) {
            if (index == 2) value = offset;
            if (index == 3) value = 1;
            negative = 0;
        }
        seed_number(root, value, negative);
    }
    assert(expected_mask() == TAIL_SCALAR_MASK);
#ifdef TAIL_BOUNDARY_TEST
    deny_geometry(ctx);
#ifdef TAIL_NATIVE_HOOKS
    deny_native_geometry(ctx, 0);
    deny_native_geometry(ctx, 1);
#endif
#endif
    uint64_t start_calls = Calls, start_fatal = FatalSeen;
    if (!admit(ctx, count, counter_maximum)) {
        assert(!memcmp(history, &before, sizeof(before)));
        verify_owner(&owner, count, offset, 0);
        assert(Calls == start_calls);
        erase(private.bytes, TAIL_TEMP_BYTES); ++Cases;
#ifdef TAIL_BOUNDARY_TEST
        release(source_stack);
#endif
#ifdef TAIL_NATIVE_HOOKS
        release(output_mapping);
        release(history_mapping);
#else
        free(history);
#endif
        release(caller_frame); release(owner); release(private); return;
    }
    Active = ctx;
#ifdef TAIL_NATIVE_HOOKS
    history->active = 1;
#endif
    if (!setjmp(Fatal)) {
        for (;;) {
#ifdef TAIL_BOUNDARY_TEST
            memset(SourceStack, 0xac, 4096);
#ifdef TAIL_NATIVE_HOOKS
            ++Calls;
#endif
            uint64_t outcome = topal_x86_tail_boundary_call_v1(PrivateExtent, SourceStack);
            assert(outcome <= 1 && !topal_x86_tail_boundary_active_v1);
            assert(zero(SourceStack, 4096) && zero(PrivateExtent, TAIL_PRIVATE_OFFSET));
#ifdef TAIL_NATIVE_HOOKS
            if (ctx->work_used > MaxWork) MaxWork = ctx->work_used;
#endif
#else
            uint64_t outcome = topal_tail_step_component_v1(ctx);
#endif
#ifdef TAIL_CONTROLLER_FAULT_TEST
            assert(!topal_x86_tail_boundary_active_v1);
            if (ControllerFault == 1) ctx->phase = 0;
            if (ControllerFault == 2) ctx->staged = 0;
            if (ControllerFault == 3) {
                /* Header/context fields cannot authorize or prevent cleanup. */
                memset(PrivateExtent, 0xee, TAIL_PRIVATE_OFFSET);
                require(0);
            }
            if (ControllerFault == 5) ctx->result_kind = 0;
#endif
            if (outcome) {
                const unsigned char *bank = ctx->banks[1 - ctx->bank];
                assert(signed_result(bank + TailRoots[TAIL_MEASURE_PARAMETER].offset) == (__int128)count - (__int128)ctx->steps * TAIL_MIN_DECREMENT);
                if (TAIL_TEST_KIND == 1 || TAIL_TEST_KIND == 2) {
                    assert(signed_result(bank + TailRoots[2].offset) == (__int128)offset + ctx->steps);
                    uint64_t available = offset < owner_size ? owner_size - offset : 0;
                    assert(signed_result(bank + TailRoots[3].offset) == (ctx->steps <= available));
                } else if (TAIL_TEST_KIND == 0) {
                    __int128 expected = (sign ? -(__int128)magnitude : (__int128)magnitude) - 3 * (__int128)ctx->steps;
                    assert(signed_result(bank + TailRoots[1].offset) == expected);
                } else if (TAIL_TEST_KIND == 6) {
                    assert(signed_result(bank + TailRoots[2].offset) == (__int128)offset);
                    assert(signed_result(bank + TailRoots[3].offset) == (__int128)offset);
                } else if (TAIL_TEST_KIND == 3) {
                    uint64_t value; memcpy(&value, bank + TailRoots[1].offset, 8); assert(value == 1);
                } else if (TAIL_TEST_KIND == 4 || TAIL_TEST_KIND == 5) {
                    const unsigned char *number = bank + TailRoots[1].offset;
                    uint64_t negative, length; uint32_t high;
                    memcpy(&negative, number, 8); memcpy(&length, number + 8, 8); memcpy(&high, number + 32, 4);
                    assert(negative == (TAIL_TEST_KIND == 5) && length == 5 && high == UINT32_C(0x80000000));
                    assert(zero(number + 16, 16));
                }
            }
#if defined(TAIL_CONTROLLER_FAULT_TEST) && defined(TAIL_NATIVE_HOOKS)
            damage_checkpoint(ctx, &outcome);
#endif
            checkpoint(ctx, outcome);
#ifdef TAIL_CONTROLLER_FAULT_TEST
            assert(ControllerFault == 4);
            require(0);
#endif
            assert(!memcmp(caller, caller_copy, sizeof(caller_copy)));
            if (!outcome) break;
        }
        assert(!fail_copy && !fail_allocation);
        uint64_t difference = count > TAIL_BASE_BOUND ? count - TAIL_BASE_BOUND : 0;
        uint64_t edges = difference / TAIL_MIN_DECREMENT + !!(difference % TAIL_MIN_DECREMENT);
        assert(ctx->steps == edges + 1); /* These fixtures have one exact decrement. */
        uint64_t available = offset < owner_size ? owner_size - offset : 0;
        if (TAIL_TEST_KIND == 1 || TAIL_TEST_KIND == 2) {
            uint64_t accepted; memcpy(&accepted, ctx->result, 8);
            assert(ctx->result_kind == 3 && accepted == (count <= available));
            uint64_t store_calls = count <= available ? count : available + 1;
            assert(history->frame == next + 2 * count + store_calls);
            assert(history->loan == next + 2 * count + store_calls);
        } else if (TAIL_TEST_KIND == 0) {
            __int128 expected = (sign ? -(__int128)magnitude : (__int128)magnitude) - 3 * (__int128)count;
            assert(ctx->result_kind == 2 && signed_result(ctx->result) == expected);
            assert(history->frame == next + count && history->loan == next);
        } else if (TAIL_TEST_KIND == 4 || TAIL_TEST_KIND == 5) {
            assert(ctx->result_kind == 2);
            if (!count) {
                assert(signed_result(ctx->result) == (sign ? -(__int128)magnitude : (__int128)magnitude));
            } else {
                uint64_t result_sign, length; uint32_t high;
                memcpy(&result_sign, ctx->result, 8); memcpy(&length, ctx->result + 8, 8);
                memcpy(&high, ctx->result + 32, 4);
                assert(result_sign == (TAIL_TEST_KIND == 5) && length == 5 && high == UINT32_C(0x80000000));
                assert(zero(ctx->result + 16, 16));
            }
            assert(history->frame == next + count && history->loan == next);
        } else if (TAIL_TEST_KIND == 6) {
            uint64_t accepted; memcpy(&accepted, ctx->result, 8);
            assert(ctx->result_kind == 3 && accepted == (count > 0 || offset == 1));
            assert(history->frame == next + count && history->loan == next + count);
        } else if (TAIL_TEST_KIND == 7) {
            assert(ctx->result_kind == 4 && zero(ctx->result, sizeof(ctx->result)));
            assert(history->frame == next + count && history->loan == next);
        } else if (TAIL_TEST_KIND == 8) {
            uint64_t accepted; memcpy(&accepted, ctx->result, 8);
            assert(ctx->result_kind == 3 && accepted == 1);
            assert(history->frame == next + count + 1 && history->loan == next + count);
        } else if (TAIL_TEST_KIND == 9) {
            uint64_t accepted; memcpy(&accepted, ctx->result, 8);
            assert(ctx->result_kind == 3 && accepted == 1);
            assert(history->frame == next + edges && history->loan == next);
        } else {
            uint64_t accepted; memcpy(&accepted, ctx->result, 8);
            assert(ctx->result_kind == 3 && accepted == 1);
            assert(history->frame == next + count && history->loan == next);
        }
        verify_owner(&owner, count, offset, count);
#ifdef TAIL_NATIVE_HOOKS
        complete_invocation(ctx);
        Active = NULL;
        assert(!history->poisoned);
#else
        erase(private.bytes, TAIL_TEMP_BYTES); Active = NULL;
#endif
    } else {
        assert(FatalSeen == start_fatal + 1 && !Active);
        assert(history->frame >= next && history->loan >= next);
        uint64_t stores = fail_copy ? ((fail_copy - 1) / 3 + 1) : 0;
#ifdef TAIL_BAD_OUTCOME_TEST
        stores = 1;
#endif
#ifdef TAIL_NATIVE_SOURCE_FAULT_TEST
        stores = TAIL_NATIVE_SOURCE_FAULT_TEST == 2 || TAIL_NATIVE_SOURCE_FAULT_TEST >= 6;
        uint64_t consumed = TAIL_NATIVE_SOURCE_FAULT_TEST == 10 ? 3 : 2 * stores;
        assert(history->frame == next + consumed && history->loan == next + consumed);
#endif
#ifdef TAIL_CONTROLLER_FAULT_TEST
        stores = terminal_controller_fault() ? 0 : 1;
        uint64_t consumed = terminal_controller_fault() ? 0 : (ControllerFault == 4 ? 3 : 2);
        assert(history->frame == next + consumed && history->loan == next + consumed);
#endif
#ifdef TAIL_NATIVE_COMPLETION_FAULT_TEST
        stores = count;
        assert(history->frame == next + 3 * count && history->loan == next + 3 * count);
#endif
        if (stores > count) stores = count;
        verify_owner(&owner, count, offset, stores);
    }
    assert(zero(private.bytes, TAIL_TEMP_BYTES));
#ifdef TAIL_BOUNDARY_TEST
    assert(!topal_x86_tail_boundary_active_v1);
    for (size_t i = TAIL_TEMP_BYTES; i < 65536; ++i) assert(private.bytes[i] == 0xd7);
    release(source_stack);
#endif
    assert(!memcmp(caller, caller_copy, sizeof(caller_copy)));
    ++Cases;
#ifdef TAIL_NATIVE_HOOKS
    release(output_mapping);
    release(history_mapping);
#else
    free(history);
#endif
    release(caller_frame); release(owner); release(private);
}

int main(void) {
#ifdef TAIL_CONTROLLER_FAULT_TEST
    invalid_controller_geometry();
    for (ControllerFault = 1; ControllerFault <= TAIL_CONTROLLER_CASES; ++ControllerFault)
        run_case(terminal_controller_fault() ? 0 : 4, 5, 32, 0, 0, 0, 0, UINT64_MAX, 2);
#elif defined(TAIL_BAD_OUTCOME_TEST) || defined(TAIL_NATIVE_SOURCE_FAULT_TEST) || defined(TAIL_NATIVE_COMPLETION_FAULT_TEST)
    run_case(4, 5, 32, 0, 0, 0, 0, UINT64_MAX, 2);
#else
    const uint64_t initials[] = {0, 1, 2, 3, 7, 31, 64, 257};
    for (unsigned i = 0; i < sizeof(initials) / sizeof(initials[0]); ++i) {
        uint64_t n = initials[i];
        for (unsigned sign = 0; sign <= 1; ++sign)
            run_case(n, 5, n + 16, sign ? UINT64_MAX : 0, sign, 0, 0, UINT64_MAX, 2);
    }
    if (TAIL_TEST_KIND == 1 || TAIL_TEST_KIND == 2) {
        run_case(70000, 11, 70032, 0, 0, 0, 0, UINT64_MAX, 2);
        run_case(5, 7, 9, 0, 0, 0, 0, UINT64_MAX, 2);
        run_case(5, UINT64_MAX, 9, 0, 0, 0, 0, UINT64_MAX, 2);
        if (TAIL_TEST_KIND == 1) {
#ifndef TAIL_NATIVE_HOOKS
            for (uint64_t copy = 1; copy <= 13; ++copy)
                run_case(4, 5, 32, 0, 0, copy, 0, UINT64_MAX, 2);
            /* Both arithmetic operands precede the first byte store. */
            for (uint64_t allocation = 1; allocation <= 2; ++allocation)
                run_case(4, 5, 32, 0, 0, 0, allocation, UINT64_MAX, 2);
#endif
        }
    }
    /* Nonunit descent can admit UINT64_MAX without counter overflow. Keep
     * this fixture a bounded admission refusal, rather than executing it. */
    run_case(UINT64_MAX, 0, 8, 0, 0, 0, 0,
             TAIL_TEST_KIND == 9 && TAIL_BASE_BOUND != UINT64_MAX ? 8 : UINT64_MAX, 2);
    run_case(4, 0, 8, 0, 0, 0, 0, 8, 2);
    run_case(1, 0, 8, 0, 0, 0, 0, UINT64_MAX, UINT64_MAX - 1);
#endif
    printf("TAIL_STEP_NATIVE_PASS cases=%" PRIu64 " calls=%" PRIu64 " checkpoints=%" PRIu64
           " fatal=%" PRIu64 " work=%" PRIu64 " private=%zu\n", Cases, Calls, Checkpoints,
           FatalSeen, MaxWork, sizeof(TailContext));
#ifdef TAIL_BOUNDARY_TEST
    printf("TAIL_BOUNDARY_PASS denials=%" PRIu64 " reentry=%" PRIu64 " temp=65536 stack=4096\n", Denials, ReentryChecks);
#endif
#ifdef TAIL_CONTROLLER_FAULT_TEST
    printf("TAIL_CONTROLLER_FAILURE_PASS admitted=%u invalid=%" PRIu64 "\n", TAIL_CONTROLLER_CASES, InvalidFatal);
#endif
#ifdef TAIL_NATIVE_HOOKS
    printf("TAIL_COMPLETION_PASS returns=%" PRIu64 " output=56\n", Completions);
#endif
    return 0;
}
