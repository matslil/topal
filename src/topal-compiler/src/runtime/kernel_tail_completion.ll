; TOPAL-SYSTEMS-TAIL-COMPLETION-001. Exact result transfer before whole retirement.
define i64 @topal_tail_complete_body_v1(ptr %provided, ptr %output) nounwind noinline {
entry:
  %context = call ptr @p.context_role(ptr %provided, ptr @topal_tail_complete_body_v1)
  %record = load ptr, ptr @topal_x86_tail_boundary_active_v1, align 8
  %bound.output = call i64 @p.word(ptr %record, i64 48)
  %output.word = ptrtoint ptr %output to i64
  %same.output = icmp eq i64 %bound.output, %output.word
  call void @p.require(i1 %same.output)
  %bound.bytes = call i64 @p.word(ptr %record, i64 56)
  %exact.bytes = icmp eq i64 %bound.bytes, 56
  call void @p.require(i1 %exact.bytes)
  %phase = call i64 @p.word(ptr %context, i64 {{PHASE}})
  %retired = icmp eq i64 %phase, 0
  call void @p.require(i1 %retired)
  call void @p.is_root(ptr %context)
  %frame = call ptr @p.current(ptr %context)
  call void @p.frame_identity(ptr %context, ptr %frame)
  %function = call i64 @p.word(ptr %frame, i64 0)
  %entry.function = icmp eq i64 %function, {{ENTRY}}
  call void @p.require(i1 %entry.function)
  %region = call i1 @p.has_region(i64 %function)
  br i1 %region, label %view, label %domains
view:
  %root.view = getelementptr i8, ptr %frame, i64 24
  call void @p.validate_current_view(ptr %context, ptr %root.view)
  br label %domains
domains:
  %helpers = getelementptr i8, ptr %context, i64 152
  %helpers.zero = call i1 @p.zero(ptr %helpers, i64 {{RETIRED_HELPER_BYTES}})
  call void @p.require(i1 %helpers.zero)
  %banks = getelementptr i8, ptr %context, i64 {{BANKS}}
  %banks.zero = call i1 @p.zero(ptr %banks, i64 {{BOTH_BANK_BYTES}})
  call void @p.require(i1 %banks.zero)
  %work = getelementptr i8, ptr %context, i64 {{WORK}}
  %work.zero = call i1 @p.zero(ptr %work, i64 {{WORK_BYTES}})
  call void @p.require(i1 %work.zero)
  %starts = getelementptr i8, ptr %context, i64 {{STARTS}}
  %starts.zero = call i1 @p.zero(ptr %starts, i64 {{START_BYTES}})
  call void @p.require(i1 %starts.zero)
  %used = call i64 @p.word(ptr %context, i64 {{USED}})
  %used.zero = icmp eq i64 %used, 0
  call void @p.require(i1 %used.zero)
  %staged = call i64 @p.word(ptr %context, i64 {{STAGED}})
  %staged.zero = icmp eq i64 %staged, 0
  call void @p.require(i1 %staged.zero)
  %steps = call i64 @p.word(ptr %context, i64 {{STEPS}})
  %limit = call i64 @p.word(ptr %context, i64 {{STEP_LIMIT}})
  %entered = icmp ugt i64 %steps, 0
  %within.limit = icmp ule i64 %steps, %limit
  %valid.steps = and i1 %entered, %within.limit
  call void @p.require(i1 %valid.steps)
  %kind = call i64 @p.word(ptr %context, i64 {{RESULT_KIND}})
  %expected = icmp eq i64 %kind, {{EXPECTED_RESULT_KIND}}
  call void @p.require(i1 %expected)
  %result = getelementptr i8, ptr %context, i64 {{RESULT}}
  %numeric = icmp eq i64 %kind, 2
  br i1 %numeric, label %number, label %other
number:
  call void @p.check_number(ptr %result, i64 48, i64 {{OPERAND_BITS}}, i64 {{RESULT_NUMBER_KIND}})
  br label %copy
other:
  %boolean = icmp eq i64 %kind, 3
  br i1 %boolean, label %bool, label %unit
bool:
  %value = call i64 @p.word(ptr %result, i64 0)
  %valid.boolean = icmp ule i64 %value, 1
  call void @p.require(i1 %valid.boolean)
  %padding = getelementptr i8, ptr %result, i64 8
  %padding.zero = call i1 @p.zero(ptr %padding, i64 40)
  call void @p.require(i1 %padding.zero)
  br label %copy
unit:
  %unit.zero = call i1 @p.zero(ptr %result, i64 48)
  call void @p.require(i1 %unit.zero)
  br label %copy
copy:
  %output.zero = call i1 @p.zero(ptr %output, i64 56)
  call void @p.require(i1 %output.zero)
  call void @p.copy(ptr %output, ptr %result, i64 56)
  br label %readback
readback:
  %index = phi i64 [ 0, %copy ], [ %next, %read ]
  %end = icmp eq i64 %index, 56
  br i1 %end, label %done, label %read
read:
  %source = getelementptr i8, ptr %result, i64 %index
  %destination = getelementptr i8, ptr %output, i64 %index
  %expected.byte = load volatile i8, ptr %source, align 1
  %actual.byte = load volatile i8, ptr %destination, align 1
  %same.byte = icmp eq i8 %expected.byte, %actual.byte
  call void @p.require(i1 %same.byte)
  %next = add i64 %index, 1
  br label %readback
done:
  ret i64 0
}
