; TOPAL-SYSTEMS-TAIL-ADMISSION-001. Independently owned mapped caller domains.
declare i64 @topal_tail_step_component_v1(ptr)
declare i64 @topal_x86_tail_boundary_call_v1(ptr, ptr)
declare i64 @topal_x86_tail_checkpoint_call_v1(ptr, ptr, ptr, i64)
declare i64 @topal_x86_tail_complete_call_v1(ptr, ptr, ptr, ptr)
declare void @topal_x86_tail_boundary_fail_v1(ptr, ptr) noreturn
define internal i64 @o.word(ptr %p, i64 %offset) nounwind noinline {
entry:
  %a = getelementptr i8, ptr %p, i64 %offset
  %v = load i64, ptr %a, align 8
  ret i64 %v
}
define internal void @o.put(ptr %p, i64 %offset, i64 %v) nounwind noinline {
entry:
  %a = getelementptr i8, ptr %p, i64 %offset
  store i64 %v, ptr %a, align 8
  ret void
}
define internal i1 @o.zero(ptr %p, i64 %length) nounwind noinline {
entry:
  br label %loop
loop:
  %i = phi i64 [0, %entry], [%next, %read]
  %end = icmp eq i64 %i, %length
  br i1 %end, label %yes, label %read
read:
  %a = getelementptr i8, ptr %p, i64 %i
  %v = load volatile i8, ptr %a, align 1
  %clean = icmp eq i8 %v, 0
  %next = add i64 %i, 1
  br i1 %clean, label %loop, label %no
yes:
  ret i1 true
no:
  ret i1 false
}
define internal void @o.number(ptr %p, i64 %v) nounwind noinline {
entry:
  %two = icmp ugt i64 %v, 4294967295
  %nonzero = icmp ne i64 %v, 0
  %one = zext i1 %nonzero to i64
  %length = select i1 %two, i64 2, i64 %one
  call void @o.put(ptr %p, i64 8, i64 %length)
  call void @o.put(ptr %p, i64 16, i64 %v)
  ret void
}
define i64 @topal_tail_owner_invoke_v1(ptr %owner, i64 %count, ptr %input, ptr %history, ptr %private, ptr %stack, ptr %output) nounwind noinline {
entry:
  %geometry = call i1 @o.geometry(ptr %owner, i64 %count, ptr %input, ptr %history, ptr %private, ptr %stack, ptr %output)
  br i1 %geometry, label %state, label %refuse
state:
  %active = load ptr, ptr @topal_x86_tail_boundary_active_v1, align 8
  %idle = icmp eq ptr %active, null
  br i1 %idle, label %empty, label %refuse
empty:
  %private.zero = call i1 @o.zero(ptr %private, i64 65528)
  %stack.zero = call i1 @o.zero(ptr %stack, i64 4096)
  %output.zero = call i1 @o.zero(ptr %output, i64 56)
  %b = and i1 %stack.zero, %output.zero
  %clean = and i1 %private.zero, %b
  br i1 %clean, label %history.state, label %refuse
history.state:
  %frame = call i64 @o.word(ptr %history, i64 0)
  %loan = call i64 @o.word(ptr %history, i64 8)
  %generation = call i64 @o.word(ptr %history, i64 16)
  %live = call i64 @o.word(ptr %history, i64 24)
  %poison = call i64 @o.word(ptr %history, i64 32)
  %virgin = call i1 @o.zero(ptr %history, i64 40)
  %frames.valid = icmp uge i64 %frame, 2
  %loans.valid = icmp uge i64 %loan, 2
  %generation.valid = icmp ugt i64 %generation, 0
  %old.ids = and i1 %frames.valid, %loans.valid
  %old.valid = and i1 %old.ids, %generation.valid
  %identities.valid = or i1 %virgin, %old.valid
  %inactive = icmp eq i64 %live, 0
  %not.poisoned = icmp eq i64 %poison, 0
  %available = and i1 %inactive, %not.poisoned
  %history.valid = and i1 %identities.valid, %available
  %next.generation = add i64 %generation, 1
  %generation.fits = icmp ugt i64 %next.generation, %generation
  %ready = and i1 %history.valid, %generation.fits
  br i1 %ready, label %budgets, label %refuse
budgets:
  %input.0 = call i64 @o.word(ptr %input, i64 0)
  %input.1 = call i64 @o.word(ptr %input, i64 8)
  %input.2 = call i64 @o.word(ptr %input, i64 16)
  %above = icmp ugt i64 %input.0, {{BASE}}
  %difference = sub i64 %input.0, {{BASE}}
  %distance = select i1 %above, i64 %difference, i64 0
  %quotient = udiv i64 %distance, {{DECREMENT}}
  %remainder = urem i64 %distance, {{DECREMENT}}
  %round = icmp ne i64 %remainder, 0
  %extra = zext i1 %round to i64
  %edges = add i64 %quotient, %extra
  %steps = add i64 %edges, 1
  %steps.fit = icmp ugt i64 %steps, %edges
  %next.frame = select i1 %virgin, i64 2, i64 %frame
  %next.loan = select i1 %virgin, i64 2, i64 %loan
  %remaining.frames = sub i64 -1, %next.frame
  %remaining.loans = sub i64 -1, %next.loan
  %frame.steps = udiv i64 %remaining.frames, {{FRAMES_PER_STEP}}
  %loan.steps = udiv i64 %remaining.loans, {{LOANS_PER_STEP}}
  %frames.fit = icmp ule i64 %steps, %frame.steps
  %loans.fit = icmp ule i64 %steps, %loan.steps
  %ids.fit = and i1 %frames.fit, %loans.fit
  %budget = and i1 %steps.fit, %ids.fit
  br i1 %budget, label %admit, label %refuse
admit:
  %frame.cost = mul i64 %steps, {{FRAMES_PER_STEP}}
  %loan.cost = mul i64 %steps, {{LOANS_PER_STEP}}
  %frame.end = add i64 %next.frame, %frame.cost
  %loan.end = add i64 %next.loan, %loan.cost
  call void @o.put(ptr %history, i64 0, i64 %next.frame)
  call void @o.put(ptr %history, i64 8, i64 %next.loan)
  call void @o.put(ptr %history, i64 16, i64 %next.generation)
  call void @o.put(ptr %history, i64 24, i64 1)
  %context = getelementptr i8, ptr %private, i64 64
  %source = ptrtoint ptr @topal_tail_step_component_v1 to i64
  %domain = ptrtoint ptr %history to i64
  %owner.word = ptrtoint ptr %owner to i64
  %context.word = ptrtoint ptr %context to i64
  call void @o.put(ptr %context, i64 0, i64 %source)
  call void @o.put(ptr %context, i64 8, i64 %domain)
  call void @o.put(ptr %context, i64 16, i64 %next.generation)
  call void @o.put(ptr %context, i64 24, i64 1)
  call void @o.put(ptr %context, i64 32, i64 %owner.word)
  call void @o.put(ptr %context, i64 40, i64 %count)
  call void @o.put(ptr %context, i64 48, i64 1)
  call void @o.put(ptr %context, i64 56, i64 1)
  call void @o.put(ptr %context, i64 64, i64 {{ENTRY}})
  call void @o.put(ptr %context, i64 72, i64 1)
  call void @o.put(ptr %context, i64 80, i64 1)
  call void @o.put(ptr %context, i64 88, i64 %context.word)
  call void @o.put(ptr %context, i64 104, i64 %count)
  call void @o.put(ptr %context, i64 112, i64 1)
  call void @o.put(ptr %context, i64 120, i64 %next.generation)
  call void @o.put(ptr %context, i64 128, i64 %domain)
  call void @o.put(ptr %context, i64 136, i64 %source)
  call void @o.put(ptr %context, i64 {{HISTORY}}, i64 %domain)
  call void @o.put(ptr %context, i64 {{FRAME_END}}, i64 %frame.end)
  call void @o.put(ptr %context, i64 {{LOAN_END}}, i64 %loan.end)
  call void @o.put(ptr %context, i64 {{DEPTH_WORD}}, i64 1)
  call void @o.put(ptr %context, i64 {{STEP_LIMIT}}, i64 %steps)
{{SEEDS}}
  br label %loop
loop:
  %outcome = call i64 @topal_x86_tail_boundary_call_v1(ptr %private, ptr %stack)
  %outcome.valid = icmp ule i64 %outcome, 1
  br i1 %outcome.valid, label %checkpoint, label %fatal
checkpoint:
  %checkpoint.outcome = call i64 @topal_x86_tail_checkpoint_call_v1(ptr %private, ptr %stack, ptr %history, i64 %outcome)
  %same.outcome = icmp eq i64 %checkpoint.outcome, %outcome
  br i1 %same.outcome, label %returned, label %fatal
returned:
  %successor = icmp eq i64 %outcome, 1
  br i1 %successor, label %loop, label %complete
complete:
  %completed = call i64 @topal_x86_tail_complete_call_v1(ptr %private, ptr %stack, ptr %history, ptr %output)
  %normal = icmp eq i64 %completed, 0
  br i1 %normal, label %done, label %fatal
done:
  ret i64 0
refuse:
  ret i64 2
fatal:
  call void @o.put(ptr %history, i64 24, i64 0)
  call void @o.put(ptr %history, i64 32, i64 1)
  br label %erase.output
erase.output:
  %i = phi i64 [0, %fatal], [%next, %write]
  %end = icmp eq i64 %i, 56
  br i1 %end, label %fail, label %write
write:
  %byte = getelementptr i8, ptr %output, i64 %i
  store volatile i8 0, ptr %byte, align 1
  %next = add i64 %i, 1
  br label %erase.output
fail:
  %erased = call i1 @o.zero(ptr %output, i64 56)
  call void @topal_x86_tail_boundary_fail_v1(ptr %private, ptr %stack)
  unreachable
}
