%topal.ListStringStorage = type { ptr, ptr }

define internal i1 @topal.runtime.list.string.starts.with(ptr %source, ptr %pattern) nounwind noinline {
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
  %source.value = load ptr, ptr %source.current, align 8
  %pattern.value = load ptr, ptr %pattern.current, align 8
  %equal = call i1 @topal.runtime.string.equal(ptr %source.value, ptr %pattern.value)
  br i1 %equal, label %advance, label %different
advance:
  %source.next.pointer = getelementptr %topal.ListStringStorage, ptr %source.current, i32 0, i32 1
  %pattern.next.pointer = getelementptr %topal.ListStringStorage, ptr %pattern.current, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %pattern.next = load ptr, ptr %pattern.next.pointer, align 8
  br label %loop
matches:
  ret i1 true
different:
  ret i1 false
}

define internal i1 @topal.runtime.list.string.contains.sequence(ptr %list, ptr %pattern) nounwind noinline {
entry:
  %pattern.empty = icmp eq ptr %pattern, null
  br i1 %pattern.empty, label %found, label %search
search:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %missing, label %inspect
inspect:
  %matches = call i1 @topal.runtime.list.string.starts.with(ptr %current, ptr %pattern)
  br i1 %matches, label %found, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStringStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  br label %search
found:
  ret i1 true
missing:
  ret i1 false
}

define internal i1 @topal.runtime.list.string.contains.subsequence(ptr %list, ptr %pattern) nounwind noinline {
entry:
  br label %loop
loop:
  %source.current = phi ptr [%list, %entry], [%source.next, %advance]
  %pattern.current = phi ptr [%pattern, %entry], [%pattern.after, %advance]
  %pattern.empty = icmp eq ptr %pattern.current, null
  br i1 %pattern.empty, label %found, label %check.source
check.source:
  %source.empty = icmp eq ptr %source.current, null
  br i1 %source.empty, label %missing, label %compare
compare:
  %source.value = load ptr, ptr %source.current, align 8
  %pattern.value = load ptr, ptr %pattern.current, align 8
  %equal = call i1 @topal.runtime.string.equal(ptr %source.value, ptr %pattern.value)
  br label %advance
advance:
  %source.next.pointer = getelementptr %topal.ListStringStorage, ptr %source.current, i32 0, i32 1
  %pattern.next.pointer = getelementptr %topal.ListStringStorage, ptr %pattern.current, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %pattern.next = load ptr, ptr %pattern.next.pointer, align 8
  %pattern.after = select i1 %equal, ptr %pattern.next, ptr %pattern.current
  br label %loop
found:
  ret i1 true
missing:
  ret i1 false
}

define internal i64 @topal.runtime.list.string.count.raw(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%count.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStringStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  %count.next = add i64 %count, 1
  br label %loop
done:
  ret i64 %count
}

define internal ptr @topal.runtime.list.string.concat(ptr %left, ptr %right) nounwind noinline {
entry:
  %count = call i64 @topal.runtime.list.string.count.raw(ptr %left)
  %empty = icmp eq i64 %count, 0
  br i1 %empty, label %share.right, label %allocate
allocate:
  %allocation.length = shl i64 %count, 4
  %copy = call ptr @topal.platform.allocate(i64 %allocation.length)
  br label %loop
loop:
  %source = phi ptr [%left, %allocate], [%source.next, %advance]
  %index = phi i64 [0, %allocate], [%index.next, %advance]
  %source.value.pointer = getelementptr %topal.ListStringStorage, ptr %source, i32 0, i32 0
  %source.value = load ptr, ptr %source.value.pointer, align 8
  %source.next.pointer = getelementptr %topal.ListStringStorage, ptr %source, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %destination.offset = shl i64 %index, 4
  %destination = getelementptr i8, ptr %copy, i64 %destination.offset
  %destination.value.pointer = getelementptr %topal.ListStringStorage, ptr %destination, i32 0, i32 0
  store ptr %source.value, ptr %destination.value.pointer, align 8
  %index.next = add i64 %index, 1
  %last = icmp eq i64 %index.next, %count
  %destination.next = getelementptr i8, ptr %destination, i64 16
  %destination.remaining = select i1 %last, ptr %right, ptr %destination.next
  %destination.remaining.pointer = getelementptr %topal.ListStringStorage, ptr %destination, i32 0, i32 1
  store ptr %destination.remaining, ptr %destination.remaining.pointer, align 8
  br i1 %last, label %done, label %advance
advance:
  br label %loop
share.right:
  ret ptr %right
done:
  ret ptr %copy
}

define internal i1 @topal.runtime.list.string.equal(ptr %left, ptr %right) nounwind noinline {
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
  %left.value.pointer = getelementptr %topal.ListStringStorage, ptr %left.current, i32 0, i32 0
  %right.value.pointer = getelementptr %topal.ListStringStorage, ptr %right.current, i32 0, i32 0
  %left.value = load ptr, ptr %left.value.pointer, align 8
  %right.value = load ptr, ptr %right.value.pointer, align 8
  %equal = call i1 @topal.runtime.string.equal(ptr %left.value, ptr %right.value)
  br i1 %equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr %topal.ListStringStorage, ptr %left.current, i32 0, i32 1
  %right.next.pointer = getelementptr %topal.ListStringStorage, ptr %right.current, i32 0, i32 1
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}

define internal i1 @topal.runtime.list.string.contains.entry(ptr %list, ptr %value) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %missing, label %inspect
inspect:
  %entry.pointer = getelementptr %topal.ListStringStorage, ptr %current, i32 0, i32 0
  %entry.value = load ptr, ptr %entry.pointer, align 8
  %equal = call i1 @topal.runtime.string.equal(ptr %entry.value, ptr %value)
  br i1 %equal, label %found, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStringStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  br label %loop
found:
  ret i1 true
missing:
  ret i1 false
}

define internal ptr @topal.runtime.list.string.entry.count(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%incremented, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStringStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  %incremented = add i64 %count, 1
  br label %loop
done:
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}
