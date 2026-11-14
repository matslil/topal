; Private translation/ABI realization. Physical selection remains ordinary Topal.
@topal_x86_frame_mm_tokens_v1 = global [6 x i64] zeroinitializer, align 8
@topal_x86_frame_mm_state_v1 = global [4 x i64] zeroinitializer, align 8
@topal_x86_frame_mm_epoch_v1 = global [2 x i64] zeroinitializer, align 8
@topal_x86_frame_details_v1 = external global [4 x i64]
@topal_bootstrap_storage = external global [0 x i8]
declare i1 @topal_frame_kernel_admit_v1(ptr,i64,i64,i64)
declare void @topal_frame_kernel_fatal_v1(ptr) noreturn

define internal i64 @m.word(ptr %p,i64 %n) nounwind noinline noredzone {
  %at = getelementptr i64,ptr %p,i64 %n
  %v = load volatile i64,ptr %at,align 8
  ret i64 %v
}
define internal void @m.put(ptr %p,i64 %n,i64 %v) nounwind noinline noredzone {
  %at = getelementptr i64,ptr %p,i64 %n
  store volatile i64 %v,ptr %at,align 8
  ret void
}
define internal void @m.require(i1 %ok) nounwind noinline noredzone {
  br i1 %ok,label %end,label %bad
bad:
  store volatile i64 3,ptr @topal_x86_frame_mm_state_v1,align 8
  call void @topal_frame_kernel_fatal_v1(ptr @topal_x86_frame_context_v1)
  unreachable
end:
  ret void
}
define internal ptr @m.context() nounwind noinline noredzone {
  %p = getelementptr i8,ptr @topal_bootstrap_storage,i64 512
  ret ptr %p
}
define internal void @m.clean_facts() nounwind noinline noredzone {
  call void @m.put(ptr @topal_x86_frame_details_v1,i64 0,i64 0)
  call void @m.put(ptr @topal_x86_frame_details_v1,i64 1,i64 0)
  call void @m.put(ptr @topal_x86_frame_details_v1,i64 2,i64 0)
  call void @m.put(ptr @topal_x86_frame_details_v1,i64 3,i64 0)
  ret void
}
define internal i64 @m.resolve(ptr %token,i64 %count,i64 %pins) nounwind noinline noredzone {
  %ok = call i1 @topal_frame_allocator_resolve_v1(ptr @topal_x86_frame_context_v1,ptr %token,ptr @topal_x86_frame_details_v1)
  call void @m.require(i1 %ok)
  %frame = call i64 @m.word(ptr @topal_x86_frame_details_v1,i64 0)
  %length = call i64 @m.word(ptr @topal_x86_frame_details_v1,i64 1)
  %alignment = call i64 @m.word(ptr @topal_x86_frame_details_v1,i64 2)
  %provenance = call i64 @m.word(ptr @topal_x86_frame_details_v1,i64 3)
  %same = icmp eq i64 %count,%length
  %one = icmp eq i64 %alignment,1
  %tag = icmp ne i64 %provenance,0
  %a = and i1 %same,%one
  %valid = and i1 %a,%tag
  call void @m.require(i1 %valid)
  %index = call i64 @m.word(ptr %token,i64 2)
  %registryWord = call i64 @m.word(ptr @topal_x86_frame_context_v1,i64 6)
  %registry = inttoptr i64 %registryWord to ptr
  %offset = mul i64 %index,8
  %record = getelementptr i64,ptr %registry,i64 %offset
  %actual = call i64 @m.word(ptr %record,i64 6)
  %pinned = icmp eq i64 %pins,%actual
  call void @m.require(i1 %pinned)
  call void @m.clean_facts()
  %base = shl i64 %frame,12
  ret i64 %base
}
define internal ptr @m.pte(i64 %index) nounwind noinline noredzone {
  %ctx = call ptr @m.context()
  %low = icmp ult i64 %index,9
  br i1 %low,label %first,label %second
first:
  %loWord = call i64 @m.word(ptr %ctx,i64 14)
  %lo = inttoptr i64 %loWord to ptr
  %loIndex = add i64 %index,503
  %a = getelementptr i64,ptr %lo,i64 %loIndex
  ret ptr %a
second:
  %root = call i64 @m.word(ptr %ctx,i64 3)
  %highWord = add i64 %root,16384
  %hi = inttoptr i64 %highWord to ptr
  %hiIndex = sub i64 %index,9
  %b = getelementptr i64,ptr %hi,i64 %hiIndex
  ret ptr %b
}
define internal void @m.hierarchy() nounwind noinline noredzone {
  %ctx = call ptr @m.context()
  %rootWord = call i64 @m.word(ptr %ctx,i64 3)
  %lowWord = call i64 @m.word(ptr %ctx,i64 14)
  %root = inttoptr i64 %rootWord to ptr
  %pml4 = call i64 @m.word(ptr %root,i64 0)
  %pdpt = call i64 @m.word(ptr %root,i64 512)
  %low = call i64 @m.word(ptr %root,i64 1026)
  %high = call i64 @m.word(ptr %root,i64 1027)
  %p = add i64 %rootWord,4096
  %d = add i64 %rootWord,8192
  %h = add i64 %rootWord,16384
  %pe = or i64 %p,7
  %de = or i64 %d,7
  %le = or i64 %lowWord,7
  %he = or i64 %h,7
  %pm = and i64 %pml4,-97
  %dm = and i64 %pdpt,-97
  %lm = and i64 %low,-97
  %hm = and i64 %high,-97
  %pOk = icmp eq i64 %pm,%pe
  %dOk = icmp eq i64 %dm,%de
  %lOk = icmp eq i64 %lm,%le
  %hOk = icmp eq i64 %hm,%he
  %a = and i1 %pOk,%dOk
  %b = and i1 %lOk,%hOk
  %ok = and i1 %a,%b
  call void @m.require(i1 %ok)
  ret void
}
define internal void @m.validate(i64 %count,i64 %base) nounwind noinline noredzone {
entry:
  call void @m.hierarchy()
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %inside = icmp ult i64 %i,519
  br i1 %inside,label %body,label %end
body:
  %pte = call ptr @m.pte(i64 %i)
  %entryWord = load volatile i64,ptr %pte,align 8
  %normalized = and i64 %entryWord,-97
  %used = icmp ult i64 %i,%count
  %delta = shl i64 %i,12
  %physical = add i64 %base,%delta
  %leaf = or i64 %physical,-9223372036854775801
  %expected = select i1 %used,i64 %leaf,i64 0
  %same = icmp eq i64 %normalized,%expected
  call void @m.require(i1 %same)
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal void @m.copy(i64 %from,i64 %to,i64 %count) nounwind noinline noredzone {
entry:
  %words = shl i64 %count,9
  %src = inttoptr i64 %from to ptr
  %dst = inttoptr i64 %to to ptr
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%words
  br i1 %more,label %body,label %end
body:
  %s = getelementptr i64,ptr %src,i64 %i
  %d = getelementptr i64,ptr %dst,i64 %i
  %v = load volatile i64,ptr %s,align 8
  store volatile i64 %v,ptr %d,align 8
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal void @m.erase(i64 %base,i64 %count) nounwind noinline noredzone {
entry:
  %p = inttoptr i64 %base to ptr
  %words = shl i64 %count,9
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%words
  br i1 %more,label %body,label %end
body:
  %at = getelementptr i64,ptr %p,i64 %i
  store volatile i64 0,ptr %at,align 8
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal void @m.publish(i64 %oldCount,i64 %newCount,i64 %base) nounwind noinline noredzone {
entry:
  %growing = icmp ugt i64 %newCount,%oldCount
  %count = select i1 %growing,i64 %newCount,i64 %oldCount
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%count
  br i1 %more,label %body,label %invalidate
body:
  %pte = call ptr @m.pte(i64 %i)
  %used = icmp ult i64 %i,%newCount
  %offset = shl i64 %i,12
  %physical = add i64 %base,%offset
  %leaf = or i64 %physical,-9223372036854775801
  %v = select i1 %used,i64 %leaf,i64 0
  store volatile i64 %v,ptr %pte,align 8
  %next = add i64 %i,1
  br label %loop
invalidate:
  fence seq_cst
  br label %tlb
tlb:
  %j = phi i64 [0,%invalidate],[%after,%flush]
  %pending = icmp ult i64 %j,%count
  br i1 %pending,label %flush,label %end
flush:
  %delta = shl i64 %j,12
  %va = add i64 %delta,6254592
  %address = inttoptr i64 %va to ptr
  call void asm sideeffect "invlpg ($0)", "r,~{memory},~{dirflag},~{fpsr},~{flags}"(ptr %address)
  %after = add i64 %j,1
  br label %tlb
end:
  fence seq_cst
  ret void
}
define i64 @topal_frame_memory_request_v1(i64 %operation,i64 %request) nounwind noinline noredzone {
entry:
  %admitted = call i1 @topal_frame_kernel_admit_v1(ptr @topal_x86_frame_context_v1,i64 0,i64 0,i64 0)
  call void @m.require(i1 %admitted)
  %f0 = call i64 @m.word(ptr @topal_x86_frame_details_v1,i64 0)
  %f1 = call i64 @m.word(ptr @topal_x86_frame_details_v1,i64 1)
  %f2 = call i64 @m.word(ptr @topal_x86_frame_details_v1,i64 2)
  %f3 = call i64 @m.word(ptr @topal_x86_frame_details_v1,i64 3)
  %f01 = or i64 %f0,%f1
  %f23 = or i64 %f2,%f3
  %facts = or i64 %f01,%f23
  %clean = icmp eq i64 %facts,0
  call void @m.require(i1 %clean)
  %ctx = call ptr @m.context()
  %candidate = getelementptr i64,ptr @topal_x86_frame_mm_tokens_v1,i64 3
  %state = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 0)
  %virgin = icmp eq i64 %state,0
  br i1 %virgin,label %initialize,label %ready
initialize:
  %v1 = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 1)
  %v2 = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %v3 = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 3)
  %v12 = or i64 %v1,%v2
  %v123 = or i64 %v12,%v3
  %fresh = icmp eq i64 %v123,0
  call void @m.require(i1 %fresh)
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 0,i64 1)
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 1,i64 6254592)
  br label %ready
