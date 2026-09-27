define internal ptr @topal.runtime.list.int-int-boolean-pair.concat(ptr %left, ptr %right) nounwind noinline {
entry:
  br label %count.loop
count.loop:
  %count.source = phi ptr [%left, %entry], [%count.next, %count.advance]
  %count = phi i64 [0, %entry], [%count.incremented, %count.advance]
  %count.empty = icmp eq ptr %count.source, null
  br i1 %count.empty, label %count.done, label %count.advance
count.advance:
  %count.next.pointer = getelementptr i8, ptr %count.source, i64 24
  %count.next = load ptr, ptr %count.next.pointer, align 8
  %count.incremented = add i64 %count, 1
  br label %count.loop
count.done:
  %empty = icmp eq i64 %count, 0
  br i1 %empty, label %share.right, label %allocate
allocate:
  %allocation.length = mul i64 %count, 32
  %copy = call ptr @topal.platform.allocate(i64 %allocation.length)
  br label %loop
loop:
  %source = phi ptr [%left, %allocate], [%source.next, %advance]
  %index = phi i64 [0, %allocate], [%index.next, %advance]
  %source.first = load ptr, ptr %source, align 8
  %source.second.pointer = getelementptr i8, ptr %source, i64 8
  %source.second = load ptr, ptr %source.second.pointer, align 8
  %source.third.pointer = getelementptr i8, ptr %source, i64 16
  %source.third = load i1, ptr %source.third.pointer, align 1
  %source.fourth.pointer = getelementptr i8, ptr %source, i64 17
  %source.fourth = load i1, ptr %source.fourth.pointer, align 1
  %source.next.pointer = getelementptr i8, ptr %source, i64 24
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %destination.offset = mul i64 %index, 32
  %destination = getelementptr i8, ptr %copy, i64 %destination.offset
  store ptr %source.first, ptr %destination, align 8
  %destination.second = getelementptr i8, ptr %destination, i64 8
  store ptr %source.second, ptr %destination.second, align 8
  %destination.third = getelementptr i8, ptr %destination, i64 16
  store i1 %source.third, ptr %destination.third, align 1
  %destination.fourth = getelementptr i8, ptr %destination, i64 17
  store i1 %source.fourth, ptr %destination.fourth, align 1
  %index.next = add i64 %index, 1
  %last = icmp eq i64 %index.next, %count
  %destination.following = getelementptr i8, ptr %destination, i64 32
  %destination.remaining = select i1 %last, ptr %right, ptr %destination.following
  %destination.next = getelementptr i8, ptr %destination, i64 24
  store ptr %destination.remaining, ptr %destination.next, align 8
  br i1 %last, label %done, label %advance
advance:
  br label %loop
share.right:
  ret ptr %right
done:
  ret ptr %copy
}

define internal i1 @topal.runtime.list.int-int-boolean-pair.equal(ptr %left, ptr %right) nounwind noinline {
entry:
  br label %loop
loop:
  %left.current = phi ptr [%left, %entry], [%left.next, %advance]
  %right.current = phi ptr [%right, %entry], [%right.next, %advance]
  %left.empty = icmp eq ptr %left.current, null
  %right.empty = icmp eq ptr %right.current, null
  %either.empty = or i1 %left.empty, %right.empty
  br i1 %either.empty, label %finish, label %compare.first
compare.first:
  %left.first = load ptr, ptr %left.current, align 8
  %right.first = load ptr, ptr %right.current, align 8
  %first.ordering = call i32 @topal.runtime.int.compare(ptr %left.first, ptr %right.first)
  %first.equal = icmp eq i32 %first.ordering, 0
  br i1 %first.equal, label %compare.second, label %different
compare.second:
  %left.second.pointer = getelementptr i8, ptr %left.current, i64 8
  %right.second.pointer = getelementptr i8, ptr %right.current, i64 8
  %left.second = load ptr, ptr %left.second.pointer, align 8
  %right.second = load ptr, ptr %right.second.pointer, align 8
  %second.ordering = call i32 @topal.runtime.int.compare(ptr %left.second, ptr %right.second)
  %second.equal = icmp eq i32 %second.ordering, 0
  br i1 %second.equal, label %compare.third, label %different
compare.third:
  %left.third.pointer = getelementptr i8, ptr %left.current, i64 16
  %right.third.pointer = getelementptr i8, ptr %right.current, i64 16
  %left.third = load i1, ptr %left.third.pointer, align 1
  %right.third = load i1, ptr %right.third.pointer, align 1
  %third.equal = icmp eq i1 %left.third, %right.third
  br i1 %third.equal, label %compare.fourth, label %different
compare.fourth:
  %left.fourth.pointer = getelementptr i8, ptr %left.current, i64 17
  %right.fourth.pointer = getelementptr i8, ptr %right.current, i64 17
  %left.fourth = load i1, ptr %left.fourth.pointer, align 1
  %right.fourth = load i1, ptr %right.fourth.pointer, align 1
  %fourth.equal = icmp eq i1 %left.fourth, %right.fourth
  br i1 %fourth.equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr i8, ptr %left.current, i64 24
  %right.next.pointer = getelementptr i8, ptr %right.current, i64 24
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}

define internal ptr @topal.runtime.list.int-int-boolean-pair.entry.count(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%incremented, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr i8, ptr %current, i64 24
  %next = load ptr, ptr %next.pointer, align 8
  %incremented = add i64 %count, 1
  br label %loop
done:
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}
