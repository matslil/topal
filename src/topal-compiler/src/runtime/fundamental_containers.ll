; Compiler-private fundamental homogeneous containers. Construction may mutate
; unpublished result nodes; published Array, Set, Bag, and Map values are immutable.

%topal.ContainerSequenceHeader = type { i64, ptr }
%topal.ContainerBagHeader = type { i64, i64, ptr }
%topal.ContainerBagNode = type { ptr, i64, ptr }
%topal.ContainerMapNode = type { ptr, ptr, ptr }

define internal i64 @topal.runtime.container.list.count(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %count = phi i64 [0, %entry], [%count.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %advance
advance:
  %next.pointer = getelementptr i8, ptr %current, i64 8
  %next = load ptr, ptr %next.pointer, align 8
  %count.next = add i64 %count, 1
  br label %loop
done:
  ret i64 %count
}

define internal ptr @topal.runtime.container.array.int.collect(ptr %list) nounwind noinline {
entry:
  %count = call i64 @topal.runtime.container.list.count(ptr %list)
  %array = call ptr @topal.platform.allocate(i64 16)
  %count.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %array, i32 0, i32 0
  %entries.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %array, i32 0, i32 1
  store i64 %count, ptr %count.pointer, align 8
  store ptr %list, ptr %entries.pointer, align 8
  ret ptr %array
}

define internal ptr @topal.runtime.container.array.function.collect(ptr %list) nounwind noinline {
entry:
  %count = call i64 @topal.runtime.container.list.count(ptr %list)
  %array = call ptr @topal.platform.allocate(i64 16)
  %count.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %array, i32 0, i32 0
  %entries.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %array, i32 0, i32 1
  store i64 %count, ptr %count.pointer, align 8
  store ptr %list, ptr %entries.pointer, align 8
  ret ptr %array
}

define internal i1 @topal.runtime.container.int.list.contains(ptr %list, ptr %value) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %absent, label %compare
compare:
  %candidate = load ptr, ptr %current, align 8
  %ordering = call i32 @topal.runtime.int.compare(ptr %candidate, ptr %value)
  %equal = icmp eq i32 %ordering, 0
  br i1 %equal, label %present, label %advance
advance:
  %next.pointer = getelementptr i8, ptr %current, i64 8
  %next = load ptr, ptr %next.pointer, align 8
  br label %loop
present:
  ret i1 true
absent:
  ret i1 false
}

define internal ptr @topal.runtime.container.set.int.collect(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%source.next, %advance]
  %head = phi ptr [null, %entry], [%head.next, %advance]
  %tail = phi ptr [null, %entry], [%tail.next, %advance]
  %count = phi i64 [0, %entry], [%count.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit
visit:
  %value = load ptr, ptr %current, align 8
  %source.next.pointer = getelementptr i8, ptr %current, i64 8
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %duplicate = call i1 @topal.runtime.container.int.list.contains(ptr %head, ptr %value)
  br i1 %duplicate, label %skip, label %add
add:
  %node = call ptr @topal.platform.allocate(i64 16)
  store ptr %value, ptr %node, align 8
  %node.next = getelementptr i8, ptr %node, i64 8
  store ptr null, ptr %node.next, align 8
  %first = icmp eq ptr %tail, null
  br i1 %first, label %start, label %link
start:
  br label %added
link:
  %tail.next.pointer = getelementptr i8, ptr %tail, i64 8
  store ptr %node, ptr %tail.next.pointer, align 8
  br label %added
added:
  %added.head = phi ptr [%node, %start], [%head, %link]
  %added.count = add i64 %count, 1
  br label %advance
skip:
  br label %advance
advance:
  %head.next = phi ptr [%head, %skip], [%added.head, %added]
  %tail.next = phi ptr [%tail, %skip], [%node, %added]
  %count.next = phi i64 [%count, %skip], [%added.count, %added]
  br label %loop
done:
  %set = call ptr @topal.platform.allocate(i64 16)
  %set.count = getelementptr %topal.ContainerSequenceHeader, ptr %set, i32 0, i32 0
  %set.entries = getelementptr %topal.ContainerSequenceHeader, ptr %set, i32 0, i32 1
  store i64 %count, ptr %set.count, align 8
  store ptr %head, ptr %set.entries, align 8
  ret ptr %set
}

