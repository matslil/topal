%topal.ListIntPairStorage = type { ptr, ptr, ptr }

define internal i1 @topal.runtime.list.int.pair.contains.entry(ptr %list, ptr %first, ptr %second) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %missing, label %inspect
inspect:
  %entry.first = load ptr, ptr %current, align 8
  %entry.second.pointer = getelementptr %topal.ListIntPairStorage, ptr %current, i32 0, i32 1
  %entry.second = load ptr, ptr %entry.second.pointer, align 8
  %first.ordering = call i32 @topal.runtime.int.compare(ptr %entry.first, ptr %first)
  %second.ordering = call i32 @topal.runtime.int.compare(ptr %entry.second, ptr %second)
  %first.equal = icmp eq i32 %first.ordering, 0
  %second.equal = icmp eq i32 %second.ordering, 0
  %equal = and i1 %first.equal, %second.equal
  br i1 %equal, label %found, label %advance
advance:
  %next.pointer = getelementptr %topal.ListIntPairStorage, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  br label %loop
found:
  ret i1 true
missing:
  ret i1 false
}

define internal i1 @topal.runtime.list.int.pair.starts.with(ptr %source, ptr %pattern) nounwind noinline {
entry:
  br label %loop
loop:
  %source.current = phi ptr [%source, %entry], [%source.next, %advance]
  %pattern.current = phi ptr [%pattern, %entry], [%pattern.next, %advance]
  %pattern.empty = icmp eq ptr %pattern.current, null
  br i1 %pattern.empty, label %matches, label %check.source
check.source:
  %source.empty = icmp eq ptr %source.current, null
  br i1 %source.empty, label %different, label %compare
compare:
  %source.first = load ptr, ptr %source.current, align 8
  %pattern.first = load ptr, ptr %pattern.current, align 8
  %source.second.pointer = getelementptr %topal.ListIntPairStorage, ptr %source.current, i32 0, i32 1
  %pattern.second.pointer = getelementptr %topal.ListIntPairStorage, ptr %pattern.current, i32 0, i32 1
  %source.second = load ptr, ptr %source.second.pointer, align 8
  %pattern.second = load ptr, ptr %pattern.second.pointer, align 8
  %first.ordering = call i32 @topal.runtime.int.compare(ptr %source.first, ptr %pattern.first)
  %second.ordering = call i32 @topal.runtime.int.compare(ptr %source.second, ptr %pattern.second)
  %first.equal = icmp eq i32 %first.ordering, 0
  %second.equal = icmp eq i32 %second.ordering, 0
  %equal = and i1 %first.equal, %second.equal
  br i1 %equal, label %advance, label %different
advance:
  %source.next.pointer = getelementptr %topal.ListIntPairStorage, ptr %source.current, i32 0, i32 2
  %pattern.next.pointer = getelementptr %topal.ListIntPairStorage, ptr %pattern.current, i32 0, i32 2
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %pattern.next = load ptr, ptr %pattern.next.pointer, align 8
  br label %loop
matches:
  ret i1 true
different:
  ret i1 false
}

define internal i1 @topal.runtime.list.int.pair.contains.sequence(ptr %list, ptr %pattern) nounwind noinline {
entry:
  %pattern.empty = icmp eq ptr %pattern, null
  br i1 %pattern.empty, label %found, label %search
search:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %missing, label %inspect
inspect:
  %matches = call i1 @topal.runtime.list.int.pair.starts.with(ptr %current, ptr %pattern)
  br i1 %matches, label %found, label %advance
advance:
  %next.pointer = getelementptr %topal.ListIntPairStorage, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  br label %search
found:
  ret i1 true
missing:
  ret i1 false
}

define internal ptr @topal.runtime.list.int.pair.first(ptr %list) nounwind noinline {
entry:
  %empty = icmp eq ptr %list, null
  br i1 %empty, label %none, label %some
some:
  %payload = call ptr @topal.platform.allocate(i64 16)
  %first = load ptr, ptr %list, align 8
  store ptr %first, ptr %payload, align 8
  %source.second = getelementptr %topal.ListIntPairStorage, ptr %list, i32 0, i32 1
  %second = load ptr, ptr %source.second, align 8
  %payload.second = getelementptr i8, ptr %payload, i64 8
  store ptr %second, ptr %payload.second, align 8
  %present = call ptr @topal.runtime.optional.some(ptr %payload)
  ret ptr %present
none:
  %absent = call ptr @topal.runtime.optional.none()
  ret ptr %absent
}

define internal ptr @topal.runtime.list.int.pair.select.index.range(ptr %source, ptr %range) nounwind noinline {
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
  %second.pointer = getelementptr %topal.ListIntPairStorage, ptr %current, i32 0, i32 1
  %second = load ptr, ptr %second.pointer, align 8
  %next.pointer = getelementptr %topal.ListIntPairStorage, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  %index.value = call ptr @topal.runtime.int.from.u64(i64 %index)
  %keep = call i1 @topal.runtime.range.int.contains(ptr %range, ptr %index.value)
  br i1 %keep, label %selected, label %skipped
skipped:
  br label %advance
selected:
  %node = call ptr @topal.platform.allocate(i64 24)
  store ptr %first, ptr %node, align 8
  %node.second = getelementptr %topal.ListIntPairStorage, ptr %node, i32 0, i32 1
  store ptr %second, ptr %node.second, align 8
  %node.next = getelementptr %topal.ListIntPairStorage, ptr %node, i32 0, i32 2
  store ptr null, ptr %node.next, align 8
  %has.previous = icmp ne ptr %previous, null
  br i1 %has.previous, label %link, label %first.node
first.node:
  br label %selected.merge
link:
  %previous.next = getelementptr %topal.ListIntPairStorage, ptr %previous, i32 0, i32 2
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
