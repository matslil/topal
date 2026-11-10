; Private actual-root target. This component alone grants no publication route.
@topal_x86_user_extent_return_v1 = external global i8

define internal void @u.require(i1 %ok) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  br i1 %ok,label %end,label %bad
bad:
  call void @topal_x86_user_extent_fatal_v1()
  unreachable
end:
  ret void
}
define internal i64 @u.msr(i32 %number) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %halves = call {i32,i32} asm sideeffect "rdmsr", "={ax},={dx},{cx},~{dirflag},~{fpsr},~{flags}"(i32 %number)
  %a = extractvalue {i32,i32} %halves,0
  %b = extractvalue {i32,i32} %halves,1
  %lo = zext i32 %a to i64
  %hi = zext i32 %b to i64
  %shift = shl i64 %hi,32
  %word = or i64 %lo,%shift
  ret i64 %word
}

; Called by g.lease for operation 8; must not recurse into g.lease itself.
; The exact emitted site and live captured entry establish caller identity;
; stack geometry alone cannot create the lease. Complete outer caller costs
; remain a separate proof before all-caller integration.
define internal i1 @u.syscall_context(i64 %saved,i64 %identity) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %ctx = getelementptr i8,ptr @topal_bootstrap_storage,i64 512
  %state = call i64 @t.word(ptr %ctx,i64 0)
  %self = call i64 @t.word(ptr %ctx,i64 32)
  %stack = call i64 @t.word(ptr %ctx,i64 1)
  %root = call i64 @t.word(ptr %ctx,i64 3)
  %epoch = call i64 @t.word(ptr @topal_x86_frame_mm_epoch_v1,i64 0)
  %gs = call i64 @u.msr(i32 -1073741567)
  %cr3 = call i64 asm sideeffect "movq %cr3, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  %live = icmp eq i64 %state,1
  %ctx_word = ptrtoint ptr %ctx to i64
  %self_ok = icmp eq i64 %self,%ctx_word
  %gs_ok = icmp eq i64 %gs,%ctx_word
  %root_ok = icmp eq i64 %root,%cr3
  %epoch_ok = icmp eq i64 %epoch,%identity
  %nonzero = icmp ne i64 %identity,0
  %caller = add i64 %saved,64
  %nowrap = icmp ugt i64 %caller,%saved
  %inside = icmp ule i64 %caller,%stack
  %top_owned = icmp ule i64 %stack,1572864
  %top_low = icmp uge i64 %stack,1085456
  %saved_low = icmp uge i64 %saved,1085456
  %saved_high = icmp ule i64 %saved,1572808
  %align_bits = and i64 %saved,15
  %aligned = icmp eq i64 %align_bits,8
  %a = and i1 %live,%self_ok
  %b = and i1 %gs_ok,%root_ok
  %c = and i1 %epoch_ok,%nonzero
  %d = and i1 %nowrap,%inside
  %e = and i1 %top_owned,%top_low
  %f = and i1 %saved_low,%saved_high
  %ab = and i1 %a,%b
  %cd = and i1 %c,%d
  %ef = and i1 %e,%f
  %abcd = and i1 %ab,%cd
  %rest = and i1 %ef,%aligned
  %safe = and i1 %abcd,%rest
  br i1 %safe,label %site,label %refuse
site:
  call void asm sideeffect "lfence", "~{memory},~{dirflag},~{fpsr},~{flags}"()
  %return_word = add i64 %saved,48
  %return_at = inttoptr i64 %return_word to ptr
  %actual_site = load volatile i64,ptr %return_at,align 8
  %site_ok = icmp eq i64 %actual_site,ptrtoint (ptr @topal_x86_user_extent_return_v1 to i64)
  ret i1 %site_ok
refuse:
  ret i1 false
}
define internal void @u.check(i64 %identity) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %admit = call i1 @topal_frame_kernel_admit_v1(ptr @topal_x86_frame_context_v1,i64 0,i64 0,i64 0)
  call void @u.require(i1 %admit)
  %op = call i64 @t.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 2)
  %entry = call i64 @t.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 5)
  %known = icmp eq i64 %op,8
  %same = icmp eq i64 %entry,%identity
  %nonzero = icmp ne i64 %identity,0
  %a = and i1 %known,%same
  %ok = and i1 %a,%nonzero
  call void @u.require(i1 %ok)
  ret void
}

