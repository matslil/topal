define internal ptr @topal.runtime.int.try.multiply.infinity(ptr %left, ptr %right, ptr %domain, ptr %source, i64 %line, i64 %column) nounwind noinline {
entry:
  %left.sign.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 0
  %right.sign.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 0
  %left.sign = load i64, ptr %left.sign.pointer, align 8
  %right.sign = load i64, ptr %right.sign.pointer, align 8
  %left.positive.infinity = icmp eq i64 %left.sign, 2
  %left.negative.infinity = icmp eq i64 %left.sign, 3
  %left.infinity = or i1 %left.positive.infinity, %left.negative.infinity
  %right.positive.infinity = icmp eq i64 %right.sign, 2
  %right.negative.infinity = icmp eq i64 %right.sign, 3
  %right.infinity = or i1 %right.positive.infinity, %right.negative.infinity
  %exactly.one.infinity = xor i1 %left.infinity, %right.infinity
  %left.length.pointer = getelementptr %topal.IntStorage, ptr %left, i32 0, i32 1
  %right.length.pointer = getelementptr %topal.IntStorage, ptr %right, i32 0, i32 1
  %left.length = load i64, ptr %left.length.pointer, align 8
  %right.length = load i64, ptr %right.length.pointer, align 8
  %left.zero.length = icmp eq i64 %left.length, 0
  %right.zero.length = icmp eq i64 %right.length, 0
  %left.not.infinity = xor i1 %left.infinity, true
  %right.not.infinity = xor i1 %right.infinity, true
  %left.valid = or i1 %left.not.infinity, %left.zero.length
  %right.valid = or i1 %right.not.infinity, %right.zero.length
  %sentinels.valid = and i1 %left.valid, %right.valid
  %valid = and i1 %exactly.one.infinity, %sentinels.valid
  br i1 %valid, label %validate.zero, label %invalid
invalid:
  call void @topal.platform.exit(i64 70)
  unreachable
validate.zero:
  %left.zero = call i1 @topal.runtime.int.is.zero(ptr %left)
  %right.zero = call i1 @topal.runtime.int.is.zero(ptr %right)
  %either.zero = or i1 %left.zero, %right.zero
  br i1 %either.zero, label %failure, label %success
failure:
  %failed = call ptr @topal.runtime.result.failure(i32 3, ptr %domain, ptr %source, i64 %line, i64 %column)
  ret ptr %failed
success:
  %product = call ptr @topal.runtime.int.multiply(ptr %left, ptr %right)
  %result = call ptr @topal.runtime.result.success(ptr %product)
  ret ptr %result
}

define internal ptr @topal.runtime.rational.try.multiply.infinity(ptr %left, ptr %right, ptr %domain, ptr %source, i64 %line, i64 %column) nounwind noinline {
entry:
  %left.numerator = call ptr @topal.runtime.rational.numerator(ptr %left)
  %right.numerator = call ptr @topal.runtime.rational.numerator(ptr %right)
  %left.denominator = call ptr @topal.runtime.rational.denominator(ptr %left)
  %right.denominator = call ptr @topal.runtime.rational.denominator(ptr %right)
  %left.sign.pointer = getelementptr %topal.IntStorage, ptr %left.numerator, i32 0, i32 0
  %right.sign.pointer = getelementptr %topal.IntStorage, ptr %right.numerator, i32 0, i32 0
  %left.sign = load i64, ptr %left.sign.pointer, align 8
  %right.sign = load i64, ptr %right.sign.pointer, align 8
  %left.positive.infinity = icmp eq i64 %left.sign, 2
  %left.negative.infinity = icmp eq i64 %left.sign, 3
  %left.infinity = or i1 %left.positive.infinity, %left.negative.infinity
  %right.positive.infinity = icmp eq i64 %right.sign, 2
  %right.negative.infinity = icmp eq i64 %right.sign, 3
  %right.infinity = or i1 %right.positive.infinity, %right.negative.infinity
  %exactly.one.infinity = xor i1 %left.infinity, %right.infinity
  %left.length.pointer = getelementptr %topal.IntStorage, ptr %left.numerator, i32 0, i32 1
  %right.length.pointer = getelementptr %topal.IntStorage, ptr %right.numerator, i32 0, i32 1
  %left.length = load i64, ptr %left.length.pointer, align 8
  %right.length = load i64, ptr %right.length.pointer, align 8
  %left.zero.length = icmp eq i64 %left.length, 0
  %right.zero.length = icmp eq i64 %right.length, 0
  %left.denominator.one = icmp eq ptr %left.denominator, @topal.runtime.int.one
  %right.denominator.one = icmp eq ptr %right.denominator, @topal.runtime.int.one
  %left.wrapper.valid = and i1 %left.zero.length, %left.denominator.one
  %right.wrapper.valid = and i1 %right.zero.length, %right.denominator.one
  %left.not.infinity = xor i1 %left.infinity, true
  %right.not.infinity = xor i1 %right.infinity, true
  %left.valid = or i1 %left.not.infinity, %left.wrapper.valid
  %right.valid = or i1 %right.not.infinity, %right.wrapper.valid
  %wrappers.valid = and i1 %left.valid, %right.valid
  %valid = and i1 %exactly.one.infinity, %wrappers.valid
  br i1 %valid, label %validate.zero, label %invalid
invalid:
  call void @topal.platform.exit(i64 70)
  unreachable
validate.zero:
  %left.zero = call i1 @topal.runtime.int.is.zero(ptr %left.numerator)
  %right.zero = call i1 @topal.runtime.int.is.zero(ptr %right.numerator)
  %either.zero = or i1 %left.zero, %right.zero
  br i1 %either.zero, label %failure, label %success
failure:
  %failed = call ptr @topal.runtime.result.failure(i32 3, ptr %domain, ptr %source, i64 %line, i64 %column)
  ret ptr %failed
success:
  %product = call ptr @topal.runtime.rational.multiply(ptr %left, ptr %right)
  %result = call ptr @topal.runtime.result.success(ptr %product)
  ret ptr %result
}
