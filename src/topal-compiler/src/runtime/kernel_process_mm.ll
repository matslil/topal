; TK-DEC-284 unpublished preparation primitive. Actual owning brk invocation.
; Receipt: live, break, count, base, root, context, syscall epoch.
@topal_x86_process_mm_prepared_token_v1 = global [3 x i64] zeroinitializer, align 8
@topal_x86_process_mm_prepared_state_v1 = global [7 x i64] zeroinitializer, align 8
@topal_x86_frame_gateway_recovery_v1 = external global [6 x i64]

define internal void @pm.require(i1 %ok) nounwind noinline noredzone {
  br i1 %ok,label %end,label %bad
bad:
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 0,i64 3)
  call void @m.require(i1 false)
  unreachable
end:
  ret void
}
define internal void @pm.zero(ptr %p,i64 %count) nounwind noinline noredzone {
entry:
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%count
  br i1 %more,label %body,label %end
body:
  %v = call i64 @m.word(ptr %p,i64 %i)
  %zero = icmp eq i64 %v,0
  call void @pm.require(i1 %zero)
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal void @pm.equal(i64 %from,i64 %to,i64 %count) nounwind noinline noredzone {
entry:
  %words = shl i64 %count,9
  %a = inttoptr i64 %from to ptr
  %b = inttoptr i64 %to to ptr
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%words
  br i1 %more,label %body,label %end
body:
  %av = call i64 @m.word(ptr %a,i64 %i)
  %bv = call i64 @m.word(ptr %b,i64 %i)
  %same = icmp eq i64 %av,%bv
  call void @pm.require(i1 %same)
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal void @pm.at_prepared() nounwind noinline noredzone {
  %v = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 0)
  ret void
}
define internal void @pm.at_released() nounwind noinline noredzone {
  %v = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 0)
  ret void
}
define internal void @pm.at_erased(i64 %base,i64 %count) nounwind noinline noredzone {
  %nonempty = icmp ugt i64 %count,0
  call void @pm.require(i1 %nonempty)
  %p = inttoptr i64 %base to ptr
  %v = call i64 @m.word(ptr %p,i64 0)
  %zero = icmp eq i64 %v,0
  call void @pm.require(i1 %zero)
  ret void
}
define internal void @pm.at_unavailable() nounwind noinline noredzone {
  %v = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 0)
  ret void
}
define internal void @pm.entry() nounwind noinline noredzone {
  %admitted = call i1 @topal_frame_kernel_admit_v1(ptr @topal_x86_frame_context_v1,i64 0,i64 0,i64 0)
  call void @pm.require(i1 %admitted)
  %op = call i64 @m.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 2)
  %brk = icmp eq i64 %op,5
  call void @pm.require(i1 %brk)
  call void @pm.zero(ptr @topal_x86_frame_details_v1,i64 4)
  %live = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 0)
  %one = icmp eq i64 %live,1
  call void @pm.require(i1 %one)
  ret void
}
define internal i1 @pm.prepare() nounwind noinline noredzone {
entry:
  call void @pm.entry()
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_state_v1,i64 7)
  %ctx = call ptr @m.context()
  %oldBreak = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 1)
  %count = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %oldBase = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 3)
  %bounded = icmp ule i64 %count,519
  call void @pm.require(i1 %bounded)
  call void @m.validate(i64 %count,i64 %oldBase)
  %empty = icmp eq i64 %count,0
  br i1 %empty,label %commit,label %allocate
allocate:
  %actual = call i64 @m.resolve(ptr @topal_x86_frame_mm_tokens_v1,i64 %count,i64 1)
  %same = icmp eq i64 %actual,%oldBase
  call void @pm.require(i1 %same)
  %budget = call i1 @topal_frame_kernel_admit_v1(ptr @topal_x86_frame_context_v1,i64 1,i64 %count,i64 1)
  call void @pm.require(i1 %budget)
  %allocated = call i1 @topal_frame_allocator_allocate_v1(ptr @topal_x86_frame_context_v1,i64 %count,i64 1,ptr @topal_x86_process_mm_prepared_token_v1)
  br i1 %allocated,label %copy,label %unavailable
