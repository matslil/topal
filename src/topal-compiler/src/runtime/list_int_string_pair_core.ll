%topal.ListIntStringPairStorage = type { ptr, ptr, ptr }

define internal ptr @topal.runtime.list.int-string.first(ptr %list) nounwind noinline {
entry:
  %empty = icmp eq ptr %list, null
  br i1 %empty, label %none, label %some
some:
  %payload = call ptr @topal.platform.allocate(i64 16)
  %first = load ptr, ptr %list, align 8
  store ptr %first, ptr %payload, align 8
  %source.second = getelementptr %topal.ListIntStringPairStorage, ptr %list, i32 0, i32 1
  %second = load ptr, ptr %source.second, align 8
  %payload.second = getelementptr i8, ptr %payload, i64 8
  store ptr %second, ptr %payload.second, align 8
  %present = call ptr @topal.runtime.optional.some(ptr %payload)
  ret ptr %present
none:
  %absent = call ptr @topal.runtime.optional.none()
  ret ptr %absent
}

define internal ptr @topal.runtime.list.int-string.select.index.range(ptr %source, ptr %range) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%source, %entry], [%next, %advance]
  %head = phi ptr [null, %entry], [%next.head, %advance]
  %previous = phi ptr [null, %entry], [%next.previous, %advance]
  %index = phi i64 [0, %entry], [%next.index, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit
visit:
  %first = load ptr, ptr %current, align 8
  %second.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %current, i32 0, i32 1
  %second = load ptr, ptr %second.pointer, align 8
  %next.pointer = getelementptr %topal.ListIntStringPairStorage, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  %index.value = call ptr @topal.runtime.int.from.u64(i64 %index)
  %keep = call i1 @topal.runtime.range.int.contains(ptr %range, ptr %index.value)
  br i1 %keep, label %selected, label %skipped
skipped:
  br label %advance
selected:
  %node = call ptr @topal.platform.allocate(i64 24)
  store ptr %first, ptr %node, align 8
  %node.second = getelementptr %topal.ListIntStringPairStorage, ptr %node, i32 0, i32 1
  store ptr %second, ptr %node.second, align 8
  %node.next = getelementptr %topal.ListIntStringPairStorage, ptr %node, i32 0, i32 2
  store ptr null, ptr %node.next, align 8
  %has.previous = icmp ne ptr %previous, null
  br i1 %has.previous, label %link, label %first.node
first.node:
  br label %selected.merge
link:
  %previous.next = getelementptr %topal.ListIntStringPairStorage, ptr %previous, i32 0, i32 2
  store ptr %node, ptr %previous.next, align 8
  br label %selected.merge
selected.merge:
  %selected.head = phi ptr [%node, %first.node], [%head, %link]
  br label %advance
advance:
  %next.head = phi ptr [%head, %skipped], [%selected.head, %selected.merge]
  %next.previous = phi ptr [%previous, %skipped], [%node, %selected.merge]
  %next.index = add i64 %index, 1
  br label %loop
done:
  ret ptr %head
}

define internal ptr @topal.runtime.list.int-string.concat(ptr %left, ptr %right) nounwind noinline {
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
define internal ptr @topal.runtime.list.int-string.reverse(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %reversed = phi ptr [null, %entry], [%node, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %first = load ptr, ptr %current, align 8
  %second.pointer = getelementptr i8, ptr %current, i64 8
  %second = load ptr, ptr %second.pointer, align 8
  %next.pointer = getelementptr i8, ptr %current, i64 16
  %next = load ptr, ptr %next.pointer, align 8
  %node = call ptr @topal.platform.allocate(i64 24)
  store ptr %first, ptr %node, align 8
  %node.second = getelementptr i8, ptr %node, i64 8
  store ptr %second, ptr %node.second, align 8
  %node.next = getelementptr i8, ptr %node, i64 16
  store ptr %reversed, ptr %node.next, align 8
  br label %loop
done:
  ret ptr %reversed
}
