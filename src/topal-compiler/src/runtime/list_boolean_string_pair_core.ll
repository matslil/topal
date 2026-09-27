%topal.ListBooleanStringPairStorage = type { i1, [7 x i8], ptr, ptr }

define internal ptr @topal.runtime.list.boolean-string.concat(ptr %left, ptr %right) nounwind noinline {
entry:
  %empty = icmp eq ptr %left, null
  br i1 %empty, label %share.right, label %copy
copy:
  %first = load i1, ptr %left, align 1
  %second.pointer = getelementptr %topal.ListBooleanStringPairStorage, ptr %left, i32 0, i32 2
  %second = load ptr, ptr %second.pointer, align 8
  %next.pointer = getelementptr %topal.ListBooleanStringPairStorage, ptr %left, i32 0, i32 3
  %next = load ptr, ptr %next.pointer, align 8
  %remaining = call ptr @topal.runtime.list.boolean-string.concat(ptr %next, ptr %right)
  %node = call ptr @topal.platform.allocate(i64 24)
  store i1 %first, ptr %node, align 1
  %node.second = getelementptr i8, ptr %node, i64 8
  store ptr %second, ptr %node.second, align 8
  %node.next = getelementptr i8, ptr %node, i64 16
  store ptr %remaining, ptr %node.next, align 8
  ret ptr %node
share.right:
  ret ptr %right
}

define internal i1 @topal.runtime.list.boolean-string.equal(ptr %left, ptr %right) nounwind noinline {
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
  %left.first = load i1, ptr %left.current, align 1
  %right.first = load i1, ptr %right.current, align 1
  %first.equal = icmp eq i1 %left.first, %right.first
  %left.second.pointer = getelementptr %topal.ListBooleanStringPairStorage, ptr %left.current, i32 0, i32 2
  %right.second.pointer = getelementptr %topal.ListBooleanStringPairStorage, ptr %right.current, i32 0, i32 2
  %left.second = load ptr, ptr %left.second.pointer, align 8
  %right.second = load ptr, ptr %right.second.pointer, align 8
  %second.equal = call i1 @topal.runtime.string.equal(ptr %left.second, ptr %right.second)
  %equal = and i1 %first.equal, %second.equal
  br i1 %equal, label %advance, label %different
advance:
  %left.next.pointer = getelementptr %topal.ListBooleanStringPairStorage, ptr %left.current, i32 0, i32 3
  %right.next.pointer = getelementptr %topal.ListBooleanStringPairStorage, ptr %right.current, i32 0, i32 3
  %left.next = load ptr, ptr %left.next.pointer, align 8
  %right.next = load ptr, ptr %right.next.pointer, align 8
  br label %loop
finish:
  %both.empty = and i1 %left.empty, %right.empty
  ret i1 %both.empty
different:
  ret i1 false
}
