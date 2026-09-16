; Ordered sequence operations over compiler-private immutable List nodes.

%topal.ListPairStorage = type { ptr, ptr, ptr }
%topal.ListSplitStorage = type { ptr, ptr }
%topal.ListEntriesStorage = type { ptr, ptr, i32, i32, ptr }

define internal ptr @topal.runtime.list.int.at(ptr %list, i64 %index) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %position = phi i64 [0, %entry], [%position.next, %advance]
  %found = icmp eq i64 %position, %index
  br i1 %found, label %done, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  %position.next = add i64 %position, 1
  br label %loop
done:
  ret ptr %current
}

define internal ptr @topal.runtime.list.int.copy.prefix(ptr %source, i64 %count, ptr %tail) nounwind noinline {
entry:
  %empty = icmp eq i64 %count, 0
  br i1 %empty, label %return.tail, label %allocate
allocate:
  %allocation.length = shl i64 %count, 4
  %copy = call ptr @topal.platform.allocate(i64 %allocation.length)
  br label %loop
loop:
  %current = phi ptr [%source, %allocate], [%source.next, %advance]
  %index = phi i64 [0, %allocate], [%index.next, %advance]
  %source.value = load ptr, ptr %current, align 8
  %source.next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %destination.offset = shl i64 %index, 4
  %destination = getelementptr i8, ptr %copy, i64 %destination.offset
  store ptr %source.value, ptr %destination, align 8
  %index.next = add i64 %index, 1
  %last = icmp eq i64 %index.next, %count
  %destination.next = getelementptr i8, ptr %destination, i64 8
  %following.node = getelementptr i8, ptr %destination, i64 16
  %following = select i1 %last, ptr %tail, ptr %following.node
  store ptr %following, ptr %destination.next, align 8
  br i1 %last, label %done, label %advance
advance:
  br label %loop
return.tail:
  ret ptr %tail
done:
  ret ptr %copy
}

