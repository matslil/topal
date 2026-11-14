; Private current invocation lease. Only the owning machine gateway writes it.
declare void @topal_x86_frame_gateway_abort_v1() noreturn
@topal_x86_frame_gateway_recovery_v1 = external global [6 x i64]
define internal i1 @g.lease() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %live = call i64 @t.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 1)
  %saved = call i64 @t.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 0)
  %pin = icmp eq i64 %live,1
  %low = icmp uge i64 %saved,1085456
  %high = icmp ule i64 %saved,1572808
  %mask = and i64 %saved,15
  %alignment = icmp eq i64 %mask,8
  %sp = call i64 asm sideeffect "movq %rsp, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  %bottom = sub i64 %saved,{{CALLER_RETIRED}}
  %above = icmp uge i64 %sp,%bottom
  %below = icmp ule i64 %sp,%saved
  %a = and i1 %pin,%low
  %b = and i1 %high,%alignment
  %c = and i1 %above,%below
  %ab = and i1 %a,%b
  %owned = and i1 %ab,%c
  ret i1 %owned
}
