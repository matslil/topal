; topal.platform.linux-x86_64/1
; Freestanding Linux services and the private topal-native/5 exact-value runtime.
; Int values are immutable sign-and-magnitude objects with little-endian
; base-2^32 limbs. A zero has sign = 0 and length = 0.

%topal.IntStorage = type { i64, i64, [0 x i32] }
%topal.IntDivmod = type { ptr, ptr }
%topal.RationalStorage = type { ptr, ptr }
%topal.RangeStorage = type { ptr, ptr, i64, i64 }
%topal.ResultStorage = type { i64, ptr }
%topal.ErrorStorage = type { ptr, i64, i32, i32, ptr, i64, ptr, ptr, i64, i64, i64 }

@topal.runtime.int.zero = private constant { i64, i64, [0 x i32] } { i64 0, i64 0, [0 x i32] zeroinitializer }, align 8
@topal.runtime.int.one = private constant { i64, i64, [1 x i32] } { i64 0, i64 1, [1 x i32] [i32 1] }, align 8
@topal.runtime.byte.zero = private constant [1 x i8] c"0", align 1
@topal.runtime.byte.minus = private constant [1 x i8] c"-", align 1
@topal.runtime.rational.prefix = private constant [11 x i8] c"Rational ( ", align 1
@topal.runtime.rational.separator = private constant [2 x i8] c", ", align 1
@topal.runtime.rational.suffix = private constant [2 x i8] c" )", align 1
@topal.runtime.range.closed.open = private constant [4 x i8] c" .. ", align 1
@topal.runtime.range.open.open = private constant [5 x i8] c" <.. ", align 1
@topal.runtime.range.closed.closed = private constant [5 x i8] c" ..= ", align 1
@topal.runtime.range.open.closed = private constant [6 x i8] c" <..= ", align 1
@topal.runtime.error.prefix = private constant [18 x i8] c"Error ( domain is ", align 1
@topal.runtime.error.code.separator = private constant [10 x i8] c", code is ", align 1
@topal.runtime.error.suffix = private constant [2 x i8] c" )", align 1
@topal.runtime.error.code.out.of.range = private constant [12 x i8] c"out-of-range", align 1
@topal.runtime.error.code.not.representable = private constant [17 x i8] c"not-representable", align 1
@topal.runtime.error.code.division.by.zero = private constant [16 x i8] c"division-by-zero", align 1
@topal.runtime.error.code.indeterminate = private constant [13 x i8] c"indeterminate", align 1

declare i32 @llvm.ctlz.i32(i32, i1 immarg)

define internal i64 @topal.platform.write(i64 %fd, ptr %buffer, i64 %length) nounwind noinline {
entry:
  %result = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 1, i64 %fd, ptr %buffer, i64 %length)
  ret i64 %result
}

define internal void @topal.platform.exit(i64 %status) noreturn nounwind noinline {
entry:
  %ignored = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},~{rcx},~{r11},~{memory}"(i64 60, i64 %status)
  unreachable
}

define internal ptr @topal.platform.allocate(i64 %length) nounwind noinline {
entry:
  %valid = icmp ule i64 %length, 9223372036854775807
  br i1 %valid, label %map, label %failure
map:
  %result = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}"(i64 9, i64 0, i64 %length, i64 3, i64 34, i64 -1, i64 0)
  %failed = icmp uge i64 %result, -4095
  br i1 %failed, label %failure, label %success
success:
  %pointer = inttoptr i64 %result to ptr
  ret ptr %pointer
failure:
  call void @topal.platform.exit(i64 71)
  unreachable
}

define internal void @topal.platform.write_all(ptr %buffer, i64 %length) nounwind noinline {
entry:
  %empty = icmp eq i64 %length, 0
  br i1 %empty, label %done, label %loop
loop:
  %offset = phi i64 [0, %entry], [%next, %progress], [%offset, %retry]
  %remaining = sub i64 %length, %offset
  %cursor = getelementptr i8, ptr %buffer, i64 %offset
  %written = call i64 @topal.platform.write(i64 1, ptr %cursor, i64 %remaining)
  %positive = icmp sgt i64 %written, 0
  br i1 %positive, label %progress, label %error
progress:
  %next = add i64 %offset, %written
  %complete = icmp eq i64 %next, %length
  br i1 %complete, label %done, label %loop
error:
  %interrupted = icmp eq i64 %written, -4
  br i1 %interrupted, label %retry, label %failure
retry:
  br label %loop
failure:
  call void @topal.platform.exit(i64 74)
  unreachable
done:
  ret void
}

define internal ptr @topal.runtime.result.success(ptr %payload) nounwind noinline {
entry:
  %result = call ptr @topal.platform.allocate(i64 16)
  %tag.pointer = getelementptr %topal.ResultStorage, ptr %result, i32 0, i32 0
  %payload.pointer = getelementptr %topal.ResultStorage, ptr %result, i32 0, i32 1
  store i64 0, ptr %tag.pointer, align 8
  store ptr %payload, ptr %payload.pointer, align 8
  ret ptr %result
}

define internal ptr @topal.runtime.result.error(ptr %payload) nounwind noinline {
entry:
  %result = call ptr @topal.platform.allocate(i64 16)
  %tag.pointer = getelementptr %topal.ResultStorage, ptr %result, i32 0, i32 0
  %payload.pointer = getelementptr %topal.ResultStorage, ptr %result, i32 0, i32 1
  store i64 1, ptr %tag.pointer, align 8
  store ptr %payload, ptr %payload.pointer, align 8
  ret ptr %result
}

define internal i1 @topal.runtime.result.is.error(ptr %result) nounwind noinline {
entry:
  %tag.pointer = getelementptr %topal.ResultStorage, ptr %result, i32 0, i32 0
  %tag = load i64, ptr %tag.pointer, align 8
  %is.error = icmp eq i64 %tag, 1
  ret i1 %is.error
}

define internal ptr @topal.runtime.result.payload(ptr %result) nounwind noinline {
entry:
  %payload.pointer = getelementptr %topal.ResultStorage, ptr %result, i32 0, i32 1
  %payload = load ptr, ptr %payload.pointer, align 8
  ret ptr %payload
}

define internal ptr @topal.runtime.error.make(i32 %code, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column) nounwind noinline {
entry:
  %error = call ptr @topal.platform.allocate(i64 80)
  %domain.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 0
  %domain.length.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 1
  %code.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 2
  %reserved.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 3
  %detail.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 4
  %detail.length.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 5
  %cause.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 6
  %source.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 7
  %source.length.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 8
  %line.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 9
  %column.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 10
  store ptr %domain, ptr %domain.pointer, align 8
  store i64 %domain.length, ptr %domain.length.pointer, align 8
  store i32 %code, ptr %code.pointer, align 4
  store i32 0, ptr %reserved.pointer, align 4
  store ptr null, ptr %detail.pointer, align 8
  store i64 0, ptr %detail.length.pointer, align 8
  store ptr null, ptr %cause.pointer, align 8
  store ptr %source, ptr %source.pointer, align 8
  store i64 %source.length, ptr %source.length.pointer, align 8
  store i64 %line, ptr %line.pointer, align 8
  store i64 %column, ptr %column.pointer, align 8
  ret ptr %error
}

define internal ptr @topal.runtime.result.failure(i32 %code, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column) nounwind noinline {
entry:
  %error = call ptr @topal.runtime.error.make(i32 %code, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column)
  %result = call ptr @topal.runtime.result.error(ptr %error)
  ret ptr %result
}