ready:
  %live = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 0)
  %valid = icmp eq i64 %live,1
  call void @m.require(i1 %valid)
  %oldBreak = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 1)
  %oldCount = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %oldBase = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 3)
  %bounded = icmp ule i64 %oldCount,519
  call void @m.require(i1 %bounded)
  %oldLow = icmp uge i64 %oldBreak,6254592
  %oldHigh = icmp ule i64 %oldBreak,8380416
  %oldRange = and i1 %oldLow,%oldHigh
  call void @m.require(i1 %oldRange)
  %oldDifference = sub i64 %oldBreak,6254592
  %oldRounded = add i64 %oldDifference,4095
  %expectedOldCount = lshr i64 %oldRounded,12
  %coherent = icmp eq i64 %oldCount,%expectedOldCount
  call void @m.require(i1 %coherent)
  %p0 = call i64 @m.word(ptr %candidate,i64 0)
  %p1 = call i64 @m.word(ptr %candidate,i64 1)
  %p2 = call i64 @m.word(ptr %candidate,i64 2)
  %p01 = or i64 %p0,%p1
  %p012 = or i64 %p01,%p2
  %emptyCandidate = icmp eq i64 %p012,0
  call void @m.require(i1 %emptyCandidate)
  %empty = icmp eq i64 %oldCount,0
  br i1 %empty,label %no_old,label %old
