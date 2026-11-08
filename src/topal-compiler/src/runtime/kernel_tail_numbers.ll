; Allocation and complete immutable numeric-root transfer.
define internal i1 @p.allocation_start(ptr %context, i64 %offset) nounwind noinline {
entry:
  %byte.index = udiv i64 %offset, 64
  %bit.word = udiv i64 %offset, 8
  %bit.index = urem i64 %bit.word, 8
  %offset.valid = icmp ult i64 %offset, {{WORK_BYTES}}
  call void @p.require(i1 %offset.valid)
  %starts = getelementptr i8, ptr %context, i64 {{STARTS}}
  %pointer = getelementptr i8, ptr %starts, i64 %byte.index
  %byte = load i8, ptr %pointer, align 1
  %shift = trunc i64 %bit.index to i8
  %mask = shl i8 1, %shift
  %bits = and i8 %byte, %mask
  %marked = icmp ne i8 %bits, 0
  ret i1 %marked
}
define ptr @topal_tail_allocate_v1(i64 %bytes) nounwind noinline {
entry:
  %record = load ptr, ptr @topal_x86_tail_boundary_active_v1, align 8
  %active = icmp ne ptr %record, null
  call void @p.require(i1 %active)
  %provided = getelementptr i8, ptr %record, i64 64
  %context = call ptr @p.phase(ptr %provided, i64 1)
  %positive = icmp ugt i64 %bytes, 0
  %bounded = icmp ule i64 %bytes, {{WORK_BYTES}}
  %valid = and i1 %positive, %bounded
  call void @p.require(i1 %valid)
  %rounded = add i64 %bytes, 7
  %aligned = and i64 %rounded, -8
  %used = call i64 @p.word(ptr %context, i64 {{USED}})
  %used.valid = icmp ule i64 %used, {{WORK_BYTES}}
  call void @p.require(i1 %used.valid)
  %remaining = sub i64 {{WORK_BYTES}}, %used
  %fits = icmp ule i64 %aligned, %remaining
  call void @p.require(i1 %fits)
  %alignment = and i64 %used, 7
  %is.aligned = icmp eq i64 %alignment, 0
  call void @p.require(i1 %is.aligned)
  %byte.index = udiv i64 %used, 64
  %bit.word = udiv i64 %used, 8
  %bit.index = urem i64 %bit.word, 8
  %starts = getelementptr i8, ptr %context, i64 {{STARTS}}
  %mark = getelementptr i8, ptr %starts, i64 %byte.index
  %old = load i8, ptr %mark, align 1
  %shift = trunc i64 %bit.index to i8
  %mask = shl i8 1, %shift
  %new = or i8 %old, %mask
  store i8 %new, ptr %mark, align 1
  %next = add i64 %used, %aligned
  call void @p.put(ptr %context, i64 {{USED}}, i64 %next)
  %work = getelementptr i8, ptr %context, i64 {{WORK}}
  %result = getelementptr i8, ptr %work, i64 %used
  call void @p.erase(ptr %result, i64 %aligned)
  ret ptr %result
}
define internal i64 @p.origin_bytes(ptr %context, ptr %source) nounwind noinline {
entry:
  %address = ptrtoint ptr %source to i64
  %work = getelementptr i8, ptr %context, i64 {{WORK}}
  %work.address = ptrtoint ptr %work to i64
  %used = call i64 @p.word(ptr %context, i64 {{USED}})
  %used.valid = icmp ule i64 %used, {{WORK_BYTES}}
  call void @p.require(i1 %used.valid)
  %work.end = call i64 @p.add(i64 %work.address, i64 %used)
  %used.alignment = and i64 %used, 7
  %used.aligned = icmp eq i64 %used.alignment, 0
  call void @p.require(i1 %used.aligned)
  %after.start = icmp uge i64 %address, %work.address
  %before.end = icmp ult i64 %address, %work.end
  %in.work = and i1 %after.start, %before.end
  br i1 %in.work, label %work.root, label %banks
work.root:
  %offset = sub i64 %address, %work.address
  %misalignment = and i64 %offset, 7
  %aligned = icmp eq i64 %misalignment, 0
  call void @p.require(i1 %aligned)
  %issued = call i1 @p.allocation_start(ptr %context, i64 %offset)
  call void @p.require(i1 %issued)
  %first.end = add i64 %offset, 8
  br label %scan
scan:
  %end = phi i64 [ %first.end, %work.root ], [ %next.end, %advance ]
  %is.end = icmp eq i64 %end, %used
  br i1 %is.end, label %extent, label %test
test:
  %next.object = call i1 @p.allocation_start(ptr %context, i64 %end)
  br i1 %next.object, label %extent, label %advance
advance:
  %next.end = add i64 %end, 8
  br label %scan
extent:
  %length = sub i64 %end, %offset
  ret i64 %length
banks:
  %bank.index = call i64 @p.word(ptr %context, i64 {{BANK}})
  %bank = call ptr @p.bank(ptr %context, i64 %bank.index)
  br label %root.loop
root.loop:
  %index = phi i64 [ 0, %banks ], [ %next.index, %root.next ]
  %roots.end = icmp eq i64 %index, {{PARAMETERS}}
  br i1 %roots.end, label %readonly, label %root.test
root.test:
  %root = call ptr @p.root(i64 %index)
  %kind = call i64 @p.word(ptr %root, i64 0)
  %is.nat = icmp eq i64 %kind, 1
  %is.int = icmp eq i64 %kind, 2
  %numeric = or i1 %is.nat, %is.int
  %root.offset = call i64 @p.word(ptr %root, i64 8)
  %pointer = getelementptr i8, ptr %bank, i64 %root.offset
  %same = icmp eq ptr %pointer, %source
  %valid.root = and i1 %numeric, %same
  br i1 %valid.root, label %root.found, label %root.next
root.next:
  %next.index = add i64 %index, 1
  br label %root.loop
root.found:
  %root.bytes = call i64 @p.word(ptr %root, i64 16)
  ret i64 %root.bytes
readonly:
  %constant.bytes = call i64 @topal_tail_readonly_bytes_v1(ptr %source)
  %large.enough = icmp uge i64 %constant.bytes, 16
  %small.enough = icmp ule i64 %constant.bytes, 48
  %constant.valid = and i1 %large.enough, %small.enough
  call void @p.require(i1 %constant.valid)
  ret i64 %constant.bytes
}
define internal i64 @p.bits(ptr %number, i64 %extent) nounwind noinline {
entry:
  %address = ptrtoint ptr %number to i64
  %alignment = and i64 %address, 7
  %aligned = icmp eq i64 %alignment, 0
  %sufficient = icmp uge i64 %extent, 16
  %geometry = and i1 %aligned, %sufficient
  call void @p.require(i1 %geometry)
  %sign = call i64 @p.word(ptr %number, i64 0)
  %count = call i64 @p.word(ptr %number, i64 8)
  %sign.valid = icmp ule i64 %sign, 1
  %count.valid = icmp ule i64 %count, 8
  %header.valid = and i1 %sign.valid, %count.valid
  call void @p.require(i1 %header.valid)
  %limb.bytes = mul i64 %count, 4
  %total = add i64 %limb.bytes, 16
  %contained = icmp ule i64 %total, %extent
  call void @p.require(i1 %contained)
  %zero = icmp eq i64 %count, 0
  br i1 %zero, label %zero.value, label %nonzero
zero.value:
  %canonical.zero = icmp eq i64 %sign, 0
  call void @p.require(i1 %canonical.zero)
  ret i64 1
nonzero:
  %high.offset = add i64 %limb.bytes, 12
  %high.pointer = getelementptr i8, ptr %number, i64 %high.offset
  %high = load i32, ptr %high.pointer, align 4
  %canonical = icmp ne i32 %high, 0
  call void @p.require(i1 %canonical)
  %count.less.one = sub i64 %count, 1
  %low.bits = mul i64 %count.less.one, 32
  br label %bits
bits:
  %word = phi i32 [ %high, %nonzero ], [ %shifted, %next.bit ]
  %size = phi i64 [ %low.bits, %nonzero ], [ %incremented, %next.bit ]
  %finished = icmp eq i32 %word, 0
  br i1 %finished, label %done, label %next.bit
next.bit:
  %shifted = lshr i32 %word, 1
  %incremented = add i64 %size, 1
  br label %bits
done:
  ret i64 %size
}
define internal void @p.copy_number(ptr %context, ptr %destination, i64 %capacity, i64 %maximum.bits, i64 %kind, ptr %source) nounwind noinline {
entry:
  %extent = call i64 @p.origin_bytes(ptr %context, ptr %source)
  %bits = call i64 @p.bits(ptr %source, i64 %extent)
  %sign = call i64 @p.word(ptr %source, i64 0)
  %length = call i64 @p.word(ptr %source, i64 8)
  %limb.bytes = mul i64 %length, 4
  %bytes = add i64 %limb.bytes, 16
  %fits.bits = icmp ule i64 %bits, %maximum.bits
  %fits.bytes = icmp ule i64 %bytes, %capacity
  %fits = and i1 %fits.bits, %fits.bytes
  call void @p.require(i1 %fits)
  %nat = icmp eq i64 %kind, 1
  %positive = icmp eq i64 %sign, 0
  %not.nat = xor i1 %nat, true
  %classifier.valid = or i1 %not.nat, %positive
  call void @p.require(i1 %classifier.valid)
  %empty = call i1 @p.zero(ptr %destination, i64 %capacity)
  call void @p.require(i1 %empty)
  call void @p.copy(ptr %destination, ptr %source, i64 %bytes)
  ret void
}
