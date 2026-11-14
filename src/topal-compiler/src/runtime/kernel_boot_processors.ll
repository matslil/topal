; Firmware participant inventory only; no processor-control or lifetime witness.
define internal {i64,i64} @k.fw.processors(i64 %base,i64 %length,i64 %bsp,i1 %save) {
  br label %walk
walk:
  %at = phi i64 [44,%0],[%next,%advance]
  %count = phi i64 [0,%0],[%newcount,%advance]
  %enabled = phi i64 [0,%0],[%newenabled,%advance]
  %matches = phi i64 [0,%0],[%newmatches,%advance]
  %finished = icmp eq i64 %at,%length
  br i1 %finished,label %done,label %header
header:
  %headend = add i64 %at,2
  %header_fits = icmp ule i64 %headend,%length
  call void @k.image.require(i1 %header_fits)
  %record = add i64 %base,%at
  %kind = call i64 @k.fw.u8(i64 %record,i64 0)
  %size = call i64 @k.fw.u8(i64 %record,i64 1)
  %next = add i64 %at,%size
  %fits = icmp ule i64 %next,%length
  %k0 = icmp eq i64 %kind,0
  %k1 = icmp eq i64 %kind,1
  %k2 = icmp eq i64 %kind,2
  %k3 = icmp eq i64 %kind,3
  %k4 = icmp eq i64 %kind,4
  %k5 = icmp eq i64 %kind,5
  %k9 = icmp eq i64 %kind,9
  %k10 = icmp eq i64 %kind,10
  %s0 = select i1 %k0,i64 8,i64 0
  %s1 = select i1 %k1,i64 12,i64 %s0
  %s2 = select i1 %k2,i64 10,i64 %s1
  %s3 = select i1 %k3,i64 8,i64 %s2
  %s4 = select i1 %k4,i64 6,i64 %s3
  %s5 = select i1 %k5,i64 12,i64 %s4
  %s9 = select i1 %k9,i64 16,i64 %s5
  %expected = select i1 %k10,i64 12,i64 %s9
  %known = icmp ne i64 %expected,0
  %exact = icmp eq i64 %size,%expected
  %a = and i1 %known,%exact
  %valid = and i1 %a,%fits
  call void @k.image.require(i1 %valid)
  %processor = or i1 %k0,%k9
  br i1 %processor,label %flags,label %nonprocessor
nonprocessor:
  br i1 %k5,label %override,label %advance
override:
  %reserved_override = call i64 @k.fw.u16(i64 %record,i64 2)
  %address = call i64 @k.fw.u64(i64 %record,i64 4)
  %reserved_zero = icmp eq i64 %reserved_override,0
  %same_apic = icmp eq i64 %address,4276092928
  %override_ok = and i1 %reserved_zero,%same_apic
  call void @k.image.require(i1 %override_ok)
  br label %advance
flags:
  %flag_offset = select i1 %k0,i64 4,i64 8
  %value = call i64 @k.fw.u32(i64 %record,i64 %flag_offset)
  %known_flags = icmp ule i64 %value,2
  call void @k.image.require(i1 %known_flags)
  %unusable = icmp eq i64 %value,0
  br i1 %unusable,label %advance,label %relevant
relevant:
  %room = icmp ult i64 %count,16
  call void @k.image.require(i1 %room)
  br i1 %k0,label %local,label %extended
local:
  %uid0 = call i64 @k.fw.u8(i64 %record,i64 2)
  %id0 = call i64 @k.fw.u8(i64 %record,i64 3)
  br label %identity
extended:
  %reserved = call i64 @k.fw.u16(i64 %record,i64 2)
  %zero = icmp eq i64 %reserved,0
  call void @k.image.require(i1 %zero)
  %uid9 = call i64 @k.fw.u32(i64 %record,i64 12)
  %id9 = call i64 @k.fw.u32(i64 %record,i64 4)
  br label %identity
identity:
  %uid = phi i64 [%uid0,%local],[%uid9,%extended]
  %id = phi i64 [%id0,%local],[%id9,%extended]
  br label %unique
unique:
  %j = phi i64 [0,%identity],[%after,%compare]
  %more = icmp ult i64 %j,%count
  br i1 %more,label %compare,label %store
compare:
  %delta = shl i64 %j,2
  %uid_index = add i64 %delta,33
  %id_index = add i64 %delta,34
  %up = getelementptr [128 x i64],ptr @topal_x86_boot_processors_v1,i64 0,i64 %uid_index
  %ip = getelementptr [128 x i64],ptr @topal_x86_boot_processors_v1,i64 0,i64 %id_index
  %previous_uid = load volatile i64,ptr %up,align 8
  %previous_id = load volatile i64,ptr %ip,align 8
  %different_uid = icmp ne i64 %previous_uid,%uid
  %different_id = icmp ne i64 %previous_id,%id
  %distinct = and i1 %different_uid,%different_id
  call void @k.image.require(i1 %distinct)
  %after = add i64 %j,1
  br label %unique
store:
  %slot = shl i64 %count,2
  %type_slot = add i64 %slot,32
  %uid_slot = add i64 %slot,33
  %id_slot = add i64 %slot,34
  %flags_slot = add i64 %slot,35
  call void @k.fw.record(i64 %type_slot,i64 %kind,i1 %save)
  call void @k.fw.record(i64 %uid_slot,i64 %uid,i1 %save)
  call void @k.fw.record(i64 %id_slot,i64 %id,i1 %save)
  call void @k.fw.record(i64 %flags_slot,i64 %value,i1 %save)
  %inc_count = add i64 %count,1
  %is_enabled = icmp eq i64 %value,1
  %on = zext i1 %is_enabled to i64
  %inc_enabled = add i64 %enabled,%on
  %bsp_id = icmp eq i64 %id,%bsp
  %bsp_enabled = and i1 %bsp_id,%is_enabled
  %one_bsp = zext i1 %bsp_enabled to i64
  %inc_matches = add i64 %matches,%one_bsp
  br label %advance
advance:
  %newcount = phi i64 [%count,%nonprocessor],[%count,%override],[%count,%flags],[%inc_count,%store]
  %newenabled = phi i64 [%enabled,%nonprocessor],[%enabled,%override],[%enabled,%flags],[%inc_enabled,%store]
  %newmatches = phi i64 [%matches,%nonprocessor],[%matches,%override],[%matches,%flags],[%inc_matches,%store]
  br label %walk
done:
  %have = icmp ugt i64 %count,0
  %online = icmp ugt i64 %enabled,0
  %one = icmp eq i64 %matches,1
  %h = and i1 %have,%online
  %complete = and i1 %h,%one
  call void @k.image.require(i1 %complete)
  %r0 = insertvalue {i64,i64} poison,i64 %count,0
  %r1 = insertvalue {i64,i64} %r0,i64 %enabled,1
  ret {i64,i64} %r1
}
define internal i64 @k.fw.bsp() {
  %p = getelementptr [128 x i64],ptr @topal_x86_image_boot_observation_v1,i64 0,i64 6
  %pat = load volatile i64,ptr %p,align 8
  %shift = lshr i64 %pat,24
  %type = and i64 %shift,255
  %uc = icmp eq i64 %type,0
  call void @k.image.require(i1 %uc)
  %raw = call i64 @k.fw.u32(i64 4276092960,i64 0)
  call void @topal_x86_image_boot_read_observe_v1(i64 1,i64 4276092960,i64 %raw)
  %actual = lshr i64 %raw,24
  %leaf = call {i32,i32,i32,i32} @k.boot.cpuid(i32 1)
  %ebx = extractvalue {i32,i32,i32,i32} %leaf,1
  %ebx64 = zext i32 %ebx to i64
  call void @topal_x86_image_boot_read_observe_v1(i64 2,i64 1,i64 %ebx64)
  %id = lshr i32 %ebx,24
  %initial = zext i32 %id to i64
  %same = icmp eq i64 %actual,%initial
  call void @k.image.require(i1 %same)
  ret i64 %actual
}
define internal void @k.fw.verify(i1 %save) {
  %cursor = alloca i64,align 8
  %madt_pointer = alloca i64,align 8
  %madt_length = alloca i64,align 8
  %madt_copy_pointer = alloca i64,align 8
  store i64 0,ptr %cursor,align 8
  store i64 0,ptr %madt_pointer,align 8
  store i64 0,ptr %madt_length,align 8
  store i64 0,ptr %madt_copy_pointer,align 8
  %state = call i64 @t.word(ptr @topal_x86_boot_processors_v1,i64 0)
  %wanted = select i1 %save,i64 0,i64 1
  %fresh = icmp eq i64 %state,%wanted
  call void @k.image.require(i1 %fresh)
  %bda_copy = call i64 @k.fw.bind(i64 1038,i64 2,ptr %cursor,i1 %save)
  %segment = call i64 @k.fw.u16(i64 %bda_copy,i64 0)
  %ebda = shl i64 %segment,4
  %rsdp = call i64 @k.fw.rsdp(i64 %ebda)
  %bios = icmp uge i64 %rsdp,917504
  %ebda_end = add i64 %ebda,1024
  %limit = select i1 %bios,i64 1048576,i64 %ebda_end
  %revision = call i64 @k.fw.u8(i64 %rsdp,i64 15)
  %legacy = icmp eq i64 %revision,0
  %extended = icmp eq i64 %revision,2
  %supported = or i1 %legacy,%extended
  call void @k.image.require(i1 %supported)
  %rsdp_length = select i1 %legacy,i64 20,i64 36
  %rsdp_end = add i64 %rsdp,%rsdp_length
  %fits = icmp ule i64 %rsdp_end,%limit
  call void @k.image.require(i1 %fits)
  %rsdp_copy = call i64 @k.fw.bind(i64 %rsdp,i64 %rsdp_length,ptr %cursor,i1 %save)
  %copy_signature = call i64 @k.fw.u64(i64 %rsdp_copy,i64 0)
  %copy_revision = call i64 @k.fw.u8(i64 %rsdp_copy,i64 15)
  %same_signature = icmp eq i64 %copy_signature,2329016660419433298
  %same_revision = icmp eq i64 %copy_revision,%revision
  %copy_sum = call i1 @k.fw.checksum(i64 %rsdp_copy,i64 20)
  %same_header = and i1 %same_signature,%same_revision
  %copy_ok = and i1 %same_header,%copy_sum
  call void @k.image.require(i1 %copy_ok)
  br i1 %legacy,label %v0,label %v2
v0:
  %root0 = call i64 @k.fw.u32(i64 %rsdp_copy,i64 16)
  br label %root
v2:
  %rsdp_size = call i64 @k.fw.u32(i64 %rsdp_copy,i64 20)
  %exact = icmp eq i64 %rsdp_size,36
  call void @k.image.require(i1 %exact)
  %reserved = call i64 @k.fw.u32(i64 %rsdp_copy,i64 32)
  %padding = and i64 %reserved,4294967040
  %padding_zero = icmp eq i64 %padding,0
  call void @k.image.require(i1 %padding_zero)
  %sum = call i1 @k.fw.checksum(i64 %rsdp_copy,i64 36)
  call void @k.image.require(i1 %sum)
  %root2 = call i64 @k.fw.u64(i64 %rsdp_copy,i64 24)
  br label %root
root:
  %root_base = phi i64 [%root0,%v0],[%root2,%v2]
  %width = select i1 %legacy,i64 4,i64 8
  %signature = select i1 %legacy,i64 1413763922,i64 1413763928
  %length = call i64 @k.fw.table(i64 %root_base)
  %root_copy = call i64 @k.fw.bind(i64 %root_base,i64 %length,ptr %cursor,i1 %save)
  call void @k.fw.checked_copy(i64 %root_copy,i64 %length)
  %actual_signature = call i64 @k.fw.u32(i64 %root_copy,i64 0)
  %header_revision = call i64 @k.fw.u8(i64 %root_copy,i64 8)
  %signature_ok = icmp eq i64 %actual_signature,%signature
  %revision_ok = icmp eq i64 %header_revision,1
  %header_ok = and i1 %signature_ok,%revision_ok
  call void @k.image.require(i1 %header_ok)
  %payload = sub i64 %length,36
  %remainder = urem i64 %payload,%width
  %aligned = icmp eq i64 %remainder,0
  %count = udiv i64 %payload,%width
  %positive = icmp ugt i64 %count,0
  %bounded = icmp ule i64 %count,64
  %a = and i1 %aligned,%positive
  %entries_ok = and i1 %a,%bounded
  call void @k.image.require(i1 %entries_ok)
  br label %children
children:
  %i = phi i64 [0,%root],[%next,%advance]
  %offset0 = mul i64 %i,%width
  %offset = add i64 %offset0,36
  br i1 %legacy,label %pointer32,label %pointer64
pointer32:
  %p32 = call i64 @k.fw.u32(i64 %root_copy,i64 %offset)
  br label %child
pointer64:
  %p64 = call i64 @k.fw.u64(i64 %root_copy,i64 %offset)
  br label %child
child:
  %pointer = phi i64 [%p32,%pointer32],[%p64,%pointer64]
  call void @k.fw.range(i64 %pointer,i64 36)
  %header_copy = call i64 @k.fw.bind(i64 %pointer,i64 36,ptr %cursor,i1 %save)
  %child_size = call i64 @k.fw.u32(i64 %header_copy,i64 4)
  %child_minimum = icmp uge i64 %child_size,36
  %child_end = add i64 %pointer,%child_size
  %child_bounded = icmp ule i64 %child_end,1073741824
  %child_valid = and i1 %child_minimum,%child_bounded
  call void @k.image.require(i1 %child_valid)
  %sig = call i64 @k.fw.u32(i64 %header_copy,i64 0)
  %apic = icmp eq i64 %sig,1128878145
  br i1 %apic,label %madt,label %advance
madt:
  %previous = load i64,ptr %madt_pointer,align 8
  %none = icmp eq i64 %previous,0
  call void @k.image.require(i1 %none)
  %bytes = call i64 @k.fw.table(i64 %pointer)
  %madt_copy = call i64 @k.fw.bind(i64 %pointer,i64 %bytes,ptr %cursor,i1 %save)
  call void @k.fw.checked_copy(i64 %madt_copy,i64 %bytes)
  call void @k.fw.equal(i64 %header_copy,i64 %madt_copy,i64 36)
  %minimum = icmp uge i64 %bytes,44
  %madt_revision = call i64 @k.fw.u8(i64 %madt_copy,i64 8)
  %rev_low = icmp uge i64 %madt_revision,1
  %rev_high = icmp ule i64 %madt_revision,7
  %rev_ok = and i1 %rev_low,%rev_high
  %madt_ok = and i1 %minimum,%rev_ok
  call void @k.image.require(i1 %madt_ok)
  store i64 %madt_copy,ptr %madt_copy_pointer,align 8
  store i64 %pointer,ptr %madt_pointer,align 8
  store i64 %bytes,ptr %madt_length,align 8
  br label %advance
advance:
  %next = add i64 %i,1
  %more = icmp ult i64 %next,%count
  br i1 %more,label %children,label %inventory
inventory:
  %madt_base = load i64,ptr %madt_pointer,align 8
  %madt_bytes = load i64,ptr %madt_length,align 8
  %inventory_copy = load i64,ptr %madt_copy_pointer,align 8
  %present = icmp ne i64 %madt_base,0
  call void @k.image.require(i1 %present)
  %local_apic = call i64 @k.fw.u32(i64 %inventory_copy,i64 36)
  %madt_flags = call i64 @k.fw.u32(i64 %inventory_copy,i64 40)
  %local = icmp eq i64 %local_apic,4276092928
  %flags = icmp ule i64 %madt_flags,1
  %madt_header = and i1 %local,%flags
  call void @k.image.require(i1 %madt_header)
  %bsp = call i64 @k.fw.bsp()
  %participants = call {i64,i64} @k.fw.processors(i64 %inventory_copy,i64 %madt_bytes,i64 %bsp,i1 %save)
  %total = extractvalue {i64,i64} %participants,0
  %enabled = extractvalue {i64,i64} %participants,1
  %retained = load i64,ptr %cursor,align 8
  call void @k.fw.record(i64 1,i64 %rsdp,i1 %save)
  call void @k.fw.record(i64 2,i64 %revision,i1 %save)
  call void @k.fw.record(i64 3,i64 %rsdp_length,i1 %save)
  call void @k.fw.record(i64 4,i64 %root_base,i1 %save)
  call void @k.fw.record(i64 5,i64 %length,i1 %save)
  call void @k.fw.record(i64 6,i64 %width,i1 %save)
  call void @k.fw.record(i64 7,i64 %count,i1 %save)
  call void @k.fw.record(i64 8,i64 %madt_base,i1 %save)
  call void @k.fw.record(i64 9,i64 %madt_bytes,i1 %save)
  call void @k.fw.record(i64 10,i64 %local_apic,i1 %save)
  call void @k.fw.record(i64 11,i64 %bsp,i1 %save)
  call void @k.fw.record(i64 12,i64 %total,i1 %save)
  call void @k.fw.record(i64 13,i64 %enabled,i1 %save)
  call void @k.fw.record(i64 14,i64 %retained,i1 %save)
  call void @k.fw.record(i64 15,i64 %ebda,i1 %save)
  fence seq_cst
  call void @k.fw.record(i64 0,i64 1,i1 true)
  ret void
}