no_old:
  %c0 = call i64 @m.word(ptr @topal_x86_frame_mm_tokens_v1,i64 0)
  %c1 = call i64 @m.word(ptr @topal_x86_frame_mm_tokens_v1,i64 1)
  %c2 = call i64 @m.word(ptr @topal_x86_frame_mm_tokens_v1,i64 2)
  %c01 = or i64 %c0,%c1
  %c012 = or i64 %c01,%c2
  %all = or i64 %c012,%oldBase
  %absent = icmp eq i64 %all,0
  call void @m.require(i1 %absent)
  br label %validated
old:
  %resolved = call i64 @m.resolve(ptr @topal_x86_frame_mm_tokens_v1,i64 %oldCount,i64 1)
  %same = icmp eq i64 %resolved,%oldBase
  call void @m.require(i1 %same)
  br label %validated
validated:
  %retainedBase = call i64 @m.word(ptr %ctx,i64 15)
  %sameBase = icmp eq i64 %retainedBase,%oldBase
  call void @m.require(i1 %sameBase)
  call void @m.validate(i64 %oldCount,i64 %oldBase)
  %brk = icmp eq i64 %operation,5
  %query = icmp eq i64 %request,0
  %read = and i1 %brk,%query
  br i1 %read,label %unchanged,label %range
range:
  %lower = icmp uge i64 %request,6254592
  %upper = icmp ule i64 %request,8380416
  %inside = and i1 %lower,%upper
  %destroy = xor i1 %brk,true
  %allowed = or i1 %inside,%destroy
  br i1 %allowed,label %sizing,label %unchanged