define internal ptr @topal.runtime.list.int.insert.at(ptr %list, i64 %boundary, ptr %inserted) nounwind noinline {
entry:
  %suffix = call ptr @topal.runtime.list.int.at(ptr %list, i64 %boundary)
  %inserted.copy = call ptr @topal.runtime.list.int.concat(ptr %inserted, ptr %suffix)
  %result = call ptr @topal.runtime.list.int.copy.prefix(ptr %list, i64 %boundary, ptr %inserted.copy)
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.take(ptr %list, i64 %count) nounwind noinline {
entry:
  %result = call ptr @topal.runtime.list.int.copy.prefix(ptr %list, i64 %count, ptr null)
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.drop(ptr %list, i64 %count) nounwind noinline {
entry:
  %result = call ptr @topal.runtime.list.int.at(ptr %list, i64 %count)
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.split.at(ptr %list, i64 %boundary) nounwind noinline {
entry:
  %suffix = call ptr @topal.runtime.list.int.at(ptr %list, i64 %boundary)
  %prefix = call ptr @topal.runtime.list.int.copy.prefix(ptr %list, i64 %boundary, ptr null)
  %result = call ptr @topal.platform.allocate(i64 16)
  %prefix.pointer = getelementptr %topal.ListSplitStorage, ptr %result, i32 0, i32 0
  %suffix.pointer = getelementptr %topal.ListSplitStorage, ptr %result, i32 0, i32 1
  store ptr %prefix, ptr %prefix.pointer, align 8
  store ptr %suffix, ptr %suffix.pointer, align 8
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.remove.index(ptr %list, i64 %index) nounwind noinline {
entry:
  %entry.node = call ptr @topal.runtime.list.int.at(ptr %list, i64 %index)
  %suffix.pointer = getelementptr %topal.ListStorage, ptr %entry.node, i32 0, i32 1
  %suffix = load ptr, ptr %suffix.pointer, align 8
  %result = call ptr @topal.runtime.list.int.copy.prefix(ptr %list, i64 %index, ptr %suffix)
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.remove.index.range(ptr %list, i64 %start, i64 %end) nounwind noinline {
entry:
  %suffix = call ptr @topal.runtime.list.int.at(ptr %list, i64 %end)
  %result = call ptr @topal.runtime.list.int.copy.prefix(ptr %list, i64 %start, ptr %suffix)
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.zip.shortest(ptr %left, ptr %right) nounwind noinline {
entry:
  br label %loop
loop:
  %left.current = phi ptr [%left, %entry], [%left.next, %advance]
  %right.current = phi ptr [%right, %entry], [%right.next, %advance]
  %head = phi ptr [null, %entry], [%head.next, %advance]
  %previous = phi ptr [null, %entry], [%node, %advance]
  %left.empty = icmp eq ptr %left.current, null
  %right.empty = icmp eq ptr %right.current, null
  %done = or i1 %left.empty, %right.empty
  br i1 %done, label %return, label %visit
visit:
  %left.value = load ptr, ptr %left.current, align 8
  %right.value = load ptr, ptr %right.current, align 8
  %left.next.pointer = getelementptr %topal.ListStorage, ptr %left.current, i32 0, i32 1
  %right.next.pointer = getelementptr %topal.ListStorage, ptr %right.current, i32 0, i32 1
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  %node = call ptr @topal.platform.allocate(i64 24)
  %node.left = getelementptr %topal.ListPairStorage, ptr %node, i32 0, i32 0
  %node.right = getelementptr %topal.ListPairStorage, ptr %node, i32 0, i32 1
  %node.next = getelementptr %topal.ListPairStorage, ptr %node, i32 0, i32 2
  store ptr %left.value, ptr %node.left, align 8
  store ptr %right.value, ptr %node.right, align 8
  store ptr null, ptr %node.next, align 8
  %first = icmp eq ptr %previous, null
  br i1 %first, label %start, label %link
start:
  br label %linked
link:
  %previous.next = getelementptr %topal.ListPairStorage, ptr %previous, i32 0, i32 2
  store ptr %node, ptr %previous.next, align 8
  br label %linked
linked:
  %head.linked = phi ptr [%node, %start], [%head, %link]
  br label %advance
advance:
  %head.next = phi ptr [%head.linked, %linked]
  br label %loop
return:
  ret ptr %head
}

define internal ptr @topal.runtime.list.int.zip.exact(ptr %left, ptr %right, ptr %domain, ptr %source, i64 %line, i64 %column) nounwind noinline {
entry:
  %left.count = call i64 @topal.runtime.list.int.count.raw(ptr %left)
  %right.count = call i64 @topal.runtime.list.int.count.raw(ptr %right)
  %equal = icmp eq i64 %left.count, %right.count
  br i1 %equal, label %success, label %failure
success:
  %pairs = call ptr @topal.runtime.list.int.zip.shortest(ptr %left, ptr %right)
  %result = call ptr @topal.runtime.result.success(ptr %pairs)
  ret ptr %result
failure:
  %failed = call ptr @topal.runtime.result.failure(i32 0, ptr %domain, ptr %source, i64 %line, i64 %column)
  ret ptr %failed
}

define internal ptr @topal.runtime.list.int.zip.longest(ptr %left, ptr %left.default, ptr %right, ptr %right.default) nounwind noinline {
entry:
  br label %loop
loop:
  %left.current = phi ptr [%left, %entry], [%left.next, %advance]
  %right.current = phi ptr [%right, %entry], [%right.next, %advance]
  %head = phi ptr [null, %entry], [%head.next, %advance]
  %previous = phi ptr [null, %entry], [%node, %advance]
  %left.empty = icmp eq ptr %left.current, null
  %right.empty = icmp eq ptr %right.current, null
  %done = and i1 %left.empty, %right.empty
  br i1 %done, label %return, label %select.left
select.left:
  br i1 %left.empty, label %left.default.block, label %left.entry
left.default.block:
  br label %left.selected
left.entry:
  %left.entry.value = load ptr, ptr %left.current, align 8
  %left.entry.next.pointer = getelementptr %topal.ListStorage, ptr %left.current, i32 0, i32 1
  %left.entry.next = load ptr, ptr %left.entry.next.pointer, align 8
  br label %left.selected
left.selected:
  %left.value = phi ptr [%left.default, %left.default.block], [%left.entry.value, %left.entry]
  %left.next = phi ptr [null, %left.default.block], [%left.entry.next, %left.entry]
  br i1 %right.empty, label %right.default.block, label %right.entry
right.default.block:
  br label %right.selected
right.entry:
  %right.entry.value = load ptr, ptr %right.current, align 8
  %right.entry.next.pointer = getelementptr %topal.ListStorage, ptr %right.current, i32 0, i32 1
  %right.entry.next = load ptr, ptr %right.entry.next.pointer, align 8
  br label %right.selected
right.selected:
  %right.value = phi ptr [%right.default, %right.default.block], [%right.entry.value, %right.entry]
  %right.next = phi ptr [null, %right.default.block], [%right.entry.next, %right.entry]
  %node = call ptr @topal.platform.allocate(i64 24)
  %node.left = getelementptr %topal.ListPairStorage, ptr %node, i32 0, i32 0
  %node.right = getelementptr %topal.ListPairStorage, ptr %node, i32 0, i32 1
  %node.next = getelementptr %topal.ListPairStorage, ptr %node, i32 0, i32 2
  store ptr %left.value, ptr %node.left, align 8
  store ptr %right.value, ptr %node.right, align 8
  store ptr null, ptr %node.next, align 8
  %first = icmp eq ptr %previous, null
  br i1 %first, label %start, label %link
start:
  br label %linked
link:
  %previous.next = getelementptr %topal.ListPairStorage, ptr %previous, i32 0, i32 2
  store ptr %node, ptr %previous.next, align 8
  br label %linked
linked:
  %head.linked = phi ptr [%node, %start], [%head, %link]
  br label %advance
advance:
  %head.next = phi ptr [%head.linked, %linked]
  br label %loop
return:
  ret ptr %head
}

define internal ptr @topal.runtime.list.int.unzip(ptr %pairs) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%pairs, %entry], [%source.next, %advance]
  %left.head = phi ptr [null, %entry], [%left.head.next, %advance]
  %right.head = phi ptr [null, %entry], [%right.head.next, %advance]
  %left.previous = phi ptr [null, %entry], [%left.node, %advance]
  %right.previous = phi ptr [null, %entry], [%right.node, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit
visit:
  %left.value = load ptr, ptr %current, align 8
  %right.value.pointer = getelementptr %topal.ListPairStorage, ptr %current, i32 0, i32 1
  %source.next.pointer = getelementptr %topal.ListPairStorage, ptr %current, i32 0, i32 2
  %right.value = load ptr, ptr %right.value.pointer, align 8
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %left.node = call ptr @topal.platform.allocate(i64 16)
  %right.node = call ptr @topal.platform.allocate(i64 16)
  store ptr %left.value, ptr %left.node, align 8
  store ptr %right.value, ptr %right.node, align 8
  %left.node.next = getelementptr i8, ptr %left.node, i64 8
  %right.node.next = getelementptr i8, ptr %right.node, i64 8
  store ptr null, ptr %left.node.next, align 8
  store ptr null, ptr %right.node.next, align 8
  %first = icmp eq ptr %left.previous, null
  br i1 %first, label %start, label %link
start:
  br label %linked
link:
  %left.previous.next = getelementptr i8, ptr %left.previous, i64 8
  %right.previous.next = getelementptr i8, ptr %right.previous, i64 8
  store ptr %left.node, ptr %left.previous.next, align 8
  store ptr %right.node, ptr %right.previous.next, align 8
  br label %linked
linked:
  %left.head.linked = phi ptr [%left.node, %start], [%left.head, %link]
  %right.head.linked = phi ptr [%right.node, %start], [%right.head, %link]
  br label %advance
advance:
  %left.head.next = phi ptr [%left.head.linked, %linked]
  %right.head.next = phi ptr [%right.head.linked, %linked]
  br label %loop
done:
  %result = call ptr @topal.platform.allocate(i64 16)
  %result.left = getelementptr %topal.ListSplitStorage, ptr %result, i32 0, i32 0
  %result.right = getelementptr %topal.ListSplitStorage, ptr %result, i32 0, i32 1
  store ptr %left.head, ptr %result.left, align 8
  store ptr %right.head, ptr %result.right, align 8
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.entries(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%source.next, %advance]
  %index = phi i64 [0, %entry], [%index.next, %advance]
  %head = phi ptr [null, %entry], [%head.next, %advance]
  %previous = phi ptr [null, %entry], [%node, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit
visit:
  %value = load ptr, ptr %current, align 8
  %source.next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %index.value = call ptr @topal.runtime.int.from.u64(i64 %index)
  %node = call ptr @topal.platform.allocate(i64 32)
  %node.index = getelementptr %topal.ListEntriesStorage, ptr %node, i32 0, i32 0
  %node.value = getelementptr %topal.ListEntriesStorage, ptr %node, i32 0, i32 1
  %node.order.index = getelementptr %topal.ListEntriesStorage, ptr %node, i32 0, i32 2
  %node.order.value = getelementptr %topal.ListEntriesStorage, ptr %node, i32 0, i32 3
  %node.next = getelementptr %topal.ListEntriesStorage, ptr %node, i32 0, i32 4
  store ptr %index.value, ptr %node.index, align 8
  store ptr %value, ptr %node.value, align 8
  store i32 0, ptr %node.order.index, align 4
  store i32 1, ptr %node.order.value, align 4
  store ptr null, ptr %node.next, align 8
  %first = icmp eq ptr %previous, null
  br i1 %first, label %start, label %link
start:
  br label %linked
link:
  %previous.next = getelementptr %topal.ListEntriesStorage, ptr %previous, i32 0, i32 4
  store ptr %node, ptr %previous.next, align 8
  br label %linked
linked:
  %head.linked = phi ptr [%node, %start], [%head, %link]
  br label %advance
advance:
  %head.next = phi ptr [%head.linked, %linked]
  %index.next = add i64 %index, 1
  br label %loop
done:
  ret ptr %head
}

define internal ptr @topal.runtime.list.string.collect(ptr %list, ptr %empty.string) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %append]
  %result = phi ptr [%empty.string, %entry], [%result.next, %append]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %append
append:
  %value = load ptr, ptr %current, align 8
  %next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  %result.next = call ptr @topal.runtime.string.concat(ptr %result, ptr %value)
  br label %loop
done:
  ret ptr %result
}
