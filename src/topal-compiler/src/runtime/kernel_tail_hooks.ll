; TOPAL-SYSTEMS-TAIL-RUNTIME-001. Provider mechanisms, no subsystem policy.
%Root = type { i64, i64, i64, i64 }
@p.roots = private constant [{{PARAMETERS}} x %Root] [{{ROOT_TABLE}}], align 8
@p.regions = private constant [{{FUNCTIONS}} x i8] [{{REGION_TABLE}}], align 8
@p.calls = private constant [{{CALL_COUNT}} x i8] [{{CALL_TABLE}}], align 8
@topal_x86_tail_boundary_active_v1 = external global ptr
declare void @topal_x86_tail_boundary_abort_v1() noreturn
declare i64 @topal_tail_readonly_bytes_v1(ptr)
declare i64 @topal_tail_step_component_v1(ptr)

define void @topal_tail_abort_v1() noreturn nounwind noinline {
entry:
  call void @topal_x86_tail_boundary_abort_v1()
  unreachable
}
define internal void @p.require(i1 %condition) nounwind noinline {
entry:
  br i1 %condition, label %done, label %fail
fail:
  call void @topal_tail_abort_v1()
  unreachable
done:
  ret void
}
define internal i64 @p.word(ptr %base, i64 %offset) nounwind noinline {
entry:
  %pointer = getelementptr i8, ptr %base, i64 %offset
  %value = load i64, ptr %pointer, align 8
  ret i64 %value
}
define internal void @p.put(ptr %base, i64 %offset, i64 %value) nounwind noinline {
entry:
  %pointer = getelementptr i8, ptr %base, i64 %offset
  store i64 %value, ptr %pointer, align 8
  ret void
}
define internal i64 @p.add(i64 %a, i64 %b) nounwind noinline {
entry:
  %sum = add i64 %a, %b
  %valid = icmp uge i64 %sum, %a
  call void @p.require(i1 %valid)
  ret i64 %sum
}
define internal i1 @p.zero(ptr %base, i64 %bytes) nounwind noinline {
entry:
  br label %loop
loop:
  %index = phi i64 [ 0, %entry ], [ %next, %read ]
  %end = icmp eq i64 %index, %bytes
  br i1 %end, label %yes, label %read
read:
  %pointer = getelementptr i8, ptr %base, i64 %index
  %byte = load volatile i8, ptr %pointer, align 1
  %is.zero = icmp eq i8 %byte, 0
  %next = add i64 %index, 1
  br i1 %is.zero, label %loop, label %no
yes:
  ret i1 true
no:
  ret i1 false
}
define internal void @p.erase(ptr %base, i64 %bytes) nounwind noinline {
entry:
  br label %loop
loop:
  %index = phi i64 [ 0, %entry ], [ %next, %write ]
  %end = icmp eq i64 %index, %bytes
  br i1 %end, label %verify, label %write
write:
  %pointer = getelementptr i8, ptr %base, i64 %index
  store volatile i8 0, ptr %pointer, align 1
  %next = add i64 %index, 1
  br label %loop
verify:
  %zero = call i1 @p.zero(ptr %base, i64 %bytes)
  call void @p.require(i1 %zero)
  ret void
}
define internal void @p.copy(ptr %to, ptr %from, i64 %bytes) nounwind noinline {
entry:
  br label %loop
loop:
  %index = phi i64 [ 0, %entry ], [ %next, %write ]
  %end = icmp eq i64 %index, %bytes
  br i1 %end, label %done, label %write
write:
  %source = getelementptr i8, ptr %from, i64 %index
  %destination = getelementptr i8, ptr %to, i64 %index
  %byte = load volatile i8, ptr %source, align 1
  store volatile i8 %byte, ptr %destination, align 1
  %next = add i64 %index, 1
  br label %loop
done:
  ret void
}
define internal ptr @p.context_role(ptr %provided, ptr %role) nounwind noinline {
entry:
  %record = load ptr, ptr @topal_x86_tail_boundary_active_v1, align 8
  %active = icmp ne ptr %record, null
  call void @p.require(i1 %active)
  %expected = getelementptr i8, ptr %record, i64 64
  %same = icmp eq ptr %provided, %expected
  call void @p.require(i1 %same)
  %armed = call i64 @p.word(ptr %record, i64 8)
  %is.armed = icmp eq i64 %armed, 1
  call void @p.require(i1 %is.armed)
  %bound.context = call i64 @p.word(ptr %record, i64 24)
  %context.word = ptrtoint ptr %provided to i64
  %same.context = icmp eq i64 %bound.context, %context.word
  call void @p.require(i1 %same.context)
  %entry.word = ptrtoint ptr @topal_tail_step_component_v1 to i64
  %role.word = ptrtoint ptr %role to i64
  %bound.code = call i64 @p.word(ptr %record, i64 32)
  %same.bound.code = icmp eq i64 %bound.code, %role.word
  call void @p.require(i1 %same.bound.code)
  %history.word = call i64 @p.word(ptr %record, i64 40)
  %history.present = icmp ne i64 %history.word, 0
  call void @p.require(i1 %history.present)
  %actual.history = call i64 @p.word(ptr %provided, i64 {{HISTORY}})
  %same.history = icmp eq i64 %actual.history, %history.word
  call void @p.require(i1 %same.history)
  %history = inttoptr i64 %history.word to ptr
  %history.active = call i64 @p.word(ptr %history, i64 24)
  %history.live = icmp eq i64 %history.active, 1
  call void @p.require(i1 %history.live)
  %poison = call i64 @p.word(ptr %history, i64 32)
  %not.poisoned = icmp eq i64 %poison, 0
  call void @p.require(i1 %not.poisoned)
  %generation = call i64 @p.word(ptr %history, i64 16)
  %has.generation = icmp ne i64 %generation, 0
  call void @p.require(i1 %has.generation)
  %control.code = call i64 @p.word(ptr %provided, i64 0)
  %code.valid = icmp eq i64 %control.code, %entry.word
  call void @p.require(i1 %code.valid)
  %owner = call i64 @p.word(ptr %provided, i64 8)
  %owner.valid = icmp eq i64 %owner, %history.word
  call void @p.require(i1 %owner.valid)
  %epoch = call i64 @p.word(ptr %provided, i64 16)
  %epoch.valid = icmp eq i64 %epoch, %generation
  call void @p.require(i1 %epoch.valid)
  %control.active = call i64 @p.word(ptr %provided, i64 24)
  %control.live = icmp eq i64 %control.active, 1
  call void @p.require(i1 %control.live)
  %windows = call i64 @p.word(ptr %provided, i64 56)
  %one.window = icmp eq i64 %windows, 1
  call void @p.require(i1 %one.window)
  %bank = call i64 @p.word(ptr %provided, i64 {{BANK}})
  %bank.valid = icmp ule i64 %bank, 1
  call void @p.require(i1 %bank.valid)
  ret ptr %provided
}
define internal ptr @p.context(ptr %provided) nounwind noinline {
entry:
  %context = call ptr @p.context_role(ptr %provided, ptr @topal_tail_step_component_v1)
  ret ptr %context
}
define internal ptr @p.phase(ptr %provided, i64 %expected) nounwind noinline {
entry:
  %context = call ptr @p.context(ptr %provided)
  %phase = call i64 @p.word(ptr %context, i64 {{PHASE}})
  %valid = icmp eq i64 %phase, %expected
  call void @p.require(i1 %valid)
  ret ptr %context
}
define internal ptr @p.history(ptr %context) nounwind noinline {
entry:
  %word = call i64 @p.word(ptr %context, i64 {{HISTORY}})
  %pointer = inttoptr i64 %word to ptr
  ret ptr %pointer
}
define internal ptr @p.current(ptr %context) nounwind noinline {
entry:
  %depth = call i64 @p.word(ptr %context, i64 {{DEPTH_WORD}})
  %nonzero = icmp ugt i64 %depth, 0
  %bounded = icmp ule i64 %depth, {{MAX_DEPTH}}
  %valid = and i1 %nonzero, %bounded
  call void @p.require(i1 %valid)
  %index = sub i64 %depth, 1
  %bytes = mul i64 %index, 88
  %offset = add i64 %bytes, 64
  %frame = getelementptr i8, ptr %context, i64 %offset
  ret ptr %frame
}
define internal i1 @p.has_region(i64 %function) nounwind noinline {
entry:
  %nonzero = icmp ugt i64 %function, 0
  %bounded = icmp ult i64 %function, {{FUNCTIONS}}
  %valid = and i1 %nonzero, %bounded
  call void @p.require(i1 %valid)
  %pointer = getelementptr [{{FUNCTIONS}} x i8], ptr @p.regions, i64 0, i64 %function
  %flag = load i8, ptr %pointer, align 1
  %present = icmp eq i8 %flag, 1
  ret i1 %present
}
define internal ptr @p.root(i64 %parameter) nounwind noinline {
entry:
  %valid = icmp ult i64 %parameter, {{PARAMETERS}}
  call void @p.require(i1 %valid)
  %root = getelementptr [{{PARAMETERS}} x %Root], ptr @p.roots, i64 0, i64 %parameter
  ret ptr %root
}
define internal ptr @p.bank(ptr %context, i64 %index) nounwind noinline {
entry:
  %valid = icmp ule i64 %index, 1
  call void @p.require(i1 %valid)
  %relative = mul i64 %index, {{BANK_BYTES}}
  %offset = add i64 %relative, {{BANKS}}
  %bank = getelementptr i8, ptr %context, i64 %offset
  ret ptr %bank
}
define internal void @p.is_root(ptr %context) nounwind noinline {
entry:
  %depth = call i64 @p.word(ptr %context, i64 {{DEPTH_WORD}})
  %valid = icmp eq i64 %depth, 1
  call void @p.require(i1 %valid)
  ret void
}
