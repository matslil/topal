; Private source-object ABI. The caller supplies live, mapped, disjoint owner,
; control and temporary storage. These words are not source authority.
; Control: code, owner, generation, active, base, count, writable, issued views.
; View: control, offset, count, writable, generation, owner, code, zero.
%topal.KernelRegionControl = type { ptr, i64, i64, i64, ptr, i64, i64, i64 }
%topal.KernelRegionView = type { ptr, i64, i64, i64, i64, i64, ptr, i64 }

define internal void @topal.kernel.region.abort() noreturn nounwind noinline {
entry:
  call void @topal_kernel_region_abort_v1()
  unreachable
}

; Returns a contained byte pointer only after complete view/control validation.
; The generated closed source graph proves view pointer provenance/nonescape.
define internal ptr @topal.kernel.region.guard(ptr %view) nounwind noinline {
entry:
  %address = ptrtoint ptr %view to i64
  %nonnull = icmp ne i64 %address, 0
  %alignment = and i64 %address, 7
  %aligned = icmp eq i64 %alignment, 0
  %valid.pointer = and i1 %nonnull, %aligned
  br i1 %valid.pointer, label %view.fields, label %invalid
view.fields:
  %control = load ptr, ptr %view, align 8
  %control.address = ptrtoint ptr %control to i64
  %control.nonnull = icmp ne i64 %control.address, 0
  %control.alignment = and i64 %control.address, 7
  %control.aligned = icmp eq i64 %control.alignment, 0
  %control.valid = and i1 %control.nonnull, %control.aligned
  br i1 %control.valid, label %control.fields, label %invalid
control.fields:
  %code = load ptr, ptr %control, align 8
  %owner.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 1
  %epoch.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 2
  %active.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 3
  %base.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 4
  %count.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 5
  %rights.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 6
  %issued.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 7
  %owner = load i64, ptr %owner.pointer, align 8
  %epoch = load i64, ptr %epoch.pointer, align 8
  %active = load i64, ptr %active.pointer, align 8
  %base = load ptr, ptr %base.pointer, align 8
  %count = load i64, ptr %count.pointer, align 8
  %rights = load i64, ptr %rights.pointer, align 8
  %issued = load i64, ptr %issued.pointer, align 8
  %offset.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 1
  %view.count.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 2
  %view.rights.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 3
  %view.epoch.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 4
  %view.owner.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 5
  %view.code.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 6
  %view.zero.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 7
  %offset = load i64, ptr %offset.pointer, align 8
  %view.count = load i64, ptr %view.count.pointer, align 8
  %view.rights = load i64, ptr %view.rights.pointer, align 8
  %view.epoch = load i64, ptr %view.epoch.pointer, align 8
  %view.owner = load i64, ptr %view.owner.pointer, align 8
  %view.code = load ptr, ptr %view.code.pointer, align 8
  %view.zero = load i64, ptr %view.zero.pointer, align 8
  %code.valid = icmp eq ptr %code, @topal_kernel_region_apply_v1
  %view.code.valid = icmp eq ptr %view.code, %code
  %owner.valid = icmp ne i64 %owner, 0
  %view.owner.valid = icmp eq i64 %view.owner, %owner
  %epoch.valid = icmp ne i64 %epoch, 0
  %view.epoch.valid = icmp eq i64 %view.epoch, %epoch
  %active.valid = icmp eq i64 %active, 1
  %count.valid = icmp ule i64 %count, 65536
  %rights.valid = icmp ule i64 %rights, 1
  %view.rights.valid = icmp ule i64 %view.rights, %rights
  %zero.valid = icmp eq i64 %view.zero, 0
  %offset.valid = icmp ule i64 %offset, %count
  %remaining = sub i64 %count, %offset
  %view.count.valid = icmp ule i64 %view.count, %remaining
  %base.address = ptrtoint ptr %base to i64
  %base.nonnull = icmp ne i64 %base.address, 0
  %end = add i64 %base.address, %count
  %end.valid = icmp uge i64 %end, %base.address
  %issued.positive = icmp uge i64 %issued, 1
  %issued.bounded = icmp ule i64 %issued, 256
  %issued.valid = and i1 %issued.positive, %issued.bounded
  %a = and i1 %code.valid, %view.code.valid
  %b = and i1 %owner.valid, %view.owner.valid
  %c = and i1 %epoch.valid, %view.epoch.valid
  %d = and i1 %active.valid, %count.valid
  %e = and i1 %rights.valid, %view.rights.valid
  %f = and i1 %zero.valid, %offset.valid
  %g = and i1 %view.count.valid, %base.nonnull
  %ab = and i1 %a, %b
  %cd = and i1 %c, %d
  %ef = and i1 %e, %f
  %g.end = and i1 %g, %end.valid
  %abcd = and i1 %ab, %cd
  %efgh = and i1 %ef, %g.end
  %all.fields = and i1 %abcd, %efgh
  %valid = and i1 %all.fields, %issued.valid
  br i1 %valid, label %success, label %invalid
success:
  %bytes = getelementptr i8, ptr %base, i64 %offset
  ret ptr %bytes
invalid:
  call void @topal.kernel.region.abort()
  unreachable
}

