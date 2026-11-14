; Boot-adapter-owned setup page. No startup dispatch, entry or release.
@topal_x86_boot_startup_storage_v1 = global [32 x i64] zeroinitializer, align 8
@s.setup_snapshot = private global [4096 x i8] zeroinitializer, align 8

define internal i64 @s.get(i64 %index) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %p = getelementptr [32 x i64],ptr @topal_x86_boot_startup_storage_v1,i64 0,i64 %index
  %v = load volatile i64,ptr %p,align 8
  ret i64 %v
}
define internal void @s.put(i64 %index,i64 %value) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %p = getelementptr [32 x i64],ptr @topal_x86_boot_startup_storage_v1,i64 0,i64 %index
  store volatile i64 %value,ptr %p,align 8
  ret void
}
define internal void @s.fact(i64 %index,i64 %value,i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  br i1 %save,label %write,label %compare
write:
  call void @s.put(i64 %index,i64 %value)
  ret void
compare:
  %old = call i64 @s.get(i64 %index)
  %same = icmp eq i64 %old,%value
  call void @k.image.require(i1 %same)
  ret void
}
define internal i64 @s.boot(i64 %index) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %p = getelementptr [128 x i64],ptr @topal_x86_image_boot_observation_v1,i64 0,i64 %index
  %v = load volatile i64,ptr %p,align 8
  ret i64 %v
}
define internal i64 @s.expected(i64 %index) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %delta = mul i64 %index,{{STARTUP_STRIDE}}
  %value = add i64 %delta,{{STARTUP_SEED}}
  ret i64 %value
}
define void @topal_x86_boot_startup_observe_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  ret void
}
define internal i64 @s.inputs(i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %boot = call i64 @s.boot(i64 16)
  %positive = icmp uge i64 %boot,4096
  %low = icmp ule i64 %boot,647168
  %mask = and i64 %boot,4095
  %aligned = icmp eq i64 %mask,0
  %a = and i1 %positive,%low
  %bounded = and i1 %a,%aligned
  call void @k.image.require(i1 %bounded)
  %base = add i64 %boot,4096
  %end = add i64 %boot,8192
  %sectors = call i64 @k.fw.u8(i64 %boot,i64 497)
  %magic = call i64 @k.fw.u32(i64 %boot,i64 514)
  %protocol = call i64 @k.fw.u16(i64 %boot,i64 518)
  %entry = call i64 @k.fw.u32(i64 %boot,i64 532)
  %extra = call i64 @k.fw.u64(i64 %boot,i64 592)
  %extcmd = call i64 @k.fw.u32(i64 %boot,i64 200)
  %xload = call i64 @k.fw.u16(i64 %boot,i64 566)
  %s = icmp eq i64 %sectors,15
  %m = icmp eq i64 %magic,1400005704
  %p = icmp eq i64 %protocol,527
  %e = icmp eq i64 %entry,1048576
  %z = icmp eq i64 %extra,0
  %x = icmp eq i64 %extcmd,0
  %l = icmp eq i64 %xload,0
  %h0 = and i1 %s,%m
  %h1 = and i1 %p,%e
  %h2 = and i1 %z,%x
  %h3 = and i1 %h0,%h1
  %h4 = and i1 %h2,%l
  %header = and i1 %h3,%h4
  call void @k.image.require(i1 %header)
  %segment = call i64 @k.fw.u16(i64 1038,i64 0)
  %ebda = shl i64 %segment,4
  %absent = icmp eq i64 %ebda,0
  %below = icmp ule i64 %end,%ebda
  %ebda_ok = or i1 %absent,%below
  call void @k.image.require(i1 %ebda_ok)
  %cmd = call i64 @k.fw.u32(i64 %boot,i64 552)
  %cmdend = add i64 %cmd,2048
  %no_cmd = icmp eq i64 %cmd,0
  %cmd_before = icmp ule i64 %cmdend,%boot
  %cmd_after = icmp uge i64 %cmd,%end
  %separate_cmd = or i1 %cmd_before,%cmd_after
  %cmd_ok = or i1 %no_cmd,%separate_cmd
  call void @k.image.require(i1 %cmd_ok)
  %count = call i64 @k.fw.u8(i64 %boot,i64 488)
  %nonempty = icmp ugt i64 %count,0
  %fits = icmp ule i64 %count,128
  %count_ok = and i1 %nonempty,%fits
  call void @k.image.require(i1 %count_ok)
  br label %claims
claims:
  %i = phi i64 [0,%0],[%next,%claim]
  %covered = phi i1 [false,%0],[%newcovered,%claim]
  %done = icmp eq i64 %i,%count
  br i1 %done,label %facts,label %claim
claim:
  %delta = mul i64 %i,20
  %offset = add i64 %delta,720
  %record = add i64 %boot,%offset
  %start = call i64 @k.fw.u64(i64 %record,i64 0)
  %length = call i64 @k.fw.u64(i64 %record,i64 8)
  %kind = call i64 @k.fw.u32(i64 %record,i64 16)
  %limit = add i64 %start,%length
  %nowrap = icmp uge i64 %limit,%start
  call void @k.image.require(i1 %nowrap)
  %ram = icmp eq i64 %kind,1
  %begins = icmp ule i64 %start,%boot
  %ends = icmp uge i64 %limit,%end
  %covers = and i1 %begins,%ends
  %available = and i1 %ram,%covers
  %newcovered = or i1 %covered,%available
  %left = icmp ult i64 %start,%end
  %right = icmp ult i64 %boot,%limit
  %nonzero = icmp ne i64 %length,0
  %overlap0 = and i1 %left,%right
  %overlap = and i1 %overlap0,%nonzero
  %outside = xor i1 %overlap,true
  %compatible = or i1 %ram,%outside
  call void @k.image.require(i1 %compatible)
  %next = add i64 %i,1
  br label %claims
facts:
  call void @k.image.require(i1 %covered)
  %reservation = ptrtoint ptr @topal_x86_kernel_reservation_end_v1 to i64
  %excluded = icmp ule i64 %end,%reservation
  call void @k.image.require(i1 %excluded)
  call void @s.fact(i64 1,i64 %boot,i1 %save)
  call void @s.fact(i64 2,i64 %base,i1 %save)
  call void @s.fact(i64 3,i64 4096,i1 %save)
  call void @s.fact(i64 4,i64 8192,i1 %save)
  call void @s.fact(i64 5,i64 1,i1 %save)
  call void @s.fact(i64 6,i64 1,i1 %save)
  call void @s.fact(i64 7,i64 0,i1 %save)
  call void @s.fact(i64 8,i64 %count,i1 %save)
  call void @s.fact(i64 9,i64 %ebda,i1 %save)
  call void @s.fact(i64 10,i64 %cmd,i1 %save)
  %root = call i64 @s.boot(i64 2)
  call void @s.fact(i64 11,i64 %root,i1 %save)
  call void @s.fact(i64 12,i64 %reservation,i1 %save)
  %resource = call i64 @s.boot(i64 17)
  %resource_end = call i64 @s.boot(i64 18)
  %high = icmp uge i64 %resource,%reservation
  call void @k.image.require(i1 %high)
  call void @s.fact(i64 13,i64 %resource,i1 %save)
  call void @s.fact(i64 14,i64 %resource_end,i1 %save)
  call void @s.fact(i64 15,i64 {{STARTUP_SEED}},i1 %save)
  call void @s.fact(i64 16,i64 {{STARTUP_STRIDE}},i1 %save)
  call void @s.fact(i64 17,i64 {{STARTUP_MASK}},i1 %save)
  br label %snapshot
snapshot:
  %b = phi i64 [0,%facts],[%after,%advance]
  %address = add i64 %boot,%b
  %value = call i64 @k.fw.u8(i64 %address,i64 0)
  %byte = trunc i64 %value to i8
  %copy = getelementptr [4096 x i8],ptr @s.setup_snapshot,i64 0,i64 %b
  br i1 %save,label %store,label %compare
store:
  store volatile i8 %byte,ptr %copy,align 1
  br label %advance
compare:
  %old = load volatile i8,ptr %copy,align 1
  %same = icmp eq i8 %old,%byte
  call void @k.image.require(i1 %same)
  br label %advance
advance:
  %after = add i64 %b,1
  %complete = icmp eq i64 %after,4096
  br i1 %complete,label %return,label %snapshot
return:
  ret i64 %base
}
define internal void @s.page(i64 %base,i1 %probe,i1 %write) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%advance]
  %expected = call i64 @s.expected(i64 %i)
  %changed = xor i64 %expected,{{STARTUP_MASK}}
  %value = select i1 %probe,i64 %changed,i64 %expected
  %offset = shl i64 %i,3
  %address = add i64 %base,%offset
  %p = inttoptr i64 %address to ptr
  br i1 %write,label %store,label %read
