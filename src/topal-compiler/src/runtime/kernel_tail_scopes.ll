; Scoped call/frame transport. No source view escapes or Window construction.
define internal void @p.frame_identity(ptr %context, ptr %frame) nounwind noinline {
entry:
  %identity = call i64 @p.word(ptr %frame, i64 8)
  %history = call ptr @p.history(ptr %context)
  %next = call i64 @p.word(ptr %history, i64 0)
  %issued = icmp ult i64 %identity, %next
  %nonzero = icmp ugt i64 %identity, 0
  %valid = and i1 %issued, %nonzero
  call void @p.require(i1 %valid)
  ret void
}
define internal void @p.validate_current_view(ptr %context, ptr %view) nounwind noinline {
entry:
  %frame = call ptr @p.current(ptr %context)
  call void @p.frame_identity(ptr %context, ptr %frame)
  %function = call i64 @p.word(ptr %frame, i64 0)
  %has.region = call i1 @p.has_region(i64 %function)
  call void @p.require(i1 %has.region)
  %expected = getelementptr i8, ptr %frame, i64 24
  %same = icmp eq ptr %view, %expected
  call void @p.require(i1 %same)
  %control = call i64 @p.word(ptr %view, i64 0)
  %control.word = ptrtoint ptr %context to i64
  %same.control = icmp eq i64 %control, %control.word
  call void @p.require(i1 %same.control)
  %offset = call i64 @p.word(ptr %view, i64 8)
  %whole = icmp eq i64 %offset, 0
  call void @p.require(i1 %whole)
  %count = call i64 @p.word(ptr %view, i64 16)
  %owner.count = call i64 @p.word(ptr %context, i64 40)
  %same.count = icmp eq i64 %count, %owner.count
  call void @p.require(i1 %same.count)
  %rights = call i64 @p.word(ptr %view, i64 24)
  %owner.rights = call i64 @p.word(ptr %context, i64 48)
  %rights.valid = icmp ule i64 %rights, 1
  %same.rights = icmp eq i64 %rights, %owner.rights
  %valid.rights = and i1 %rights.valid, %same.rights
  call void @p.require(i1 %valid.rights)
  %epoch = call i64 @p.word(ptr %view, i64 32)
  %generation = call i64 @p.word(ptr %context, i64 16)
  %same.epoch = icmp eq i64 %epoch, %generation
  call void @p.require(i1 %same.epoch)
  %owner = call i64 @p.word(ptr %view, i64 40)
  %control.owner = call i64 @p.word(ptr %context, i64 8)
  %same.owner = icmp eq i64 %owner, %control.owner
  call void @p.require(i1 %same.owner)
  %code = call i64 @p.word(ptr %view, i64 48)
  %control.code = call i64 @p.word(ptr %context, i64 0)
  %same.code = icmp eq i64 %code, %control.code
  call void @p.require(i1 %same.code)
  %zero = call i64 @p.word(ptr %view, i64 56)
  %reserved.zero = icmp eq i64 %zero, 0
  call void @p.require(i1 %reserved.zero)
  %history = call ptr @p.history(ptr %context)
  %loan = call i64 @p.word(ptr %frame, i64 16)
  %next.loan = call i64 @p.word(ptr %history, i64 8)
  %loan.nonzero = icmp ugt i64 %loan, 0
  %loan.issued = icmp ult i64 %loan, %next.loan
  %valid.loan = and i1 %loan.nonzero, %loan.issued
  call void @p.require(i1 %valid.loan)
  ret void
}
define void @topal_tail_validate_view_v1(ptr %provided, ptr %view) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  call void @p.validate_current_view(ptr %context, ptr %view)
  ret void
}
define ptr @topal_tail_input_view_v1(ptr %provided) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %frame = call ptr @p.current(ptr %context)
  %view = getelementptr i8, ptr %frame, i64 24
  call void @topal_tail_validate_view_v1(ptr %context, ptr %view)
  ret ptr %view
}
define ptr @topal_tail_helper_enter_v1(ptr %provided, i64 %function, ptr %view) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %parent = call ptr @p.current(ptr %context)
  call void @p.frame_identity(ptr %context, ptr %parent)
  %parent.function = call i64 @p.word(ptr %parent, i64 0)
  %parent.has.region = call i1 @p.has_region(i64 %parent.function)
  %has.region = call i1 @p.has_region(i64 %function)
  %row = mul i64 %parent.function, {{FUNCTIONS}}
  %index = add i64 %row, %function
  %edge.pointer = getelementptr [{{CALL_COUNT}} x i8], ptr @p.calls, i64 0, i64 %index
  %edge = load i8, ptr %edge.pointer, align 1
  %allowed = icmp eq i8 %edge, 1
  call void @p.require(i1 %allowed)
  %depth = call i64 @p.word(ptr %context, i64 {{DEPTH_WORD}})
  %capacity = icmp ult i64 %depth, {{MAX_DEPTH}}
  call void @p.require(i1 %capacity)
  %history = call ptr @p.history(ptr %context)
  %identity = call i64 @p.word(ptr %history, i64 0)
  %end = call i64 @p.word(ptr %context, i64 {{FRAME_END}})
  %step.end = call i64 @p.word(ptr %context, i64 {{STEP_FRAME_END}})
  %global.budget = icmp ult i64 %identity, %end
  %step.budget = icmp ult i64 %identity, %step.end
  %budget = and i1 %global.budget, %step.budget
  call void @p.require(i1 %budget)
  %next.identity = call i64 @p.add(i64 %identity, i64 1)
  br i1 %has.region, label %region, label %no.region
