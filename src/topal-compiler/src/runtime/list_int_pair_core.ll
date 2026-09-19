%topal.ListIntPairStorage = type { ptr, ptr, ptr }

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
