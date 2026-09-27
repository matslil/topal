%topal.ListIntIntStringIntStorage = type { ptr, ptr, ptr, ptr, ptr }

define internal ptr @topal.runtime.list.int-int-string-int.concat(ptr %left, ptr %right) nounwind noinline {
entry:
  %empty = icmp eq ptr %left, null
  br i1 %empty, label %share.right, label %copy
copy:
  %first = load ptr, ptr %left, align 8
  %second.pointer = getelementptr %topal.ListIntIntStringIntStorage, ptr %left, i32 0, i32 1
  %second = load ptr, ptr %second.pointer, align 8
  %third.pointer = getelementptr %topal.ListIntIntStringIntStorage, ptr %left, i32 0, i32 2
  %third = load ptr, ptr %third.pointer, align 8
  %fourth.pointer = getelementptr %topal.ListIntIntStringIntStorage, ptr %left, i32 0, i32 3
  %fourth = load ptr, ptr %fourth.pointer, align 8
  %next.pointer = getelementptr %topal.ListIntIntStringIntStorage, ptr %left, i32 0, i32 4
  %next = load ptr, ptr %next.pointer, align 8
  %remaining = call ptr @topal.runtime.list.int-int-string-int.concat(ptr %next, ptr %right)
  %node = call ptr @topal.platform.allocate(i64 40)
  store ptr %first, ptr %node, align 8
  %node.second = getelementptr i8, ptr %node, i64 8
  store ptr %second, ptr %node.second, align 8
  %node.third = getelementptr i8, ptr %node, i64 16
  store ptr %third, ptr %node.third, align 8
  %node.fourth = getelementptr i8, ptr %node, i64 24
  store ptr %fourth, ptr %node.fourth, align 8
  %node.next = getelementptr i8, ptr %node, i64 32
  store ptr %remaining, ptr %node.next, align 8
  ret ptr %node
share.right:
  ret ptr %right
}

define internal ptr @topal.runtime.list.int-int-string-int.select.index.range(ptr %source, ptr %range) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%source, %entry], [%next, %advance]
  %head = phi ptr [null, %entry], [%next.head, %advance]
  %tail = phi ptr [null, %entry], [%next.tail, %advance]
  %index = phi i64 [0, %entry], [%next.index, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit
visit:
  %next.pointer = getelementptr %topal.ListIntIntStringIntStorage, ptr %current, i32 0, i32 4
  %next = load ptr, ptr %next.pointer, align 8
  %index.value = call ptr @topal.runtime.int.from.u64(i64 %index)
  %keep = call i1 @topal.runtime.range.int.contains(ptr %range, ptr %index.value)
  br i1 %keep, label %selected, label %skipped
selected:
  %first = load ptr, ptr %current, align 8
  %second.pointer = getelementptr i8, ptr %current, i64 8
  %second = load ptr, ptr %second.pointer, align 8
  %third.pointer = getelementptr i8, ptr %current, i64 16
  %third = load ptr, ptr %third.pointer, align 8
  %fourth.pointer = getelementptr i8, ptr %current, i64 24
  %fourth = load ptr, ptr %fourth.pointer, align 8
  %node = call ptr @topal.platform.allocate(i64 40)
  store ptr %first, ptr %node, align 8
  %node.second = getelementptr i8, ptr %node, i64 8
  store ptr %second, ptr %node.second, align 8
  %node.third = getelementptr i8, ptr %node, i64 16
  store ptr %third, ptr %node.third, align 8
  %node.fourth = getelementptr i8, ptr %node, i64 24
  store ptr %fourth, ptr %node.fourth, align 8
  %node.next = getelementptr i8, ptr %node, i64 32
  store ptr null, ptr %node.next, align 8
  %has.tail = icmp ne ptr %tail, null
  br i1 %has.tail, label %link, label %first.node
link:
  %tail.next = getelementptr i8, ptr %tail, i64 32
  store ptr %node, ptr %tail.next, align 8
  br label %selected.merge
first.node:
  br label %selected.merge
selected.merge:
  %selected.head = phi ptr [%head, %link], [%node, %first.node]
  br label %advance
skipped:
  br label %advance
advance:
  %next.head = phi ptr [%selected.head, %selected.merge], [%head, %skipped]
  %next.tail = phi ptr [%node, %selected.merge], [%tail, %skipped]
  %next.index = add i64 %index, 1
  br label %loop
done:
  ret ptr %head
}

define internal ptr @topal.runtime.list.int-int-string-int.first(ptr %list) nounwind noinline {
entry:
  %empty = icmp eq ptr %list, null
  br i1 %empty, label %none, label %some
some:
  %payload = call ptr @topal.platform.allocate(i64 32)
  %first = load ptr, ptr %list, align 8
  store ptr %first, ptr %payload, align 8
  %second.pointer = getelementptr i8, ptr %list, i64 8
  %second = load ptr, ptr %second.pointer, align 8
  %payload.second = getelementptr i8, ptr %payload, i64 8
  store ptr %second, ptr %payload.second, align 8
  %third.pointer = getelementptr i8, ptr %list, i64 16
  %third = load ptr, ptr %third.pointer, align 8
  %payload.third = getelementptr i8, ptr %payload, i64 16
  store ptr %third, ptr %payload.third, align 8
  %fourth.pointer = getelementptr i8, ptr %list, i64 24
  %fourth = load ptr, ptr %fourth.pointer, align 8
  %payload.fourth = getelementptr i8, ptr %payload, i64 24
  store ptr %fourth, ptr %payload.fourth, align 8
  %present = call ptr @topal.runtime.optional.some(ptr %payload)
  ret ptr %present
none:
  %absent = call ptr @topal.runtime.optional.none()
  ret ptr %absent
}

define internal ptr @topal.runtime.list.int-int-string-int.entry.count(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%incremented, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr i8, ptr %current, i64 32
  %next = load ptr, ptr %next.pointer, align 8
  %incremented = add i64 %count, 1
  br label %loop
done:
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}
