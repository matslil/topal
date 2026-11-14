; Whole-owner inactive table construction; never writes CPU controls.
define void @topal_x86_image_tables_observe_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void asm sideeffect "", "~{memory},~{dirflag},~{fpsr},~{flags}"()
  ret void
}
define internal {i64,i64,i64,i64} @k.tables.owners() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %banks = call {i64,i64,i64} @k.image.owners()
  %b = extractvalue {i64,i64,i64} %banks,0
  %p = extractvalue {i64,i64,i64} %banks,1
  %cr3 = extractvalue {i64,i64,i64} %banks,2
  %token = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{LOW_SLOT}}
  %query = call i64 @k.image.query(ptr %token,i64 1)
  %low = call i64 @k.image.owner(ptr %token,i64 1,i64 1)
  %same = icmp eq i64 %low,%query
  call void @k.image.require(i1 %same)
  %low_end = add i64 %low,4096
  %bend = add i64 %b,{{BYTES}}
  %pend = add i64 %p,{{BYTES}}
  %bl = icmp ule i64 %low_end,%b
  %br = icmp uge i64 %low,%bend
  %pl = icmp ule i64 %low_end,%p
  %pr = icmp uge i64 %low,%pend
  %bd = or i1 %bl,%br
  %pd = or i1 %pl,%pr
  %disjoint = and i1 %bd,%pd
  %reservation = ptrtoint ptr @topal_x86_kernel_reservation_end_v1 to i64
  %outside = icmp uge i64 %low,%reservation
  %active = and i64 %cr3,-4096
  %before = icmp ult i64 %active,%low
  %after = icmp uge i64 %active,%low_end
  %inactive = or i1 %before,%after
  %a = and i1 %disjoint,%outside
  %ok = and i1 %a,%inactive
  call void @k.image.require(i1 %ok)
  %x0 = insertvalue {i64,i64,i64,i64} poison,i64 %b,0
  %x1 = insertvalue {i64,i64,i64,i64} %x0,i64 %p,1
  %x2 = insertvalue {i64,i64,i64,i64} %x1,i64 %low,2
  %x3 = insertvalue {i64,i64,i64,i64} %x2,i64 %cr3,3
  ret {i64,i64,i64,i64} %x3
}
define internal void @k.tables.identity(ptr %token,i64 %index,i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %id = call i64 @t.word(ptr %token,i64 1)
  %slot = call i64 @t.word(ptr %token,i64 2)
  %next = add i64 %index,1
  br i1 %save,label %initial,label %check
initial:
  call void @k.image.store(i64 %index,i64 %id)
  call void @k.image.store(i64 %next,i64 %slot)
  ret void
check:
  %saved_id = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 %index)
  %saved_slot = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 %next)
  %si = icmp eq i64 %id,%saved_id
  %ss = icmp eq i64 %slot,%saved_slot
  %same = and i1 %si,%ss
  call void @k.image.require(i1 %same)
  ret void
}
define internal void @k.tables.verify(i64 %b,i64 %p,i64 %low) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %bp = inttoptr i64 %b to ptr
  %pp = inttoptr i64 %p to ptr
  %bv = call i1 @topal_x86_image_table_verify_data_v1(ptr %bp,i64 %b,i64 0,i64 0)
  %pv = call i1 @topal_x86_image_table_verify_data_v1(ptr %pp,i64 %p,i64 1,i64 %low)
  %ok = and i1 %bv,%pv
  call void @k.image.require(i1 %ok)
  call void @k.image.verify_zero(i64 %low,i64 512)
  ret void
}
define internal void @k.tables.erase(i64 %b,i64 %p,i64 %low) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void @k.image.zero(i64 %b,i64 {{WORDS}})
  call void @k.image.zero(i64 %p,i64 {{WORDS}})
  call void @k.image.zero(i64 %low,i64 512)
  call void @k.image.verify_zero(i64 %b,i64 {{WORDS}})
  call void @k.image.verify_zero(i64 %p,i64 {{WORDS}})
  call void @k.image.verify_zero(i64 %low,i64 512)
  ret void
}
define i64 @topal_x86_image_tables_prepare_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 0)
  %fresh = icmp eq i64 %phase,0
  call void @k.image.require(i1 %fresh)
  %owners = call {i64,i64,i64,i64} @k.tables.owners()
  call void @k.boot.verify(i1 true)
  %b = extractvalue {i64,i64,i64,i64} %owners,0
  %p = extractvalue {i64,i64,i64,i64} %owners,1
  %low = extractvalue {i64,i64,i64,i64} %owners,2
  %cr3 = extractvalue {i64,i64,i64,i64} %owners,3
  call void @k.image.store(i64 1,i64 {{PAGES}})
  call void @k.image.store(i64 2,i64 %b)
  call void @k.image.store(i64 3,i64 %p)
  call void @k.image.store(i64 4,i64 %cr3)
  call void @k.image.store(i64 10,i64 %low)
  %bt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{BOOT_SLOT}}
  %pt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{PROCESS_SLOT}}
  %lt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{LOW_SLOT}}
  call void @k.tables.identity(ptr %bt,i64 5,i1 true)
  call void @k.tables.identity(ptr %pt,i64 7,i1 true)
  call void @k.tables.identity(ptr %lt,i64 11,i1 true)
  %next = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 2)
  call void @k.image.store(i64 9,i64 %next)
  call void @k.image.seed(i64 %b,i64 {{WORDS}})
  call void @k.image.seed(i64 %p,i64 {{WORDS}})
  call void @k.image.seed(i64 %low,i64 512)
  call void @topal_x86_image_tables_observe_v1()
  call void @k.tables.erase(i64 %b,i64 %p,i64 %low)
  call void @k.image.store(i64 0,i64 1)
  call void @topal_x86_image_tables_observe_v1()
  %bp = inttoptr i64 %b to ptr
  %pp = inttoptr i64 %p to ptr
  %bc = call i1 @topal_x86_image_table_prepare_data_v1(ptr %bp,i64 {{BYTES}},i64 %b,i64 0,i64 0)
  call void @k.image.require(i1 %bc)
  %pc = call i1 @topal_x86_image_table_prepare_data_v1(ptr %pp,i64 {{BYTES}},i64 %p,i64 1,i64 %low)
  call void @k.image.require(i1 %pc)
  fence seq_cst
  call void @k.tables.verify(i64 %b,i64 %p,i64 %low)
  call void @k.image.store(i64 0,i64 2)
  call void @topal_x86_image_tables_observe_v1()
  ret i64 1
}
define i64 @topal_x86_image_tables_discard_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 0)
  %prepared = icmp eq i64 %phase,2
  call void @k.image.require(i1 %prepared)
  %owners = call {i64,i64,i64,i64} @k.tables.owners()
  call void @k.boot.verify(i1 false)
  %b = extractvalue {i64,i64,i64,i64} %owners,0
  %p = extractvalue {i64,i64,i64,i64} %owners,1
  %low = extractvalue {i64,i64,i64,i64} %owners,2
  %cr3 = extractvalue {i64,i64,i64,i64} %owners,3
  %sb = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 2)
  %sp = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 3)
  %sl = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 10)
  %sc = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 4)
  %eqb = icmp eq i64 %b,%sb
  %eqp = icmp eq i64 %p,%sp
  %eql = icmp eq i64 %low,%sl
  %eqc = icmp eq i64 %cr3,%sc
  %a = and i1 %eqb,%eqp
  %z = and i1 %eql,%eqc
  %same = and i1 %a,%z
  call void @k.image.require(i1 %same)
  %bt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{BOOT_SLOT}}
  %pt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{PROCESS_SLOT}}
  %lt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{LOW_SLOT}}
  call void @k.tables.identity(ptr %bt,i64 5,i1 false)
  call void @k.tables.identity(ptr %pt,i64 7,i1 false)
  call void @k.tables.identity(ptr %lt,i64 11,i1 false)
  call void @k.tables.verify(i64 %b,i64 %p,i64 %low)
  call void @k.tables.erase(i64 %b,i64 %p,i64 %low)
  call void @k.image.store(i64 0,i64 3)
  call void @topal_x86_image_tables_observe_v1()
  ret i64 1
}
define internal void @k.tables.empty(ptr %token) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %a = call i64 @t.word(ptr %token,i64 0)
  %b = call i64 @t.word(ptr %token,i64 1)
  %c = call i64 @t.word(ptr %token,i64 2)
  %ab = or i64 %a,%b
  %all = or i64 %ab,%c
  %ok = icmp eq i64 %all,0
  call void @k.image.require(i1 %ok)
  ret void
}
define i64 @topal_x86_image_tables_released_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 0)
  %erased = icmp eq i64 %phase,3
  call void @k.image.require(i1 %erased)
  %bt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{BOOT_SLOT}}
  %pt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{PROCESS_SLOT}}
  %lt = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{LOW_SLOT}}
  call void @k.tables.empty(ptr %bt)
  call void @k.tables.empty(ptr %pt)
  call void @k.tables.empty(ptr %lt)
  %next = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 2)
  %saved = call i64 @t.word(ptr @topal_x86_image_tables_observation_v1,i64 9)
  %retained = icmp eq i64 %next,%saved
  call void @k.image.require(i1 %retained)
  call void @k.image.store(i64 0,i64 4)
  call void @topal_x86_image_tables_observe_v1()
  ret i64 1
}
