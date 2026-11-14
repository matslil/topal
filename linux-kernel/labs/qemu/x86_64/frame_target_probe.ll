; Unpublished privileged component workload, not the production kernel root.
@topal_frame_target_probe_mode_v1 = global i64 0,align 8
@topal_frame_target_probe_request_v1 = global [2 x i64] [i64 3,i64 4],align 8
@topal_frame_target_probe_result_v1 = global i64 0,align 8
@topal_x86_frame_tokens_v1 = external global [6291456 x i8]
@topal_x86_frame_details_v1 = external global [4 x i64]
declare ptr @topal_x86_frame_allocator_create_v1(ptr)
declare i1 @topal_frame_allocator_allocate_v1(ptr,i64,i64,ptr)
declare i1 @topal_frame_allocator_release_v1(ptr,ptr)
declare i1 @topal_frame_allocator_pin_v1(ptr,ptr,i1)
declare i1 @topal_frame_allocator_resolve_v1(ptr,ptr,ptr)
define void @topal_frame_target_created_v1(ptr %ctx,ptr %boot) noinline nounwind {
  call void asm sideeffect "", "~{memory}"()
  ret void
}
define void @topal_frame_target_complete_v1() noinline nounwind noreturn {
entry:
  br label %halt
halt:
  call void asm sideeffect "cli; hlt", "~{memory}"()
  br label %halt
}
define internal void @probe.fill(i64 %frame,i64 %count,i8 %value) noinline nounwind {
entry:
  %base = shl i64 %frame,12
  %bytes = shl i64 %count,12
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%bytes
  br i1 %more,label %body,label %end
body:
  %address = add i64 %base,%i
  %p = inttoptr i64 %address to ptr
  store volatile i8 %value,ptr %p,align 1
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal i1 @probe.zero(i64 %frame,i64 %count) noinline nounwind {
entry:
  %base = shl i64 %frame,12
  %bytes = shl i64 %count,12
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%bytes
  br i1 %more,label %body,label %end
body:
  %address = add i64 %base,%i
  %p = inttoptr i64 %address to ptr
  %v = load volatile i8,ptr %p,align 1
  %zero = icmp eq i8 %v,0
  %next = add i64 %i,1
  br i1 %zero,label %loop,label %bad
end:
  ret i1 true
bad:
  ret i1 false
}
define void @_topal_kernel_entry(ptr %unused,ptr %boot) noinline nounwind noreturn "stackrealign" noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %ctx = call ptr @topal_x86_frame_allocator_create_v1(ptr %boot)
  call void @topal_frame_target_created_v1(ptr %ctx,ptr %boot)
  %created = icmp ne ptr %ctx,null
  br i1 %created,label %select_mode,label %bad
select_mode:
  %mode = load i64,ptr @topal_frame_target_probe_mode_v1,align 8
  %duplicate = icmp eq i64 %mode,2
  br i1 %duplicate,label %duplicateCreate,label %allocationMode
duplicateCreate:
  %secondContext = call ptr @topal_x86_frame_allocator_create_v1(ptr %boot)
  %refusedCreate = icmp eq ptr %secondContext,null
  br i1 %refusedCreate,label %good,label %bad
allocationMode:
  %allocate = icmp eq i64 %mode,1
  br i1 %allocate,label %first,label %good
first:
  %count = load i64,ptr @topal_frame_target_probe_request_v1,align 8
  %alignmentPtr = getelementptr i64,ptr @topal_frame_target_probe_request_v1,i64 1
  %alignment = load i64,ptr %alignmentPtr,align 8
  %allocated = call i1 @topal_frame_allocator_allocate_v1(ptr %ctx,i64 %count,i64 %alignment,ptr @topal_x86_frame_tokens_v1)
  br i1 %allocated,label %resolve,label %bad
resolve:
  %resolved = call i1 @topal_frame_allocator_resolve_v1(ptr %ctx,ptr @topal_x86_frame_tokens_v1,ptr @topal_x86_frame_details_v1)
  br i1 %resolved,label %observe,label %bad
observe:
  %physical = load i64,ptr @topal_x86_frame_details_v1,align 8
  %zero = call i1 @probe.zero(i64 %physical,i64 %count)
  br i1 %zero,label %second,label %bad
second:
  %other = getelementptr i8,ptr @topal_x86_frame_tokens_v1,i64 24
  %secondCount = add i64 %count,1
  %another = call i1 @topal_frame_allocator_allocate_v1(ptr %ctx,i64 %secondCount,i64 1,ptr %other)
  br i1 %another,label %pin,label %bad
pin:
  %pinned = call i1 @topal_frame_allocator_pin_v1(ptr %ctx,ptr @topal_x86_frame_tokens_v1,i1 true)
  %refused = call i1 @topal_frame_allocator_release_v1(ptr %ctx,ptr @topal_x86_frame_tokens_v1)
  %notReleased = xor i1 %refused,true
  %unpin = call i1 @topal_frame_allocator_pin_v1(ptr %ctx,ptr @topal_x86_frame_tokens_v1,i1 false)
  %a = and i1 %pinned,%notReleased
  %b = and i1 %a,%unpin
  br i1 %b,label %dirty,label %bad
dirty:
  call void @probe.fill(i64 %physical,i64 %count,i8 -91)
  %released = call i1 @topal_frame_allocator_release_v1(ptr %ctx,ptr @topal_x86_frame_tokens_v1)
  br i1 %released,label %reuse,label %bad
reuse:
  %again = call i1 @topal_frame_allocator_allocate_v1(ptr %ctx,i64 %count,i64 %alignment,ptr @topal_x86_frame_tokens_v1)
  br i1 %again,label %resolveAgain,label %bad
resolveAgain:
  %againResolved = call i1 @topal_frame_allocator_resolve_v1(ptr %ctx,ptr @topal_x86_frame_tokens_v1,ptr @topal_x86_frame_details_v1)
  br i1 %againResolved,label %observeAgain,label %bad
observeAgain:
  %reusedPhysical = load i64,ptr @topal_x86_frame_details_v1,align 8
  %same = icmp eq i64 %reusedPhysical,%physical
  %againZero = call i1 @probe.zero(i64 %reusedPhysical,i64 %count)
  %correct = and i1 %same,%againZero
  br i1 %correct,label %releaseAll,label %bad
releaseAll:
  %one = call i1 @topal_frame_allocator_release_v1(ptr %ctx,ptr @topal_x86_frame_tokens_v1)
  %two = call i1 @topal_frame_allocator_release_v1(ptr %ctx,ptr %other)
  %all = and i1 %one,%two
  br i1 %all,label %good,label %bad
good:
  store i64 1,ptr @topal_frame_target_probe_result_v1,align 8
  call void @topal_frame_target_complete_v1()
  unreachable
bad:
  store i64 -1,ptr @topal_frame_target_probe_result_v1,align 8
  call void @topal_frame_target_complete_v1()
  unreachable
}
define void @topal_x86_systems_debug_break_entry() noreturn {
  call void @topal_frame_target_complete_v1()
  unreachable
}
define void @topal_x86_systems_local_notification_entry() noreturn {
  call void @topal_frame_target_complete_v1()
  unreachable
}
define void @topal_x86_systems_deadline_entry() noreturn {
  call void @topal_frame_target_complete_v1()
  unreachable
}
define void @topal_x86_systems_user_copy_page_fault() noreturn {
  call void @topal_frame_target_complete_v1()
  unreachable
}
