; Immutable List Int removal over the compiler-private node layout.

define internal ptr @topal.runtime.list.int.remove.first(ptr %list, ptr %value) nounwind noinline {
entry:
  br label %search
search:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %prefix.count = phi i64 [0, %entry], [%prefix.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %not.found, label %inspect
inspect:
  %entry.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 0
  %entry.value = load ptr, ptr %entry.pointer, align 8
  %ordering = call i32 @topal.runtime.int.compare(ptr %entry.value, ptr %value)
  %equal = icmp eq i32 %ordering, 0
  br i1 %equal, label %found, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  %prefix.next = add i64 %prefix.count, 1
  br label %search
found:
  %remaining.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %remaining = load ptr, ptr %remaining.pointer, align 8
  %at.front = icmp eq i64 %prefix.count, 0
  br i1 %at.front, label %share.suffix, label %allocate
allocate:
  %allocation.length = shl i64 %prefix.count, 4
  %copy = call ptr @topal.platform.allocate(i64 %allocation.length)
  br label %copy.loop
copy.loop:
  %source = phi ptr [%list, %allocate], [%source.next, %copy.advance]
  %index = phi i64 [0, %allocate], [%index.next, %copy.advance]
  %source.value.pointer = getelementptr %topal.ListStorage, ptr %source, i32 0, i32 0
  %source.value = load ptr, ptr %source.value.pointer, align 8
  %destination.offset = shl i64 %index, 4
  %destination = getelementptr i8, ptr %copy, i64 %destination.offset
  %destination.value.pointer = getelementptr %topal.ListStorage, ptr %destination, i32 0, i32 0
  store ptr %source.value, ptr %destination.value.pointer, align 8
  %index.next = add i64 %index, 1
  %last = icmp eq i64 %index.next, %prefix.count
  %destination.next = getelementptr i8, ptr %destination, i64 16
  %destination.remaining = select i1 %last, ptr %remaining, ptr %destination.next
  %destination.remaining.pointer = getelementptr %topal.ListStorage, ptr %destination, i32 0, i32 1
  store ptr %destination.remaining, ptr %destination.remaining.pointer, align 8
  br i1 %last, label %copied, label %copy.advance
copy.advance:
  %source.next.pointer = getelementptr %topal.ListStorage, ptr %source, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  br label %copy.loop
share.suffix:
  ret ptr %remaining
copied:
  ret ptr %copy
not.found:
  ret ptr %list
}

define internal ptr @topal.runtime.list.int.remove.all(ptr %list, ptr %value) nounwind noinline {
entry:
  br label %scan
scan:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %kept = phi i64 [0, %entry], [%kept.after, %advance]
  %removed = phi i1 [false, %entry], [%removed.after, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %summarize, label %inspect
inspect:
  %entry.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 0
  %entry.value = load ptr, ptr %entry.pointer, align 8
  %ordering = call i32 @topal.runtime.int.compare(ptr %entry.value, ptr %value)
  %equal = icmp eq i32 %ordering, 0
  %kept.next = add i64 %kept, 1
  %kept.after = select i1 %equal, i64 %kept, i64 %kept.next
  %removed.after = or i1 %removed, %equal
  br label %advance
advance:
  %next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  br label %scan
summarize:
  br i1 %removed, label %check.empty, label %unchanged
check.empty:
  %all.removed = icmp eq i64 %kept, 0
  br i1 %all.removed, label %removed.all, label %allocate
allocate:
  %allocation.length = shl i64 %kept, 4
  %copy = call ptr @topal.platform.allocate(i64 %allocation.length)
  br label %copy.loop
copy.loop:
  %source = phi ptr [%list, %allocate], [%source.next, %copy.advance]
  %written = phi i64 [0, %allocate], [%written.after, %copy.advance]
  %source.value.pointer = getelementptr %topal.ListStorage, ptr %source, i32 0, i32 0
  %source.value = load ptr, ptr %source.value.pointer, align 8
  %source.ordering = call i32 @topal.runtime.int.compare(ptr %source.value, ptr %value)
  %source.equal = icmp eq i32 %source.ordering, 0
  br i1 %source.equal, label %copy.advance, label %copy.value
copy.value:
  %destination.offset = shl i64 %written, 4
  %destination = getelementptr i8, ptr %copy, i64 %destination.offset
  %destination.value.pointer = getelementptr %topal.ListStorage, ptr %destination, i32 0, i32 0
  store ptr %source.value, ptr %destination.value.pointer, align 8
  %written.next = add i64 %written, 1
  %last = icmp eq i64 %written.next, %kept
  %destination.next = getelementptr i8, ptr %destination, i64 16
  %destination.remaining = select i1 %last, ptr null, ptr %destination.next
  %destination.remaining.pointer = getelementptr %topal.ListStorage, ptr %destination, i32 0, i32 1
  store ptr %destination.remaining, ptr %destination.remaining.pointer, align 8
  br label %copy.advance
copy.advance:
  %written.after = phi i64 [%written, %copy.loop], [%written.next, %copy.value]
  %source.next.pointer = getelementptr %topal.ListStorage, ptr %source, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %complete = icmp eq i64 %written.after, %kept
  br i1 %complete, label %copied, label %copy.loop
unchanged:
  ret ptr %list
removed.all:
  ret ptr null
copied:
  ret ptr %copy
}