define internal ptr @topal.runtime.container.bag.int.find(ptr %entries, ptr %value) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%entries, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %absent, label %compare
compare:
  %candidate = load ptr, ptr %current, align 8
  %ordering = call i32 @topal.runtime.int.compare(ptr %candidate, ptr %value)
  %equal = icmp eq i32 %ordering, 0
  br i1 %equal, label %present, label %advance
advance:
  %next.pointer = getelementptr %topal.ContainerBagNode, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  br label %loop
present:
  ret ptr %current
absent:
  ret ptr null
}

define internal ptr @topal.runtime.container.bag.int.collect(ptr %list) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%list, %entry], [%source.next, %advance]
  %head = phi ptr [null, %entry], [%head.next, %advance]
  %tail = phi ptr [null, %entry], [%tail.next, %advance]
  %total = phi i64 [0, %entry], [%total.next, %advance]
  %distinct = phi i64 [0, %entry], [%distinct.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit
visit:
  %value = load ptr, ptr %current, align 8
  %source.next.pointer = getelementptr i8, ptr %current, i64 8
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %existing = call ptr @topal.runtime.container.bag.int.find(ptr %head, ptr %value)
  %found = icmp ne ptr %existing, null
  br i1 %found, label %increment, label %add
increment:
  %existing.count.pointer = getelementptr %topal.ContainerBagNode, ptr %existing, i32 0, i32 1
  %existing.count = load i64, ptr %existing.count.pointer, align 8
  %incremented = add i64 %existing.count, 1
  store i64 %incremented, ptr %existing.count.pointer, align 8
  br label %advance
add:
  %node = call ptr @topal.platform.allocate(i64 24)
  %node.value = getelementptr %topal.ContainerBagNode, ptr %node, i32 0, i32 0
  %node.count = getelementptr %topal.ContainerBagNode, ptr %node, i32 0, i32 1
  %node.next = getelementptr %topal.ContainerBagNode, ptr %node, i32 0, i32 2
  store ptr %value, ptr %node.value, align 8
  store i64 1, ptr %node.count, align 8
  store ptr null, ptr %node.next, align 8
  %first = icmp eq ptr %tail, null
  br i1 %first, label %start, label %link
start:
  br label %added
link:
  %tail.next.pointer = getelementptr %topal.ContainerBagNode, ptr %tail, i32 0, i32 2
  store ptr %node, ptr %tail.next.pointer, align 8
  br label %added
added:
  %added.head = phi ptr [%node, %start], [%head, %link]
  %added.distinct = add i64 %distinct, 1
  br label %advance
advance:
  %head.next = phi ptr [%head, %increment], [%added.head, %added]
  %tail.next = phi ptr [%tail, %increment], [%node, %added]
  %distinct.next = phi i64 [%distinct, %increment], [%added.distinct, %added]
  %total.next = add i64 %total, 1
  br label %loop
done:
  %bag = call ptr @topal.platform.allocate(i64 24)
  %bag.total = getelementptr %topal.ContainerBagHeader, ptr %bag, i32 0, i32 0
  %bag.distinct = getelementptr %topal.ContainerBagHeader, ptr %bag, i32 0, i32 1
  %bag.entries = getelementptr %topal.ContainerBagHeader, ptr %bag, i32 0, i32 2
  store i64 %total, ptr %bag.total, align 8
  store i64 %distinct, ptr %bag.distinct, align 8
  store ptr %head, ptr %bag.entries, align 8
  ret ptr %bag
}

