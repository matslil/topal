; The root compiler allocates one private slot per affine IR definition. These
; are value storage, not registry capacity or an application allocation quota.
define i1 @topal_frame_kernel_token_domain_v1(ptr %ctx,ptr %token,i1 %write) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %pool = call i1 @r.pool_token_domain(ptr %ctx,ptr %token,i1 %write)
  %owned = icmp eq ptr %ctx,@topal_x86_frame_context_v1
  %p = ptrtoint ptr %token to i64
  %start = ptrtoint ptr @topal_x86_frame_root_tokens_v1 to i64
  %end = add i64 %start,{{ROOT_TOKEN_BYTES}}
  %nowrap = icmp ugt i64 %end,%start
  %offset = sub i64 %p,%start
  %lower = icmp uge i64 %p,%start
  %upper = icmp ult i64 %offset,{{ROOT_TOKEN_BYTES}}
  %remainder = urem i64 %offset,24
  %aligned = icmp eq i64 %remainder,0
  %a = and i1 %owned,%nowrap
  %b = and i1 %lower,%upper
  %ab = and i1 %a,%b
  %root = and i1 %ab,%aligned
  %valid = or i1 %pool,%root
  ret i1 %valid
}
