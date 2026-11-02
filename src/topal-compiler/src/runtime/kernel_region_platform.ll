; Qualified hooks for the explicitly owned root-region invocation.
; No process/GS context, address or allocator function is exposed to source.
declare ptr @topal_kernel_region_allocate_v1(i64)
declare void @topal_kernel_region_abort_v1() noreturn

define internal ptr @topal.platform.allocate(i64 %length) nounwind noinline {
entry:
  %pointer = call ptr @topal_kernel_region_allocate_v1(i64 %length)
  ret ptr %pointer
}

define internal void @topal.platform.exit(i64 %status) noreturn nounwind noinline {
entry:
  call void @topal_kernel_region_abort_v1()
  unreachable
}

define internal i64 @topal.platform.write(i64 %fd, ptr %buffer, i64 %length) nounwind noinline {
entry:
  call void @topal_kernel_region_abort_v1()
  unreachable
}