define internal ptr @topal.runtime.container.map.string-int.find(ptr %entries, ptr %key) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%entries, %entry], [%next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %absent, label %compare
compare:
  %candidate = load ptr, ptr %current, align 8
  %equal = call i1 @topal.runtime.string.equal(ptr %candidate, ptr %key)
  br i1 %equal, label %present, label %advance
advance:
  %next.pointer = getelementptr %topal.ContainerMapNode, ptr %current, i32 0, i32 2
  %next = load ptr, ptr %next.pointer, align 8
  br label %loop
present:
  ret ptr %current
absent:
  ret ptr null
}

define internal ptr @topal.runtime.container.map.string-int.collect(ptr %pairs, i1 %keep.last) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%pairs, %entry], [%source.next, %advance]
  %head = phi ptr [null, %entry], [%head.next, %advance]
  %tail = phi ptr [null, %entry], [%tail.next, %advance]
  %count = phi i64 [0, %entry], [%count.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit
visit:
  %key = load ptr, ptr %current, align 8
  %source.value.pointer = getelementptr i8, ptr %current, i64 8
  %source.next.pointer = getelementptr i8, ptr %current, i64 16
  %value = load ptr, ptr %source.value.pointer, align 8
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %existing = call ptr @topal.runtime.container.map.string-int.find(ptr %head, ptr %key)
  %found = icmp ne ptr %existing, null
  br i1 %found, label %collision, label %add
collision:
  br i1 %keep.last, label %replace, label %keep
replace:
  %existing.value = getelementptr %topal.ContainerMapNode, ptr %existing, i32 0, i32 1
  store ptr %value, ptr %existing.value, align 8
  br label %advance
keep:
  br label %advance
add:
  %node = call ptr @topal.platform.allocate(i64 24)
  %node.key = getelementptr %topal.ContainerMapNode, ptr %node, i32 0, i32 0
  %node.value = getelementptr %topal.ContainerMapNode, ptr %node, i32 0, i32 1
  %node.next = getelementptr %topal.ContainerMapNode, ptr %node, i32 0, i32 2
  store ptr %key, ptr %node.key, align 8
  store ptr %value, ptr %node.value, align 8
  store ptr null, ptr %node.next, align 8
  %first = icmp eq ptr %tail, null
  br i1 %first, label %start, label %link
start:
  br label %added
link:
  %tail.next.pointer = getelementptr %topal.ContainerMapNode, ptr %tail, i32 0, i32 2
  store ptr %node, ptr %tail.next.pointer, align 8
  br label %added
added:
  %added.head = phi ptr [%node, %start], [%head, %link]
  %added.count = add i64 %count, 1
  br label %advance
advance:
  %head.next = phi ptr [%head, %replace], [%head, %keep], [%added.head, %added]
  %tail.next = phi ptr [%tail, %replace], [%tail, %keep], [%node, %added]
  %count.next = phi i64 [%count, %replace], [%count, %keep], [%added.count, %added]
  br label %loop
done:
  %mapping = call ptr @topal.platform.allocate(i64 16)
  %mapping.count = getelementptr %topal.ContainerSequenceHeader, ptr %mapping, i32 0, i32 0
  %mapping.entries = getelementptr %topal.ContainerSequenceHeader, ptr %mapping, i32 0, i32 1
  store i64 %count, ptr %mapping.count, align 8
  store ptr %head, ptr %mapping.entries, align 8
  ret ptr %mapping
}

