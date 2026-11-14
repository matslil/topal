; TK-DEC-284 private process caller identities. Hardware observations remain
; in the existing g.memory_context and owning gateway; no source entry is added.
@topal_x86_process_mm_inactive_token_v1 = external global [3 x i64]
@topal_x86_process_mm_prepare_return_v1 = external global i8
@topal_x86_process_mm_commit_return_v1 = external global i8
@topal_x86_process_mm_switch_return_v1 = external global i8
@topal_x86_process_mm_child_exit_return_v1 = external global i8
@topal_x86_process_mm_cancel_return_v1 = external global i8

define internal i1 @p.abi(i64 %op,i64 %number) nounwind noinline noredzone {
entry:
  switch i64 %op,label %bad [i64 9,label %creation
    i64 10,label %creation i64 11,label %switch i64 12,label %exit i64 13,label %exit]
creation:
  %fork = icmp eq i64 %number,57
  %clone = icmp eq i64 %number,56
  %create = or i1 %fork,%clone
  ret i1 %create
switch:
  %read = icmp eq i64 %number,0
  %write = icmp eq i64 %number,1
  %readv = icmp eq i64 %number,19
  %writev = icmp eq i64 %number,20
  %yield = icmp eq i64 %number,24
  %wait = icmp eq i64 %number,61
  %a = or i1 %read,%write
  %b = or i1 %readv,%writev
  %c = or i1 %yield,%wait
  %ab = or i1 %a,%b
  %ok = or i1 %ab,%c
  ret i1 %ok
exit:
  %one = icmp eq i64 %number,60
  %group = icmp eq i64 %number,231
  %leave = or i1 %one,%group
  ret i1 %leave
bad:
  ret i1 false
}
define internal i64 @p.return_site(i64 %op) nounwind noinline noredzone {
entry:
  switch i64 %op,label %bad [i64 9,label %prepare
    i64 10,label %commit i64 11,label %switch i64 12,label %exit i64 13,label %cancel]
prepare:
  ret i64 ptrtoint (ptr @topal_x86_process_mm_prepare_return_v1 to i64)
commit:
  ret i64 ptrtoint (ptr @topal_x86_process_mm_commit_return_v1 to i64)
switch:
  ret i64 ptrtoint (ptr @topal_x86_process_mm_switch_return_v1 to i64)
exit:
  ret i64 ptrtoint (ptr @topal_x86_process_mm_child_exit_return_v1 to i64)
cancel:
  ret i64 ptrtoint (ptr @topal_x86_process_mm_cancel_return_v1 to i64)
bad:
  ret i64 0
}
