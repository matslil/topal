; Source entry roots, successor staging and actual terminal-result transport.
define void @topal_tail_step_begin_v1(ptr %provided) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 0)
  call void @p.is_root(ptr %context)
  %frame = call ptr @p.current(ptr %context)
  call void @p.frame_identity(ptr %context, ptr %frame)
  %function = call i64 @p.word(ptr %frame, i64 0)
  %correct.entry = icmp eq i64 %function, {{ENTRY}}
  call void @p.require(i1 %correct.entry)
  %result.kind = call i64 @p.word(ptr %context, i64 {{RESULT_KIND}})
  %not.ended = icmp eq i64 %result.kind, 0
  call void @p.require(i1 %not.ended)
  %work = getelementptr i8, ptr %context, i64 {{WORK}}
  %work.zero = call i1 @p.zero(ptr %work, i64 {{WORK_BYTES}})
  call void @p.require(i1 %work.zero)
  %starts = getelementptr i8, ptr %context, i64 {{STARTS}}
  %starts.zero = call i1 @p.zero(ptr %starts, i64 {{START_BYTES}})
  call void @p.require(i1 %starts.zero)
  %bank.index = call i64 @p.word(ptr %context, i64 {{BANK}})
  %other.index = xor i64 %bank.index, 1
  %other = call ptr @p.bank(ptr %context, i64 %other.index)
  %other.zero = call i1 @p.zero(ptr %other, i64 {{BANK_BYTES}})
  call void @p.require(i1 %other.zero)
  %steps = call i64 @p.word(ptr %context, i64 {{STEPS}})
  %limit = call i64 @p.word(ptr %context, i64 {{STEP_LIMIT}})
  %remaining = icmp ult i64 %steps, %limit
  call void @p.require(i1 %remaining)
  %history = call ptr @p.history(ptr %context)
  %next.frame = call i64 @p.word(ptr %history, i64 0)
  %next.loan = call i64 @p.word(ptr %history, i64 8)
  %frames.valid = icmp uge i64 %next.frame, 2
  %loans.valid = icmp uge i64 %next.loan, 2
  %history.valid = and i1 %frames.valid, %loans.valid
  call void @p.require(i1 %history.valid)
  %frame.end = call i64 @p.add(i64 %next.frame, i64 {{FRAMES_PER_STEP}})
  %loan.end = call i64 @p.add(i64 %next.loan, i64 {{LOANS_PER_STEP}})
  %admitted.frame.end = call i64 @p.word(ptr %context, i64 {{FRAME_END}})
  %admitted.loan.end = call i64 @p.word(ptr %context, i64 {{LOAN_END}})
  %frame.budget = icmp ule i64 %frame.end, %admitted.frame.end
  %loan.budget = icmp ule i64 %loan.end, %admitted.loan.end
  %within.budget = and i1 %frame.budget, %loan.budget
  call void @p.require(i1 %within.budget)
  call void @p.put(ptr %context, i64 {{STEP_FRAME_END}}, i64 %frame.end)
  call void @p.put(ptr %context, i64 {{STEP_LOAN_END}}, i64 %loan.end)
  %next.step = call i64 @p.add(i64 %steps, i64 1)
  call void @p.put(ptr %context, i64 {{STEPS}}, i64 %next.step)
  call void @p.put(ptr %context, i64 {{STAGED}}, i64 0)
  call void @p.put(ptr %context, i64 {{USED}}, i64 0)
  call void @p.put(ptr %context, i64 {{PHASE}}, i64 1)
  ret void
}
define ptr @topal_tail_input_number_v1(ptr %provided, i64 %parameter) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %root = call ptr @p.root(i64 %parameter)
  %kind = call i64 @p.word(ptr %root, i64 0)
  %nat = icmp eq i64 %kind, 1
  %integer = icmp eq i64 %kind, 2
  %numeric = or i1 %nat, %integer
  call void @p.require(i1 %numeric)
  %index = call i64 @p.word(ptr %context, i64 {{BANK}})
  %bank = call ptr @p.bank(ptr %context, i64 %index)
  %offset = call i64 @p.word(ptr %root, i64 8)
  %number = getelementptr i8, ptr %bank, i64 %offset
  %extent = call i64 @p.word(ptr %root, i64 16)
  %maximum = call i64 @p.word(ptr %root, i64 24)
  %bits = call i64 @p.bits(ptr %number, i64 %extent)
  %bounded = icmp ule i64 %bits, %maximum
  call void @p.require(i1 %bounded)
  %sign = call i64 @p.word(ptr %number, i64 0)
  %nonnegative = icmp eq i64 %sign, 0
  %not.nat = xor i1 %nat, true
  %valid.nat = or i1 %not.nat, %nonnegative
  call void @p.require(i1 %valid.nat)
  ret ptr %number
}
define i64 @topal_tail_input_boolean_v1(ptr %provided, i64 %parameter) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %root = call ptr @p.root(i64 %parameter)
  %kind = call i64 @p.word(ptr %root, i64 0)
  %boolean = icmp eq i64 %kind, 3
  call void @p.require(i1 %boolean)
  %index = call i64 @p.word(ptr %context, i64 {{BANK}})
  %bank = call ptr @p.bank(ptr %context, i64 %index)
  %offset = call i64 @p.word(ptr %root, i64 8)
  %value = call i64 @p.word(ptr %bank, i64 %offset)
  %valid = icmp ule i64 %value, 1
  call void @p.require(i1 %valid)
  ret i64 %value
}
define internal ptr @p.stage(ptr %provided, i64 %parameter, i64 %kind) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  call void @p.is_root(ptr %context)
  %root = call ptr @p.root(i64 %parameter)
  %actual = call i64 @p.word(ptr %root, i64 0)
  %same = icmp eq i64 %actual, %kind
  call void @p.require(i1 %same)
  %result = call i64 @p.word(ptr %context, i64 {{RESULT_KIND}})
  %no.result = icmp eq i64 %result, 0
  call void @p.require(i1 %no.result)
  %mask = shl i64 1, %parameter
  %staged = call i64 @p.word(ptr %context, i64 {{STAGED}})
  %bits = and i64 %staged, %mask
  %new = icmp eq i64 %bits, 0
  call void @p.require(i1 %new)
  %updated = or i64 %staged, %mask
  call void @p.put(ptr %context, i64 {{STAGED}}, i64 %updated)
  %index = call i64 @p.word(ptr %context, i64 {{BANK}})
  %other = xor i64 %index, 1
  %bank = call ptr @p.bank(ptr %context, i64 %other)
  %offset = call i64 @p.word(ptr %root, i64 8)
  %destination = getelementptr i8, ptr %bank, i64 %offset
  ret ptr %destination
}
define void @topal_tail_stage_number_v1(ptr %provided, i64 %parameter, ptr %source) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %root = call ptr @p.root(i64 %parameter)
  %kind = call i64 @p.word(ptr %root, i64 0)
  %nat = icmp eq i64 %kind, 1
  %integer = icmp eq i64 %kind, 2
  %numeric = or i1 %nat, %integer
  call void @p.require(i1 %numeric)
  %destination = call ptr @p.stage(ptr %context, i64 %parameter, i64 %kind)
  %capacity = call i64 @p.word(ptr %root, i64 16)
  %bits = call i64 @p.word(ptr %root, i64 24)
  call void @p.copy_number(ptr %context, ptr %destination, i64 %capacity, i64 %bits, i64 %kind, ptr %source)
  ret void
}
define void @topal_tail_stage_boolean_v1(ptr %provided, i64 %parameter, i64 %value) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %valid = icmp ule i64 %value, 1
  call void @p.require(i1 %valid)
  %destination = call ptr @p.stage(ptr %context, i64 %parameter, i64 3)
  %empty = call i1 @p.zero(ptr %destination, i64 8)
  call void @p.require(i1 %empty)
  call void @p.put(ptr %destination, i64 0, i64 %value)
  ret void
}
define internal ptr @p.result(ptr %provided, i64 %kind) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  call void @p.is_root(ptr %context)
  %expected = icmp eq i64 %kind, {{EXPECTED_RESULT_KIND}}
  call void @p.require(i1 %expected)
  %staged = call i64 @p.word(ptr %context, i64 {{STAGED}})
  %no.staged = icmp eq i64 %staged, 0
  call void @p.require(i1 %no.staged)
  %old.kind = call i64 @p.word(ptr %context, i64 {{RESULT_KIND}})
  %not.finished = icmp eq i64 %old.kind, 0
  call void @p.require(i1 %not.finished)
  %destination = getelementptr i8, ptr %context, i64 {{RESULT}}
  %empty = call i1 @p.zero(ptr %destination, i64 48)
  call void @p.require(i1 %empty)
  ret ptr %destination
}
define void @topal_tail_result_number_v1(ptr %provided, ptr %source) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %destination = call ptr @p.result(ptr %context, i64 2)
  call void @p.copy_number(ptr %context, ptr %destination, i64 48, i64 {{OPERAND_BITS}}, i64 {{RESULT_NUMBER_KIND}}, ptr %source)
  call void @p.put(ptr %context, i64 {{RESULT_KIND}}, i64 2)
  ret void
}
define void @topal_tail_result_boolean_v1(ptr %provided, i64 %value) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %valid = icmp ule i64 %value, 1
  call void @p.require(i1 %valid)
  %destination = call ptr @p.result(ptr %context, i64 3)
  call void @p.put(ptr %destination, i64 0, i64 %value)
  call void @p.put(ptr %context, i64 {{RESULT_KIND}}, i64 3)
  ret void
}
define void @topal_tail_result_unit_v1(ptr %provided) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %destination = call ptr @p.result(ptr %context, i64 4)
  call void @p.put(ptr %context, i64 {{RESULT_KIND}}, i64 4)
  ret void
}
