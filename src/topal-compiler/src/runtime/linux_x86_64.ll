; topal.platform.linux-x86_64/1
; Hosted process hooks; the shared value algorithms have no host syscalls.

define internal i64 @topal.platform.write(i64 %fd, ptr %buffer, i64 %length) nounwind noinline {
entry:
  %result = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 1, i64 %fd, ptr %buffer, i64 %length)
  ret i64 %result
}

define internal void @topal.platform.exit(i64 %status) noreturn nounwind noinline {
entry:
  %ignored = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},~{rcx},~{r11},~{memory}"(i64 60, i64 %status)
  unreachable
}

define internal ptr @topal.platform.allocate(i64 %length) nounwind noinline {
entry:
  %valid = icmp ule i64 %length, 9223372036854775807
  br i1 %valid, label %map, label %failure
map:
  %result = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}"(i64 9, i64 0, i64 %length, i64 3, i64 34, i64 -1, i64 0)
  %failed = icmp uge i64 %result, -4095
  br i1 %failed, label %failure, label %success
success:
  %pointer = inttoptr i64 %result to ptr
  ret ptr %pointer
failure:
  call void @topal.platform.exit(i64 71)
  unreachable
}