sizing:
  %newBreak = select i1 %brk,i64 %request,i64 6254592
  %difference = sub i64 %newBreak,6254592
  %rounded = add i64 %difference,4095
  %newCount = lshr i64 %rounded,12
  %sameCount = icmp eq i64 %newCount,%oldCount
  br i1 %sameCount,label %commit,label %replace
replace:
  %none = icmp eq i64 %newCount,0
  br i1 %none,label %publish,label %allocate
allocate:
  %budget = call i1 @topal_frame_kernel_admit_v1(ptr @topal_x86_frame_context_v1,i64 1,i64 %newCount,i64 1)
  call void @m.require(i1 %budget)
  %nextId = call i64 @m.word(ptr @topal_x86_frame_context_v1,i64 2)
  %notzero = icmp ne i64 %nextId,0
  %notmax = icmp ne i64 %nextId,-1
  %idOk = and i1 %notzero,%notmax
  call void @m.require(i1 %idOk)
  %allocated = call i1 @topal_frame_allocator_allocate_v1(ptr @topal_x86_frame_context_v1,i64 %newCount,i64 1,ptr %candidate)
  br i1 %allocated,label %prepare,label %unchanged
prepare:
  %physical = call i64 @m.resolve(ptr %candidate,i64 %newCount,i64 0)
  %smaller = icmp ult i64 %newCount,%oldCount
  %kept = select i1 %smaller,i64 %newCount,i64 %oldCount
  call void @m.copy(i64 %oldBase,i64 %physical,i64 %kept)
  %pinned = call i1 @topal_frame_allocator_pin_v1(ptr @topal_x86_frame_context_v1,ptr %candidate,i1 true)
  call void @m.require(i1 %pinned)
  br label %publish
publish:
  %newBase = phi i64 [0,%replace],[%physical,%prepare]
  call void @m.publish(i64 %oldCount,i64 %newCount,i64 %newBase)
  br i1 %empty,label %transfer,label %release
release:
  %unpinned = call i1 @topal_frame_allocator_pin_v1(ptr @topal_x86_frame_context_v1,ptr @topal_x86_frame_mm_tokens_v1,i1 false)
  call void @m.require(i1 %unpinned)
  call void @m.erase(i64 %oldBase,i64 %oldCount)
  %released = call i1 @topal_frame_allocator_release_v1(ptr @topal_x86_frame_context_v1,ptr @topal_x86_frame_mm_tokens_v1)
  call void @m.require(i1 %released)
  br label %transfer
transfer:
  %t0 = call i64 @m.word(ptr %candidate,i64 0)
  %t1 = call i64 @m.word(ptr %candidate,i64 1)
  %t2 = call i64 @m.word(ptr %candidate,i64 2)
  call void @m.put(ptr @topal_x86_frame_mm_tokens_v1,i64 0,i64 %t0)
  call void @m.put(ptr @topal_x86_frame_mm_tokens_v1,i64 1,i64 %t1)
  call void @m.put(ptr @topal_x86_frame_mm_tokens_v1,i64 2,i64 %t2)
  call void @m.put(ptr %candidate,i64 0,i64 0)
  call void @m.put(ptr %candidate,i64 1,i64 0)
  call void @m.put(ptr %candidate,i64 2,i64 0)
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 2,i64 %newCount)
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 3,i64 %newBase)
  call void @m.put(ptr %ctx,i64 15,i64 %newBase)
  br label %commit
commit:
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 1,i64 %newBreak)
  br i1 %brk,label %return_break,label %retire
retire:
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 0,i64 0)
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 1,i64 0)
  ret i64 1
return_break:
  ret i64 %newBreak
unchanged:
  ret i64 %oldBreak
}

define i64 @topal_frame_kernel_dispatch_v1(i64 %op,i64 %a,i64 %b,ptr %token,ptr %facts) nounwind noinline noredzone {
  %mm = icmp uge i64 %op,5
  br i1 %mm,label %memory,label %boot
memory:
  %known = icmp ule i64 %op,7
  %zero = icmp eq i64 %b,0
  %noToken = icmp eq ptr %token,null
  %noFacts = icmp eq ptr %facts,null
  %a0 = and i1 %known,%zero
  %b0 = and i1 %noToken,%noFacts
  %ok = and i1 %a0,%b0
  call void @m.require(i1 %ok)
  %result = call i64 @topal_frame_memory_request_v1(i64 %op,i64 %a)
  ret i64 %result
boot:
  %normal = call i64 @topal_frame_kernel_boot_dispatch_v1(i64 %op,i64 %a,i64 %b,ptr %token,ptr %facts)
  ret i64 %normal
}
