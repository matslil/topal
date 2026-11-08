/* TK-DEC-282, TOPAL-SYSTEMS-TAIL-STEP-001.
 * Independent guarded test hooks. This C bridge does not qualify private
 * kernel stack/register reclamation or grant artifact-publication authority. */
#include <assert.h>
#include <inttypes.h>
#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifndef TAIL_TEMP_BYTES
#define TAIL_TEMP_BYTES 65536
#define TAIL_PRIVATE_OFFSET 0
#endif
#ifdef TAIL_BOUNDARY_TEST
/* Integer-only callback memory operations; no host libc SIMD on source stack. */
static void *tail_test_copy(void *destination, const void *source, size_t bytes) {
    volatile unsigned char *to = destination;
    const volatile unsigned char *from = source;
    for (size_t i = 0; i < bytes; ++i) to[i] = from[i];
    return destination;
}
static void *tail_test_fill(void *destination, int value, size_t bytes) {
    volatile unsigned char *to = destination;
    for (size_t i = 0; i < bytes; ++i) to[i] = (unsigned char)value;
    return destination;
}
static int tail_test_compare(const void *left, const void *right, size_t bytes) {
    const volatile unsigned char *a = left, *b = right;
    for (size_t i = 0; i < bytes; ++i) if (a[i] != b[i]) return (int)a[i] - (int)b[i];
    return 0;
}
#define memcpy tail_test_copy
#define memset tail_test_fill
#define memcmp tail_test_compare
extern uint64_t topal_x86_tail_boundary_call_v1(void *, void *);
extern void topal_x86_tail_boundary_abort_v1(void) __attribute__((noreturn));
extern void topal_x86_tail_boundary_fail_v1(void *, void *) __attribute__((noreturn));
extern uintptr_t topal_x86_tail_boundary_active_v1;
static unsigned char *SourceStack;
#endif

typedef struct {
    void *code;
    uint64_t owner, epoch, active;
    unsigned char *base;
    uint64_t count, writable, windows;
} Control;
typedef struct {
    Control *control;
    uint64_t offset, count, writable, epoch, owner;
    void *code;
    uint64_t zero;
} View;
typedef struct {
    uint64_t function, identity, loan;
    View view;
} Frame;
typedef struct {
    uint64_t frame, loan;
#ifdef TAIL_NATIVE_HOOKS
    uint64_t generation, active, poisoned;
#endif
} History;
typedef struct {
    Control control;
    Frame frames[TAIL_DEPTH];
    History *history;
    uint64_t frame_end, loan_end;
    uint64_t depth, bank, phase, staged, work_used, steps;
#ifndef TAIL_NATIVE_HOOKS
    uint64_t allocations;
    uint64_t fail_allocation, fail_copy, copies;
#endif
#ifdef TAIL_NATIVE_HOOKS
    uint64_t step_limit, step_frame_end, step_loan_end;
#endif
    _Alignas(16) unsigned char banks[2][TAIL_BANK_BYTES];
    _Alignas(16) unsigned char work[TAIL_WORK_BYTES];
    unsigned char starts[(TAIL_WORK_BYTES + 63) / 64];
    _Alignas(8) unsigned char result[48];
    uint64_t result_kind;
} TailContext;
_Static_assert(sizeof(Control) == 64 && sizeof(View) == 64, "private byte view layout");
_Static_assert(sizeof(TailContext) + TAIL_PRIVATE_OFFSET <= TAIL_TEMP_BYTES, "complete test context must fit existing reserve");