define internal void @topal.runtime.error.print(ptr %error) nounwind noinline {
entry:
  %domain.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 0
  %domain.length.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 1
  %code.pointer = getelementptr %topal.ErrorStorage, ptr %error, i32 0, i32 2
  %domain = load ptr, ptr %domain.pointer, align 8
  %domain.length = load i64, ptr %domain.length.pointer, align 8
  %code = load i32, ptr %code.pointer, align 4
  call void @topal.platform.write_all(ptr @topal.runtime.error.prefix, i64 18)
  call void @topal.platform.write_all(ptr %domain, i64 %domain.length)
  call void @topal.platform.write_all(ptr @topal.runtime.error.code.separator, i64 10)
  switch i32 %code, label %indeterminate [ i32 0, label %out.of.range i32 1, label %not.representable i32 2, label %division.by.zero ]
out.of.range:
  call void @topal.platform.write_all(ptr @topal.runtime.error.code.out.of.range, i64 12)
  br label %done
not.representable:
  call void @topal.platform.write_all(ptr @topal.runtime.error.code.not.representable, i64 17)
  br label %done
division.by.zero:
  call void @topal.platform.write_all(ptr @topal.runtime.error.code.division.by.zero, i64 16)
  br label %done
indeterminate:
  call void @topal.platform.write_all(ptr @topal.runtime.error.code.indeterminate, i64 13)
  br label %done
done:
  call void @topal.platform.write_all(ptr @topal.runtime.error.suffix, i64 2)
  ret void
}

define internal ptr @topal.runtime.int.allocate(i64 %length, i64 %negative) nounwind noinline {
entry:
  %length.valid = icmp ule i64 %length, 2305843009213693947
  br i1 %length.valid, label %allocate, label %failure
allocate:
  %limb.bytes = mul i64 %length, 4
  %bytes = add i64 %limb.bytes, 16
  %value = call ptr @topal.platform.allocate(i64 %bytes)
  %negative.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 0
  %length.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 1
  store i64 %negative, ptr %negative.pointer, align 8
  store i64 %length, ptr %length.pointer, align 8
  ret ptr %value
failure:
  call void @topal.platform.exit(i64 71)
  unreachable
}

define internal i64 @topal.runtime.int.limb.or.zero(ptr %value, i64 %index) nounwind noinline {
entry:
  %length.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %present = icmp ult i64 %index, %length
  br i1 %present, label %load, label %zero
load:
  %limb.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 2, i64 %index
  %limb = load i32, ptr %limb.pointer, align 4
  %extended = zext i32 %limb to i64
  ret i64 %extended
zero:
  ret i64 0
}

define internal ptr @topal.runtime.int.normalize(ptr %value) nounwind noinline {
entry:
  %length.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %empty = icmp eq i64 %length, 0
  br i1 %empty, label %zero, label %trim
trim:
  %candidate = phi i64 [%length, %entry], [%next, %leading.zero]
  %index = sub i64 %candidate, 1
  %limb.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 2, i64 %index
  %limb = load i32, ptr %limb.pointer, align 4
  %is.zero = icmp eq i32 %limb, 0
  br i1 %is.zero, label %leading.zero, label %done
leading.zero:
  %next = sub i64 %candidate, 1
  %now.empty = icmp eq i64 %next, 0
  br i1 %now.empty, label %zero, label %trim
zero:
  %negative.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 0
  store i64 0, ptr %negative.pointer, align 8
  store i64 0, ptr %length.pointer, align 8
  ret ptr %value
done:
  store i64 %candidate, ptr %length.pointer, align 8
  ret ptr %value
}

define internal ptr @topal.runtime.int.copy.with.sign(ptr %value, i64 %negative) nounwind noinline {
entry:
  %length.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %empty = icmp eq i64 %length, 0
  br i1 %empty, label %zero, label %allocate
zero:
  ret ptr @topal.runtime.int.zero
allocate:
  %copy = call ptr @topal.runtime.int.allocate(i64 %length, i64 %negative)
  br label %loop
loop:
  %index = phi i64 [0, %allocate], [%next, %loop]
  %source = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 2, i64 %index
  %limb = load i32, ptr %source, align 4
  %destination = getelementptr %topal.IntStorage, ptr %copy, i32 0, i32 2, i64 %index
  store i32 %limb, ptr %destination, align 4
  %next = add i64 %index, 1
  %more = icmp ult i64 %next, %length
  br i1 %more, label %loop, label %done
done:
  ret ptr %copy
}

define internal ptr @topal.runtime.int.negate(ptr %value) nounwind noinline {
entry:
  %length.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %empty = icmp eq i64 %length, 0
  br i1 %empty, label %zero, label %copy
zero:
  ret ptr @topal.runtime.int.zero
copy:
  %negative.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 0
  %negative = load i64, ptr %negative.pointer, align 8
  %opposite = xor i64 %negative, 1
  %result = call ptr @topal.runtime.int.copy.with.sign(ptr %value, i64 %opposite)
  ret ptr %result
}

define internal ptr @topal.runtime.int.absolute(ptr %value) nounwind noinline {
entry:
  %negative.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 0
  %negative = load i64, ptr %negative.pointer, align 8
  %already.absolute = icmp eq i64 %negative, 0
  br i1 %already.absolute, label %same, label %copy
same:
  ret ptr %value
copy:
  %result = call ptr @topal.runtime.int.copy.with.sign(ptr %value, i64 0)
  ret ptr %result
}

define internal i32 @topal.runtime.int.compare.absolute(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.length.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 1
  %right.length.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 1
  %left.length = load i64, ptr %left.length.pointer, align 8
  %right.length = load i64, ptr %right.length.pointer, align 8
  %shorter = icmp ult i64 %left.length, %right.length
  br i1 %shorter, label %less, label %check.longer
check.longer:
  %longer = icmp ugt i64 %left.length, %right.length
  br i1 %longer, label %greater, label %check.empty
check.empty:
  %empty = icmp eq i64 %left.length, 0
  br i1 %empty, label %equal, label %limbs
limbs:
  %position = phi i64 [%left.length, %check.empty], [%index, %limbs.equal]
  %index = sub i64 %position, 1
  %left.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 2, i64 %index
  %right.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 2, i64 %index
  %left.limb = load i32, ptr %left.pointer, align 4
  %right.limb = load i32, ptr %right.pointer, align 4
  %limb.less = icmp ult i32 %left.limb, %right.limb
  br i1 %limb.less, label %less, label %check.limb.greater
check.limb.greater:
  %limb.greater = icmp ugt i32 %left.limb, %right.limb
  br i1 %limb.greater, label %greater, label %limbs.equal
limbs.equal:
  %more = icmp ne i64 %index, 0
  br i1 %more, label %limbs, label %equal
less:
  ret i32 -1
equal:
  ret i32 0
greater:
  ret i32 1
}

define internal i32 @topal.runtime.int.compare(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.negative.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 0
  %right.negative.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 0
  %left.negative = load i64, ptr %left.negative.pointer, align 8
  %right.negative = load i64, ptr %right.negative.pointer, align 8
  %different.signs = icmp ne i64 %left.negative, %right.negative
  br i1 %different.signs, label %signed.result, label %absolute
signed.result:
  %left.is.negative = icmp ne i64 %left.negative, 0
  %result = select i1 %left.is.negative, i32 -1, i32 1
  ret i32 %result
absolute:
  %magnitude = call i32 @topal.runtime.int.compare.absolute(ptr %left, ptr %right)
  %both.negative = icmp ne i64 %left.negative, 0
  %reversed = sub i32 0, %magnitude
  %ordered = select i1 %both.negative, i32 %reversed, i32 %magnitude
  ret i32 %ordered
}

