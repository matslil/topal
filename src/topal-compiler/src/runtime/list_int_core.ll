; Basic immutable List Int operations over the compiler-private node layout.

%topal.ListUnconsStorage = type { ptr, ptr }

define internal i64 @topal.runtime.list.int.count.raw(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%count.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  %count.next = add i64 %count, 1
  br label %loop
done:
  ret i64 %count
}

define internal ptr @topal.runtime.list.int.concat(ptr %left, ptr %right) nounwind noinline {
entry:
  %count = call i64 @topal.runtime.list.int.count.raw(ptr %left)
  %empty = icmp eq i64 %count, 0
  br i1 %empty, label %share.right, label %allocate
allocate:
  %allocation.length = shl i64 %count, 4
  %copy = call ptr @topal.platform.allocate(i64 %allocation.length)
  br label %loop
loop:
  %source = phi ptr [%left, %allocate], [%source.next, %advance]
  %index = phi i64 [0, %allocate], [%index.next, %advance]
  %source.value.pointer = getelementptr %topal.ListStorage, ptr %source, i32 0, i32 0
  %source.value = load ptr, ptr %source.value.pointer, align 8
  %source.next.pointer = getelementptr %topal.ListStorage, ptr %source, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %destination.offset = shl i64 %index, 4
  %destination = getelementptr i8, ptr %copy, i64 %destination.offset
  %destination.value.pointer = getelementptr %topal.ListStorage, ptr %destination, i32 0, i32 0
  store ptr %source.value, ptr %destination.value.pointer, align 8
  %index.next = add i64 %index, 1
  %last = icmp eq i64 %index.next, %count
  %destination.next = getelementptr i8, ptr %destination, i64 16
  %destination.remaining = select i1 %last, ptr %right, ptr %destination.next
  %destination.remaining.pointer = getelementptr %topal.ListStorage, ptr %destination, i32 0, i32 1
  store ptr %destination.remaining, ptr %destination.remaining.pointer, align 8
  br i1 %last, label %done, label %advance
advance:
  br label %loop
share.right:
  ret ptr %right
done:
  ret ptr %copy
}

define internal ptr @topal.runtime.list.int.reverse(ptr %list) nounwind noinline {
entry:
  %count = call i64 @topal.runtime.list.int.count.raw(ptr %list)
  %empty = icmp eq i64 %count, 0
  br i1 %empty, label %return.empty, label %allocate
allocate:
  %allocation.length = shl i64 %count, 4
  %copy = call ptr @topal.platform.allocate(i64 %allocation.length)
  %last.index = sub i64 %count, 1
  br label %loop
loop:
  %source = phi ptr [%list, %allocate], [%source.next, %loop]
  %remaining = phi i64 [%count, %allocate], [%destination.index, %loop]
  %destination.index = sub i64 %remaining, 1
  %destination.offset = shl i64 %destination.index, 4
  %destination = getelementptr i8, ptr %copy, i64 %destination.offset
  %source.value.pointer = getelementptr %topal.ListStorage, ptr %source, i32 0, i32 0
  %source.value = load ptr, ptr %source.value.pointer, align 8
  %destination.value.pointer = getelementptr %topal.ListStorage, ptr %destination, i32 0, i32 0
  store ptr %source.value, ptr %destination.value.pointer, align 8
  %tail = icmp eq i64 %destination.index, %last.index
  %destination.next = getelementptr i8, ptr %destination, i64 16
  %destination.remaining = select i1 %tail, ptr null, ptr %destination.next
  %destination.remaining.pointer = getelementptr %topal.ListStorage, ptr %destination, i32 0, i32 1
  store ptr %destination.remaining, ptr %destination.remaining.pointer, align 8
  %source.next.pointer = getelementptr %topal.ListStorage, ptr %source, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %complete = icmp eq ptr %source.next, null
  br i1 %complete, label %done, label %loop
return.empty:
  ret ptr null
done:
  ret ptr %copy
}

define internal i1 @topal.runtime.list.int.equal(ptr %left, ptr %right) nounwind noinline {
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
  %left.value.pointer = getelementptr %topal.ListStorage, ptr %left.current, i32 0, i32 0
  %right.value.pointer = getelementptr %topal.ListStorage, ptr %right.current, i32 0, i32 0
  %left.value = load ptr, ptr %left.value.pointer, align 8
  %right.value = load ptr, ptr %right.value.pointer, align 8
  %ordering = call i32 @topal.runtime.int.compare(ptr %left.value, ptr %right.value)
  %equal = icmp eq i32 %ordering, 0
  br i1 %equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr %topal.ListStorage, ptr %left.current, i32 0, i32 1
  %right.next.pointer = getelementptr %topal.ListStorage, ptr %right.current, i32 0, i32 1
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}

define internal ptr @topal.runtime.list.int.entry.count(ptr %list) nounwind noinline {
entry:
  %count = call i64 @topal.runtime.list.int.count.raw(ptr %list)
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}

define internal ptr @topal.runtime.list.int.first(ptr %list) nounwind noinline {
entry:
  %empty = icmp eq ptr %list, null
  br i1 %empty, label %none, label %some
some:
  %value.pointer = getelementptr %topal.ListStorage, ptr %list, i32 0, i32 0
  %value = load ptr, ptr %value.pointer, align 8
  %present = call ptr @topal.runtime.optional.some(ptr %value)
  ret ptr %present
none:
  %absent = call ptr @topal.runtime.optional.none()
  ret ptr %absent
}

define internal ptr @topal.runtime.list.int.rest(ptr %list) nounwind noinline {
entry:
  %empty = icmp eq ptr %list, null
  br i1 %empty, label %none, label %some
some:
  %rest.pointer = getelementptr %topal.ListStorage, ptr %list, i32 0, i32 1
  %rest = load ptr, ptr %rest.pointer, align 8
  %present = call ptr @topal.runtime.optional.some(ptr %rest)
  ret ptr %present
none:
  %absent = call ptr @topal.runtime.optional.none()
  ret ptr %absent
}

define internal ptr @topal.runtime.list.int.uncons(ptr %list) nounwind noinline {
entry:
  %empty = icmp eq ptr %list, null
  br i1 %empty, label %none, label %some
some:
  %first.pointer = getelementptr %topal.ListStorage, ptr %list, i32 0, i32 0
  %rest.pointer = getelementptr %topal.ListStorage, ptr %list, i32 0, i32 1
  %first = load ptr, ptr %first.pointer, align 8
  %rest = load ptr, ptr %rest.pointer, align 8
  %pair = call ptr @topal.platform.allocate(i64 16)
  %pair.first.pointer = getelementptr %topal.ListUnconsStorage, ptr %pair, i32 0, i32 0
  %pair.rest.pointer = getelementptr %topal.ListUnconsStorage, ptr %pair, i32 0, i32 1
  store ptr %first, ptr %pair.first.pointer, align 8
  store ptr %rest, ptr %pair.rest.pointer, align 8
  %present = call ptr @topal.runtime.optional.some(ptr %pair)
  ret ptr %present
none:
  %absent = call ptr @topal.runtime.optional.none()
  ret ptr %absent
}
