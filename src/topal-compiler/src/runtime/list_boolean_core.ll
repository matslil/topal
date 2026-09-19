%topal.ListBooleanStorage = type { i1, ptr }

define internal i1 @topal.runtime.list.boolean.equal(ptr %left, ptr %right) nounwind noinline {
entry:
  br label %loop
loop:
  %left.current = phi ptr [%left, %entry], [%left.next, %advance]
  %right.current = phi ptr [%right, %entry], [%right.next, %advance]
  %left.empty = icmp eq ptr %left.current, null
  %right.empty = icmp eq ptr %right.current, null
  %either.empty = or i1 %left.empty, %right.empty
  br i1 %either.empty, label %finish, label %compare
compare:
  %left.value.pointer = getelementptr %topal.ListBooleanStorage, ptr %left.current, i32 0, i32 0
  %right.value.pointer = getelementptr %topal.ListBooleanStorage, ptr %right.current, i32 0, i32 0
  %left.value = load i1, ptr %left.value.pointer, align 1
  %right.value = load i1, ptr %right.value.pointer, align 1
  %equal = icmp eq i1 %left.value, %right.value
  br i1 %equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr %topal.ListBooleanStorage, ptr %left.current, i32 0, i32 1
  %right.next.pointer = getelementptr %topal.ListBooleanStorage, ptr %right.current, i32 0, i32 1
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}

define internal ptr @topal.runtime.list.boolean.entry.count(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%incremented, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr %topal.ListBooleanStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  %incremented = add i64 %count, 1
  br label %loop
done:
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}