define internal ptr @topal.runtime.int.add.absolute(ptr %left, ptr %right, i64 %negative) nounwind noinline {
entry:
  %left.length.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 1
  %right.length.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 1
  %left.length = load i64, ptr %left.length.pointer, align 8
  %right.length = load i64, ptr %right.length.pointer, align 8
  %left.longer = icmp ugt i64 %left.length, %right.length
  %maximum = select i1 %left.longer, i64 %left.length, i64 %right.length
  %result.length = add i64 %maximum, 1
  %result = call ptr @topal.runtime.int.allocate(i64 %result.length, i64 %negative)
  %none = icmp eq i64 %maximum, 0
  br i1 %none, label %finish, label %loop
loop:
  %index = phi i64 [0, %entry], [%next, %loop]
  %carry = phi i64 [0, %entry], [%next.carry, %loop]
  %left.limb = call i64 @topal.runtime.int.limb.or.zero(ptr %left, i64 %index)
  %right.limb = call i64 @topal.runtime.int.limb.or.zero(ptr %right, i64 %index)
  %pair = add i64 %left.limb, %right.limb
  %sum = add i64 %pair, %carry
  %low = trunc i64 %sum to i32
  %destination = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 2, i64 %index
  store i32 %low, ptr %destination, align 4
  %next.carry = lshr i64 %sum, 32
  %next = add i64 %index, 1
  %more = icmp ult i64 %next, %maximum
  br i1 %more, label %loop, label %finish
finish:
  %final.carry = phi i64 [0, %entry], [%next.carry, %loop]
  %carry.present = icmp ne i64 %final.carry, 0
  br i1 %carry.present, label %store.carry, label %without.carry
store.carry:
  %carry.limb = trunc i64 %final.carry to i32
  %carry.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 2, i64 %maximum
  store i32 %carry.limb, ptr %carry.pointer, align 4
  ret ptr %result
without.carry:
  %result.length.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 1
  store i64 %maximum, ptr %result.length.pointer, align 8
  %normalized = call ptr @topal.runtime.int.normalize(ptr %result)
  ret ptr %normalized
}

define internal ptr @topal.runtime.int.subtract.absolute(ptr %larger, ptr %smaller, i64 %negative) nounwind noinline {
entry:
  %length.pointer = getelementptr %topal.IntStorage, ptr %larger, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %result = call ptr @topal.runtime.int.allocate(i64 %length, i64 %negative)
  br label %loop
loop:
  %index = phi i64 [0, %entry], [%next, %loop]
  %borrow = phi i64 [0, %entry], [%next.borrow, %loop]
  %left = call i64 @topal.runtime.int.limb.or.zero(ptr %larger, i64 %index)
  %right = call i64 @topal.runtime.int.limb.or.zero(ptr %smaller, i64 %index)
  %subtrahend = add i64 %right, %borrow
  %needs.borrow = icmp ult i64 %left, %subtrahend
  %extension = select i1 %needs.borrow, i64 4294967296, i64 0
  %extended.left = add i64 %left, %extension
  %difference = sub i64 %extended.left, %subtrahend
  %low = trunc i64 %difference to i32
  %destination = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 2, i64 %index
  store i32 %low, ptr %destination, align 4
  %next.borrow = zext i1 %needs.borrow to i64
  %next = add i64 %index, 1
  %more = icmp ult i64 %next, %length
  br i1 %more, label %loop, label %done
done:
  %normalized = call ptr @topal.runtime.int.normalize(ptr %result)
  ret ptr %normalized
}

define internal ptr @topal.runtime.int.add(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.zero = call i1 @topal.runtime.int.is.zero(ptr %left)
  br i1 %left.zero, label %return.right, label %check.right
return.right:
  ret ptr %right
check.right:
  %right.zero = call i1 @topal.runtime.int.is.zero(ptr %right)
  br i1 %right.zero, label %return.left, label %nonzero
return.left:
  ret ptr %left
nonzero:
  %left.negative.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 0
  %right.negative.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 0
  %left.negative = load i64, ptr %left.negative.pointer, align 8
  %right.negative = load i64, ptr %right.negative.pointer, align 8
  %same.sign = icmp eq i64 %left.negative, %right.negative
  br i1 %same.sign, label %sum, label %difference
sum:
  %sum.value = call ptr @topal.runtime.int.add.absolute(ptr %left, ptr %right, i64 %left.negative)
  ret ptr %sum.value
difference:
  %ordering = call i32 @topal.runtime.int.compare.absolute(ptr %left, ptr %right)
  %equal = icmp eq i32 %ordering, 0
  br i1 %equal, label %zero, label %select.larger
zero:
  ret ptr @topal.runtime.int.zero
select.larger:
  %left.larger = icmp sgt i32 %ordering, 0
  br i1 %left.larger, label %left.difference, label %right.difference
left.difference:
  %left.value = call ptr @topal.runtime.int.subtract.absolute(ptr %left, ptr %right, i64 %left.negative)
  ret ptr %left.value
right.difference:
  %right.value = call ptr @topal.runtime.int.subtract.absolute(ptr %right, ptr %left, i64 %right.negative)
  ret ptr %right.value
}

define internal ptr @topal.runtime.int.subtract(ptr %left, ptr %right) nounwind noinline {
entry:
  %opposite = call ptr @topal.runtime.int.negate(ptr %right)
  %result = call ptr @topal.runtime.int.add(ptr %left, ptr %opposite)
  ret ptr %result
}

define internal ptr @topal.runtime.int.multiply(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.length.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 1
  %right.length.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 1
  %left.length = load i64, ptr %left.length.pointer, align 8
  %right.length = load i64, ptr %right.length.pointer, align 8
  %left.empty = icmp eq i64 %left.length, 0
  %right.empty = icmp eq i64 %right.length, 0
  %empty = or i1 %left.empty, %right.empty
  br i1 %empty, label %zero, label %check.length
zero:
  ret ptr @topal.runtime.int.zero
check.length:
  %available = sub i64 -1, %left.length
  %overflow = icmp ugt i64 %right.length, %available
  br i1 %overflow, label %failure, label %allocate
failure:
  call void @topal.platform.exit(i64 71)
  unreachable
allocate:
  %result.length = add i64 %left.length, %right.length
  %left.negative.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 0
  %right.negative.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 0
  %left.negative = load i64, ptr %left.negative.pointer, align 8
  %right.negative = load i64, ptr %right.negative.pointer, align 8
  %negative = xor i64 %left.negative, %right.negative
  %result = call ptr @topal.runtime.int.allocate(i64 %result.length, i64 %negative)
  br label %clear
clear:
  %clear.index = phi i64 [0, %allocate], [%clear.next, %clear]
  %clear.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 2, i64 %clear.index
  store i32 0, ptr %clear.pointer, align 4
  %clear.next = add i64 %clear.index, 1
  %clear.more = icmp ult i64 %clear.next, %result.length
  br i1 %clear.more, label %clear, label %outer.entry
outer.entry:
  br label %outer
outer:
  %left.index = phi i64 [0, %outer.entry], [%left.next, %outer.done]
  %left.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 2, i64 %left.index
  %left.limb.raw = load i32, ptr %left.pointer, align 4
  %left.limb = zext i32 %left.limb.raw to i64
  br label %inner
inner:
  %right.index = phi i64 [0, %outer], [%right.next, %inner]
  %carry = phi i64 [0, %outer], [%next.carry, %inner]
  %right.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 2, i64 %right.index
  %right.limb.raw = load i32, ptr %right.pointer, align 4
  %right.limb = zext i32 %right.limb.raw to i64
  %result.index = add i64 %left.index, %right.index
  %result.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 2, i64 %result.index
  %existing.raw = load i32, ptr %result.pointer, align 4
  %existing = zext i32 %existing.raw to i64
  %product = mul i64 %left.limb, %right.limb
  %partial = add i64 %product, %existing
  %total = add i64 %partial, %carry
  %low = trunc i64 %total to i32
  store i32 %low, ptr %result.pointer, align 4
  %next.carry = lshr i64 %total, 32
  %right.next = add i64 %right.index, 1
  %right.more = icmp ult i64 %right.next, %right.length
  br i1 %right.more, label %inner, label %outer.done
outer.done:
  %carry.index = add i64 %left.index, %right.length
  %carry.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 2, i64 %carry.index
  %carry.limb = trunc i64 %next.carry to i32
  store i32 %carry.limb, ptr %carry.pointer, align 4
  %left.next = add i64 %left.index, 1
  %left.more = icmp ult i64 %left.next, %left.length
  br i1 %left.more, label %outer, label %done
done:
  %normalized = call ptr @topal.runtime.int.normalize(ptr %result)
  ret ptr %normalized
}

