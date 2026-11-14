; TOPAL-SYSTEMS-BOOT-PROCESSOR-TOPOLOGY-001. Private bounded read-only discovery.
@topal_x86_boot_processors_v1 = global [128 x i64] zeroinitializer, align 8
@topal_x86_boot_firmware_bytes_v1 = global [8192 x i8] zeroinitializer, align 8

define internal i64 @k.fw.u8(i64 %base,i64 %offset) {
  %address = add i64 %base,%offset
  %p = inttoptr i64 %address to ptr
  %v = load volatile i8,ptr %p,align 1
  %word = zext i8 %v to i64
  ret i64 %word
}
define internal i64 @k.fw.u16(i64 %base,i64 %offset) {
  %address = add i64 %base,%offset
  %p = inttoptr i64 %address to ptr
  %v = load volatile i16,ptr %p,align 1
  %word = zext i16 %v to i64
  ret i64 %word
}
define internal i64 @k.fw.u32(i64 %base,i64 %offset) {
  %address = add i64 %base,%offset
  %p = inttoptr i64 %address to ptr
  %v = load volatile i32,ptr %p,align 1
  %word = zext i32 %v to i64
  ret i64 %word
}
define internal i64 @k.fw.u64(i64 %base,i64 %offset) {
  %address = add i64 %base,%offset
  %p = inttoptr i64 %address to ptr
  %v = load volatile i64,ptr %p,align 1
  ret i64 %v
}
define internal void @k.fw.record(i64 %index,i64 %value,i1 %save) {
  %p = getelementptr [128 x i64],ptr @topal_x86_boot_processors_v1,i64 0,i64 %index
  br i1 %save,label %initial,label %compare
initial:
  store volatile i64 %value,ptr %p,align 8
  ret void
compare:
  %old = load volatile i64,ptr %p,align 8
  %same = icmp eq i64 %old,%value
  call void @k.image.require(i1 %same)
  ret void
}
define internal i1 @k.fw.checksum(i64 %base,i64 %length) {
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%loop]
  %sum = phi i64 [0,%0],[%added,%loop]
  %v = call i64 @k.fw.u8(i64 %base,i64 %i)
  %added = add i64 %sum,%v
  %next = add i64 %i,1
  %more = icmp ult i64 %next,%length
  br i1 %more,label %loop,label %done
done:
  %byte = and i64 %added,255
  %ok = icmp eq i64 %byte,0
  ret i1 %ok
}
define internal i64 @k.fw.bind(i64 %base,i64 %length,ptr %cursor,i1 %save) {
  %at = load i64,ptr %cursor,align 8
  %positive = icmp ugt i64 %length,0
  %bounded = icmp ule i64 %length,8192
  %space = sub i64 8192,%length
  %fits = icmp ule i64 %at,%space
  %a = and i1 %positive,%bounded
  %ok = and i1 %a,%fits
  call void @k.image.require(i1 %ok)
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%advance]
  %value = call i64 @k.fw.u8(i64 %base,i64 %i)
  %v = trunc i64 %value to i8
  %offset = add i64 %at,%i
  %p = getelementptr [8192 x i8],ptr @topal_x86_boot_firmware_bytes_v1,i64 0,i64 %offset
  br i1 %save,label %initial,label %compare
initial:
  store volatile i8 %v,ptr %p,align 1
  br label %advance
compare:
  %old = load volatile i8,ptr %p,align 1
  %same = icmp eq i8 %old,%v
  call void @k.image.require(i1 %same)
  br label %advance
advance:
  %next = add i64 %i,1
  %more = icmp ult i64 %next,%length
  br i1 %more,label %loop,label %done
done:
  %end = add i64 %at,%length
  store i64 %end,ptr %cursor,align 8
  %start = getelementptr [8192 x i8],ptr @topal_x86_boot_firmware_bytes_v1,i64 0,i64 %at
  %snapshot = ptrtoint ptr %start to i64
  ret i64 %snapshot
}
define internal void @k.fw.equal(i64 %left,i64 %right,i64 %length) {
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%loop]
  %a = call i64 @k.fw.u8(i64 %left,i64 %i)
  %b = call i64 @k.fw.u8(i64 %right,i64 %i)
  %same = icmp eq i64 %a,%b
  call void @k.image.require(i1 %same)
  %next = add i64 %i,1
  %more = icmp ult i64 %next,%length
  br i1 %more,label %loop,label %done
