; Inactive allocation preparation only. No CR3/control writes or table activation.
@topal_x86_frame_context_v1 = external global i8
@topal_x86_frame_registry_v1 = external global i8
@topal_x86_frame_table_v1 = external global i8
@topal_x86_frame_root_tokens_v1 = external global i8
@topal_x86_frame_details_v1 = external global i8
@topal_x86_kernel_reservation_end_v1 = external global i8
@topal_x86_image_backing_observation_v1 = global [32 x i64] zeroinitializer, align 8
declare void @topal_x86_systems_fatal() noreturn
declare i64 @topal_x86_frame_gateway_v1(i64,i64,i64,ptr,ptr)

define internal void @k.image.require(i1 %ok) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  br i1 %ok,label %end,label %bad
bad:
  call void @topal_x86_systems_fatal()
  unreachable
end:
  ret void
}
define internal i64 @t.word(ptr %base,i64 %index) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %at = getelementptr i64,ptr %base,i64 %index
  %word = load volatile i64,ptr %at,align 8
  ret i64 %word
}
define internal void @k.image.store(i64 %index,i64 %word) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %at = getelementptr i64,ptr @topal_x86_image_backing_observation_v1,i64 %index
  store volatile i64 %word,ptr %at,align 8
  ret void
}
define internal i64 @k.image.active_root() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %value = call i64 asm sideeffect "movq %cr3, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  ret i64 %value
}
; Enter the existing owning gateway normally. Its complete hardware/context/
; lease/domain checks remain in force; direct admission outside a lease is
; intentionally invalid. Consume and clear the private query facts as Root does.
define internal i64 @k.image.query(ptr %token) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %f0 = call i64 @t.word(ptr @topal_x86_frame_details_v1,i64 0)
  %f1 = call i64 @t.word(ptr @topal_x86_frame_details_v1,i64 1)
  %f2 = call i64 @t.word(ptr @topal_x86_frame_details_v1,i64 2)
  %f3 = call i64 @t.word(ptr @topal_x86_frame_details_v1,i64 3)
  %f01 = or i64 %f0,%f1
  %f23 = or i64 %f2,%f3
  %all = or i64 %f01,%f23
  %empty = icmp eq i64 %all,0
  call void @k.image.require(i1 %empty)
  %result = call i64 @topal_x86_frame_gateway_v1(i64 4,i64 0,i64 0,ptr %token,ptr @topal_x86_frame_details_v1)
  %ok = icmp eq i64 %result,1
  call void @k.image.require(i1 %ok)
  %frame = call i64 @t.word(ptr @topal_x86_frame_details_v1,i64 0)
  %count = call i64 @t.word(ptr @topal_x86_frame_details_v1,i64 1)
  %alignment = call i64 @t.word(ptr @topal_x86_frame_details_v1,i64 2)
  %provenance = call i64 @t.word(ptr @topal_x86_frame_details_v1,i64 3)
  %whole = icmp eq i64 %count,{{PAGES}}
  %aligned = icmp eq i64 %alignment,1
  %tagged = icmp ne i64 %provenance,0
  %shape = and i1 %whole,%aligned
  %valid = and i1 %shape,%tagged
  call void @k.image.require(i1 %valid)
  store volatile i64 0,ptr @topal_x86_frame_details_v1,align 8
  %a1 = getelementptr i64,ptr @topal_x86_frame_details_v1,i64 1
  %a2 = getelementptr i64,ptr @topal_x86_frame_details_v1,i64 2
  %a3 = getelementptr i64,ptr @topal_x86_frame_details_v1,i64 3
  store volatile i64 0,ptr %a1,align 8
  store volatile i64 0,ptr %a2,align 8
  store volatile i64 0,ptr %a3,align 8
  fence seq_cst
  %base = shl i64 %frame,12
  ret i64 %base
}
define internal {i64,i64,i64} @k.image.owners() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %bt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{BOOT_SLOT}}
  %pt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{PROCESS_SLOT}}
  %query_b = call i64 @k.image.query(ptr %bt)
  %query_p = call i64 @k.image.query(ptr %pt)
  %bootstrap = call i64 @k.image.owner(ptr %bt,i64 {{PAGES}},i64 1)
  %process = call i64 @k.image.owner(ptr %pt,i64 {{PAGES}},i64 1)
  %same_b = icmp eq i64 %bootstrap,%query_b
  %same_p = icmp eq i64 %process,%query_p
  %same = and i1 %same_b,%same_p
  call void @k.image.require(i1 %same)
  %bend = add i64 %bootstrap,{{BYTES}}
  %pend = add i64 %process,{{BYTES}}
  %left = icmp ule i64 %bend,%process
  %right = icmp ule i64 %pend,%bootstrap
  %disjoint = or i1 %left,%right
  call void @k.image.require(i1 %disjoint)
  %reservation = ptrtoint ptr @topal_x86_kernel_reservation_end_v1 to i64
  %beyond_b = icmp uge i64 %bootstrap,%reservation
  %beyond_p = icmp uge i64 %process,%reservation
  %outside = and i1 %beyond_b,%beyond_p
  call void @k.image.require(i1 %outside)
  %cr3 = call i64 @k.image.active_root()
  %active = and i64 %cr3,-4096
  %inside_b1 = icmp uge i64 %active,%bootstrap
  %inside_b2 = icmp ult i64 %active,%bend
  %inside_b = and i1 %inside_b1,%inside_b2
  %inside_p1 = icmp uge i64 %active,%process
  %inside_p2 = icmp ult i64 %active,%pend
  %inside_p = and i1 %inside_p1,%inside_p2
  %live = or i1 %inside_b,%inside_p
  %inactive = xor i1 %live,true
  call void @k.image.require(i1 %inactive)
  %a = insertvalue {i64,i64,i64} poison,i64 %bootstrap,0
  %b = insertvalue {i64,i64,i64} %a,i64 %process,1
  %c = insertvalue {i64,i64,i64} %b,i64 %cr3,2
  ret {i64,i64,i64} %c
}
define internal void @k.image.zero(i64 %base) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %p = inttoptr i64 %base to ptr
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%loop]
  %at = getelementptr i64,ptr %p,i64 %i
  store volatile i64 0,ptr %at,align 8
  %next = add i64 %i,1
  %more = icmp ult i64 %next,{{WORDS}}
  br i1 %more,label %loop,label %done