; Read-only whole-token/registry/cell validation. No resolve fact staging and no
; source call. The source/controller protocol preserves normalized physical and
; provenance columns; all token pointers below are private compiled bindings.
define internal i64 @u.owner(ptr %token,i64 %count,i64 %pins) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %n = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 3)
  %issuer = call i64 @t.word(ptr %token,i64 0)
  %id = call i64 @t.word(ptr %token,i64 1)
  %slot = call i64 @t.word(ptr %token,i64 2)
  %next = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 2)
  %issuer_ok = icmp eq i64 %issuer,1
  %id_nonzero = icmp ne i64 %id,0
  %id_issued = icmp ult i64 %id,%next
  %inside = icmp ult i64 %slot,%n
  %positive = icmp ne i64 %count,0
  %a = and i1 %issuer_ok,%id_nonzero
  %b = and i1 %id_issued,%inside
  %ab = and i1 %a,%b
  %valid = and i1 %ab,%positive
  call void @u.require(i1 %valid)
  call void asm sideeffect "lfence", "~{memory},~{dirflag},~{fpsr},~{flags}"()
  %offset = mul i64 %slot,64
  %record = getelementptr i8,ptr @topal_x86_frame_registry_v1,i64 %offset
  %r0 = call i64 @t.word(ptr %record,i64 0)
  %r1 = call i64 @t.word(ptr %record,i64 1)
  %start = call i64 @t.word(ptr %record,i64 2)
  %r3 = call i64 @t.word(ptr %record,i64 3)
  %r4 = call i64 @t.word(ptr %record,i64 4)
  %provenance = call i64 @t.word(ptr %record,i64 5)
  %r6 = call i64 @t.word(ptr %record,i64 6)
  %r7 = call i64 @t.word(ptr %record,i64 7)
  %end = add i64 %start,%count
  %end_wrap = icmp ugt i64 %end,%start
  %end_inside = icmp ule i64 %end,%n
  %same_issuer = icmp eq i64 %r0,%issuer
  %same_id = icmp eq i64 %r1,%id
  %same_count = icmp eq i64 %r3,%count
  %alignment = icmp eq i64 %r4,1
  %tag = icmp ne i64 %provenance,0
  %pinned = icmp eq i64 %r6,%pins
  %live = icmp eq i64 %r7,1
  %c = and i1 %end_wrap,%end_inside
  %d = and i1 %same_issuer,%same_id
  %e = and i1 %same_count,%alignment
  %f = and i1 %tag,%pinned
  %cd = and i1 %c,%d
  %ef = and i1 %e,%f
  %cdef = and i1 %cd,%ef
  %registry_valid = and i1 %cdef,%live
  call void @u.require(i1 %registry_valid)
  call void asm sideeffect "lfence", "~{memory},~{dirflag},~{fpsr},~{flags}"()
  %cell_offset = mul i64 %start,24
  %first_offset = add i64 %cell_offset,128
  %first = getelementptr i8,ptr @topal_x86_frame_table_v1,i64 %first_offset
  %frame = call i64 @t.word(ptr %first,i64 2)
  %last_frame = add i64 %frame,%count
  %frame_wrap = icmp ugt i64 %last_frame,%frame
  %accessible = icmp ule i64 %last_frame,262144
  %physical_valid = and i1 %frame_wrap,%accessible
  call void @u.require(i1 %physical_valid)
  br label %cells
cells:
  %i = phi i64 [0,%entry],[%after,%cell]
  %more = icmp ult i64 %i,%count
  br i1 %more,label %cell,label %done
cell:
  %delta = mul i64 %i,24
  %at = getelementptr i8,ptr %first,i64 %delta
  %actual_id = call i64 @t.word(ptr %at,i64 0)
  %actual_tag = call i64 @t.word(ptr %at,i64 1)
  %actual_frame = call i64 @t.word(ptr %at,i64 2)
  %expected_frame = add i64 %frame,%i
  %id_ok = icmp eq i64 %actual_id,%id
  %tag_ok = icmp eq i64 %actual_tag,%provenance
  %frame_ok = icmp eq i64 %actual_frame,%expected_frame
  %both = and i1 %id_ok,%tag_ok
  %cell_valid = and i1 %both,%frame_ok
  call void @u.require(i1 %cell_valid)
  %after = add i64 %i,1
  br label %cells
done:
  %base = shl i64 %frame,12
  ret i64 %base
}

define internal i64 @u.static_owner(i64 %role) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  switch i64 %role,label %bad [i64 0,label %hierarchy i64 1,label %low i64 2,label %image i64 3,label %stack]
hierarchy:
  %h = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{HIERARCHY_OFFSET}}
  %hv = call i64 @u.owner(ptr %h,i64 5,i64 1)
  ret i64 %hv
