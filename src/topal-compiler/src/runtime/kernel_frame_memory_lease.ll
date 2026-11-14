@topal_bootstrap_storage = external global [0 x i8]
@topal_x86_frame_mm_tokens_v1 = external global [6 x i64]
@topal_x86_frame_mm_state_v1 = external global [4 x i64]
@topal_x86_frame_mm_epoch_v1 = external global [2 x i64]
@topal_x86_frame_mm_brk_return_v1 = external global i8
@topal_x86_frame_mm_exec_return_v1 = external global i8
@topal_x86_frame_mm_cleanup_return_v1 = external global i8

define internal i1 @g.memory_context() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %op = call i64 @t.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 2)
  %ctx = getelementptr i8,ptr @topal_bootstrap_storage,i64 512
  %state = call i64 @t.word(ptr %ctx,i64 0)
  %saved = call i64 @t.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 0)
  %identity = call i64 @t.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 5)
  %boot = icmp ult i64 %op,5
  br i1 %boot,label %bootstrap,label %memory
bootstrap:
  %notUser = icmp ne i64 %state,1
  %noIdentity = icmp eq i64 %identity,0
  %normal = and i1 %notUser,%noIdentity
  ret i1 %normal
memory:
  %known = icmp ule i64 %op,7
  %cleanup = icmp eq i64 %op,6
  %liveState = select i1 %cleanup,i64 2,i64 1
  %live = icmp eq i64 %state,%liveState
  %stack = call i64 @t.word(ptr %ctx,i64 1)
  %expected = add i64 %saved,64
  %sameStack = icmp eq i64 %stack,%expected
  %offset = select i1 %cleanup,i64 2,i64 3
  %root = call i64 @t.word(ptr %ctx,i64 %offset)
  %cr3 = call i64 asm sideeffect "movq %cr3, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  %sameRoot = icmp eq i64 %root,%cr3
  %msr = select i1 %cleanup,i32 -1073741566,i32 -1073741567
  %halves = call {i32,i32} asm sideeffect "rdmsr", "={ax},={dx},{cx},~{dirflag},~{fpsr},~{flags}"(i32 %msr)
  %lo32 = extractvalue {i32,i32} %halves,0
  %hi32 = extractvalue {i32,i32} %halves,1
  %lo = zext i32 %lo32 to i64
  %hi = zext i32 %hi32 to i64
  %upper = shl i64 %hi,32
  %gs = or i64 %lo,%upper
  %contextWord = ptrtoint ptr %ctx to i64
  %sameGs = icmp eq i64 %gs,%contextWord
  %epoch = call i64 @t.word(ptr @topal_x86_frame_mm_epoch_v1,i64 0)
  %number = call i64 @t.word(ptr @topal_x86_frame_mm_epoch_v1,i64 1)
  %sameEpoch = icmp eq i64 %epoch,%identity
  %nonzero = icmp ne i64 %epoch,0
  %brk = icmp eq i64 %op,5
  %request = select i1 %brk,i64 12,i64 59
  %requestOk = icmp eq i64 %number,%request
  %exit = icmp eq i64 %number,60
  %exitGroup = icmp eq i64 %number,231
  %exitOk = or i1 %exit,%exitGroup
  %abi = select i1 %cleanup,i1 %exitOk,i1 %requestOk
  %brkReturn = ptrtoint ptr @topal_x86_frame_mm_brk_return_v1 to i64
  %execReturn = ptrtoint ptr @topal_x86_frame_mm_exec_return_v1 to i64
  %cleanupReturn = ptrtoint ptr @topal_x86_frame_mm_cleanup_return_v1 to i64
  %syscallReturn = select i1 %brk,i64 %brkReturn,i64 %execReturn
  %returnSite = select i1 %cleanup,i64 %cleanupReturn,i64 %syscallReturn
  %wordAt = add i64 %saved,48
  %returnAt = inttoptr i64 %wordAt to ptr
  %actual = load volatile i64,ptr %returnAt,align 8
  %sameSite = icmp eq i64 %actual,%returnSite
  %a = and i1 %known,%live
  %b = and i1 %sameStack,%sameRoot
  %c = and i1 %sameGs,%sameEpoch
  %d = and i1 %nonzero,%abi
  %ab = and i1 %a,%b
  %cd = and i1 %c,%d
  %abcd = and i1 %ab,%cd
  %valid = and i1 %abcd,%sameSite
  ret i1 %valid
}

define i1 @topal_frame_kernel_token_domain_v1(ptr %ctx,ptr %token,i1 %write) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %old = call i1 @mm.previous_token_domain(ptr %ctx,ptr %token,i1 %write)
  %owned = icmp eq ptr %ctx,@topal_x86_frame_context_v1
  %p = ptrtoint ptr %token to i64
  %base = ptrtoint ptr @topal_x86_frame_mm_tokens_v1 to i64
  %second = add i64 %base,24
  %first = icmp eq i64 %p,%base
  %next = icmp eq i64 %p,%second
  %either = or i1 %first,%next
  %mm = and i1 %owned,%either
  %ok = or i1 %old,%mm
  ret i1 %ok
}
