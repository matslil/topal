%topal.ListStringIntPairStorage = type { ptr, ptr, ptr }

define internal i1 @topal.runtime.list.string-int.equal(ptr %left, ptr %right) nounwind noinline {
entry:
  br label %loop
loop:
  %left.current = phi ptr [%left, %entry], [%left.next, %advance]
  %right.current = phi ptr [%right, %entry], [%right.next, %advance]
  %left.empty = icmp eq ptr %left.current, null
  %right.empty = icmp eq ptr %right.current, null
  %either.empty = or i1 %left.empty, %right.empty
  br i1 %either.empty, label %finish, label %compare.string
compare.string:
  %left.string.pointer = getelementptr %topal.ListStringIntPairStorage, ptr %left.current, i32 0, i32 0
  %right.string.pointer = getelementptr %topal.ListStringIntPairStorage, ptr %right.current, i32 0, i32 0
  %left.string = load ptr, ptr %left.string.pointer, align 8
  %right.string = load ptr, ptr %right.string.pointer, align 8
  %string.equal = call i1 @topal.runtime.string.equal(ptr %left.string, ptr %right.string)
  br i1 %string.equal, label %compare.int, label %different
compare.int:
  %left.int.pointer = getelementptr %topal.ListStringIntPairStorage, ptr %left.current, i32 0, i32 1
  %right.int.pointer = getelementptr %topal.ListStringIntPairStorage, ptr %right.current, i32 0, i32 1
  %left.int = load ptr, ptr %left.int.pointer, align 8
  %right.int = load ptr, ptr %right.int.pointer, align 8
  %int.ordering = call i32 @topal.runtime.int.compare(ptr %left.int, ptr %right.int)
  %int.equal = icmp eq i32 %int.ordering, 0
  br i1 %int.equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr %topal.ListStringIntPairStorage, ptr %left.current, i32 0, i32 2
  %right.next.pointer = getelementptr %topal.ListStringIntPairStorage, ptr %right.current, i32 0, i32 2
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}

define internal ptr @topal.runtime.list.string-int.entry.count(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%incremented, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStringIntPairStorage, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  %incremented = add i64 %count, 1
  br label %loop
done:
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}

define internal ptr @topal.runtime.list.string-int.cartesian(ptr %left, ptr %right) nounwind noinline {
entry:
  br label %outer.loop
outer.loop:
  %left.current = phi ptr [%left, %entry], [%left.next, %outer.advance]
  %outer.head = phi ptr [null, %entry], [%inner.head.done, %outer.advance]
  %outer.previous = phi ptr [null, %entry], [%inner.previous.done, %outer.advance]
  %left.empty = icmp eq ptr %left.current, null
  br i1 %left.empty, label %done, label %outer.value
outer.value:
  %left.value = load ptr, ptr %left.current, align 8
  br label %inner.loop
inner.loop:
  %right.current = phi ptr [%right, %outer.value], [%right.next, %inner.advance]
  %inner.head = phi ptr [%outer.head, %outer.value], [%next.head, %inner.advance]
  %inner.previous = phi ptr [%outer.previous, %outer.value], [%node, %inner.advance]
  %right.empty = icmp eq ptr %right.current, null
  br i1 %right.empty, label %outer.advance, label %inner.value
inner.value:
  %right.value = load ptr, ptr %right.current, align 8
  %node = call ptr @topal.platform.allocate(i64 24)
  store ptr %left.value, ptr %node, align 8
  %node.int.pointer = getelementptr i8, ptr %node, i64 8
  store ptr %right.value, ptr %node.int.pointer, align 8
  %node.next.pointer = getelementptr i8, ptr %node, i64 16
  store ptr null, ptr %node.next.pointer, align 8
  %has.previous = icmp ne ptr %inner.previous, null
  br i1 %has.previous, label %link, label %first
first:
  br label %linked
link:
  %previous.next.pointer = getelementptr i8, ptr %inner.previous, i64 16
  store ptr %node, ptr %previous.next.pointer, align 8
  br label %linked
linked:
  %next.head = phi ptr [%node, %first], [%inner.head, %link]
  br label %inner.advance
inner.advance:
  %right.next.pointer = getelementptr i8, ptr %right.current, i64 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %inner.loop
outer.advance:
  %inner.head.done = phi ptr [%inner.head, %inner.loop]
  %inner.previous.done = phi ptr [%inner.previous, %inner.loop]
  %left.next.pointer = getelementptr i8, ptr %left.current, i64 8
  %left.next = load ptr, ptr %left.next.pointer, align 8
  br label %outer.loop
done:
  ret ptr %outer.head
}