done:
  fence seq_cst
  ret void
}
; Private qualification initialization, before any zeroed-resource witness.
; This is ordinary data written under the complete live owner, not injected
; metadata or executable instructions. It makes zeroing observable on real RAM.
define internal void @k.image.seed(i64 %base) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %p = inttoptr i64 %base to ptr
  %salt = xor i64 %base,6510615555426900570
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%loop]
  %at = getelementptr i64,ptr %p,i64 %i
  %word = xor i64 %salt,%i
  store volatile i64 %word,ptr %at,align 8
  %next = add i64 %i,1
  %more = icmp ult i64 %next,{{WORDS}}
  br i1 %more,label %loop,label %done
done:
  fence seq_cst
  ret void
}
define internal void @k.image.verify_zero(i64 %base) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %p = inttoptr i64 %base to ptr
  br label %loop
loop:
  %i = phi i64 [0,%0],[%next,%loop]
  %at = getelementptr i64,ptr %p,i64 %i
  %word = load volatile i64,ptr %at,align 8
  %zero = icmp eq i64 %word,0
  call void @k.image.require(i1 %zero)
  %next = add i64 %i,1
  %more = icmp ult i64 %next,{{WORDS}}
  br i1 %more,label %loop,label %done
done:
  ret void
}
define void @topal_x86_image_backing_observe_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void asm sideeffect "", "~{memory},~{dirflag},~{fpsr},~{flags}"()
  ret void
}
define void @topal_x86_image_backing_prepared_observe_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void @topal_x86_image_backing_observe_v1()
  ret void
}
define void @topal_x86_image_backing_erased_observe_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void @topal_x86_image_backing_observe_v1()
  ret void
}
define void @topal_x86_image_backing_released_observe_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void @topal_x86_image_backing_observe_v1()
  ret void
}
define i1 @topal_x86_image_backing_prepare_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_image_backing_observation_v1,i64 0)
  %fresh = icmp eq i64 %phase,0
  call void @k.image.require(i1 %fresh)
  %owners = call {i64,i64,i64} @k.image.owners()
  %bootstrap = extractvalue {i64,i64,i64} %owners,0
  %process = extractvalue {i64,i64,i64} %owners,1
  %cr3 = extractvalue {i64,i64,i64} %owners,2
  call void @k.image.store(i64 1,i64 {{PAGES}})
  call void @k.image.store(i64 2,i64 %bootstrap)
  call void @k.image.store(i64 3,i64 %process)
  call void @k.image.store(i64 4,i64 %cr3)
  %bt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{BOOT_SLOT}}
  %pt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{PROCESS_SLOT}}
  %bid = call i64 @t.word(ptr %bt,i64 1)
  %bslot = call i64 @t.word(ptr %bt,i64 2)
  %pid = call i64 @t.word(ptr %pt,i64 1)
  %pslot = call i64 @t.word(ptr %pt,i64 2)
  call void @k.image.store(i64 5,i64 %bid)
  call void @k.image.store(i64 6,i64 %bslot)
  call void @k.image.store(i64 7,i64 %pid)
  call void @k.image.store(i64 8,i64 %pslot)
  %next = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 2)
  call void @k.image.store(i64 9,i64 %next)
  call void @k.image.seed(i64 %bootstrap)
  call void @k.image.seed(i64 %process)
  call void @topal_x86_image_backing_observe_v1()
  call void @k.image.zero(i64 %bootstrap)
  call void @k.image.zero(i64 %process)
  call void @k.image.verify_zero(i64 %bootstrap)
  call void @k.image.verify_zero(i64 %process)
  call void @k.image.store(i64 0,i64 1)
  call void @topal_x86_image_backing_prepared_observe_v1()
  ret i1 true
}
define i1 @topal_x86_image_backing_discard_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_image_backing_observation_v1,i64 0)
  %prepared = icmp eq i64 %phase,1
  call void @k.image.require(i1 %prepared)
  %owners = call {i64,i64,i64} @k.image.owners()
  %bootstrap = extractvalue {i64,i64,i64} %owners,0
  %process = extractvalue {i64,i64,i64} %owners,1
  %cr3 = extractvalue {i64,i64,i64} %owners,2
  %saved_b = call i64 @t.word(ptr @topal_x86_image_backing_observation_v1,i64 2)
  %saved_p = call i64 @t.word(ptr @topal_x86_image_backing_observation_v1,i64 3)
  %saved_cr3 = call i64 @t.word(ptr @topal_x86_image_backing_observation_v1,i64 4)
  %same_b = icmp eq i64 %bootstrap,%saved_b
  %same_p = icmp eq i64 %process,%saved_p
  %same_cr3 = icmp eq i64 %cr3,%saved_cr3
  %same_bp = and i1 %same_b,%same_p
  %same = and i1 %same_bp,%same_cr3
  call void @k.image.require(i1 %same)
  call void @k.image.verify_zero(i64 %bootstrap)
  call void @k.image.verify_zero(i64 %process)
  call void @k.image.zero(i64 %bootstrap)
  call void @k.image.zero(i64 %process)
  call void @k.image.verify_zero(i64 %bootstrap)
  call void @k.image.verify_zero(i64 %process)
  call void @k.image.store(i64 0,i64 2)
  call void @topal_x86_image_backing_erased_observe_v1()
  ret i1 true
}
define i1 @topal_x86_image_backing_released_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_image_backing_observation_v1,i64 0)
  %erased = icmp eq i64 %phase,2
  call void @k.image.require(i1 %erased)
  %bt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{BOOT_SLOT}}
  %pt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{PROCESS_SLOT}}
  %b0 = call i64 @t.word(ptr %bt,i64 0)
  %b1 = call i64 @t.word(ptr %bt,i64 1)
  %b2 = call i64 @t.word(ptr %bt,i64 2)
  %p0 = call i64 @t.word(ptr %pt,i64 0)
  %p1 = call i64 @t.word(ptr %pt,i64 1)
  %p2 = call i64 @t.word(ptr %pt,i64 2)
  %b01 = or i64 %b0,%b1
  %b = or i64 %b01,%b2
  %p01 = or i64 %p0,%p1
  %p = or i64 %p01,%p2
  %all = or i64 %b,%p
  %clear = icmp eq i64 %all,0
  call void @k.image.require(i1 %clear)
  %next = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 2)
  %saved = call i64 @t.word(ptr @topal_x86_image_backing_observation_v1,i64 9)
  %retained = icmp eq i64 %next,%saved
  call void @k.image.require(i1 %retained)
  call void @k.image.store(i64 0,i64 3)
  call void @topal_x86_image_backing_released_observe_v1()
  ret i1 true
}