done:
  ret void
}
define internal void @k.fw.checked_copy(i64 %snapshot,i64 %length) {
  %actual = call i64 @k.fw.u32(i64 %snapshot,i64 4)
  %same = icmp eq i64 %actual,%length
  call void @k.image.require(i1 %same)
  %valid = call i1 @k.fw.checksum(i64 %snapshot,i64 %length)
  call void @k.image.require(i1 %valid)
  ret void
}
; SDT bytes must remain outside allocator-eligible RAM and conflicting claims.
define internal void @k.fw.range(i64 %base,i64 %length) {
  %end = add i64 %base,%length
  %nonzero = icmp ne i64 %base,0
  %positive = icmp ugt i64 %end,%base
  %limit = icmp ule i64 %end,1073741824
  %a = and i1 %nonzero,%positive
  %valid = and i1 %a,%limit
  call void @k.image.require(i1 %valid)
  %boot = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 0)
  %count = call i64 @k.fw.u8(i64 %boot,i64 488)
  %have = icmp ugt i64 %count,0
  %bounded = icmp ule i64 %count,128
  %map_ok = and i1 %have,%bounded
  call void @k.image.require(i1 %map_ok)
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%loop]
  %covered = phi i1 [false,%0],[%found,%loop]
  %safe = phi i1 [true,%0],[%clean,%loop]
  %delta = mul i64 %i,20
  %at = add i64 %delta,720
  %pointer = add i64 %boot,%at
  %start = call i64 @k.fw.u64(i64 %pointer,i64 0)
  %bytes = call i64 @k.fw.u64(i64 %pointer,i64 8)
  %kind = call i64 @k.fw.u32(i64 %pointer,i64 16)
  %last = add i64 %start,%bytes
  %nowrap = icmp uge i64 %last,%start
  call void @k.image.require(i1 %nowrap)
  %type_low = icmp uge i64 %kind,2
  %type_high = icmp ule i64 %kind,4
  %retained = and i1 %type_low,%type_high
  %first = icmp ule i64 %start,%base
  %final = icmp uge i64 %last,%end
  %contains = and i1 %first,%final
  %cover = and i1 %contains,%retained
  %found = or i1 %covered,%cover
  %left = icmp ult i64 %start,%end
  %right = icmp ult i64 %base,%last
  %overlap = and i1 %left,%right
  %empty = icmp eq i64 %bytes,0
  %none = xor i1 %overlap,true
  %absent = or i1 %none,%empty
  %allowed = or i1 %retained,%absent
  %clean = and i1 %safe,%allowed
  %next = add i64 %i,1
  %more = icmp ult i64 %next,%count
  br i1 %more,label %loop,label %done
done:
  %ok = and i1 %found,%clean
  call void @k.image.require(i1 %ok)
  ret void
}
define internal i64 @k.fw.table(i64 %base) {
  call void @k.fw.range(i64 %base,i64 36)
  %length = call i64 @k.fw.u32(i64 %base,i64 4)
  %minimum = icmp uge i64 %length,36
  %maximum = icmp ule i64 %length,4096
  %size_ok = and i1 %minimum,%maximum
  call void @k.image.require(i1 %size_ok)
  call void @k.fw.range(i64 %base,i64 %length)
  %sum = call i1 @k.fw.checksum(i64 %base,i64 %length)
  call void @k.image.require(i1 %sum)
  ret i64 %length
}
; Qualified legacy ROM/EBDA scan. A nonmatching or bad first checksum is skipped.
define internal i64 @k.fw.scan(i64 %base,i64 %limit) {
  br label %loop
loop:
  %at = phi i64 [%base,%0],[%next,%advance]
  %end = add i64 %at,20
  %fits = icmp ule i64 %end,%limit
  br i1 %fits,label %signature,label %absent
signature:
  %value = call i64 @k.fw.u64(i64 %at,i64 0)
  %matches = icmp eq i64 %value,2329016660419433298
  br i1 %matches,label %checksum,label %advance
checksum:
  %sum = call i1 @k.fw.checksum(i64 %at,i64 20)
  br i1 %sum,label %found,label %advance
advance:
  %next = add i64 %at,16
  br label %loop
found:
  ret i64 %at
absent:
  ret i64 0
}
define internal i64 @k.fw.rsdp(i64 %ebda) {
  %none = icmp eq i64 %ebda,0
  br i1 %none,label %bios,label %check
check:
  %low = icmp uge i64 %ebda,524288
  %high = icmp ule i64 %ebda,654336
  %bounded = and i1 %low,%high
  call void @k.image.require(i1 %bounded)
  call void @k.fw.range(i64 %ebda,i64 1024)
  %end = add i64 %ebda,1024
  %first = call i64 @k.fw.scan(i64 %ebda,i64 %end)
  %present = icmp ne i64 %first,0
  br i1 %present,label %found,label %bios
found:
  ret i64 %first
bios:
  %second = call i64 @k.fw.scan(i64 917504,i64 1048576)
  %has = icmp ne i64 %second,0
  call void @k.image.require(i1 %has)
  ret i64 %second
}