store:
  store volatile i64 %value,ptr %p,align 8
  br label %advance
read:
  %actual = load volatile i64,ptr %p,align 8
  %same = icmp eq i64 %actual,%value
  call void @k.image.require(i1 %same)
  br label %advance
advance:
  %next = add i64 %i,1
  %done = icmp eq i64 %next,512
  br i1 %done,label %return,label %loop
return:
  ret void
}
define internal void @s.cover(i64 %base,i64 %b,i64 %p,i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void @k.cover.page(i64 %b,i64 %base,i64 0)
  call void @k.cover.page(i64 %p,i64 %base,i64 0)
  call void @s.fact(i64 18,i64 1,i1 %save)
  call void @s.fact(i64 19,i64 1,i1 %save)
  ret void
}
define internal void @s.startup.claim(i64 %b,i64 %p) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @s.get(i64 0)
  %fresh = icmp eq i64 %phase,0
  call void @k.image.require(i1 %fresh)
  %base = call i64 @s.inputs(i1 true)
  call void @s.page(i64 %base,i1 false,i1 false)
  call void @s.cover(i64 %base,i64 %b,i64 %p,i1 true)
  call void @s.put(i64 0,i64 1)
  call void @topal_x86_boot_startup_observe_v1()
  call void @s.page(i64 %base,i1 true,i1 true)
  call void @s.page(i64 %base,i1 true,i1 false)
  call void @s.put(i64 0,i64 2)
  call void @topal_x86_boot_startup_observe_v1()
  call void @s.page(i64 %base,i1 false,i1 true)
  call void @s.page(i64 %base,i1 false,i1 false)
  call void @s.put(i64 0,i64 3)
  call void @topal_x86_boot_startup_observe_v1()
  ret void
}
define internal void @s.startup.verify(i64 %b,i64 %p) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @s.get(i64 0)
  %held = icmp eq i64 %phase,3
  call void @k.image.require(i1 %held)
  %base = call i64 @s.inputs(i1 false)
  call void @s.page(i64 %base,i1 false,i1 false)
  call void @s.cover(i64 %base,i64 %b,i64 %p,i1 false)
  call void @s.put(i64 0,i64 4)
  ret void
}
