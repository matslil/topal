%topal.ListIntStringPairStorage = type { ptr, ptr, ptr }

define internal i1 @topal.runtime.list.int-string.equal(ptr %left, ptr %right) nounwind noinline {
entry:
  br label %loop
loop:
  %left.current = phi ptr [%left, %entry], [%left.next, %advance]
  %right.current = phi ptr [%right, %entry], [%right.next, %advance]
  %left.empty = icmp eq ptr %left.current, null
  %right.empty = icmp eq ptr %right.current, null
  %either.empty = or i1 %left.empty, %right.empty
  br i1 %either.empty, label %finish, label %compare.int
compare.int:
  %left.int.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %left.current, i32 0, i32 0
  %right.int.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %right.current, i32 0, i32 0
  %left.int = load ptr, ptr %left.int.pointer, align 8
  %right.int = load ptr, ptr %right.int.pointer, align 8
  %int.ordering = call i32 @topal.runtime.int.compare(ptr %left.int, ptr %right.int)
  %int.equal = icmp eq i32 %int.ordering, 0
  br i1 %int.equal, label %compare.string, label %different
compare.string:
  %left.string.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %left.current, i32 0, i32 1
  %right.string.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %right.current, i32 0, i32 1
  %left.string = load ptr, ptr %left.string.pointer, align 8
  %right.string = load ptr, ptr %right.string.pointer, align 8
  %string.equal = call i1 @topal.runtime.string.equal(ptr %left.string, ptr %right.string)
  br i1 %string.equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %left.current, i32 0, i32 2
  %right.next.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %right.current, i32 0, i32 2
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}

define internal ptr @topal.runtime.list.int-string.entry.count(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%incremented, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  %incremented = add i64 %count, 1
  br label %loop
done:
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}