define internal ptr @topal.runtime.container.map.string-function.collect(ptr %pairs, i1 %keep.last) nounwind noinline {
entry:
  br label %loop
loop:
  %current = phi ptr [%pairs, %entry], [%source.next, %advance]
  %head = phi ptr [null, %entry], [%head.next, %advance]
  %tail = phi ptr [null, %entry], [%tail.next, %advance]
  %count = phi i64 [0, %entry], [%count.next, %advance]
  %empty = icmp eq ptr %current, null
  br i1 %empty, label %done, label %visit
visit:
  %key = load ptr, ptr %current, align 8
  %source.value.pointer = getelementptr i8, ptr %current, i64 8
  %source.next.pointer = getelementptr i8, ptr %current, i64 16
  %value = load i32, ptr %source.value.pointer, align 4
  %source.next = load ptr, ptr %source.next.pointer, align 8
  %existing = call ptr @topal.runtime.container.map.string-int.find(ptr %head, ptr %key)
  %found = icmp ne ptr %existing, null
  br i1 %found, label %collision, label %add
collision:
  br i1 %keep.last, label %replace, label %keep
replace:
  %existing.value = getelementptr %topal.ContainerMapNode, ptr %existing, i32 0, i32 1
  store i32 %value, ptr %existing.value, align 4
  br label %advance
keep:
  br label %advance
add:
  %node = call ptr @topal.platform.allocate(i64 24)
  %node.key = getelementptr %topal.ContainerMapNode, ptr %node, i32 0, i32 0
  %node.value = getelementptr %topal.ContainerMapNode, ptr %node, i32 0, i32 1
  %node.next = getelementptr %topal.ContainerMapNode, ptr %node, i32 0, i32 2
  store ptr %key, ptr %node.key, align 8
  store i32 %value, ptr %node.value, align 4
  store ptr null, ptr %node.next, align 8
  %first = icmp eq ptr %tail, null
  br i1 %first, label %start, label %link
start:
  br label %added
link:
  %tail.next.pointer = getelementptr %topal.ContainerMapNode, ptr %tail, i32 0, i32 2
  store ptr %node, ptr %tail.next.pointer, align 8
  br label %added
added:
  %added.head = phi ptr [%node, %start], [%head, %link]
  %added.count = add i64 %count, 1
  br label %advance
advance:
  %head.next = phi ptr [%head, %replace], [%head, %keep], [%added.head, %added]
  %tail.next = phi ptr [%tail, %replace], [%tail, %keep], [%node, %added]
  %count.next = phi i64 [%count, %replace], [%count, %keep], [%added.count, %added]
  br label %loop
done:
  %mapping = call ptr @topal.platform.allocate(i64 16)
  %mapping.count = getelementptr %topal.ContainerSequenceHeader, ptr %mapping, i32 0, i32 0
  %mapping.entries = getelementptr %topal.ContainerSequenceHeader, ptr %mapping, i32 0, i32 1
  store i64 %count, ptr %mapping.count, align 8
  store ptr %head, ptr %mapping.entries, align 8
  ret ptr %mapping
}

define internal ptr @topal.runtime.container.entry.count(ptr %container) nounwind noinline {
entry:
  %count = load i64, ptr %container, align 8
  %value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %value
}

define internal i1 @topal.runtime.container.empty(ptr %container) nounwind noinline {
entry:
  %count = load i64, ptr %container, align 8
  %empty = icmp eq i64 %count, 0
  ret i1 %empty
}

define internal ptr @topal.runtime.container.array.int.at(ptr %array, i64 %index) nounwind noinline {
entry:
  %count = load i64, ptr %array, align 8
  %in.bounds = icmp ult i64 %index, %count
  br i1 %in.bounds, label %find, label %absent
find:
  %entries.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %array, i32 0, i32 1
  %entries = load ptr, ptr %entries.pointer, align 8
  br label %loop
loop:
  %current = phi ptr [%entries, %find], [%next, %advance]
  %position = phi i64 [0, %find], [%position.next, %advance]
  %found = icmp eq i64 %position, %index
  br i1 %found, label %present, label %advance
advance:
  %next.pointer = getelementptr i8, ptr %current, i64 8
  %next = load ptr, ptr %next.pointer, align 8
  %position.next = add i64 %position, 1
  br label %loop
present:
  %value = load ptr, ptr %current, align 8
  %some = call ptr @topal.runtime.optional.some(ptr %value)
  ret ptr %some
absent:
  %none = call ptr @topal.runtime.optional.none()
  ret ptr %none
}