define internal void @topal.runtime.int.print(ptr %value) nounwind noinline {
entry:
  %length.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %empty = icmp eq i64 %length, 0
  br i1 %empty, label %zero, label %check.sign
zero:
  call void @topal.platform.write_all(ptr @topal.runtime.byte.zero, i64 1)
  ret void
check.sign:
  %negative.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 0
  %negative = load i64, ptr %negative.pointer, align 8
  %is.negative = icmp ne i64 %negative, 0
  br i1 %is.negative, label %write.sign, label %prepare
write.sign:
  call void @topal.platform.write_all(ptr @topal.runtime.byte.minus, i64 1)
  br label %prepare
prepare:
  %capacity.valid = icmp ule i64 %length, 922337203685477580
  br i1 %capacity.valid, label %allocate, label %failure
failure:
  call void @topal.platform.exit(i64 71)
  unreachable
allocate:
  %scratch.bytes = mul i64 %length, 4
  %capacity = mul i64 %length, 10
  %scratch = call ptr @topal.platform.allocate(i64 %scratch.bytes)
  %buffer = call ptr @topal.platform.allocate(i64 %capacity)
  br label %copy
copy:
  %copy.index = phi i64 [0, %allocate], [%copy.next, %copy]
  %source = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 2, i64 %copy.index
  %limb = load i32, ptr %source, align 4
  %destination = getelementptr i32, ptr %scratch, i64 %copy.index
  store i32 %limb, ptr %destination, align 4
  %copy.next = add i64 %copy.index, 1
  %copy.more = icmp ult i64 %copy.next, %length
  br i1 %copy.more, label %copy, label %digits
digits:
  %work.length = phi i64 [%length, %copy], [%next.length, %division.finish]
  %position = phi i64 [%capacity, %copy], [%next.position, %division.finish]
  br label %divide
divide:
  %division.position = phi i64 [%work.length, %digits], [%division.index, %divide.step]
  %remainder = phi i64 [0, %digits], [%next.remainder, %divide.step]
  %division.index = sub i64 %division.position, 1
  %work.pointer = getelementptr i32, ptr %scratch, i64 %division.index
  %work.raw = load i32, ptr %work.pointer, align 4
  %work = zext i32 %work.raw to i64
  %high = shl i64 %remainder, 32
  %current = or i64 %high, %work
  %quotient = udiv i64 %current, 10
  %next.remainder = urem i64 %current, 10
  %quotient.limb = trunc i64 %quotient to i32
  store i32 %quotient.limb, ptr %work.pointer, align 4
  %division.more = icmp ne i64 %division.index, 0
  br i1 %division.more, label %divide.step, label %division.done
divide.step:
  br label %divide
division.done:
  %next.position = sub i64 %position, 1
  %digit = trunc i64 %next.remainder to i8
  %ascii = add i8 %digit, 48
  %digit.pointer = getelementptr i8, ptr %buffer, i64 %next.position
  store i8 %ascii, ptr %digit.pointer, align 1
  br label %trim
trim:
  %candidate = phi i64 [%work.length, %division.done], [%trim.next, %trim.zero]
  %trim.index = sub i64 %candidate, 1
  %trim.pointer = getelementptr i32, ptr %scratch, i64 %trim.index
  %trim.limb = load i32, ptr %trim.pointer, align 4
  %leading.zero = icmp eq i32 %trim.limb, 0
  br i1 %leading.zero, label %trim.zero, label %trim.done
trim.zero:
  %trim.next = sub i64 %candidate, 1
  %trim.empty = icmp eq i64 %trim.next, 0
  br i1 %trim.empty, label %trim.empty.done, label %trim
trim.empty.done:
  br label %division.finish
trim.done:
  br label %division.finish
division.finish:
  %next.length = phi i64 [0, %trim.empty.done], [%candidate, %trim.done]
  %more.digits = icmp ne i64 %next.length, 0
  br i1 %more.digits, label %digits, label %emit
emit:
  %count = sub i64 %capacity, %next.position
  %first = getelementptr i8, ptr %buffer, i64 %next.position
  call void @topal.platform.write_all(ptr %first, i64 %count)
  ret void
}

define internal i1 @topal.runtime.int.is.zero(ptr %value) nounwind noinline {
entry:
  %length.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %zero = icmp eq i64 %length, 0
  ret i1 %zero
}

define internal ptr @topal.runtime.int.divmod.pair(ptr %quotient, ptr %remainder) nounwind noinline {
entry:
  %pair = call ptr @topal.platform.allocate(i64 16)
  %quotient.pointer = getelementptr %topal.IntDivmod, ptr %pair, i32 0, i32 0
  %remainder.pointer = getelementptr %topal.IntDivmod, ptr %pair, i32 0, i32 1
  store ptr %quotient, ptr %quotient.pointer, align 8
  store ptr %remainder, ptr %remainder.pointer, align 8
  ret ptr %pair
}

define internal ptr @topal.runtime.int.divmod.quotient(ptr %pair) nounwind noinline {
entry:
  %pointer = getelementptr %topal.IntDivmod, ptr %pair, i32 0, i32 0
  %value = load ptr, ptr %pointer, align 8
  ret ptr %value
}

define internal ptr @topal.runtime.int.divmod.remainder(ptr %pair) nounwind noinline {
entry:
  %pointer = getelementptr %topal.IntDivmod, ptr %pair, i32 0, i32 1
  %value = load ptr, ptr %pointer, align 8
  ret ptr %value
}

define internal i64 @topal.runtime.int.bit.length(ptr %value) nounwind noinline {
entry:
  %length.pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 1
  %length = load i64, ptr %length.pointer, align 8
  %empty = icmp eq i64 %length, 0
  br i1 %empty, label %zero, label %check
zero:
  ret i64 0
check:
  %valid = icmp ule i64 %length, 576460752303423487
  br i1 %valid, label %calculate, label %failure
failure:
  call void @topal.platform.exit(i64 71)
  unreachable
calculate:
  %last = sub i64 %length, 1
  %pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 2, i64 %last
  %limb = load i32, ptr %pointer, align 4
  %leading = call i32 @llvm.ctlz.i32(i32 %limb, i1 false)
  %top.bits.raw = sub i32 32, %leading
  %top.bits = zext i32 %top.bits.raw to i64
  %lower.bits = mul i64 %last, 32
  %bits = add i64 %lower.bits, %top.bits
  ret i64 %bits
}

define internal i1 @topal.runtime.int.bit.at(ptr %value, i64 %index) nounwind noinline {
entry:
  %limb.index = lshr i64 %index, 5
  %offset.raw = and i64 %index, 31
  %offset = trunc i64 %offset.raw to i32
  %pointer = getelementptr %topal.IntStorage, ptr %value, i32 0, i32 2, i64 %limb.index
  %limb = load i32, ptr %pointer, align 4
  %shifted = lshr i32 %limb, %offset
  %bit.raw = and i32 %shifted, 1
  %bit = icmp ne i32 %bit.raw, 0
  ret i1 %bit
}

define internal ptr @topal.runtime.int.divmod.absolute(ptr %dividend, ptr %divisor) nounwind noinline {
entry:
  %divisor.zero = call i1 @topal.runtime.int.is.zero(ptr %divisor)
  br i1 %divisor.zero, label %failure, label %check.dividend
failure:
  call void @topal.platform.exit(i64 65)
  unreachable
check.dividend:
  %dividend.zero = call i1 @topal.runtime.int.is.zero(ptr %dividend)
  br i1 %dividend.zero, label %zero, label %prepare
zero:
  %zero.pair = call ptr @topal.runtime.int.divmod.pair(ptr @topal.runtime.int.zero, ptr @topal.runtime.int.zero)
  ret ptr %zero.pair
prepare:
  %bits = call i64 @topal.runtime.int.bit.length(ptr %dividend)
  br label %loop
loop:
  %position = phi i64 [%bits, %prepare], [%next.position, %merge]
  %quotient = phi ptr [@topal.runtime.int.zero, %prepare], [%next.quotient, %merge]
  %remainder = phi ptr [@topal.runtime.int.zero, %prepare], [%next.remainder, %merge]
  %bit.index = sub i64 %position, 1
  %bit = call i1 @topal.runtime.int.bit.at(ptr %dividend, i64 %bit.index)
  %quotient.twice = call ptr @topal.runtime.int.add(ptr %quotient, ptr %quotient)
  %remainder.twice = call ptr @topal.runtime.int.add(ptr %remainder, ptr %remainder)
  br i1 %bit, label %append.bit, label %compare
append.bit:
  %remainder.with.bit = call ptr @topal.runtime.int.add(ptr %remainder.twice, ptr @topal.runtime.int.one)
  br label %compare
compare:
  %candidate = phi ptr [%remainder.twice, %loop], [%remainder.with.bit, %append.bit]
  %ordering = call i32 @topal.runtime.int.compare.absolute(ptr %candidate, ptr %divisor)
  %subtract = icmp sge i32 %ordering, 0
  br i1 %subtract, label %subtract.divisor, label %retain
subtract.divisor:
  %reduced = call ptr @topal.runtime.int.subtract.absolute(ptr %candidate, ptr %divisor, i64 0)
  %incremented = call ptr @topal.runtime.int.add(ptr %quotient.twice, ptr @topal.runtime.int.one)
  br label %merge
retain:
  br label %merge
merge:
  %next.quotient = phi ptr [%incremented, %subtract.divisor], [%quotient.twice, %retain]
  %next.remainder = phi ptr [%reduced, %subtract.divisor], [%candidate, %retain]
  %next.position = sub i64 %position, 1
  %more = icmp ne i64 %next.position, 0
  br i1 %more, label %loop, label %done
done:
  %pair = call ptr @topal.runtime.int.divmod.pair(ptr %next.quotient, ptr %next.remainder)
  ret ptr %pair
}

