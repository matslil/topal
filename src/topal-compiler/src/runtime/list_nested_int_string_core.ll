; Exact core operations for List (List (Int, String)). The outer node is two
; pointers; each inner pair node contains Int, String, and remaining pointers.

define internal i64 @topal.runtime.list.nested.int-string.count.raw(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%count.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr i8, ptr %current, i64 8
  %next = load ptr, ptr %next.pointer, align 8
  %count.next = add i64 %count, 1
  br label %loop
done:
  ret i64 %count
}

define internal ptr @topal.runtime.list.nested.int-string.entry.count(ptr %list) nounwind noinline {
entry:
  %count = call i64 @topal.runtime.list.nested.int-string.count.raw(ptr %list)
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}

define internal ptr @topal.runtime.list.nested.int-string.first(ptr %list) nounwind noinline {
entry:
  %empty = icmp eq ptr %list, null
  br i1 %empty, label %none, label %some
some:
  %value = load ptr, ptr %list, align 8
  %present = call ptr @topal.runtime.optional.some(ptr %value)
  ret ptr %present
none:
  %absent = call ptr @topal.runtime.optional.none()
  ret ptr %absent
}

define internal i1 @topal.runtime.list.int-string.equal(ptr %left, ptr %right) nounwind noinline {
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
  %left.int = load ptr, ptr %left.current, align 8
  %right.int = load ptr, ptr %right.current, align 8
  %int.ordering = call i32 @topal.runtime.int.compare(ptr %left.int, ptr %right.int)
  %int.equal = icmp eq i32 %int.ordering, 0
  br i1 %int.equal, label %compare.string, label %different
compare.string:
  %left.string.pointer = getelementptr i8, ptr %left.current, i64 8
  %right.string.pointer = getelementptr i8, ptr %right.current, i64 8
  %left.string = load ptr, ptr %left.string.pointer, align 8
  %right.string = load ptr, ptr %right.string.pointer, align 8
  %string.equal = call i1 @topal.runtime.string.equal(ptr %left.string, ptr %right.string)
  br i1 %string.equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr i8, ptr %left.current, i64 16
  %right.next.pointer = getelementptr i8, ptr %right.current, i64 16
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}

define internal i1 @topal.runtime.list.nested.int-string.equal(ptr %left, ptr %right) nounwind noinline {
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
  %left.value = load ptr, ptr %left.current, align 8
  %right.value = load ptr, ptr %right.current, align 8
  %equal = call i1 @topal.runtime.list.int-string.equal(ptr %left.value, ptr %right.value)
  br i1 %equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr i8, ptr %left.current, i64 8
  %right.next.pointer = getelementptr i8, ptr %right.current, i64 8
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}