region:
  call void @topal_tail_validate_view_v1(ptr %context, ptr %view)
  %loan = call i64 @p.word(ptr %history, i64 8)
  %loan.end = call i64 @p.word(ptr %context, i64 {{LOAN_END}})
  %step.loan.end = call i64 @p.word(ptr %context, i64 {{STEP_LOAN_END}})
  %global.loan.budget = icmp ult i64 %loan, %loan.end
  %step.loan.budget = icmp ult i64 %loan, %step.loan.end
  %loan.budget = and i1 %global.loan.budget, %step.loan.budget
  call void @p.require(i1 %loan.budget)
  %next.loan = call i64 @p.add(i64 %loan, i64 1)
  br label %prepare
no.region:
  %no.view = icmp eq ptr %view, null
  call void @p.require(i1 %no.view)
  br label %prepare
prepare:
  %issued.loan = phi i64 [ %loan, %region ], [ 0, %no.region ]
  %loan.successor = phi i64 [ %next.loan, %region ], [ 0, %no.region ]
  %relative = mul i64 %depth, 88
  %offset = add i64 %relative, 64
  %frame = getelementptr i8, ptr %context, i64 %offset
  %empty = call i1 @p.zero(ptr %frame, i64 88)
  call void @p.require(i1 %empty)
  call void @p.put(ptr %frame, i64 0, i64 %function)
  call void @p.put(ptr %frame, i64 8, i64 %identity)
  call void @p.put(ptr %frame, i64 16, i64 %issued.loan)
  %new.view = getelementptr i8, ptr %frame, i64 24
  br i1 %has.region, label %copy.view, label %publish
copy.view:
  call void @p.copy(ptr %new.view, ptr %view, i64 64)
  call void @p.put(ptr %history, i64 8, i64 %loan.successor)
  br label %publish
publish:
  call void @p.put(ptr %history, i64 0, i64 %next.identity)
  %next.depth = add i64 %depth, 1
  call void @p.put(ptr %context, i64 {{DEPTH_WORD}}, i64 %next.depth)
  %result = select i1 %has.region, ptr %new.view, ptr null
  ret ptr %result
}
define void @topal_tail_helper_leave_v1(ptr %provided, i64 %function) nounwind noinline {
entry:
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %depth = call i64 @p.word(ptr %context, i64 {{DEPTH_WORD}})
  %nested = icmp ugt i64 %depth, 1
  call void @p.require(i1 %nested)
  %frame = call ptr @p.current(ptr %context)
  call void @p.frame_identity(ptr %context, ptr %frame)
  %actual = call i64 @p.word(ptr %frame, i64 0)
  %same = icmp eq i64 %actual, %function
  call void @p.require(i1 %same)
  call void @p.erase(ptr %frame, i64 88)
  %next = sub i64 %depth, 1
  call void @p.put(ptr %context, i64 {{DEPTH_WORD}}, i64 %next)
  ret void
}