extern uint64_t topal_tail_step_component_v1(void *);
extern uint64_t topal_tail_readonly_bytes_v1(const void *);
static TailContext *Active;
static jmp_buf Fatal;
static uint64_t FatalSeen, Calls, Checkpoints, MaxWork;
#ifdef TAIL_BOUNDARY_TEST
static uint64_t ReentryChecks;
#endif
#ifdef TAIL_CONTROLLER_FAULT_TEST
#ifdef TAIL_NATIVE_HOOKS
#define TAIL_CONTROLLER_CASES 26
#else
#define TAIL_CONTROLLER_CASES 5
#endif
static uint64_t InvalidController, InvalidFatal;
static uintptr_t InvalidExpectedActive;
#endif
static unsigned char *PrivateExtent;
#ifdef TAIL_NATIVE_HOOKS
static History *NativeHistory;
typedef struct { unsigned char value[48]; uint64_t kind; } TailOutput;
static TailOutput *NativeOutput;
static uint64_t Completions;
extern void topal_tail_abort_v1(void) __attribute__((noreturn));
extern uint64_t topal_x86_tail_checkpoint_call_v1(void *, void *, History *, uint64_t);
extern uint64_t topal_x86_tail_complete_call_v1(void *, void *, History *, TailOutput *);
_Static_assert(sizeof(TailOutput) == 56, "native caller result frame");
_Static_assert(sizeof(TailContext) == TAIL_NATIVE_CONTEXT_BYTES, "complete native context layout");
_Static_assert(sizeof(Frame) == 88 && sizeof(History) == 40, "native frame/history extents");
_Static_assert(offsetof(TailContext, history) == TAIL_NATIVE_HISTORY, "native history offset");
_Static_assert(offsetof(TailContext, banks) == TAIL_NATIVE_BANKS, "native banks offset");
_Static_assert(offsetof(TailContext, work) == TAIL_NATIVE_WORK, "native work offset");
_Static_assert(offsetof(TailContext, starts) == TAIL_NATIVE_STARTS, "native provenance offset");
_Static_assert(offsetof(TailContext, result) == TAIL_NATIVE_RESULT, "native result offset");
_Static_assert(offsetof(TailContext, step_limit) == TAIL_NATIVE_STEP_LIMIT, "native admission offset");
#endif

