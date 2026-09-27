%topal.ListIntPairStorage = type { ptr, ptr, ptr }

define internal ptr @topal.runtime.list.int.pair.concat(ptr %left, ptr %right) nounwind noinline {
entry:
  br label %count.loop
count.loop:
  %count.source = phi ptr [%left, %entry], [%count.next, %count.advance]
  %count = phi i64 [0, %entry], [%count.incremented, %count.advance]
  %count.empty = icmp eq ptr %count.source, null
  br i1 %count.empty, label %count.done, label %count.advance
count.advance:
  %count.next.pointer = getelementptr i8, ptr %count.source, i64 16
  %count.next = load ptr, ptr %count.next.pointer, align 8
  %count.incremented = add i64 %count, 1
  br label %count.loop
count.done:
  %empty = icmp eq i64 %count, 0
  br i1 %empty, label %share.right, label %allocate
allocate:
  %allocation.length = mul i64 %count, 24
  %copy = call ptr @topal.platform.allocate(i64 %allocation.length)
  br label %loop
loop:
  %source = phi ptr [%left, %allocate], [%source.next, %advance]
  %index = phi i64 [0, %allocate], [%index.next, %advance]
  %source.first = load ptr, ptr %source, align 8
  %source.second.pointer = getelementptr i8, ptr %source, i64 8
  %source.second = load ptr, ptr %source.second.pointer, align 8
  %source.next.pointer = getelementptr i8, ptr %source, i64 16
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %destination.offset = mul i64 %index, 24
  %destination = getelementptr i8, ptr %copy, i64 %destination.offset
  store ptr %source.first, ptr %destination, align 8
  %destination.second = getelementptr i8, ptr %destination, i64 8
  store ptr %source.second, ptr %destination.second, align 8
  %index.next = add i64 %index, 1
  %last = icmp eq i64 %index.next, %count
  %destination.following = getelementptr i8, ptr %destination, i64 24
  %destination.remaining = select i1 %last, ptr %right, ptr %destination.following
  %destination.next = getelementptr i8, ptr %destination, i64 16
  store ptr %destination.remaining, ptr %destination.next, align 8
  br i1 %last, label %done, label %advance
advance:
  br label %loop
share.right:
  ret ptr %right
done:
  ret ptr %copy
}

define internal i1 @topal.runtime.list.int.pair.equal(ptr %left, ptr %right) nounwind noinline {
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
  %left.first.pointer = getelementptr %topal.ListIntPairStorage, ptr %left.current, i32 0, i32 0
  %right.first.pointer = getelementptr %topal.ListIntPairStorage, ptr %right.current, i32 0, i32 0
  %left.first = load ptr, ptr %left.first.pointer, align 8
  %right.first = load ptr, ptr %right.first.pointer, align 8
  %first.ordering = call i32 @topal.runtime.int.compare(ptr %left.first, ptr %right.first)
  %first.equal = icmp eq i32 %first.ordering, 0
  br i1 %first.equal, label %compare.second, label %different
compare.second:
  %left.second.pointer = getelementptr %topal.ListIntPairStorage, ptr %left.current, i32 0, i32 1
  %right.second.pointer = getelementptr %topal.ListIntPairStorage, ptr %right.current, i32 0, i32 1
  %left.second = load ptr, ptr %left.second.pointer, align 8
  %right.second = load ptr, ptr %right.second.pointer, align 8
  %second.ordering = call i32 @topal.runtime.int.compare(ptr %left.second, ptr %right.second)
  %second.equal = icmp eq i32 %second.ordering, 0
  br i1 %second.equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr %topal.ListIntPairStorage, ptr %left.current, i32 0, i32 2
  %right.next.pointer = getelementptr %topal.ListIntPairStorage, ptr %right.current, i32 0, i32 2
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}

define internal ptr @topal.runtime.list.int.pair.entry.count(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%incremented, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr %topal.ListIntPairStorage, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  %incremented = add i64 %count, 1
  br label %loop
done:
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}
