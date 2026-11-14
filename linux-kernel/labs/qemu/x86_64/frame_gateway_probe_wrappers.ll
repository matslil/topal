; Fixture calls the real gateway, preserving the same private provider ABI.
declare i64 @topal_x86_frame_gateway_v1(i64,i64,i64,ptr,ptr)
define ptr @probe.gateway_create(ptr %boot) noinline nounwind noredzone noimplicitfloat "disable-tail-calls"="true" {
  %input = ptrtoint ptr %boot to i64
  %result = call i64 @topal_x86_frame_gateway_v1(i64 0,i64 %input,i64 0,ptr null,ptr null)
  %ctx = inttoptr i64 %result to ptr
  ret ptr %ctx
}
define i1 @probe.gateway_allocate(ptr %ctx,i64 %count,i64 %alignment,ptr %token) noinline nounwind noredzone noimplicitfloat "disable-tail-calls"="true" {
  %result = call i64 @topal_x86_frame_gateway_v1(i64 1,i64 %count,i64 %alignment,ptr %token,ptr null)
  %ok = icmp eq i64 %result,1
  ret i1 %ok
}
define i1 @probe.gateway_release(ptr %ctx,ptr %token) noinline nounwind noredzone noimplicitfloat "disable-tail-calls"="true" {
  %result = call i64 @topal_x86_frame_gateway_v1(i64 2,i64 0,i64 0,ptr %token,ptr null)
  %ok = icmp eq i64 %result,1
  ret i1 %ok
}
define i1 @probe.gateway_pin(ptr %ctx,ptr %token,i1 %acquire) noinline nounwind noredzone noimplicitfloat "disable-tail-calls"="true" {
  %value = zext i1 %acquire to i64
  %result = call i64 @topal_x86_frame_gateway_v1(i64 3,i64 %value,i64 0,ptr %token,ptr null)
  %ok = icmp eq i64 %result,1
  ret i1 %ok
}
define i1 @probe.gateway_resolve(ptr %ctx,ptr %token,ptr %facts) noinline nounwind noredzone noimplicitfloat "disable-tail-calls"="true" {
  %result = call i64 @topal_x86_frame_gateway_v1(i64 4,i64 0,i64 0,ptr %token,ptr %facts)
  %ok = icmp eq i64 %result,1
  ret i1 %ok
}
