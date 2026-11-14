; Read-only architectural observations and complete retained boot checks.
define internal i64 @k.boot.cr0() {
  %v = call i64 asm sideeffect "movq %cr0, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  ret i64 %v
}
define internal i64 @k.boot.cr4() {
  %v = call i64 asm sideeffect "movq %cr4, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  ret i64 %v
}
define internal i64 @k.boot.sp() {
  %v = call i64 asm sideeffect "movq %rsp, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  ret i64 %v
}
define internal i64 @k.boot.flags() {
  %v = call i64 asm sideeffect "pushfq; popq $0", "=r,~{dirflag},~{fpsr},~{flags},~{memory}"()
  ret i64 %v
}
define internal i64 @k.boot.cs() {
  %v = call i16 asm sideeffect "movw %cs, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  %n = zext i16 %v to i64
  ret i64 %n
}
define internal {i32,i32,i32,i32} @k.boot.cpuid(i32 %leaf) {
  %v = call {i32,i32,i32,i32} asm sideeffect "cpuid", "={ax},={bx},={cx},={dx},0,2,~{dirflag},~{fpsr},~{flags}"(i32 %leaf,i32 0)
  ret {i32,i32,i32,i32} %v
}
define void @topal_x86_image_boot_msr_observe_v1(i64 %number,i64 %value) {
  call void asm sideeffect "", "~{memory},~{dirflag},~{fpsr},~{flags}"()
  ret void
}
define internal i64 @k.boot.msr(i32 %number) {
  %v = call {i32,i32} asm sideeffect "rdmsr", "={ax},={dx},{cx},~{dirflag},~{fpsr},~{flags}"(i32 %number)
  %l = extractvalue {i32,i32} %v,0
  %h = extractvalue {i32,i32} %v,1
  %lo = zext i32 %l to i64
  %hi = zext i32 %h to i64
  %up = shl i64 %hi,32
  %word = or i64 %lo,%up
  %id = zext i32 %number to i64
  call void @topal_x86_image_boot_msr_observe_v1(i64 %id,i64 %word)
  ret i64 %word
}
define internal void @k.boot.record(i64 %index,i64 %value,i1 %save) {
  %p = getelementptr i64,ptr @topal_x86_image_boot_observation_v1,i64 %index
  br i1 %save,label %put,label %check
put:
  store volatile i64 %value,ptr %p,align 8
  ret void
check:
  %old = load volatile i64,ptr %p,align 8
  %same = icmp eq i64 %old,%value
  call void @k.image.require(i1 %same)
  ret void
}
define internal void @k.boot.tables() {
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%loop]
  %table = lshr i64 %i,9
  %index = and i64 %i,511
  %at = getelementptr [4 x i64],ptr @k.boot.table_addresses,i64 0,i64 %table
  %base = load i64,ptr %at,align 8
  %bp = inttoptr i64 %base to ptr
  %actual = call i64 @t.word(ptr %bp,i64 %index)
  %ep = getelementptr [2048 x i64],ptr @k.boot.table_words,i64 0,i64 %i
  %expected = load i64,ptr %ep,align 8
  %leaf = icmp uge i64 %table,2
  %ad = select i1 %leaf,i64 96,i64 32
  %present = icmp ne i64 %expected,0
  %mask = select i1 %present,i64 %ad,i64 0
  %keep = xor i64 %mask,-1
  %normalized = and i64 %actual,%keep
  %same = icmp eq i64 %normalized,%expected
  call void @k.image.require(i1 %same)
  %next = add i64 %i,1
  %more = icmp ult i64 %next,2048
  br i1 %more,label %loop,label %done
done:
  ret void
}
define internal void @k.boot.transition() {
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%loop]
  %p = getelementptr i8,ptr inttoptr (i64 1048576 to ptr),i64 %i
  %actual = load volatile i8,ptr %p,align 1
  %reference = getelementptr [{{TRANSITION_BYTES}} x i8],ptr @k.boot.transition_bytes,i64 0,i64 %i
  %byte = load i8,ptr %reference,align 1
  %start = icmp uge i64 %i,{{ENTRY_FIXUP}}
  %end = icmp ult i64 %i,{{ENTRY_FIXUP_END}}
  %immediate = and i1 %start,%end
  %offset = sub i64 %i,{{ENTRY_FIXUP}}
  %bits = shl i64 %offset,3
  %shift = and i64 %bits,63
  %entry = ptrtoint ptr @{{KERNEL_ENTRY}} to i64
  %part = lshr i64 %entry,%shift
  %entrybyte = trunc i64 %part to i8
  %expected = select i1 %immediate,i8 %entrybyte,i8 %byte
  %same = icmp eq i8 %actual,%expected
  call void @k.image.require(i1 %same)
  %next = add i64 %i,1
  %more = icmp ult i64 %next,{{TRANSITION_BYTES}}
  br i1 %more,label %loop,label %done
