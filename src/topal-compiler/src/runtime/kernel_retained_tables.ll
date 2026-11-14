; TOPAL-SYSTEMS-KERNEL-IMAGE-RETAIN-001. No activation or control writes.
@topal_x86_retained_tables_v1 = global [32 x i64] zeroinitializer, align 8

define internal void @r.fact(i64 %index,i64 %value,i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %at = getelementptr i64,ptr @topal_x86_retained_tables_v1,i64 %index
  br i1 %save,label %store,label %check
store:
  store volatile i64 %value,ptr %at,align 8
  ret void
check:
  %saved = load volatile i64,ptr %at,align 8
  %same = icmp eq i64 %saved,%value
  call void @k.image.require(i1 %same)
  ret void
}
define internal void @r.owner(ptr %token,i64 %index,i64 %base,i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  ; Called only after complete token/registry/cell/whole/pin validation.
  %id = call i64 @t.word(ptr %token,i64 1)
  %slot = call i64 @t.word(ptr %token,i64 2)
  %offset = mul i64 %slot,64
  %record = getelementptr i8,ptr @topal_x86_frame_registry_v1,i64 %offset
  %generation = call i64 @t.word(ptr %record,i64 5)
  call void @r.fact(i64 %index,i64 %base,i1 %save)
  %i1 = add i64 %index,1
  call void @r.fact(i64 %i1,i64 %id,i1 %save)
  %i2 = add i64 %index,2
  call void @r.fact(i64 %i2,i64 %slot,i1 %save)
  %i3 = add i64 %index,3
  call void @r.fact(i64 %i3,i64 %generation,i1 %save)
  ret void
}
define internal {i64,i64,i64} @r.verify(i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_retained_tables_v1,i64 0)
  %wanted = select i1 %save,i64 0,i64 1
  %ordered = icmp eq i64 %phase,%wanted
  call void @k.image.require(i1 %ordered)
  %owners = call {i64,i64,i64,i64} @k.tables.owners()
  %b = extractvalue {i64,i64,i64,i64} %owners,0
  %p = extractvalue {i64,i64,i64,i64} %owners,1
  %low = extractvalue {i64,i64,i64,i64} %owners,2
  %root = extractvalue {i64,i64,i64,i64} %owners,3
  %issuer = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 1)
  call void @r.fact(i64 1,i64 %issuer,i1 %save)
  call void @r.fact(i64 6,i64 {{PAGES}},i1 %save)
  %bt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{BOOT_SLOT}}
  %pt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{PROCESS_SLOT}}
  %lt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{LOW_SLOT}}
  call void @r.owner(ptr %bt,i64 8,i64 %b,i1 %save)
  call void @r.owner(ptr %pt,i64 12,i64 %p,i1 %save)
  call void @r.owner(ptr %lt,i64 16,i64 %low,i1 %save)
  %next = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 2)
  br i1 %save,label %initial,label %retirement
initial:
  call void @r.fact(i64 2,i64 %next,i1 true)
  call void @r.fact(i64 4,i64 %root,i1 true)
  br label %done
retirement:
  %first = call i64 @t.word(ptr @topal_x86_retained_tables_v1,i64 2)
  %monotonic = icmp uge i64 %next,%first
  call void @k.image.require(i1 %monotonic)
  call void @r.fact(i64 3,i64 %next,i1 true)
  call void @r.fact(i64 5,i64 %root,i1 true)
  br label %done
done:
  %a0 = insertvalue {i64,i64,i64} poison,i64 %b,0
  %a1 = insertvalue {i64,i64,i64} %a0,i64 %p,1
  %a2 = insertvalue {i64,i64,i64} %a1,i64 %low,2
  ret {i64,i64,i64} %a2
}
define i64 @topal_x86_retained_tables_discard_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 0)
  %retained = icmp eq i64 %phase,3
  call void @k.image.require(i1 %retained)
  %owners = call {i64,i64,i64} @r.verify(i1 false)
  %b = extractvalue {i64,i64,i64} %owners,0
  %p = extractvalue {i64,i64,i64} %owners,1
  %low = extractvalue {i64,i64,i64} %owners,2
  call void @k.tables.verify(i64 %b,i64 %p,i64 %low,i1 false)
  call void @k.image.store(i64 0,i64 4)
  call void @topal_x86_image_tables_observe_v1()
  call void @k.tables.erase(i64 %b,i64 %p,i64 %low)
  store volatile i64 2,ptr @topal_x86_retained_tables_v1,align 8
  call void @k.image.store(i64 0,i64 5)
  call void @topal_x86_image_tables_observe_v1()
  ret i64 1
}
define i64 @topal_x86_retained_tables_released_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 0)
  %erased = icmp eq i64 %phase,5
  call void @k.image.require(i1 %erased)
  %retained_phase = call i64 @t.word(ptr @topal_x86_retained_tables_v1,i64 0)
  %retired = icmp eq i64 %retained_phase,2
  call void @k.image.require(i1 %retired)
  %bt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{BOOT_SLOT}}
  %pt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{PROCESS_SLOT}}
  %lt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{LOW_SLOT}}
  call void @k.tables.empty(ptr %bt)
  call void @k.tables.empty(ptr %pt)
  call void @k.tables.empty(ptr %lt)
  %issuer = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 1)
  call void @r.fact(i64 1,i64 %issuer,i1 false)
  %next = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 2)
  call void @r.fact(i64 3,i64 %next,i1 false)
  store volatile i64 3,ptr @topal_x86_retained_tables_v1,align 8
  call void @k.image.store(i64 0,i64 6)
  call void @topal_x86_image_tables_observe_v1()
  ret i64 1
}