; Valid arbitrary-precision Nat outside u64 refuses; it is not truncated.
define internal { i1, i64 } @topal.kernel.region.index(ptr %number) nounwind noinline {
entry:
  %sign = load i64, ptr %number, align 8
  %nonnegative = icmp eq i64 %sign, 0
  br i1 %nonnegative, label %number.length, label %invalid
number.length:
  %length.pointer = getelementptr %topal.IntStorage, ptr %number, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %zero = icmp eq i64 %length, 0
  br i1 %zero, label %zero.result, label %bounded
zero.result:
  ret { i1, i64 } { i1 true, i64 0 }
bounded:
  %fits = icmp ule i64 %length, 2
  br i1 %fits, label %low, label %refuse
low:
  %low.pointer = getelementptr %topal.IntStorage, ptr %number, i32 0, i32 2, i64 0
  %low.word = load i32, ptr %low.pointer, align 4
  %low.value = zext i32 %low.word to i64
  %single = icmp eq i64 %length, 1
  br i1 %single, label %single.result, label %high
single.result:
  %one = insertvalue { i1, i64 } { i1 true, i64 0 }, i64 %low.value, 1
  ret { i1, i64 } %one
high:
  %high.pointer = getelementptr %topal.IntStorage, ptr %number, i32 0, i32 2, i64 1
  %high.word = load i32, ptr %high.pointer, align 4
  %high.value = zext i32 %high.word to i64
  %shifted = shl i64 %high.value, 32
  %value = or i64 %shifted, %low.value
  %two = insertvalue { i1, i64 } { i1 true, i64 0 }, i64 %value, 1
  ret { i1, i64 } %two
refuse:
  ret { i1, i64 } { i1 false, i64 0 }
invalid:
  call void @topal.kernel.region.abort()
  unreachable
}

define internal ptr @topal.kernel.region.count(ptr %view) nounwind noinline {
entry:
  %bytes = call ptr @topal.kernel.region.guard(ptr %view)
  %count.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 2
  %count = load i64, ptr %count.pointer, align 8
  %number = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %number
}

define internal ptr @topal.kernel.region.load(ptr %view, ptr %index) nounwind noinline {
entry:
  %bytes = call ptr @topal.kernel.region.guard(ptr %view)
  %decoded = call { i1, i64 } @topal.kernel.region.index(ptr %index)
  %fits = extractvalue { i1, i64 } %decoded, 0
  %offset = extractvalue { i1, i64 } %decoded, 1
  %count.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 2
  %count = load i64, ptr %count.pointer, align 8
  %contained = icmp ult i64 %offset, %count
  %valid = and i1 %fits, %contained
  br i1 %valid, label %read, label %refuse
read:
  %pointer = getelementptr i8, ptr %bytes, i64 %offset
  %byte = load i8, ptr %pointer, align 1
  %value = zext i8 %byte to i64
  %number = call ptr @topal.runtime.int.from.u64(i64 %value)
  %some = call ptr @topal.runtime.optional.some(ptr %number)
  ret ptr %some
refuse:
  %none = call ptr @topal.runtime.optional.none()
  ret ptr %none
}