low:
  %l = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{LOW_OFFSET}}
  %lv = call i64 @u.owner(ptr %l,i64 1,i64 1)
  ret i64 %lv
image:
  %m = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{IMAGE_OFFSET}}
  %mv = call i64 @u.owner(ptr %m,i64 503,i64 1)
  ret i64 %mv
stack:
  %s = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 {{STACK_OFFSET}}
  %sv = call i64 @u.owner(ptr %s,i64 2,i64 2)
  ret i64 %sv
bad:
  call void @topal_x86_user_extent_fatal_v1()
  unreachable
}

define i64 @topal_x86_user_extent_begin_v1() nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %identity = call i64 @t.word(ptr @topal_x86_frame_gateway_recovery_v1,i64 5)
  call void @u.check(i64 %identity)
  %cr0 = call i64 asm sideeffect "movq %cr0, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  %cr4 = call i64 asm sideeffect "movq %cr4, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  %efer = call i64 @u.msr(i32 -1073741696)
  %apic = call i64 @u.msr(i32 27)
  %required_cr0 = and i64 %cr0,2147549185
  %cr0_ok = icmp eq i64 %required_cr0,2147549185
  ; LA57, PCID, SMAP, PKE, CET and PKS need distinct qualification.
  %extra_modes = and i64 %cr4,31592448
  %cr4_ok = icmp eq i64 %extra_modes,0
  %required_efer = and i64 %efer,3328
  %efer_ok = icmp eq i64 %required_efer,3328
  %bsp_bit = and i64 %apic,256
  %bsp = icmp ne i64 %bsp_bit,0
  %a = and i1 %cr0_ok,%cr4_ok
  %b = and i1 %efer_ok,%bsp
  %mode = and i1 %a,%b
  %result = select i1 %mode,i64 %identity,i64 0
  ret i64 %result
}
define i64 @topal_x86_user_extent_root_v1(i64 %identity) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void @u.check(i64 %identity)
  %owned = call i64 @u.static_owner(i64 0)
  %ctx = getelementptr i8,ptr @topal_bootstrap_storage,i64 512
  %actual = call i64 @t.word(ptr %ctx,i64 3)
  %same = icmp eq i64 %owned,%actual
  call void @u.require(i1 %same)
  ret i64 %owned
}
define ptr @topal_x86_user_extent_table_v1(i64 %identity,i64 %root,i64 %physical,i64 %level) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %current = call i64 @topal_x86_user_extent_root_v1(i64 %identity)
  %same = icmp eq i64 %root,%current
  call void @u.require(i1 %same)
  %low = call i64 @u.static_owner(i64 1)
  %l4 = icmp eq i64 %level,4
  %p4 = icmp eq i64 %physical,%root
  %a = and i1 %l4,%p4
  %p3word = add i64 %root,4096
  %l3 = icmp eq i64 %level,3
  %p3 = icmp eq i64 %physical,%p3word
  %b = and i1 %l3,%p3
  %p2word = add i64 %root,8192
  %p2other = add i64 %root,12288
  %l2 = icmp eq i64 %level,2
  %p2a = icmp eq i64 %physical,%p2word
  %p2b = icmp eq i64 %physical,%p2other
  %p2 = or i1 %p2a,%p2b
  %c = and i1 %l2,%p2
  %high = add i64 %root,16384
  %l1 = icmp eq i64 %level,1
  %p1a = icmp eq i64 %physical,%low
  %p1b = icmp eq i64 %physical,%high
  %p1 = or i1 %p1a,%p1b
  %d = and i1 %l1,%p1
  %ab = or i1 %a,%b
  %cd = or i1 %c,%d
  %valid = or i1 %ab,%cd
  %pointer = inttoptr i64 %physical to ptr
  %result = select i1 %valid,ptr %pointer,ptr null
  ret ptr %result
}

; A runtime heap allocation is distinct from every statically bound root
; allocation, including retained translation storage. Issued identities are
; monotonic within the single validated allocator issuer. Historical root slots
; can only cause rejection; they cannot supply backing authority.
define internal i64 @u.heap_owner(i64 %count) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %base = call i64 @u.owner(ptr @topal_x86_frame_mm_tokens_v1,i64 %count,i64 1)
  %identity = call i64 @t.word(ptr @topal_x86_frame_mm_tokens_v1,i64 1)
  br label %slots