define internal ptr @topal.runtime.container.array.function.at(ptr %array, i64 %index) nounwind noinline {
entry:
  %count = load i64, ptr %array, align 8
  %in.bounds = icmp ult i64 %index, %count
  br i1 %in.bounds, label %find, label %absent
find:
  %entries.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %array, i32 0, i32 1
  %entries = load ptr, ptr %entries.pointer, align 8
  br label %loop
loop:
  %current = phi ptr [%entries, %find], [%next, %advance]
  %position = phi i64 [0, %find], [%position.next, %advance]
  %found = icmp eq i64 %position, %index
  br i1 %found, label %present, label %advance
advance:
  %next.pointer = getelementptr i8, ptr %current, i64 8
  %next = load ptr, ptr %next.pointer, align 8
  %position.next = add i64 %position, 1
  br label %loop
present:
  %value = load i32, ptr %current, align 4
  %payload = call ptr @topal.platform.allocate(i64 4)
  store i32 %value, ptr %payload, align 4
  %some = call ptr @topal.runtime.optional.some(ptr %payload)
  ret ptr %some
absent:
  %none = call ptr @topal.runtime.optional.none()
  ret ptr %none
}

define internal i1 @topal.runtime.container.set.int.contains(ptr %set, ptr %value) nounwind noinline {
entry:
  %entries.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %set, i32 0, i32 1
  %entries = load ptr, ptr %entries.pointer, align 8
  %present = call i1 @topal.runtime.container.int.list.contains(ptr %entries, ptr %value)
  ret i1 %present
}

define internal ptr @topal.runtime.container.bag.int.multiplicity(ptr %bag, ptr %value) nounwind noinline {
entry:
  %entries.pointer = getelementptr %topal.ContainerBagHeader, ptr %bag, i32 0, i32 2
  %entries = load ptr, ptr %entries.pointer, align 8
  %node = call ptr @topal.runtime.container.bag.int.find(ptr %entries, ptr %value)
  %absent = icmp eq ptr %node, null
  br i1 %absent, label %zero, label %present
zero:
  %zero.value = call ptr @topal.runtime.int.from.u64(i64 0)
  ret ptr %zero.value
present:
  %count.pointer = getelementptr %topal.ContainerBagNode, ptr %node, i32 0, i32 1
  %count = load i64, ptr %count.pointer, align 8
  %count.value = call ptr @topal.runtime.int.from.u64(i64 %count)
  ret ptr %count.value
}

define internal ptr @topal.runtime.container.map.string-int.lookup(ptr %mapping, ptr %key) nounwind noinline {
entry:
  %entries.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %mapping, i32 0, i32 1
  %entries = load ptr, ptr %entries.pointer, align 8
  %node = call ptr @topal.runtime.container.map.string-int.find(ptr %entries, ptr %key)
  %absent = icmp eq ptr %node, null
  br i1 %absent, label %none, label %some
some:
  %value.pointer = getelementptr %topal.ContainerMapNode, ptr %node, i32 0, i32 1
  %value = load ptr, ptr %value.pointer, align 8
  %some.value = call ptr @topal.runtime.optional.some(ptr %value)
  ret ptr %some.value
none:
  %none.value = call ptr @topal.runtime.optional.none()
  ret ptr %none.value
}

define internal ptr @topal.runtime.container.map.string-function.lookup(ptr %mapping, ptr %key) nounwind noinline {
entry:
  %entries.pointer = getelementptr %topal.ContainerSequenceHeader, ptr %mapping, i32 0, i32 1
  %entries = load ptr, ptr %entries.pointer, align 8
  %node = call ptr @topal.runtime.container.map.string-int.find(ptr %entries, ptr %key)
  %absent = icmp eq ptr %node, null
  br i1 %absent, label %none, label %some
some:
  %value.pointer = getelementptr %topal.ContainerMapNode, ptr %node, i32 0, i32 1
  %value = load i32, ptr %value.pointer, align 4
  %payload = call ptr @topal.platform.allocate(i64 4)
  store i32 %value, ptr %payload, align 4
  %some.value = call ptr @topal.runtime.optional.some(ptr %payload)
  ret ptr %some.value
none:
  %none.value = call ptr @topal.runtime.optional.none()
  ret ptr %none.value
}