define internal i1 @topal.kernel.region.store(ptr %view, ptr %index, ptr %number) nounwind noinline {
entry:
  %bytes = call ptr @topal.kernel.region.guard(ptr %view)
  %decoded.index = call { i1, i64 } @topal.kernel.region.index(ptr %index)
  %decoded.value = call { i1, i64 } @topal.kernel.region.index(ptr %number)
  %index.fits = extractvalue { i1, i64 } %decoded.index, 0
  %offset = extractvalue { i1, i64 } %decoded.index, 1
  %value.fits = extractvalue { i1, i64 } %decoded.value, 0
  %value = extractvalue { i1, i64 } %decoded.value, 1
  %count.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 2
  %rights.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 3
  %count = load i64, ptr %count.pointer, align 8
  %rights = load i64, ptr %rights.pointer, align 8
  %contained = icmp ult i64 %offset, %count
  %writable = icmp eq i64 %rights, 1
  %byte.valid = icmp ule i64 %value, 255
  %both.fit = and i1 %index.fits, %value.fits
  %access = and i1 %contained, %writable
  %range = and i1 %both.fit, %byte.valid
  %valid = and i1 %access, %range
  br i1 %valid, label %write, label %refuse
write:
  %pointer = getelementptr i8, ptr %bytes, i64 %offset
  %byte = trunc i64 %value to i8
  store i8 %byte, ptr %pointer, align 1
  ret i1 true
refuse:
  ret i1 false
}

define internal ptr @topal.kernel.region.window(ptr %view, ptr %start.number, ptr %count.number, i1 %write) nounwind noinline {
entry:
  %bytes = call ptr @topal.kernel.region.guard(ptr %view)
  %decoded.start = call { i1, i64 } @topal.kernel.region.index(ptr %start.number)
  %decoded.count = call { i1, i64 } @topal.kernel.region.index(ptr %count.number)
  %start.fits = extractvalue { i1, i64 } %decoded.start, 0
  %start = extractvalue { i1, i64 } %decoded.start, 1
  %count.fits = extractvalue { i1, i64 } %decoded.count, 0
  %count = extractvalue { i1, i64 } %decoded.count, 1
  %parent.offset.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 1
  %parent.count.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 2
  %parent.rights.pointer = getelementptr %topal.KernelRegionView, ptr %view, i32 0, i32 3
  %parent.offset = load i64, ptr %parent.offset.pointer, align 8
  %parent.count = load i64, ptr %parent.count.pointer, align 8
  %parent.rights = load i64, ptr %parent.rights.pointer, align 8
  %start.valid = icmp ule i64 %start, %parent.count
  %remaining = sub i64 %parent.count, %start
  %count.valid = icmp ule i64 %count, %remaining
  %rights = zext i1 %write to i64
  %rights.valid = icmp ule i64 %rights, %parent.rights
  %range = and i1 %start.valid, %count.valid
  %facts = and i1 %start.fits, %count.fits
  %allowed = and i1 %range, %rights.valid
  %valid = and i1 %allowed, %facts
  br i1 %valid, label %capacity, label %refuse
capacity:
  %control = load ptr, ptr %view, align 8
  %issued.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 7
  %issued = load i64, ptr %issued.pointer, align 8
  %available = icmp ult i64 %issued, 256
  br i1 %available, label %derive, label %invalid
derive:
  %new = call ptr @topal.platform.allocate(i64 64)
  %fields = load %topal.KernelRegionView, ptr %view, align 8
  store %topal.KernelRegionView %fields, ptr %new, align 8
  %new.offset = add i64 %parent.offset, %start
  %new.offset.pointer = getelementptr %topal.KernelRegionView, ptr %new, i32 0, i32 1
  %new.count.pointer = getelementptr %topal.KernelRegionView, ptr %new, i32 0, i32 2
  %new.rights.pointer = getelementptr %topal.KernelRegionView, ptr %new, i32 0, i32 3
  store i64 %new.offset, ptr %new.offset.pointer, align 8
  store i64 %count, ptr %new.count.pointer, align 8
  store i64 %rights, ptr %new.rights.pointer, align 8
  %next.issued = add i64 %issued, 1
  store i64 %next.issued, ptr %issued.pointer, align 8
  %some = call ptr @topal.runtime.optional.some(ptr %new)
  ret ptr %some
refuse:
  %none = call ptr @topal.runtime.optional.none()
  ret ptr %none
invalid:
  call void @topal.kernel.region.abort()
  unreachable
}