slots:
  %i = phi i64 [0,%entry],[%next,%slot]
  %more = icmp ult i64 %i,{{ROOT_TOKEN_SLOTS}}
  br i1 %more,label %slot,label %done
slot:
  %offset = mul i64 %i,24
  %token = getelementptr i8,ptr @topal_x86_frame_root_tokens_v1,i64 %offset
  %root_identity = call i64 @t.word(ptr %token,i64 1)
  %distinct = icmp ne i64 %identity,%root_identity
  call void @u.require(i1 %distinct)
  %next = add i64 %i,1
  br label %slots
done:
  ret i64 %base
}

define i64 @topal_x86_user_extent_backing_v1(i64 %identity,i64 %root,i64 %virtual,i64 %physical,i64 %write) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %current = call i64 @topal_x86_user_extent_root_v1(i64 %identity)
  %same = icmp eq i64 %root,%current
  call void @u.require(i1 %same)
  %image_low = icmp uge i64 %virtual,4194304
  %image_high = icmp ult i64 %virtual,6254592
  %image = and i1 %image_low,%image_high
  br i1 %image,label %image_owner,label %stack_range
image_owner:
  %image_base = call i64 @u.static_owner(i64 2)
  %image_offset = sub i64 %virtual,4194304
  %image_expected = add i64 %image_base,%image_offset
  %image_same = icmp eq i64 %physical,%image_expected
  call void @u.require(i1 %image_same)
  %ctx = getelementptr i8,ptr @topal_bootstrap_storage,i64 512
  %phase_at = getelementptr i8,ptr %ctx,i64 104
  %phase = load volatile i8,ptr %phase_at,align 1
  %relro = icmp uge i8 %phase,9
  %write_start = select i1 %relro,i64 6213632,i64 6180864
  %image_writable = icmp uge i64 %virtual,%write_start
  %output = icmp eq i64 %write,1
  %readonly = xor i1 %image_writable,true
  %denied = and i1 %output,%readonly
  %image_result = select i1 %denied,i64 1,i64 0
  ret i64 %image_result
stack_range:
  %stack_low = icmp uge i64 %virtual,8380416
  %stack_high = icmp ult i64 %virtual,8388608
  %stack = and i1 %stack_low,%stack_high
  br i1 %stack,label %stack_owner,label %heap
stack_owner:
  %stack_base = call i64 @u.static_owner(i64 3)
  %stack_offset = sub i64 %virtual,8380416
  %stack_expected = add i64 %stack_base,%stack_offset
  %stack_same = icmp eq i64 %physical,%stack_expected
  call void @u.require(i1 %stack_same)
  ret i64 0
heap:
  %state = call i64 @t.word(ptr @topal_x86_frame_mm_state_v1,i64 0)
  %count = call i64 @t.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %base = call i64 @t.word(ptr @topal_x86_frame_mm_state_v1,i64 3)
  %live = icmp eq i64 %state,1
  %positive = icmp ne i64 %count,0
  %bounded = icmp ule i64 %count,519
  %size = shl i64 %count,12
  %end = add i64 %size,6254592
  %heap_low = icmp uge i64 %virtual,6254592
  %heap_high = icmp ult i64 %virtual,%end
  %a = and i1 %live,%positive
  %b = and i1 %bounded,%heap_low
  %ab = and i1 %a,%b
  %valid = and i1 %ab,%heap_high
  call void @u.require(i1 %valid)
  %heap_base = call i64 @u.heap_owner(i64 %count)
  %heap_offset = sub i64 %virtual,6254592
  %heap_expected = add i64 %heap_base,%heap_offset
  %heap_same = icmp eq i64 %physical,%heap_expected
  %base_same = icmp eq i64 %base,%heap_base
  %ok = and i1 %heap_same,%base_same
  call void @u.require(i1 %ok)
  ret i64 0
}
; Validation closes here; actual entry/table/backing owners must remain held
; through the caller's copy. A future target must not drop its mapping lock here.
define void @topal_x86_user_extent_end_v1(i64 %identity,i64 %root) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %actual = call i64 @topal_x86_user_extent_root_v1(i64 %identity)
  %same = icmp eq i64 %actual,%root
  call void @u.require(i1 %same)
  ret void
}
define void @topal_x86_user_extent_fatal_v1() noreturn nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  call void @topal_frame_kernel_fatal_v1(ptr @topal_x86_frame_context_v1)
  unreachable
}