static void erase(void *pointer, size_t bytes) {
    volatile unsigned char *p = pointer;
    while (bytes--) *p++ = 0;
}
static int zero(const void *pointer, size_t bytes) {
    const unsigned char *p = pointer;
    while (bytes--) if (*p++) return 0;
    return 1;
}
#ifndef TAIL_NATIVE_HOOKS
__attribute__((noreturn)) void topal_tail_abort_v1(void) {
#ifdef TAIL_BOUNDARY_TEST
    if (topal_x86_tail_boundary_active_v1) topal_x86_tail_boundary_abort_v1();
    topal_x86_tail_boundary_fail_v1(PrivateExtent, SourceStack);
#else
    ++FatalSeen;
    erase(PrivateExtent, TAIL_TEMP_BYTES);
    Active = NULL;
    longjmp(Fatal, 1);
#endif
}
#endif
#ifdef TAIL_BOUNDARY_TEST
__attribute__((noreturn)) void topal_tail_boundary_fatal_v1(void) {
#ifdef TAIL_CONTROLLER_FAULT_TEST
    if (InvalidController) {
        /* Outside the admitted controller contract: no unproven extent may be
         * erased, and the fatal entry still cannot return to an owner. */
        assert(topal_x86_tail_boundary_active_v1 == InvalidExpectedActive);
        for (size_t i = 0; i < TAIL_TEMP_BYTES; ++i) assert(PrivateExtent[i] == 0x5d);
        for (size_t i = 0; i < 4096; ++i) assert(SourceStack[i] == 0xbc);
        ++InvalidFatal;
        longjmp(Fatal, 1);
    }
#endif
    /* Independent observer after actual provider recovery and reclamation. */
#ifdef TAIL_NATIVE_HOOKS
    assert(NativeHistory && !NativeHistory->active && NativeHistory->poisoned == 1);
    assert(NativeOutput && zero(NativeOutput, sizeof(*NativeOutput)));
#endif
    assert(!topal_x86_tail_boundary_active_v1);
    assert(zero(PrivateExtent, TAIL_TEMP_BYTES) && zero(SourceStack, 4096));
    ++FatalSeen; Active = NULL; longjmp(Fatal, 1);
}
#endif
static void require(int condition) {
    if (condition) return;
#ifdef TAIL_NATIVE_HOOKS
    if (!topal_x86_tail_boundary_active_v1) {
        /* Independent controller owns these facts across source return. */
        NativeHistory->active = 0; NativeHistory->poisoned = 1;
        topal_x86_tail_boundary_fail_v1(PrivateExtent, SourceStack);
    }
#endif
    topal_tail_abort_v1();
}
#ifndef TAIL_NATIVE_HOOKS
static TailContext *context(void *pointer) {
    require(Active && pointer == Active);
    return Active;
}
static Frame *current(TailContext *ctx) {
    require(ctx->depth > 0 && ctx->depth <= TAIL_DEPTH);
    return &ctx->frames[ctx->depth - 1];
}
static int allocation_start(TailContext *ctx, uint64_t offset) {
    return (ctx->starts[offset / 64] >> ((offset / 8) % 8)) & 1;
}
void *topal_tail_allocate_v1(uint64_t bytes) {
    TailContext *ctx = context(Active);
    require(ctx->phase == 1 && bytes > 0 && bytes <= TAIL_WORK_BYTES);
    ++ctx->allocations;
    if (ctx->fail_allocation && ctx->allocations == ctx->fail_allocation) topal_tail_abort_v1();
    uint64_t aligned = (bytes + 7) & ~UINT64_C(7);
    require(ctx->work_used <= TAIL_WORK_BYTES && aligned <= TAIL_WORK_BYTES - ctx->work_used);
    uint64_t offset = ctx->work_used;
    ctx->starts[offset / 64] |= (unsigned char)(1u << ((offset / 8) % 8));
    memset(ctx->work + offset, 0xa5, (size_t)aligned);
    ctx->work_used += aligned;
    if (ctx->work_used > MaxWork) MaxWork = ctx->work_used;
    return ctx->work + offset;
}
static uint64_t origin_bytes(TailContext *ctx, const void *pointer) {
    uintptr_t address = (uintptr_t)pointer, work = (uintptr_t)ctx->work;
    if (address >= work && address < work + ctx->work_used) {
        uint64_t offset = address - work;
        require(!(offset & 7) && allocation_start(ctx, offset));
        uint64_t end = offset + 8;
        while (end < ctx->work_used && !allocation_start(ctx, end)) end += 8;
        return end - offset;
    }
    for (unsigned index = 0; index < TAIL_PARAMETERS; ++index) {
        if (TailRoots[index].kind == 1 || TailRoots[index].kind == 2) {
            if (pointer == ctx->banks[ctx->bank] + TailRoots[index].offset)
                return TailRoots[index].bytes;
        }
    }
    uint64_t bytes = topal_tail_readonly_bytes_v1(pointer);
    require(bytes >= 16 && bytes <= 48);
    return bytes;
}
static unsigned number_bits(const unsigned char *number, uint64_t extent) {
    uint64_t sign, length;
    require(extent >= 16 && !((uintptr_t)number & 7));
    memcpy(&sign, number, 8); memcpy(&length, number + 8, 8);
    require(sign <= 1 && length <= 8 && 16 + length * 4 <= extent);
    if (!length) { require(!sign); return 1; }
    uint32_t high; memcpy(&high, number + 16 + 4 * (length - 1), 4);
    require(high != 0);
    unsigned bits = (unsigned)(32 * (length - 1));
    do { ++bits; high >>= 1; } while (high);
    return bits;
}
static void copy_number(TailContext *ctx, unsigned char *destination, uint64_t capacity,
                        unsigned maximum_bits, unsigned kind, const void *source) {
    uint64_t extent = origin_bytes(ctx, source), length, sign;
    unsigned bits = number_bits(source, extent);
    memcpy(&length, (const unsigned char *)source + 8, 8);
    memcpy(&sign, source, 8);
    require(bits <= maximum_bits && 16 + 4 * length <= capacity && (kind != 1 || !sign));
    ++ctx->copies;
    if (ctx->fail_copy && ctx->copies == ctx->fail_copy) topal_tail_abort_v1();
    require(zero(destination, (size_t)capacity));
    memcpy(destination, source, (size_t)(16 + 4 * length));
}
void topal_tail_step_begin_v1(void *pointer) {
    TailContext *ctx = context(pointer);
    require(ctx->phase == 0 && !ctx->result_kind && ctx->depth == 1 && current(ctx)->function == TAIL_ENTRY_ID);
    require(zero(ctx->work, TAIL_WORK_BYTES) && zero(ctx->starts, sizeof(ctx->starts)));
    require(zero(ctx->banks[1 - ctx->bank], TAIL_BANK_BYTES));
    ctx->phase = 1; ctx->staged = 0; ctx->work_used = 0;
    ++ctx->steps; ++Calls;
}
void *topal_tail_input_number_v1(void *pointer, uint64_t parameter) {
    TailContext *ctx = context(pointer);
    require(ctx->phase == 1 && parameter < TAIL_PARAMETERS);
    require(TailRoots[parameter].kind == 1 || TailRoots[parameter].kind == 2);
    unsigned char *number = ctx->banks[ctx->bank] + TailRoots[parameter].offset;
    require(number_bits(number, TailRoots[parameter].bytes) <= TailRoots[parameter].bits);
    return number;
}
uint64_t topal_tail_input_boolean_v1(void *pointer, uint64_t parameter) {
    TailContext *ctx = context(pointer); uint64_t value;
    require(ctx->phase == 1 && parameter < TAIL_PARAMETERS && TailRoots[parameter].kind == 3);
    memcpy(&value, ctx->banks[ctx->bank] + TailRoots[parameter].offset, 8);
    require(value <= 1); return value;
}
void topal_tail_validate_view_v1(void *pointer, void *view) {
    TailContext *ctx = context(pointer);
    Frame *frame = current(ctx);
    require(ctx->phase == 1 && TailHasRegion[frame->function] && view == &frame->view);
    View *actual = view;
    require(actual->control == &ctx->control && actual->offset == 0 && actual->count == ctx->control.count);
    require(actual->writable == ctx->control.writable && actual->epoch == ctx->control.epoch);
    require(actual->owner == ctx->control.owner && actual->code == ctx->control.code && actual->zero == 0);
    require(frame->loan > 0 && frame->loan < ctx->history->loan);
}
void *topal_tail_input_view_v1(void *pointer) {
    TailContext *ctx = context(pointer);
    topal_tail_validate_view_v1(pointer, &current(ctx)->view);
    return &current(ctx)->view;
}
void *topal_tail_helper_enter_v1(void *pointer, uint64_t function, void *view) {
    TailContext *ctx = context(pointer); Frame *parent = current(ctx);
#ifdef TAIL_BOUNDARY_TEST
    if (!ReentryChecks) {
        unsigned char saved[64]; History before = *ctx->history;
        memcpy(saved, PrivateExtent, 64);
        uintptr_t active = topal_x86_tail_boundary_active_v1;
        assert(topal_x86_tail_boundary_call_v1(PrivateExtent, SourceStack) == 2);
        assert(active == topal_x86_tail_boundary_active_v1);
        assert(!memcmp(saved, PrivateExtent, 64) && !memcmp(&before, ctx->history, sizeof(before)));
        ++ReentryChecks;
    }
#endif
    require(ctx->phase == 1 && function > 0 && function <= TAIL_FUNCTIONS);
    require(TailCalls[parent->function][function] && ctx->depth < TAIL_DEPTH);
    require(ctx->history->frame < ctx->frame_end);
    View next = {0}; uint64_t loan = 0;
    if (TailHasRegion[function]) {
        topal_tail_validate_view_v1(pointer, view);
        require(ctx->history->loan < ctx->loan_end);
        next = *(View *)view;
        loan = ctx->history->loan++;
    } else require(!view);
    Frame *frame = &ctx->frames[ctx->depth++];
    require(zero(frame, sizeof(*frame)));
    frame->function = function; frame->identity = ctx->history->frame++;
    frame->loan = loan; frame->view = next;
    return TailHasRegion[function] ? &frame->view : NULL;
}
void topal_tail_helper_leave_v1(void *pointer, uint64_t function) {
    TailContext *ctx = context(pointer);
    require(ctx->depth > 1 && current(ctx)->function == function);
    erase(current(ctx), sizeof(Frame)); --ctx->depth;
}
static void stage_check(TailContext *ctx, uint64_t parameter) {
    require(ctx->phase == 1 && ctx->depth == 1 && parameter < TAIL_PARAMETERS);
    require(!(ctx->staged & (UINT64_C(1) << parameter)));
    ctx->staged |= UINT64_C(1) << parameter;
}
void topal_tail_stage_number_v1(void *pointer, uint64_t parameter, void *source) {
    TailContext *ctx = context(pointer); stage_check(ctx, parameter);
    require(TailRoots[parameter].kind == 1 || TailRoots[parameter].kind == 2);
    copy_number(ctx, ctx->banks[1 - ctx->bank] + TailRoots[parameter].offset,
                TailRoots[parameter].bytes, TailRoots[parameter].bits, TailRoots[parameter].kind, source);
}
void topal_tail_stage_boolean_v1(void *pointer, uint64_t parameter, uint64_t value) {
    TailContext *ctx = context(pointer); stage_check(ctx, parameter);
    require(TailRoots[parameter].kind == 3 && value <= 1);
    ++ctx->copies;
    if (ctx->fail_copy && ctx->copies == ctx->fail_copy) topal_tail_abort_v1();
    memcpy(ctx->banks[1 - ctx->bank] + TailRoots[parameter].offset, &value, 8);
}
void topal_tail_result_number_v1(void *pointer, void *source) {
    TailContext *ctx = context(pointer);
    require(ctx->phase == 1 && ctx->depth == 1 && !ctx->staged && !ctx->result_kind);
    copy_number(ctx, ctx->result, sizeof(ctx->result), TAIL_OPERAND_BITS, 2, source);
    ctx->result_kind = 2;
}
void topal_tail_result_boolean_v1(void *pointer, uint64_t value) {
    TailContext *ctx = context(pointer);
    require(ctx->phase == 1 && ctx->depth == 1 && !ctx->staged && !ctx->result_kind && value <= 1);
    ++ctx->copies;
    if (ctx->fail_copy && ctx->copies == ctx->fail_copy) topal_tail_abort_v1();
    memcpy(ctx->result, &value, 8); ctx->result_kind = 3;
}
void topal_tail_result_unit_v1(void *pointer) {
    TailContext *ctx = context(pointer);
    require(ctx->phase == 1 && ctx->depth == 1 && !ctx->staged && !ctx->result_kind);
    ctx->result_kind = 4;
}
#endif