define internal ptr @topal.runtime.int.quotient.modulo(ptr %left, ptr %right) nounwind noinline {
entry:
  %right.zero = call i1 @topal.runtime.int.is.zero(ptr %right)
  br i1 %right.zero, label %failure, label %divide
failure:
  call void @topal.platform.exit(i64 65)
  unreachable
divide:
  %left.absolute = call ptr @topal.runtime.int.absolute(ptr %left)
  %right.absolute = call ptr @topal.runtime.int.absolute(ptr %right)
  %unsigned = call ptr @topal.runtime.int.divmod.absolute(ptr %left.absolute, ptr %right.absolute)
  %quotient.absolute = call ptr @topal.runtime.int.divmod.quotient(ptr %unsigned)
  %remainder.absolute = call ptr @topal.runtime.int.divmod.remainder(ptr %unsigned)
  %left.sign.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 0
  %right.sign.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 0
  %left.sign = load i64, ptr %left.sign.pointer, align 8
  %right.sign = load i64, ptr %right.sign.pointer, align 8
  %left.negative = icmp ne i64 %left.sign, 0
  %remainder.zero = call i1 @topal.runtime.int.is.zero(ptr %remainder.absolute)
  %remainder.present = xor i1 %remainder.zero, true
  %needs.adjustment = and i1 %left.negative, %remainder.present
  %quotient.sign = xor i64 %left.sign, %right.sign
  br i1 %needs.adjustment, label %adjust, label %direct
direct:
  %direct.quotient = call ptr @topal.runtime.int.copy.with.sign(ptr %quotient.absolute, i64 %quotient.sign)
  %direct.pair = call ptr @topal.runtime.int.divmod.pair(ptr %direct.quotient, ptr %remainder.absolute)
  ret ptr %direct.pair
adjust:
  %larger.quotient = call ptr @topal.runtime.int.add(ptr %quotient.absolute, ptr @topal.runtime.int.one)
  %adjusted.quotient = call ptr @topal.runtime.int.copy.with.sign(ptr %larger.quotient, i64 %quotient.sign)
  %adjusted.remainder = call ptr @topal.runtime.int.subtract.absolute(ptr %right.absolute, ptr %remainder.absolute, i64 0)
  %adjusted.pair = call ptr @topal.runtime.int.divmod.pair(ptr %adjusted.quotient, ptr %adjusted.remainder)
  ret ptr %adjusted.pair
}

define internal ptr @topal.runtime.int.modulo(ptr %left, ptr %right) nounwind noinline {
entry:
  %pair = call ptr @topal.runtime.int.quotient.modulo(ptr %left, ptr %right)
  %remainder = call ptr @topal.runtime.int.divmod.remainder(ptr %pair)
  ret ptr %remainder
}

define internal ptr @topal.runtime.int.try.modulo(ptr %left, ptr %right, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column) nounwind noinline {
entry:
  %right.zero = call i1 @topal.runtime.int.is.zero(ptr %right)
  br i1 %right.zero, label %failure, label %success
failure:
  %failed = call ptr @topal.runtime.result.failure(i32 2, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column)
  ret ptr %failed
success:
  %remainder = call ptr @topal.runtime.int.modulo(ptr %left, ptr %right)
  %result = call ptr @topal.runtime.result.success(ptr %remainder)
  ret ptr %result
}

define internal ptr @topal.runtime.int.try.quotient.modulo(ptr %left, ptr %right, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column) nounwind noinline {
entry:
  %right.zero = call i1 @topal.runtime.int.is.zero(ptr %right)
  br i1 %right.zero, label %failure, label %success
failure:
  %failed = call ptr @topal.runtime.result.failure(i32 2, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column)
  ret ptr %failed
success:
  %pair = call ptr @topal.runtime.int.quotient.modulo(ptr %left, ptr %right)
  %result = call ptr @topal.runtime.result.success(ptr %pair)
  ret ptr %result
}

define internal ptr @topal.runtime.int.power(ptr %base, ptr %exponent) nounwind noinline {
entry:
  %sign.pointer = getelementptr %topal.IntStorage, ptr %exponent, i32 0, i32 0
  %sign = load i64, ptr %sign.pointer, align 8
  %negative = icmp ne i64 %sign, 0
  br i1 %negative, label %failure, label %prepare
failure:
  call void @topal.platform.exit(i64 65)
  unreachable
prepare:
  %bits = call i64 @topal.runtime.int.bit.length(ptr %exponent)
  %empty = icmp eq i64 %bits, 0
  br i1 %empty, label %one, label %loop
one:
  ret ptr @topal.runtime.int.one
loop:
  %position = phi i64 [0, %prepare], [%next.position, %square]
  %result = phi ptr [@topal.runtime.int.one, %prepare], [%next.result, %square]
  %factor = phi ptr [%base, %prepare], [%next.factor, %square]
  %bit = call i1 @topal.runtime.int.bit.at(ptr %exponent, i64 %position)
  br i1 %bit, label %multiply, label %retain
multiply:
  %product = call ptr @topal.runtime.int.multiply(ptr %result, ptr %factor)
  br label %advance
retain:
  br label %advance
advance:
  %next.result = phi ptr [%product, %multiply], [%result, %retain]
  %next.position = add i64 %position, 1
  %complete = icmp eq i64 %next.position, %bits
  br i1 %complete, label %done, label %square
square:
  %next.factor = call ptr @topal.runtime.int.multiply(ptr %factor, ptr %factor)
  br label %loop
done:
  ret ptr %next.result
}

define internal ptr @topal.runtime.int.greatest.common.divisor(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.absolute = call ptr @topal.runtime.int.absolute(ptr %left)
  %right.absolute = call ptr @topal.runtime.int.absolute(ptr %right)
  %right.zero = call i1 @topal.runtime.int.is.zero(ptr %right.absolute)
  br i1 %right.zero, label %initial.done, label %loop
initial.done:
  ret ptr %left.absolute
loop:
  %current.left = phi ptr [%left.absolute, %entry], [%current.right, %again]
  %current.right = phi ptr [%right.absolute, %entry], [%remainder, %again]
  %division = call ptr @topal.runtime.int.divmod.absolute(ptr %current.left, ptr %current.right)
  %remainder = call ptr @topal.runtime.int.divmod.remainder(ptr %division)
  %done = call i1 @topal.runtime.int.is.zero(ptr %remainder)
  br i1 %done, label %result, label %again
again:
  br label %loop
result:
  ret ptr %current.right
}

