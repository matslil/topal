; TOPAL-SYSTEMS-TAIL-CHECKPOINT-001. Runs after source return on the reused stack.
@p.decrements = private constant [{{DECREMENT_COUNT}} x i64] [{{DECREMENT_TABLE}}], align 8

define internal void @p.check_number(ptr %number, i64 %extent, i64 %maximum, i64 %kind) nounwind noinline {
entry:
  %bits = call i64 @p.bits(ptr %number, i64 %extent)
  %bounded = icmp ule i64 %bits, %maximum
  call void @p.require(i1 %bounded)
  %sign = call i64 @p.word(ptr %number, i64 0)
  %nat = icmp eq i64 %kind, 1
  %positive = icmp eq i64 %sign, 0
  %not.nat = xor i1 %nat, true
  %classifier = or i1 %not.nat, %positive
  call void @p.require(i1 %classifier)
  %length = call i64 @p.word(ptr %number, i64 8)
  %limbs = mul i64 %length, 4
  %bytes = add i64 %limbs, 16
  %padding.bytes = sub i64 %extent, %bytes
  %padding = getelementptr i8, ptr %number, i64 %bytes
  %clean = call i1 @p.zero(ptr %padding, i64 %padding.bytes)
  call void @p.require(i1 %clean)
  ret void
}
define internal void @p.check_bank(ptr %context, ptr %bank) nounwind noinline {
entry:
  br label %loop
loop:
  %index = phi i64 [ 0, %entry ], [ %next, %advance ]
  %end = icmp eq i64 %index, {{PARAMETERS}}
  br i1 %end, label %done, label %root
root:
  %descriptor = call ptr @p.root(i64 %index)
  %kind = call i64 @p.word(ptr %descriptor, i64 0)
  %region = icmp eq i64 %kind, 4
  br i1 %region, label %advance, label %value
value:
  %offset = call i64 @p.word(ptr %descriptor, i64 8)
  %pointer = getelementptr i8, ptr %bank, i64 %offset
  %boolean = icmp eq i64 %kind, 3
  br i1 %boolean, label %bool, label %number
bool:
  %word = call i64 @p.word(ptr %pointer, i64 0)
  %valid.bool = icmp ule i64 %word, 1
  call void @p.require(i1 %valid.bool)
  br label %advance
number:
  %bytes = call i64 @p.word(ptr %descriptor, i64 16)
  %maximum = call i64 @p.word(ptr %descriptor, i64 24)
  call void @p.check_number(ptr %pointer, i64 %bytes, i64 %maximum, i64 %kind)
  br label %advance
advance:
  %next = add i64 %index, 1
  br label %loop
done:
  ret void
}
define internal i64 @p.measure(ptr %bank) nounwind noinline {
entry:
  %descriptor = call ptr @p.root(i64 {{MEASURE_PARAMETER}})
  %offset = call i64 @p.word(ptr %descriptor, i64 8)
  %extent = call i64 @p.word(ptr %descriptor, i64 16)
  %number = getelementptr i8, ptr %bank, i64 %offset
  call void @p.check_number(ptr %number, i64 %extent, i64 64, i64 1)
  %length = call i64 @p.word(ptr %number, i64 8)
  %zero = icmp eq i64 %length, 0
  br i1 %zero, label %zero.value, label %low
zero.value:
  ret i64 0
low:
  %low.pointer = getelementptr i8, ptr %number, i64 16
  %low.limb = load i32, ptr %low.pointer, align 4
  %low.word = zext i32 %low.limb to i64
  %one = icmp eq i64 %length, 1
  br i1 %one, label %one.value, label %two
one.value:
  ret i64 %low.word
two:
  %high.pointer = getelementptr i8, ptr %number, i64 20
  %high.limb = load i32, ptr %high.pointer, align 4
  %high.word = zext i32 %high.limb to i64
  %shifted = shl i64 %high.word, 32
  %value = or i64 %shifted, %low.word
  ret i64 %value
}
define internal i1 @p.decrement(i64 %difference) nounwind noinline {
entry:
  br label %loop
loop:
  %index = phi i64 [ 0, %entry ], [ %next, %advance ]
  %end = icmp eq i64 %index, {{DECREMENT_COUNT}}
  br i1 %end, label %no, label %test
test:
  %pointer = getelementptr [{{DECREMENT_COUNT}} x i64], ptr @p.decrements, i64 0, i64 %index
  %value = load i64, ptr %pointer, align 8
  %same = icmp eq i64 %value, %difference
  br i1 %same, label %yes, label %advance
advance:
  %next = add i64 %index, 1
  br label %loop
yes:
  ret i1 true
no:
  ret i1 false
}
define i64 @topal_tail_checkpoint_body_v1(ptr %provided, i64 %outcome) nounwind noinline {
entry:
  %context = call ptr @p.context_role(ptr %provided, ptr @topal_tail_checkpoint_body_v1)
  %valid.outcome = icmp ule i64 %outcome, 1
  call void @p.require(i1 %valid.outcome)
  %phase = call i64 @p.word(ptr %context, i64 {{PHASE}})
  %returned = icmp eq i64 %phase, 1
  call void @p.require(i1 %returned)
  call void @p.is_root(ptr %context)
  %frame = call ptr @p.current(ptr %context)
  call void @p.frame_identity(ptr %context, ptr %frame)
  %function = call i64 @p.word(ptr %frame, i64 0)
  %entry.function = icmp eq i64 %function, {{ENTRY}}
  call void @p.require(i1 %entry.function)
  %region = call i1 @p.has_region(i64 %function)
  br i1 %region, label %check.view, label %check.helpers
check.view:
  %view = getelementptr i8, ptr %frame, i64 24
  call void @p.validate_current_view(ptr %context, ptr %view)
  br label %check.helpers
check.helpers:
  %helpers = getelementptr i8, ptr %context, i64 152
  %helpers.zero = call i1 @p.zero(ptr %helpers, i64 {{RETIRED_HELPER_BYTES}})
  call void @p.require(i1 %helpers.zero)
  %used = call i64 @p.word(ptr %context, i64 {{USED}})
  %used.bounded = icmp ule i64 %used, {{WORK_BYTES}}
  %used.alignment = and i64 %used, 7
  %used.aligned = icmp eq i64 %used.alignment, 0
  %used.valid = and i1 %used.bounded, %used.aligned
  call void @p.require(i1 %used.valid)
  %bank.index = call i64 @p.word(ptr %context, i64 {{BANK}})
  %other.index = xor i64 %bank.index, 1
  %old.bank = call ptr @p.bank(ptr %context, i64 %bank.index)
  %new.bank = call ptr @p.bank(ptr %context, i64 %other.index)
  %staged = call i64 @p.word(ptr %context, i64 {{STAGED}})
  %result.kind = call i64 @p.word(ptr %context, i64 {{RESULT_KIND}})
  %history = call ptr @p.history(ptr %context)
  %successor = icmp eq i64 %outcome, 1
  br i1 %successor, label %next.state, label %terminal
next.state:
  %complete = icmp eq i64 %staged, {{SCALAR_MASK}}
  %no.result = icmp eq i64 %result.kind, 0
  %state.valid = and i1 %complete, %no.result
  call void @p.require(i1 %state.valid)
  call void @p.check_bank(ptr %context, ptr %new.bank)
  %old.measure = call i64 @p.measure(ptr %old.bank)
  %new.measure = call i64 @p.measure(ptr %new.bank)
  %above.base = icmp ugt i64 %old.measure, {{BASE_WORD}}
  %decreases = icmp ult i64 %new.measure, %old.measure
  %natural.progress = and i1 %above.base, %decreases
  call void @p.require(i1 %natural.progress)
  %difference = sub i64 %old.measure, %new.measure
  %admitted.decrement = call i1 @p.decrement(i64 %difference)
  call void @p.require(i1 %admitted.decrement)
  %identity = call i64 @p.word(ptr %history, i64 0)
  %frame.end = call i64 @p.word(ptr %context, i64 {{FRAME_END}})
  %step.frame.end = call i64 @p.word(ptr %context, i64 {{STEP_FRAME_END}})
  %global.frame.budget = icmp ult i64 %identity, %frame.end
  %step.frame.budget = icmp ult i64 %identity, %step.frame.end
  %frame.budget = and i1 %global.frame.budget, %step.frame.budget
  call void @p.require(i1 %frame.budget)
  %next.identity = call i64 @p.add(i64 %identity, i64 1)
  br i1 %region, label %next.loan, label %no.loan
next.loan:
  %loan = call i64 @p.word(ptr %history, i64 8)
  %loan.end = call i64 @p.word(ptr %context, i64 {{LOAN_END}})
  %step.loan.end = call i64 @p.word(ptr %context, i64 {{STEP_LOAN_END}})
  %global.loan.budget = icmp ult i64 %loan, %loan.end
  %step.loan.budget = icmp ult i64 %loan, %step.loan.end
  %loan.budget = and i1 %global.loan.budget, %step.loan.budget
  call void @p.require(i1 %loan.budget)
  %next.loan.identity = call i64 @p.add(i64 %loan, i64 1)
  br label %retire
no.loan:
  br label %retire
terminal:
  %nothing.staged = icmp eq i64 %staged, 0
  %expected.result = icmp eq i64 %result.kind, {{EXPECTED_RESULT_KIND}}
  %terminal.valid = and i1 %nothing.staged, %expected.result
  call void @p.require(i1 %terminal.valid)
  %empty.next = call i1 @p.zero(ptr %new.bank, i64 {{BANK_BYTES}})
  call void @p.require(i1 %empty.next)
  %result = getelementptr i8, ptr %context, i64 {{RESULT}}
  %numeric.result = icmp eq i64 %result.kind, 2
  br i1 %numeric.result, label %result.number, label %result.other
result.number:
  call void @p.check_number(ptr %result, i64 48, i64 {{OPERAND_BITS}}, i64 {{RESULT_NUMBER_KIND}})
  br label %retire
result.other:
  %boolean.result = icmp eq i64 %result.kind, 3
  br i1 %boolean.result, label %result.bool, label %result.unit
result.bool:
  %boolean = call i64 @p.word(ptr %result, i64 0)
  %boolean.valid = icmp ule i64 %boolean, 1
  call void @p.require(i1 %boolean.valid)
  %result.padding = getelementptr i8, ptr %result, i64 8
  %padding.zero = call i1 @p.zero(ptr %result.padding, i64 40)
  call void @p.require(i1 %padding.zero)
  br label %retire
result.unit:
  %unit.zero = call i1 @p.zero(ptr %result, i64 48)
  call void @p.require(i1 %unit.zero)
  br label %retire
retire:
  %fresh.frame = phi i64 [ %identity, %next.loan ], [ %identity, %no.loan ], [ 0, %result.number ], [ 0, %result.bool ], [ 0, %result.unit ]
  %fresh.next.frame = phi i64 [ %next.identity, %next.loan ], [ %next.identity, %no.loan ], [ 0, %result.number ], [ 0, %result.bool ], [ 0, %result.unit ]
  %fresh.loan = phi i64 [ %loan, %next.loan ], [ 0, %no.loan ], [ 0, %result.number ], [ 0, %result.bool ], [ 0, %result.unit ]
  %fresh.next.loan = phi i64 [ %next.loan.identity, %next.loan ], [ 0, %no.loan ], [ 0, %result.number ], [ 0, %result.bool ], [ 0, %result.unit ]
  %work = getelementptr i8, ptr %context, i64 {{WORK}}
  %starts = getelementptr i8, ptr %context, i64 {{STARTS}}
  call void @p.erase(ptr %work, i64 {{WORK_BYTES}})
  call void @p.erase(ptr %starts, i64 {{START_BYTES}})
  call void @p.erase(ptr %old.bank, i64 {{BANK_BYTES}})
  br i1 %successor, label %publish.frame, label %finish
publish.frame:
  call void @p.put(ptr %frame, i64 8, i64 %fresh.frame)
  call void @p.put(ptr %history, i64 0, i64 %fresh.next.frame)
  br i1 %region, label %publish.loan, label %finish
publish.loan:
  call void @p.put(ptr %frame, i64 16, i64 %fresh.loan)
  call void @p.put(ptr %history, i64 8, i64 %fresh.next.loan)
  br label %finish
finish:
  call void @p.put(ptr %context, i64 {{BANK}}, i64 %other.index)
  call void @p.put(ptr %context, i64 {{USED}}, i64 0)
  call void @p.put(ptr %context, i64 {{STAGED}}, i64 0)
  call void @p.put(ptr %context, i64 {{PHASE}}, i64 0)
  ret i64 %outcome
}
