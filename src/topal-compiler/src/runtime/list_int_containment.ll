; Allocation-free List Int containment over the compiler-private node layout.

define internal i1 @topal.runtime.list.int.contains.entry(ptr %list, ptr %value) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %missing, label %inspect
inspect:
  %entry.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 0
  %entry.value = load ptr, ptr %entry.pointer, align 8
  %ordering = call i32 @topal.runtime.int.compare(ptr %entry.value, ptr %value)
  %equal = icmp eq i32 %ordering, 0
  br i1 %equal, label %found, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  br label %loop
found:
  ret i1 true
missing:
  ret i1 false
}

define internal i1 @topal.runtime.list.int.starts.with(ptr %source, ptr %pattern) nounwind noinline {
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
  %source.value.pointer = getelementptr %topal.ListStorage, ptr %source.current, i32 0, i32 0
  %pattern.value.pointer = getelementptr %topal.ListStorage, ptr %pattern.current, i32 0, i32 0
  %source.value = load ptr, ptr %source.value.pointer, align 8
  %pattern.value = load ptr, ptr %pattern.value.pointer, align 8
  %ordering = call i32 @topal.runtime.int.compare(ptr %source.value, ptr %pattern.value)
  %equal = icmp eq i32 %ordering, 0
  br i1 %equal, label %advance, label %different
advance:
  %source.next.pointer = getelementptr %topal.ListStorage, ptr %source.current, i32 0, i32 1
  %pattern.next.pointer = getelementptr %topal.ListStorage, ptr %pattern.current, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %pattern.next = load ptr, ptr %pattern.next.pointer, align 8
  br label %loop
matches:
  ret i1 true
different:
  ret i1 false
}

define internal i1 @topal.runtime.list.int.contains.sequence(ptr %list, ptr %pattern) nounwind noinline {
entry:
  %pattern.empty = icmp eq ptr %pattern, null
  br i1 %pattern.empty, label %found, label %search
search:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %missing, label %inspect
inspect:
  %matches = call i1 @topal.runtime.list.int.starts.with(ptr %current, ptr %pattern)
  br i1 %matches, label %found, label %advance
advance:
  %next.pointer = getelementptr %topal.ListStorage, ptr %current, i32 0, i32 1
  %next = load ptr, ptr %next.pointer, align 8
  br label %search
found:
  ret i1 true
missing:
  ret i1 false
}

define internal i1 @topal.runtime.list.int.contains.subsequence(ptr %list, ptr %pattern) nounwind noinline {
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
  %source.value.pointer = getelementptr %topal.ListStorage, ptr %source.current, i32 0, i32 0
  %pattern.value.pointer = getelementptr %topal.ListStorage, ptr %pattern.current, i32 0, i32 0
  %source.value = load ptr, ptr %source.value.pointer, align 8
  %pattern.value = load ptr, ptr %pattern.value.pointer, align 8
  %ordering = call i32 @topal.runtime.int.compare(ptr %source.value, ptr %pattern.value)
  %equal = icmp eq i32 %ordering, 0
  br label %advance
advance:
  %source.next.pointer = getelementptr %topal.ListStorage, ptr %source.current, i32 0, i32 1
  %pattern.next.pointer = getelementptr %topal.ListStorage, ptr %pattern.current, i32 0, i32 1
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %pattern.next = load ptr, ptr %pattern.next.pointer, align 8
  %pattern.after = select i1 %equal, ptr %pattern.next, ptr %pattern.current
  br label %loop
found:
  ret i1 true
missing:
  ret i1 false
}
