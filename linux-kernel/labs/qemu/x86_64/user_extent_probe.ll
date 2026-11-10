; First-party unpublished observation only. Requests remain actual ABI requests;
; this probe never changes allocation results, page tables or user data.
@topal_x86_user_extent_probe_counts_v1 = global [4 x i64] zeroinitializer, align 8
@probe.last_count = internal global i64 -1, align 8
@topal_x86_frame_mm_state_v1 = external global [4 x i64]
declare i64 @topal_x86_user_extent_call_v1(i64,i64,i64)
declare void @topal_x86_systems_fatal() noreturn

define void @topal_x86_user_extent_probe_fail_v1() noreturn nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void @topal_x86_systems_fatal()
  unreachable
}
define internal void @probe.check(i64 %start,i64 %length,i64 %write,i64 %expected) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %actual = call i64 @topal_x86_user_extent_call_v1(i64 %start,i64 %length,i64 %write)
  %ok = icmp eq i64 %actual,%expected
  br i1 %ok,label %count,label %bad
count:
  %at = getelementptr i64,ptr @topal_x86_user_extent_probe_counts_v1,i64 1
  %old = load volatile i64,ptr %at,align 8
  %next = add i64 %old,1
  store volatile i64 %next,ptr %at,align 8
  ret void
bad:
  call void @topal_x86_user_extent_probe_fail_v1()
  unreachable
}
define void @topal_x86_user_extent_probe_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %count_at = getelementptr i64,ptr @topal_x86_frame_mm_state_v1,i64 2
  %count = load volatile i64,ptr %count_at,align 8
  %previous = load volatile i64,ptr @probe.last_count,align 8
  %same = icmp eq i64 %count,%previous
  br i1 %same,label %done,label %changed
changed:
  store volatile i64 %count,ptr @probe.last_count,align 8
  %old = load volatile i64,ptr @topal_x86_user_extent_probe_counts_v1,align 8
  %next = add i64 %old,1
  store volatile i64 %next,ptr @topal_x86_user_extent_probe_counts_v1,align 8
  call void @probe.check(i64 4198400,i64 1,i64 0,i64 0)
  call void @probe.check(i64 4198400,i64 1,i64 1,i64 3)
  call void @probe.check(i64 8388592,i64 16,i64 1,i64 0)
  call void @probe.check(i64 -1,i64 1,i64 0,i64 1)
  call void @probe.check(i64 -1,i64 0,i64 0,i64 0)
  %has_heap = icmp ne i64 %count,0
  br i1 %has_heap,label %heap_tail,label %cross_check
heap_tail:
  %bytes = shl i64 %count,12
  %end = add i64 %bytes,6254592
  %last = sub i64 %end,1
  call void @probe.check(i64 %last,i64 1,i64 1,i64 0)
  %below_stack = icmp ult i64 %end,8380416
  br i1 %below_stack,label %hole,label %cross_check
hole:
  call void @probe.check(i64 %end,i64 1,i64 0,i64 2)
  br label %cross_check
cross_check:
  %crosses = icmp uge i64 %count,10
  br i1 %crosses,label %cross,label %large_check
cross:
  call void @probe.check(i64 6291448,i64 16,i64 1,i64 0)
  %at2 = getelementptr i64,ptr @topal_x86_user_extent_probe_counts_v1,i64 2
  %old2 = load volatile i64,ptr %at2,align 8
  %new2 = add i64 %old2,1
  store volatile i64 %new2,ptr %at2,align 8
  br label %large_check
large_check:
  %large = icmp ugt i64 %count,34
  br i1 %large,label %large_heap,label %done
large_heap:
  call void @probe.check(i64 6393856,i64 1,i64 1,i64 0)
  %at3 = getelementptr i64,ptr @topal_x86_user_extent_probe_counts_v1,i64 3
  %old3 = load volatile i64,ptr %at3,align 8
  %new3 = add i64 %old3,1
  store volatile i64 %new3,ptr %at3,align 8
  br label %done
done:
  ret void
}