define internal ptr @topal.runtime.rational.raw(ptr %numerator, ptr %denominator) nounwind noinline {
entry:
  %value = call ptr @topal.platform.allocate(i64 16)
  %numerator.pointer = getelementptr %topal.RationalStorage, ptr %value, i32 0, i32 0
  %denominator.pointer = getelementptr %topal.RationalStorage, ptr %value, i32 0, i32 1
  store ptr %numerator, ptr %numerator.pointer, align 8
  store ptr %denominator, ptr %denominator.pointer, align 8
  ret ptr %value
}

define internal ptr @topal.runtime.rational.numerator(ptr %value) nounwind noinline {
entry:
  %pointer = getelementptr %topal.RationalStorage, ptr %value, i32 0, i32 0
  %numerator = load ptr, ptr %pointer, align 8
  ret ptr %numerator
}

define internal ptr @topal.runtime.rational.denominator(ptr %value) nounwind noinline {
entry:
  %pointer = getelementptr %topal.RationalStorage, ptr %value, i32 0, i32 1
  %denominator = load ptr, ptr %pointer, align 8
  ret ptr %denominator
}

define internal ptr @topal.runtime.rational.make(ptr %numerator, ptr %denominator) nounwind noinline {
entry:
  %denominator.zero = call i1 @topal.runtime.int.is.zero(ptr %denominator)
  br i1 %denominator.zero, label %failure, label %check.numerator
failure:
  call void @topal.platform.exit(i64 65)
  unreachable
check.numerator:
  %numerator.zero = call i1 @topal.runtime.int.is.zero(ptr %numerator)
  br i1 %numerator.zero, label %zero, label %normalize
zero:
  %zero.value = call ptr @topal.runtime.rational.raw(ptr @topal.runtime.int.zero, ptr @topal.runtime.int.one)
  ret ptr %zero.value
normalize:
  %numerator.absolute = call ptr @topal.runtime.int.absolute(ptr %numerator)
  %denominator.absolute = call ptr @topal.runtime.int.absolute(ptr %denominator)
  %divisor = call ptr @topal.runtime.int.greatest.common.divisor(ptr %numerator.absolute, ptr %denominator.absolute)
  %numerator.division = call ptr @topal.runtime.int.divmod.absolute(ptr %numerator.absolute, ptr %divisor)
  %denominator.division = call ptr @topal.runtime.int.divmod.absolute(ptr %denominator.absolute, ptr %divisor)
  %reduced.numerator.absolute = call ptr @topal.runtime.int.divmod.quotient(ptr %numerator.division)
  %reduced.denominator = call ptr @topal.runtime.int.divmod.quotient(ptr %denominator.division)
  %numerator.sign.pointer = getelementptr %topal.IntStorage, ptr %numerator, i32 0, i32 0
  %denominator.sign.pointer = getelementptr %topal.IntStorage, ptr %denominator, i32 0, i32 0
  %numerator.sign = load i64, ptr %numerator.sign.pointer, align 8
  %denominator.sign = load i64, ptr %denominator.sign.pointer, align 8
  %sign = xor i64 %numerator.sign, %denominator.sign
  %reduced.numerator = call ptr @topal.runtime.int.copy.with.sign(ptr %reduced.numerator.absolute, i64 %sign)
  %result = call ptr @topal.runtime.rational.raw(ptr %reduced.numerator, ptr %reduced.denominator)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.from.int(ptr %value) nounwind noinline {
entry:
  %result = call ptr @topal.runtime.rational.raw(ptr %value, ptr @topal.runtime.int.one)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.negate(ptr %value) nounwind noinline {
entry:
  %numerator = call ptr @topal.runtime.rational.numerator(ptr %value)
  %denominator = call ptr @topal.runtime.rational.denominator(ptr %value)
  %negative = call ptr @topal.runtime.int.negate(ptr %numerator)
  %result = call ptr @topal.runtime.rational.raw(ptr %negative, ptr %denominator)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.absolute(ptr %value) nounwind noinline {
entry:
  %numerator = call ptr @topal.runtime.rational.numerator(ptr %value)
  %denominator = call ptr @topal.runtime.rational.denominator(ptr %value)
  %absolute = call ptr @topal.runtime.int.absolute(ptr %numerator)
  %result = call ptr @topal.runtime.rational.raw(ptr %absolute, ptr %denominator)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.add(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.numerator = call ptr @topal.runtime.rational.numerator(ptr %left)
  %left.denominator = call ptr @topal.runtime.rational.denominator(ptr %left)
  %right.numerator = call ptr @topal.runtime.rational.numerator(ptr %right)
  %right.denominator = call ptr @topal.runtime.rational.denominator(ptr %right)
  %left.scaled = call ptr @topal.runtime.int.multiply(ptr %left.numerator, ptr %right.denominator)
  %right.scaled = call ptr @topal.runtime.int.multiply(ptr %right.numerator, ptr %left.denominator)
  %numerator = call ptr @topal.runtime.int.add(ptr %left.scaled, ptr %right.scaled)
  %denominator = call ptr @topal.runtime.int.multiply(ptr %left.denominator, ptr %right.denominator)
  %result = call ptr @topal.runtime.rational.make(ptr %numerator, ptr %denominator)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.subtract(ptr %left, ptr %right) nounwind noinline {
entry:
  %negative = call ptr @topal.runtime.rational.negate(ptr %right)
  %result = call ptr @topal.runtime.rational.add(ptr %left, ptr %negative)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.multiply(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.numerator = call ptr @topal.runtime.rational.numerator(ptr %left)
  %left.denominator = call ptr @topal.runtime.rational.denominator(ptr %left)
  %right.numerator = call ptr @topal.runtime.rational.numerator(ptr %right)
  %right.denominator = call ptr @topal.runtime.rational.denominator(ptr %right)
  %numerator = call ptr @topal.runtime.int.multiply(ptr %left.numerator, ptr %right.numerator)
  %denominator = call ptr @topal.runtime.int.multiply(ptr %left.denominator, ptr %right.denominator)
  %result = call ptr @topal.runtime.rational.make(ptr %numerator, ptr %denominator)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.divide(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.numerator = call ptr @topal.runtime.rational.numerator(ptr %left)
  %left.denominator = call ptr @topal.runtime.rational.denominator(ptr %left)
  %right.numerator = call ptr @topal.runtime.rational.numerator(ptr %right)
  %right.denominator = call ptr @topal.runtime.rational.denominator(ptr %right)
  %zero = call i1 @topal.runtime.int.is.zero(ptr %right.numerator)
  br i1 %zero, label %failure, label %calculate
failure:
  call void @topal.platform.exit(i64 65)
  unreachable
calculate:
  %numerator = call ptr @topal.runtime.int.multiply(ptr %left.numerator, ptr %right.denominator)
  %denominator = call ptr @topal.runtime.int.multiply(ptr %left.denominator, ptr %right.numerator)
  %result = call ptr @topal.runtime.rational.make(ptr %numerator, ptr %denominator)
  ret ptr %result
}

define internal i32 @topal.runtime.rational.compare(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.numerator = call ptr @topal.runtime.rational.numerator(ptr %left)
  %left.denominator = call ptr @topal.runtime.rational.denominator(ptr %left)
  %right.numerator = call ptr @topal.runtime.rational.numerator(ptr %right)
  %right.denominator = call ptr @topal.runtime.rational.denominator(ptr %right)
  %left.scaled = call ptr @topal.runtime.int.multiply(ptr %left.numerator, ptr %right.denominator)
  %right.scaled = call ptr @topal.runtime.int.multiply(ptr %right.numerator, ptr %left.denominator)
  %result = call i32 @topal.runtime.int.compare(ptr %left.scaled, ptr %right.scaled)
  ret i32 %result
}

define internal ptr @topal.runtime.rational.power(ptr %base, ptr %exponent) nounwind noinline {
entry:
  %numerator = call ptr @topal.runtime.rational.numerator(ptr %base)
  %denominator = call ptr @topal.runtime.rational.denominator(ptr %base)
  %exponent.sign.pointer = getelementptr %topal.IntStorage, ptr %exponent, i32 0, i32 0
  %exponent.sign = load i64, ptr %exponent.sign.pointer, align 8
  %negative = icmp ne i64 %exponent.sign, 0
  %absolute.exponent = call ptr @topal.runtime.int.absolute(ptr %exponent)
  %powered.numerator = call ptr @topal.runtime.int.power(ptr %numerator, ptr %absolute.exponent)
  %powered.denominator = call ptr @topal.runtime.int.power(ptr %denominator, ptr %absolute.exponent)
  br i1 %negative, label %reciprocal, label %direct
direct:
  %direct.result = call ptr @topal.runtime.rational.raw(ptr %powered.numerator, ptr %powered.denominator)
  ret ptr %direct.result
reciprocal:
  %zero = call i1 @topal.runtime.int.is.zero(ptr %powered.numerator)
  br i1 %zero, label %failure, label %invert
failure:
  call void @topal.platform.exit(i64 65)
  unreachable
invert:
  %result = call ptr @topal.runtime.rational.make(ptr %powered.denominator, ptr %powered.numerator)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.try.make(ptr %numerator, ptr %denominator, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column) nounwind noinline {
entry:
  %denominator.zero = call i1 @topal.runtime.int.is.zero(ptr %denominator)
  br i1 %denominator.zero, label %failure, label %success
failure:
  %numerator.zero = call i1 @topal.runtime.int.is.zero(ptr %numerator)
  %code = select i1 %numerator.zero, i32 3, i32 2
  %failed = call ptr @topal.runtime.result.failure(i32 %code, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column)
  ret ptr %failed
success:
  %ratio = call ptr @topal.runtime.rational.make(ptr %numerator, ptr %denominator)
  %result = call ptr @topal.runtime.result.success(ptr %ratio)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.try.divide(ptr %left, ptr %right, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column) nounwind noinline {
entry:
  %right.numerator = call ptr @topal.runtime.rational.numerator(ptr %right)
  %right.zero = call i1 @topal.runtime.int.is.zero(ptr %right.numerator)
  br i1 %right.zero, label %failure, label %success
failure:
  %failed = call ptr @topal.runtime.result.failure(i32 2, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column)
  ret ptr %failed
success:
  %ratio = call ptr @topal.runtime.rational.divide(ptr %left, ptr %right)
  %result = call ptr @topal.runtime.result.success(ptr %ratio)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.try.power(ptr %base, ptr %exponent, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column) nounwind noinline {
entry:
  %exponent.sign.pointer = getelementptr %topal.IntStorage, ptr %exponent, i32 0, i32 0
  %exponent.sign = load i64, ptr %exponent.sign.pointer, align 8
  %negative = icmp ne i64 %exponent.sign, 0
  br i1 %negative, label %check.base, label %success
check.base:
  %numerator = call ptr @topal.runtime.rational.numerator(ptr %base)
  %base.zero = call i1 @topal.runtime.int.is.zero(ptr %numerator)
  br i1 %base.zero, label %failure, label %success
failure:
  %failed = call ptr @topal.runtime.result.failure(i32 2, ptr %domain, i64 %domain.length, ptr %source, i64 %source.length, i64 %line, i64 %column)
  ret ptr %failed
success:
  %ratio = call ptr @topal.runtime.rational.power(ptr %base, ptr %exponent)
  %result = call ptr @topal.runtime.result.success(ptr %ratio)
  ret ptr %result
}

define internal void @topal.runtime.rational.print(ptr %value) nounwind noinline {
entry:
  %numerator = call ptr @topal.runtime.rational.numerator(ptr %value)
  %denominator = call ptr @topal.runtime.rational.denominator(ptr %value)
  call void @topal.platform.write_all(ptr @topal.runtime.rational.prefix, i64 11)
  call void @topal.runtime.int.print(ptr %numerator)
  call void @topal.platform.write_all(ptr @topal.runtime.rational.separator, i64 2)
  call void @topal.runtime.int.print(ptr %denominator)
  call void @topal.platform.write_all(ptr @topal.runtime.rational.suffix, i64 2)
  ret void
}

define internal ptr @topal.runtime.range.make(ptr %lower, ptr %upper, i64 %lower.inclusive, i64 %upper.inclusive) nounwind noinline {
entry:
  %value = call ptr @topal.platform.allocate(i64 32)
  %lower.pointer = getelementptr %topal.RangeStorage, ptr %value, i32 0, i32 0
  %upper.pointer = getelementptr %topal.RangeStorage, ptr %value, i32 0, i32 1
  %lower.inclusive.pointer = getelementptr %topal.RangeStorage, ptr %value, i32 0, i32 2
  %upper.inclusive.pointer = getelementptr %topal.RangeStorage, ptr %value, i32 0, i32 3
  store ptr %lower, ptr %lower.pointer, align 8
  store ptr %upper, ptr %upper.pointer, align 8
  store i64 %lower.inclusive, ptr %lower.inclusive.pointer, align 8
  store i64 %upper.inclusive, ptr %upper.inclusive.pointer, align 8
  ret ptr %value
}

define internal ptr @topal.runtime.range.lower(ptr %range) nounwind noinline {
entry:
  %pointer = getelementptr %topal.RangeStorage, ptr %range, i32 0, i32 0
  %value = load ptr, ptr %pointer, align 8
  ret ptr %value
}

define internal ptr @topal.runtime.range.upper(ptr %range) nounwind noinline {
entry:
  %pointer = getelementptr %topal.RangeStorage, ptr %range, i32 0, i32 1
  %value = load ptr, ptr %pointer, align 8
  ret ptr %value
}

define internal i1 @topal.runtime.range.lower.inclusive(ptr %range) nounwind noinline {
entry:
  %pointer = getelementptr %topal.RangeStorage, ptr %range, i32 0, i32 2
  %raw = load i64, ptr %pointer, align 8
  %value = icmp ne i64 %raw, 0
  ret i1 %value
}

define internal i1 @topal.runtime.range.upper.inclusive(ptr %range) nounwind noinline {
entry:
  %pointer = getelementptr %topal.RangeStorage, ptr %range, i32 0, i32 3
  %raw = load i64, ptr %pointer, align 8
  %value = icmp ne i64 %raw, 0
  ret i1 %value
}

define internal i1 @topal.runtime.range.empty.from.comparison(ptr %range, i32 %ordering) nounwind noinline {
entry:
  %reversed = icmp sgt i32 %ordering, 0
  %equal = icmp eq i32 %ordering, 0
  %lower.inclusive = call i1 @topal.runtime.range.lower.inclusive(ptr %range)
  %upper.inclusive = call i1 @topal.runtime.range.upper.inclusive(ptr %range)
  %both.inclusive = and i1 %lower.inclusive, %upper.inclusive
  %open.equal = xor i1 %both.inclusive, true
  %empty.equal = and i1 %equal, %open.equal
  %empty = or i1 %reversed, %empty.equal
  ret i1 %empty
}

define internal i1 @topal.runtime.range.int.empty(ptr %range) nounwind noinline {
entry:
  %lower = call ptr @topal.runtime.range.lower(ptr %range)
  %upper = call ptr @topal.runtime.range.upper(ptr %range)
  %ordering = call i32 @topal.runtime.int.compare(ptr %lower, ptr %upper)
  %empty = call i1 @topal.runtime.range.empty.from.comparison(ptr %range, i32 %ordering)
  ret i1 %empty
}

define internal i1 @topal.runtime.range.rational.empty(ptr %range) nounwind noinline {
entry:
  %lower = call ptr @topal.runtime.range.lower(ptr %range)
  %upper = call ptr @topal.runtime.range.upper(ptr %range)
  %ordering = call i32 @topal.runtime.rational.compare(ptr %lower, ptr %upper)
  %empty = call i1 @topal.runtime.range.empty.from.comparison(ptr %range, i32 %ordering)
  ret i1 %empty
}

define internal i1 @topal.runtime.range.int.contains(ptr %range, ptr %value) nounwind noinline {
entry:
  %lower = call ptr @topal.runtime.range.lower(ptr %range)
  %upper = call ptr @topal.runtime.range.upper(ptr %range)
  %lower.ordering = call i32 @topal.runtime.int.compare(ptr %value, ptr %lower)
  %upper.ordering = call i32 @topal.runtime.int.compare(ptr %value, ptr %upper)
  %lower.inclusive = call i1 @topal.runtime.range.lower.inclusive(ptr %range)
  %upper.inclusive = call i1 @topal.runtime.range.upper.inclusive(ptr %range)
  %above.lower = icmp sgt i32 %lower.ordering, 0
  %at.lower = icmp eq i32 %lower.ordering, 0
  %included.lower = and i1 %at.lower, %lower.inclusive
  %lower.accepted = or i1 %above.lower, %included.lower
  %below.upper = icmp slt i32 %upper.ordering, 0
  %at.upper = icmp eq i32 %upper.ordering, 0
  %included.upper = and i1 %at.upper, %upper.inclusive
  %upper.accepted = or i1 %below.upper, %included.upper
  %accepted = and i1 %lower.accepted, %upper.accepted
  ret i1 %accepted
}

define internal i1 @topal.runtime.range.rational.contains(ptr %range, ptr %value) nounwind noinline {
entry:
  %lower = call ptr @topal.runtime.range.lower(ptr %range)
  %upper = call ptr @topal.runtime.range.upper(ptr %range)
  %lower.ordering = call i32 @topal.runtime.rational.compare(ptr %value, ptr %lower)
  %upper.ordering = call i32 @topal.runtime.rational.compare(ptr %value, ptr %upper)
  %lower.inclusive = call i1 @topal.runtime.range.lower.inclusive(ptr %range)
  %upper.inclusive = call i1 @topal.runtime.range.upper.inclusive(ptr %range)
  %above.lower = icmp sgt i32 %lower.ordering, 0
  %at.lower = icmp eq i32 %lower.ordering, 0
  %included.lower = and i1 %at.lower, %lower.inclusive
  %lower.accepted = or i1 %above.lower, %included.lower
  %below.upper = icmp slt i32 %upper.ordering, 0
  %at.upper = icmp eq i32 %upper.ordering, 0
  %included.upper = and i1 %at.upper, %upper.inclusive
  %upper.accepted = or i1 %below.upper, %included.upper
  %accepted = and i1 %lower.accepted, %upper.accepted
  ret i1 %accepted
}

define internal ptr @topal.runtime.range.intersection.from.orderings(ptr %left, ptr %right, i32 %lower.ordering, i32 %upper.ordering) nounwind noinline {
entry:
  %left.lower = call ptr @topal.runtime.range.lower(ptr %left)
  %left.upper = call ptr @topal.runtime.range.upper(ptr %left)
  %right.lower = call ptr @topal.runtime.range.lower(ptr %right)
  %right.upper = call ptr @topal.runtime.range.upper(ptr %right)
  %left.lower.inclusive = call i1 @topal.runtime.range.lower.inclusive(ptr %left)
  %left.upper.inclusive = call i1 @topal.runtime.range.upper.inclusive(ptr %left)
  %right.lower.inclusive = call i1 @topal.runtime.range.lower.inclusive(ptr %right)
  %right.upper.inclusive = call i1 @topal.runtime.range.upper.inclusive(ptr %right)
  %lower.equal = icmp eq i32 %lower.ordering, 0
  %left.lower.stricter = icmp sgt i32 %lower.ordering, 0
  %selected.lower = select i1 %left.lower.stricter, ptr %left.lower, ptr %right.lower
  %selected.lower.inclusive = select i1 %left.lower.stricter, i1 %left.lower.inclusive, i1 %right.lower.inclusive
  %equal.lower.inclusive = and i1 %left.lower.inclusive, %right.lower.inclusive
  %lower.inclusive = select i1 %lower.equal, i1 %equal.lower.inclusive, i1 %selected.lower.inclusive
  %upper.equal = icmp eq i32 %upper.ordering, 0
  %left.upper.stricter = icmp slt i32 %upper.ordering, 0
  %selected.upper = select i1 %left.upper.stricter, ptr %left.upper, ptr %right.upper
  %selected.upper.inclusive = select i1 %left.upper.stricter, i1 %left.upper.inclusive, i1 %right.upper.inclusive
  %equal.upper.inclusive = and i1 %left.upper.inclusive, %right.upper.inclusive
  %upper.inclusive = select i1 %upper.equal, i1 %equal.upper.inclusive, i1 %selected.upper.inclusive
  %lower.inclusive.raw = zext i1 %lower.inclusive to i64
  %upper.inclusive.raw = zext i1 %upper.inclusive to i64
  %result = call ptr @topal.runtime.range.make(ptr %selected.lower, ptr %selected.upper, i64 %lower.inclusive.raw, i64 %upper.inclusive.raw)
  ret ptr %result
}

define internal ptr @topal.runtime.range.int.intersection(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.lower = call ptr @topal.runtime.range.lower(ptr %left)
  %left.upper = call ptr @topal.runtime.range.upper(ptr %left)
  %right.lower = call ptr @topal.runtime.range.lower(ptr %right)
  %right.upper = call ptr @topal.runtime.range.upper(ptr %right)
  %lower.ordering = call i32 @topal.runtime.int.compare(ptr %left.lower, ptr %right.lower)
  %upper.ordering = call i32 @topal.runtime.int.compare(ptr %left.upper, ptr %right.upper)
  %result = call ptr @topal.runtime.range.intersection.from.orderings(ptr %left, ptr %right, i32 %lower.ordering, i32 %upper.ordering)
  ret ptr %result
}

define internal ptr @topal.runtime.range.rational.intersection(ptr %left, ptr %right) nounwind noinline {
entry:
  %left.lower = call ptr @topal.runtime.range.lower(ptr %left)
  %left.upper = call ptr @topal.runtime.range.upper(ptr %left)
  %right.lower = call ptr @topal.runtime.range.lower(ptr %right)
  %right.upper = call ptr @topal.runtime.range.upper(ptr %right)
  %lower.ordering = call i32 @topal.runtime.rational.compare(ptr %left.lower, ptr %right.lower)
  %upper.ordering = call i32 @topal.runtime.rational.compare(ptr %left.upper, ptr %right.upper)
  %result = call ptr @topal.runtime.range.intersection.from.orderings(ptr %left, ptr %right, i32 %lower.ordering, i32 %upper.ordering)
  ret ptr %result
}

define internal void @topal.runtime.range.print.symbol(ptr %range) nounwind noinline {
entry:
  %lower.inclusive = call i1 @topal.runtime.range.lower.inclusive(ptr %range)
  %upper.inclusive = call i1 @topal.runtime.range.upper.inclusive(ptr %range)
  br i1 %lower.inclusive, label %lower.closed, label %lower.open
lower.closed:
  br i1 %upper.inclusive, label %closed.closed, label %closed.open
lower.open:
  br i1 %upper.inclusive, label %open.closed, label %open.open
closed.closed:
  call void @topal.platform.write_all(ptr @topal.runtime.range.closed.closed, i64 5)
  ret void
closed.open:
  call void @topal.platform.write_all(ptr @topal.runtime.range.closed.open, i64 4)
  ret void
open.closed:
  call void @topal.platform.write_all(ptr @topal.runtime.range.open.closed, i64 6)
  ret void
open.open:
  call void @topal.platform.write_all(ptr @topal.runtime.range.open.open, i64 5)
  ret void
}

define internal void @topal.runtime.range.int.print(ptr %range) nounwind noinline {
entry:
  %lower = call ptr @topal.runtime.range.lower(ptr %range)
  %upper = call ptr @topal.runtime.range.upper(ptr %range)
  call void @topal.runtime.int.print(ptr %lower)
  call void @topal.runtime.range.print.symbol(ptr %range)
  call void @topal.runtime.int.print(ptr %upper)
  ret void
}

define internal void @topal.runtime.range.rational.print(ptr %range) nounwind noinline {
entry:
  %lower = call ptr @topal.runtime.range.lower(ptr %range)
  %upper = call ptr @topal.runtime.range.upper(ptr %range)
  call void @topal.runtime.rational.print(ptr %lower)
  call void @topal.runtime.range.print.symbol(ptr %range)
  call void @topal.runtime.rational.print(ptr %upper)
  ret void
}