copy:
  %newBase = call i64 @m.resolve(ptr @topal_x86_process_mm_prepared_token_v1,i64 %count,i64 0)
  %bytes = shl i64 %count,12
  %oldEnd = add i64 %oldBase,%bytes
  %newEnd = add i64 %newBase,%bytes
  %before = icmp ule i64 %newEnd,%oldBase
  %after = icmp ule i64 %oldEnd,%newBase
  %disjoint = or i1 %before,%after
  call void @pm.require(i1 %disjoint)
  %newPointer = inttoptr i64 %newBase to ptr
  %newWords = shl i64 %count,9
  call void @pm.zero(ptr %newPointer,i64 %newWords)
  call void @m.copy(i64 %oldBase,i64 %newBase,i64 %count)
  call void @pm.equal(i64 %oldBase,i64 %newBase,i64 %count)
  %pinned = call i1 @topal_frame_allocator_pin_v1(ptr @topal_x86_frame_context_v1,ptr @topal_x86_process_mm_prepared_token_v1,i1 true)
  call void @pm.require(i1 %pinned)
  %resolved = call i64 @m.resolve(ptr @topal_x86_process_mm_prepared_token_v1,i64 %count,i64 1)
  %sameNew = icmp eq i64 %resolved,%newBase
  call void @pm.require(i1 %sameNew)
  br label %commit
commit:
  %base = phi i64 [0,%entry],[%newBase,%copy]
  %root = call i64 @m.word(ptr %ctx,i64 3)
  %ctxWord = ptrtoint ptr %ctx to i64
  %epoch = call i64 @m.word(ptr @topal_x86_frame_mm_epoch_v1,i64 0)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 1,i64 %oldBreak)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 2,i64 %count)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 3,i64 %base)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 4,i64 %root)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 5,i64 %ctxWord)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 6,i64 %epoch)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 0,i64 1)
  call void @pm.at_prepared()
  ret i1 true
unavailable:
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_state_v1,i64 7)
  call void @pm.at_unavailable()
  ret i1 false
}
define internal void @pm.cancel(i1 %prepared) nounwind noinline noredzone {
entry:
  call void @pm.entry()
  br i1 %prepared,label %validate,label %empty
empty:
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_state_v1,i64 7)
  call void @pm.at_released()
  ret void
validate:
  %ctx = call ptr @m.context()
  %live = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 0)
  %one = icmp eq i64 %live,1
  call void @pm.require(i1 %one)
  %root = call i64 @m.word(ptr %ctx,i64 3)
  %ctxWord = ptrtoint ptr %ctx to i64
  %epoch = call i64 @m.word(ptr @topal_x86_frame_mm_epoch_v1,i64 0)
  %savedRoot = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 4)
  %savedCtx = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 5)
  %savedEpoch = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 6)
  %sameRoot = icmp eq i64 %savedRoot,%root
  %sameCtx = icmp eq i64 %savedCtx,%ctxWord
  %sameEpoch = icmp eq i64 %savedEpoch,%epoch
  %a = and i1 %sameRoot,%sameCtx
  %all = and i1 %a,%sameEpoch
  call void @pm.require(i1 %all)
  %count = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 2)
  %base = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 3)
  %oldBreak = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 1)
  %oldCount = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %oldBase = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 3)
  %savedBreak = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 1)
  %sameCount = icmp eq i64 %count,%oldCount
  %sameBreak = icmp eq i64 %savedBreak,%oldBreak
  %unchanged = and i1 %sameCount,%sameBreak
  call void @pm.require(i1 %unchanged)
  call void @m.validate(i64 %oldCount,i64 %oldBase)
  %none = icmp eq i64 %count,0
  br i1 %none,label %clear,label %release
release:
  %resolved = call i64 @m.resolve(ptr @topal_x86_process_mm_prepared_token_v1,i64 %count,i64 1)
  %same = icmp eq i64 %resolved,%base
  call void @pm.require(i1 %same)
  call void @pm.equal(i64 %oldBase,i64 %base,i64 %count)
  %unpinned = call i1 @topal_frame_allocator_pin_v1(ptr @topal_x86_frame_context_v1,ptr @topal_x86_process_mm_prepared_token_v1,i1 false)
  call void @pm.require(i1 %unpinned)
  call void @m.erase(i64 %base,i64 %count)
  %pointer = inttoptr i64 %base to ptr
  %words = shl i64 %count,9
  call void @pm.zero(ptr %pointer,i64 %words)
  call void @pm.at_erased(i64 %base,i64 %count)
  %released = call i1 @topal_frame_allocator_release_v1(ptr @topal_x86_frame_context_v1,ptr @topal_x86_process_mm_prepared_token_v1)
  call void @pm.require(i1 %released)
  br label %clear
clear:
  br i1 %none,label %empty_token,label %clear_state
empty_token:
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  %zeroBase = icmp eq i64 %base,0
  call void @pm.require(i1 %zeroBase)
  br label %clear_state
clear_state:
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 0,i64 0)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 1,i64 0)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 2,i64 0)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 3,i64 0)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 4,i64 0)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 5,i64 0)
  call void @m.put(ptr @topal_x86_process_mm_prepared_state_v1,i64 6,i64 0)
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_state_v1,i64 7)
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  call void @pm.at_released()
  ret void
}
