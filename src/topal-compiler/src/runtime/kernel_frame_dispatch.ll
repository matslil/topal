; Actual bootstrap gateway dispatch; these are private provider calls.
declare ptr @topal_x86_frame_allocator_create_v1(ptr)
declare i1 @topal_frame_allocator_allocate_v1(ptr,i64,i64,ptr)
declare i1 @topal_frame_allocator_release_v1(ptr,ptr)
declare i1 @topal_frame_allocator_pin_v1(ptr,ptr,i1)
declare i1 @topal_frame_allocator_resolve_v1(ptr,ptr,ptr)
@topal_x86_frame_context_v1 = external global [16 x i64]
define i64 @topal_frame_kernel_dispatch_v1(i64 %op,i64 %a,i64 %b,ptr %token,ptr %facts) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  switch i64 %op,label %bad [i64 0,label %create
    i64 1,label %allocate
    i64 2,label %release
    i64 3,label %pin
    i64 4,label %resolve]
create:
  %boot = inttoptr i64 %a to ptr
  %ctx = call ptr @topal_x86_frame_allocator_create_v1(ptr %boot)
  %created = ptrtoint ptr %ctx to i64
  ret i64 %created
allocate:
  %allocated = call i1 @topal_frame_allocator_allocate_v1(ptr @topal_x86_frame_context_v1,i64 %a,i64 %b,ptr %token)
  %allocation = zext i1 %allocated to i64
  ret i64 %allocation
release:
  %released = call i1 @topal_frame_allocator_release_v1(ptr @topal_x86_frame_context_v1,ptr %token)
  %releaseResult = zext i1 %released to i64
  ret i64 %releaseResult
pin:
  %valid = icmp ule i64 %a,1
  br i1 %valid,label %changePin,label %bad
changePin:
  %acquire = icmp eq i64 %a,1
  %pinned = call i1 @topal_frame_allocator_pin_v1(ptr @topal_x86_frame_context_v1,ptr %token,i1 %acquire)
  %pinResult = zext i1 %pinned to i64
  ret i64 %pinResult
resolve:
  %resolved = call i1 @topal_frame_allocator_resolve_v1(ptr @topal_x86_frame_context_v1,ptr %token,ptr %facts)
  %resolution = zext i1 %resolved to i64
  ret i64 %resolution
bad:
  ret i64 0
}