/* The bridge is independently implemented; no source policy is translated into
 * this loop. The tests below supply separate source-result/effect oracles. */
static void checkpoint(TailContext *ctx, uint64_t successor) {
#ifdef TAIL_NATIVE_HOOKS
    unsigned bank=ctx->bank;
    memset(SourceStack,0xbd,4096);
    uint64_t outcome=topal_x86_tail_checkpoint_call_v1(PrivateExtent,SourceStack,NativeHistory,successor);
    require(outcome==successor && outcome<=1);
    assert(!topal_x86_tail_boundary_active_v1 && zero(SourceStack,4096));
    assert(zero(PrivateExtent,TAIL_PRIVATE_OFFSET));
    assert(zero(ctx->work,TAIL_WORK_BYTES) && zero(ctx->starts,sizeof(ctx->starts)));
    assert(zero(ctx->banks[bank],TAIL_BANK_BYTES));
    assert(ctx->bank==1-bank && !ctx->phase && !ctx->work_used && !ctx->staged);
    ++Checkpoints;
#else
    require(ctx->phase == 1 && ctx->depth == 1 && successor <= 1);
    require(zero(ctx->frames + 1, sizeof(Frame) * (TAIL_DEPTH - 1)));
    if (successor) {
        require(!ctx->result_kind && ctx->staged == TAIL_SCALAR_MASK);
        require(ctx->history->frame < ctx->frame_end);
#ifdef TAIL_NATIVE_HOOKS
        require(ctx->history->frame < ctx->step_frame_end);
#endif
        ctx->frames[0].identity = ctx->history->frame++;
        if (TailHasRegion[TAIL_ENTRY_ID]) {
            require(ctx->history->loan < ctx->loan_end);
#ifdef TAIL_NATIVE_HOOKS
            require(ctx->history->loan < ctx->step_loan_end);
#endif
            ctx->frames[0].loan = ctx->history->loan++;
        }
    } else require(ctx->result_kind && !ctx->staged);
    erase(ctx->work, (size_t)ctx->work_used);
    erase(ctx->starts, sizeof(ctx->starts));
    erase(ctx->banks[ctx->bank], TAIL_BANK_BYTES);
    assert(zero(ctx->work, TAIL_WORK_BYTES));
    assert(zero(ctx->banks[ctx->bank], TAIL_BANK_BYTES));
    ctx->bank = 1 - ctx->bank; ctx->work_used = 0; ctx->phase = 0;
    ++Checkpoints;
#endif
}