done:
  ret void
}
define internal void @k.boot.descriptors(i1 %save) {
  %g = alloca [10 x i8],align 8
  %t = alloca [10 x i8],align 8
  call void asm sideeffect "sgdt ($0)", "r,~{memory},~{dirflag},~{fpsr},~{flags}"(ptr %g)
  call void asm sideeffect "sidt ($0)", "r,~{memory},~{dirflag},~{fpsr},~{flags}"(ptr %t)
  %gl = load volatile i16,ptr %g,align 1
  %tl = load volatile i16,ptr %t,align 1
  %gp = getelementptr i8,ptr %g,i64 2
  %tp = getelementptr i8,ptr %t,i64 2
  %gb = load volatile i64,ptr %gp,align 1
  %tb = load volatile i64,ptr %tp,align 1
  %glen = zext i16 %gl to i64
  %tlen = zext i16 %tl to i64
  %gbase = icmp eq i64 %gb,1064960
  %tbase = icmp eq i64 %tb,1069056
  %gsize = icmp eq i16 %gl,47
  %tsize = icmp eq i16 %tl,4095
  %a = and i1 %gbase,%tbase
  %b = and i1 %gsize,%tsize
  %ok = and i1 %a,%b
  call void @k.image.require(i1 %ok)
  call void @k.boot.record(i64 9,i64 %gb,i1 %save)
  call void @k.boot.record(i64 10,i64 %glen,i1 %save)
  call void @k.boot.record(i64 11,i64 %tb,i1 %save)
  call void @k.boot.record(i64 12,i64 %tlen,i1 %save)
  %saved_limit = load volatile i16,ptr inttoptr (i64 1073152 to ptr),align 1
  %saved_base = load volatile i64,ptr inttoptr (i64 1073154 to ptr),align 1
  %same_limit = icmp eq i16 %saved_limit,%tl
  %same_base = icmp eq i64 %saved_base,%tb
  %same_pointer = and i1 %same_limit,%same_base
  call void @k.image.require(i1 %same_pointer)
  br label %gdt
gdt:
  %i = phi i64 [0,%0],[%next,%gdt]
  %p = inttoptr i64 %gb to ptr
  %actual = call i64 @t.word(ptr %p,i64 %i)
  %ep = getelementptr [6 x i64],ptr @k.boot.gdt,i64 0,i64 %i
  %expected = load i64,ptr %ep,align 8
  %with_access_masked = and i64 %actual,-1099511627777
  %null = icmp eq i64 %i,0
  %normalized = select i1 %null,i64 %actual,i64 %with_access_masked
  %same = icmp eq i64 %normalized,%expected
  call void @k.image.require(i1 %same)
  %next = add i64 %i,1
  %more = icmp ult i64 %next,6
  br i1 %more,label %gdt,label %idt
idt:
  %j = phi i64 [0,%gdt],[%after,%idt]
  %vector = lshr i64 %j,1
  %which = and i64 %j,1
  %low = icmp eq i64 %which,0
  %debug = icmp eq i64 %vector,3
  %fault = icmp eq i64 %vector,14
  %local = icmp eq i64 %vector,241
  %deadline = icmp eq i64 %vector,242
  %debug_target = ptrtoint ptr @{{DEBUG_ENTRY}} to i64
  %fault_target = ptrtoint ptr @{{FAULT_ENTRY}} to i64
  %local_target = ptrtoint ptr @{{LOCAL_ENTRY}} to i64
  %deadline_target = ptrtoint ptr @{{DEADLINE_ENTRY}} to i64
  %s0 = select i1 %debug,i64 %debug_target,i64 0
  %s1 = select i1 %fault,i64 %fault_target,i64 %s0
  %s2 = select i1 %local,i64 %local_target,i64 %s1
  %target = select i1 %deadline,i64 %deadline_target,i64 %s2
  %has = icmp ne i64 %target,0
  %bottom = and i64 %target,65535
  %middle_raw = and i64 %target,4294901760
  %middle = shl i64 %middle_raw,32
  %attr = select i1 %debug,i64 157230162771968,i64 156130651144192
  %with_selector = or i64 %bottom,1048576
  %with_attr = or i64 %with_selector,%attr
  %lower = or i64 %with_attr,%middle
  %upper = lshr i64 %target,32
  %part = select i1 %low,i64 %lower,i64 %upper
  %ew = select i1 %has,i64 %part,i64 0
  %it = inttoptr i64 %tb to ptr
  %aw = call i64 @t.word(ptr %it,i64 %j)
  %equal = icmp eq i64 %aw,%ew
  call void @k.image.require(i1 %equal)
  %after = add i64 %j,1
  %rest = icmp ult i64 %after,512
  br i1 %rest,label %idt,label %done
done:
  ret void
}
define internal void @k.boot.handoff(i1 %save) {
  %boot = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 0)
  %end = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 1)
  %resource = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 2)
  %resource_end = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 3)
  %n = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 4)
  %actualn = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 3)
  %nonzero = icmp ne i64 %boot,0
  %bounded = icmp ule i64 %boot,16773120
  %image_end = ptrtoint ptr @topal_x86_kernel_reservation_end_v1 to i64
  %same_end = icmp eq i64 %end,%image_end
  %same_n = icmp eq i64 %n,%actualn
  %after = icmp uge i64 %resource,%end
  %inside = icmp ule i64 %resource_end,1073741824
  %size = icmp ugt i64 %resource_end,%resource
  %a = and i1 %nonzero,%bounded
  %b = and i1 %same_end,%same_n
  %c = and i1 %after,%inside
  %ab = and i1 %a,%b
  %cs = and i1 %c,%size
  %ok = and i1 %ab,%cs
  call void @k.image.require(i1 %ok)
  %p = inttoptr i64 %boot to ptr
  %rp = getelementptr i8,ptr %p,i64 536
  %lp = getelementptr i8,ptr %p,i64 540
  %rv = load volatile i32,ptr %rp,align 1
  %lv = load volatile i32,ptr %lp,align 1
  %r64 = zext i32 %rv to i64
  %l64 = zext i32 %lv to i64
  %limit = add i64 %r64,%l64
  %same_r = icmp eq i64 %r64,%resource
  %same_l = icmp eq i64 %limit,%resource_end
  %same = and i1 %same_r,%same_l
  call void @k.image.require(i1 %same)
  call void @k.boot.record(i64 16,i64 %boot,i1 %save)
  call void @k.boot.record(i64 17,i64 %resource,i1 %save)
  call void @k.boot.record(i64 18,i64 %resource_end,i1 %save)
  call void @k.boot.record(i64 19,i64 %end,i1 %save)
  call void @k.boot.record(i64 23,i64 %n,i1 %save)
  ret void
}
define internal void @k.boot.verify(i1 %save) {
  %phase = call i64 @t.word(ptr @topal_x86_image_boot_observation_v1,i64 0)
  %wanted = select i1 %save,i64 0,i64 1
  %fresh = icmp eq i64 %phase,%wanted
  call void @k.image.require(i1 %fresh)
  %cr0 = call i64 @k.boot.cr0()
  %cr3 = call i64 @k.image.active_root()
  %cr4 = call i64 @k.boot.cr4()
  %flags = call i64 @k.boot.flags()
  %cs = call i64 @k.boot.cs()
  %sp = call i64 @k.boot.sp()
  %selected0 = and i64 %cr0,3758096385
  %mode0 = icmp eq i64 %selected0,2147483649
  %mode3 = icmp eq i64 %cr3,1052672
  %mode4 = icmp eq i64 %cr4,32
  %privileged = icmp eq i64 %cs,16
  %critical = and i64 %flags,1792
  %masked = icmp eq i64 %critical,0
  %above = icmp uge i64 %sp,1085504
  %below = icmp ult i64 %sp,1572864
  %a = and i1 %mode0,%mode3
  %b = and i1 %mode4,%privileged
  %c = and i1 %masked,%above
  %ab = and i1 %a,%b
  %cb = and i1 %c,%below
  %valid = and i1 %ab,%cb
  call void @k.image.require(i1 %valid)
  %maximum = call {i32,i32,i32,i32} @k.boot.cpuid(i32 0)
  %max = extractvalue {i32,i32,i32,i32} %maximum,0
  %has = icmp uge i32 %max,1
  call void @k.image.require(i1 %has)
  %basic = call {i32,i32,i32,i32} @k.boot.cpuid(i32 1)
  %edx = extractvalue {i32,i32,i32,i32} %basic,3
  %ecx = extractvalue {i32,i32,i32,i32} %basic,2
  %needed = and i32 %edx,70248
  %features = icmp eq i32 %needed,70248
  call void @k.image.require(i1 %features)
  %extended_max = call {i32,i32,i32,i32} @k.boot.cpuid(i32 -2147483648)
  %xm = extractvalue {i32,i32,i32,i32} %extended_max,0
  %extended = icmp uge i32 %xm,-2147483647
  call void @k.image.require(i1 %extended)
  %extra = call {i32,i32,i32,i32} @k.boot.cpuid(i32 -2147483647)
  %xd = extractvalue {i32,i32,i32,i32} %extra,3
  %nx_lm = and i32 %xd,537919488
  %nx = icmp eq i32 %nx_lm,537919488
  call void @k.image.require(i1 %nx)
  %efer = call i64 @k.boot.msr(i32 -1073741696)
  %lm = and i64 %efer,1280
  %lma = icmp eq i64 %lm,1280
  %other = and i64 %efer,-3330
  %plain = icmp eq i64 %other,0
  %efok = and i1 %lma,%plain
  call void @k.image.require(i1 %efok)
  %apic = call i64 @k.boot.msr(i32 27)
  %apic_ok = icmp eq i64 %apic,4276095232
  call void @k.image.require(i1 %apic_ok)
  %pat = call i64 @k.boot.msr(i32 631)
  %cap = call i64 @k.boot.msr(i32 254)
  %def = call i64 @k.boot.msr(i32 767)
  %count = and i64 %cap,255
  %fits = icmp ule i64 %count,16
  call void @k.image.require(i1 %fits)
  call void @k.boot.record(i64 1,i64 %cr0,i1 %save)
  call void @k.boot.record(i64 2,i64 %cr3,i1 %save)
  call void @k.boot.record(i64 3,i64 %cr4,i1 %save)
  call void @k.boot.record(i64 4,i64 %efer,i1 %save)
  call void @k.boot.record(i64 5,i64 %apic,i1 %save)
  call void @k.boot.record(i64 6,i64 %pat,i1 %save)
  call void @k.boot.record(i64 7,i64 %cap,i1 %save)
  call void @k.boot.record(i64 8,i64 %def,i1 %save)
  call void @k.boot.record(i64 14,i64 %cs,i1 %save)
  %edx64 = zext i32 %edx to i64
  %ecx64 = zext i32 %ecx to i64
  %xd64 = zext i32 %xd to i64
  call void @k.boot.record(i64 20,i64 %edx64,i1 %save)
  call void @k.boot.record(i64 21,i64 %ecx64,i1 %save)
  call void @k.boot.record(i64 22,i64 %xd64,i1 %save)
  call void @k.boot.record(i64 24,i64 2048,i1 %save)
  call void @k.boot.record(i64 25,i64 512,i1 %save)
  call void @k.boot.record(i64 26,i64 {{TRANSITION_BYTES}},i1 %save)
  %fixed = and i64 %cap,256
  call void @k.boot.record(i64 27,i64 %fixed,i1 %save)
  call void @k.boot.record(i64 28,i64 %count,i1 %save)
  br i1 %save,label %initial,label %attributes
initial:
  call void @k.boot.record(i64 13,i64 %sp,i1 true)
  call void @k.boot.record(i64 15,i64 %flags,i1 true)
  br label %attributes
attributes:
  %registers = shl i64 %count,1
  br label %variable
variable:
  %i = phi i64 [0,%attributes],[%next,%varbody]
  %more = icmp ult i64 %i,%registers
  br i1 %more,label %varbody,label %fixed_start
varbody:
  %number = add i64 %i,512
  %msr = trunc i64 %number to i32
  %value = call i64 @k.boot.msr(i32 %msr)
  %slot = add i64 %i,64
  call void @k.boot.record(i64 %slot,i64 %value,i1 %save)
  %next = add i64 %i,1
  br label %variable
fixed_start:
  %have_fixed = icmp ne i64 %fixed,0
  br i1 %have_fixed,label %fixed_loop,label %contents
fixed_loop:
  %j = phi i64 [0,%fixed_start],[%after,%fixed_loop]
  %pointer = getelementptr [11 x i32],ptr @k.boot.fixed_msrs,i64 0,i64 %j
  %address = load i32,ptr %pointer,align 4
  %v = call i64 @k.boot.msr(i32 %address)
  %at = add i64 %j,40
  call void @k.boot.record(i64 %at,i64 %v,i1 %save)
  %after = add i64 %j,1
  %remaining = icmp ult i64 %after,11
  br i1 %remaining,label %fixed_loop,label %contents
contents:
  call void @k.boot.tables()
  call void @k.boot.transition()
  call void @k.boot.descriptors(i1 %save)
  call void @k.boot.handoff(i1 %save)
  fence seq_cst
  call void @k.boot.record(i64 0,i64 1,i1 true)
  ret void
}
