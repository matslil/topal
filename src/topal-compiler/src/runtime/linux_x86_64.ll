; topal.platform.linux-x86_64/1
; Freestanding Linux services and the private topal-native/2 Int runtime.
; Int values are immutable sign-and-magnitude objects with little-endian
; base-2^32 limbs. A zero has sign = 0 and length = 0.

%topal.IntStorage = type { i64, i64, [0 x i32] }

@topal.runtime.int.zero = private constant { i64, i64, [0 x i32] } { i64 0, i64 0, [0 x i32] zeroinitializer }, align 8
@topal.runtime.byte.zero = private constant [1 x i8] c"0", align 1
@topal.runtime.byte.minus = private constant [1 x i8] c"-", align 1

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
