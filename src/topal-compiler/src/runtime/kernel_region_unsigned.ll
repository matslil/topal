; Optional checked field encoding runtime. Core values and the base region
; implementation remain shared. Static source formats admit one to eight bytes.

define internal { i1, ptr } @topal.kernel.region.unsigned.span(ptr %view, ptr %index, i64 %width) nounwind noinline {
entry:
  %bytes = call ptr @topal.kernel.region.guard(ptr %view)
  %positive = icmp uge i64 %width, 1
  %bounded = icmp ule i64 %width, 8
  %format.valid = and i1 %positive, %bounded
  br i1 %format.valid, label %decode, label %invalid
decode:
  %decoded = call { i1, i64 } @topal.kernel.region.index(ptr %index)
  %fits = extractvalue { i1, i64 } %decoded, 0
  %offset = extractvalue { i1, i64 } %decoded, 1
  %count.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 2
  %count = load i64, ptr %count.pointer, align 8
  %starts = icmp ule i64 %offset, %count
  %remaining = sub i64 %count, %offset
  %ends = icmp ule i64 %width, %remaining
  %contained = and i1 %starts, %ends
  %valid = and i1 %fits, %contained
  br i1 %valid, label %success, label %refuse
success:
  %pointer = getelementptr i8, ptr %bytes, i64 %offset
  %result = insertvalue { i1, ptr } { i1 true, ptr null }, ptr %pointer, 1
  ret { i1, ptr } %result
refuse:
  ret { i1, ptr } { i1 false, ptr null }
invalid:
  call void @topal.kernel.region.abort()
  unreachable
}

define internal ptr @topal.kernel.region.unsigned.load(ptr %view, ptr %index, i64 %width, i1 %big) nounwind noinline {
entry:
  %checked = call { i1, ptr } @topal.kernel.region.unsigned.span(ptr %view, ptr %index, i64 %width)
  %valid = extractvalue { i1, ptr } %checked, 0
  %base = extractvalue { i1, ptr } %checked, 1
  br i1 %valid, label %loop, label %refuse
loop:
  %index.byte = phi i64 [ 0, %entry ], [ %next, %loop ]
  %value = phi i64 [ 0, %entry ], [ %merged, %loop ]
  %last = sub i64 %width, 1
  %reverse = sub i64 %last, %index.byte
  %place = select i1 %big, i64 %reverse, i64 %index.byte
  %shift = mul i64 %place, 8
  %pointer = getelementptr i8, ptr %base, i64 %index.byte
  %byte = load i8, ptr %pointer, align 1
  %extended = zext i8 %byte to i64
  %part = shl i64 %extended, %shift
  %merged = or i64 %value, %part
  %next = add i64 %index.byte, 1
  %done = icmp eq i64 %next, %width
  br i1 %done, label %result, label %loop
result:
  %number = call ptr @topal.runtime.int.from.u64(i64 %merged)
  %some = call ptr @topal.runtime.optional.some(ptr %number)
  ret ptr %some
refuse:
  %none = call ptr @topal.runtime.optional.none()
  ret ptr %none
}

define internal i1 @topal.kernel.region.unsigned.store(ptr %view, ptr %index, ptr %number, i64 %width, i1 %big) nounwind noinline {
entry:
  %checked = call { i1, ptr } @topal.kernel.region.unsigned.span(ptr %view, ptr %index, i64 %width)
  %span.valid = extractvalue { i1, ptr } %checked, 0
  %base = extractvalue { i1, ptr } %checked, 1
  br i1 %span.valid, label %value.check, label %refuse
value.check:
  %decoded = call { i1, i64 } @topal.kernel.region.index(ptr %number)
  %fits = extractvalue { i1, i64 } %decoded, 0
  %value = extractvalue { i1, i64 } %decoded, 1
  %rights.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 3
  %rights = load i64, ptr %rights.pointer, align 8
  %writable = icmp eq i64 %rights, 1
  ; Width is proved in [1,8], so this shift is always in [0,56]. In particular
  ; width eight never attempts a poison-producing shift by 64.
  %bits = mul i64 %width, 8
  %unused = sub i64 64, %bits
  %mask = lshr i64 -1, %unused
  %outside.mask = xor i64 %mask, -1
  %outside = and i64 %value, %outside.mask
  %value.valid = icmp eq i64 %outside, 0
  %range = and i1 %fits, %value.valid
  %valid = and i1 %range, %writable
  br i1 %valid, label %loop, label %refuse
loop:
  %index.byte = phi i64 [ 0, %value.check ], [ %next, %loop ]
  %last = sub i64 %width, 1
  %reverse = sub i64 %last, %index.byte
  %place = select i1 %big, i64 %reverse, i64 %index.byte
  %shift = mul i64 %place, 8
  %part = lshr i64 %value, %shift
  %byte = trunc i64 %part to i8
  %pointer = getelementptr i8, ptr %base, i64 %index.byte
  store i8 %byte, ptr %pointer, align 1
  %next = add i64 %index.byte, 1
  %done = icmp eq i64 %next, %width
  br i1 %done, label %result, label %loop
result:
  ret i1 true
refuse:
  ret i1 false
}
