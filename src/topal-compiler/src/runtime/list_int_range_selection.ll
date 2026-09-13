define internal ptr @topal.runtime.list.int.select.value.range(ptr %source, ptr %range) nounwind noinline {
entry:
  %result = call ptr @topal.runtime.list.int.select.range(ptr %source, ptr %range, i1 false)
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.select.index.range(ptr %source, ptr %range) nounwind noinline {
entry:
  %result = call ptr @topal.runtime.list.int.select.range(ptr %source, ptr %range, i1 true)
  ret ptr %result
}

define internal ptr @topal.runtime.list.int.select.range(ptr %source, ptr %range, i1 %indexes) nounwind noinline {
entry:
  br label %loop

loop:
  %current = phi ptr [ %source, %entry ], [ %next, %advance ]
  %head = phi ptr [ null, %entry ], [ %next.head, %advance ]
  %previous = phi ptr [ null, %entry ], [ %next.previous, %advance ]
  %index = phi i64 [ 0, %entry ], [ %next.index, %advance ]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit

visit:
  %value.address = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 0
  %value = load ptr, ptr %value.address, align 8
  %next.address = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.address, align 8
  br i1 %indexes, label %by.index, label %by.value

by.index:
  %index.value = call ptr @topal.runtime.int.from.u64(i64 %index)
  br label %test

by.value:
  br label %test

test:
  %candidate = phi ptr [ %index.value, %by.index ], [ %value, %by.value ]
  %keep = call i1 @topal.runtime.range.int.contains(ptr %range, ptr %candidate)
  br i1 %keep, label %selected, label %skipped

skipped:
  br label %advance

selected:
  %node = call ptr @topal.platform.allocate(i64 16)
  %node.value.address = getelementptr %topal.ListStorage, ptr %node, i32 0, i32 0
  store ptr %value, ptr %node.value.address, align 8
  %node.next.address = getelementptr %topal.ListStorage, ptr %node, i32 0, i32 1
  store ptr null, ptr %node.next.address, align 8
  %has.previous = icmp ne ptr %previous, null
  br i1 %has.previous, label %link, label %first

first:
  br label %selected.merge

link:
  %previous.next.address = getelementptr %topal.ListStorage, ptr %previous, i32 0, i32 1
  store ptr %node, ptr %previous.next.address, align 8
  br label %selected.merge

selected.merge:
  %selected.head = phi ptr [ %node, %first ], [ %head, %link ]
  br label %advance

advance:
  %next.head = phi ptr [ %head, %skipped ], [ %selected.head, %selected.merge ]
  %next.previous = phi ptr [ %previous, %skipped ], [ %node, %selected.merge ]
  %next.index = add i64 %index, 1
  br label %loop

done:
  ret ptr %head
}
