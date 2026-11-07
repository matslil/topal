; Qualified hooks for the owned pure numeric kernel invocation.
; No context, address or allocator function is exposed to Topal source.
declare ptr @topal_kernel_numeric_allocate_v1(i64)
declare void @topal_kernel_numeric_abort_v1() noreturn

define internal ptr @topal.platform.allocate(i64 %length) nounwind noinline {
entry:
  %pointer = call ptr @topal_kernel_numeric_allocate_v1(i64 %length)
  ret ptr %pointer
}

define internal void @topal.platform.exit(i64 %status) noreturn nounwind noinline {
entry:
  call void @topal_kernel_numeric_abort_v1()
  unreachable
}

define internal i64 @topal.platform.write(i64 %fd, ptr %buffer, i64 %length) nounwind noinline {
entry:
  call void @topal_kernel_numeric_abort_v1()
  unreachable
}
