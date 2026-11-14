; Bootstrap-only target implementation, not a source address constructor.
; The 1GiB bound is the existing bootstrap identity mapping, not pool policy.
; All complete normalized allocatable frames must fit, or creation fails.
; Capacities cover that architectural window; actual N is derived from E820.
@topal_x86_frame_context_v1 = global [16 x i64] zeroinitializer, align 64
@topal_x86_frame_table_v1 = global [6291584 x i8] zeroinitializer, align 64
@topal_x86_frame_snapshot_v1 = global [6291584 x i8] zeroinitializer, align 64
@topal_x86_frame_registry_v1 = global [16777216 x i8] zeroinitializer, align 64
@topal_x86_frame_tokens_v1 = global [6291456 x i8] zeroinitializer, align 64
@topal_x86_frame_history_v1 = global [5 x i64] zeroinitializer, align 16
@topal_x86_frame_private_v1 = global [65536 x i8] zeroinitializer, align 16
@topal_x86_frame_stack_v1 = global [4096 x i8] zeroinitializer, align 16
@topal_x86_frame_facts_v1 = global [3 x i64] zeroinitializer, align 16
@topal_x86_frame_output_v1 = global [7 x i64] zeroinitializer, align 16
@topal_x86_frame_details_v1 = global [4 x i64] zeroinitializer, align 16
@topal_x86_frame_handoff_v1 = global [5 x i64] zeroinitializer, align 16
; Linker/packager must bind the actual complete allocated image end, including
; this backing. No hand-written floor may stand in for that binding.
@topal_x86_kernel_reservation_end_v1 = external global i8
@topal_frame_allocator_active_v1 = external global ptr
@topal_x86_tail_boundary_active_v1 = external global ptr

define internal i64 @t.word(ptr %p,i64 %i) nounwind alwaysinline {
  %a = getelementptr i64,ptr %p,i64 %i
  %v = load i64,ptr %a,align 8
  ret i64 %v
}
define internal void @t.put(ptr %p,i64 %i,i64 %v) nounwind alwaysinline {
  %a = getelementptr i64,ptr %p,i64 %i
  store i64 %v,ptr %a,align 8
  ret void
}
define internal i64 @t.load64(ptr %p,i64 %offset) nounwind alwaysinline {
  %a = getelementptr i8,ptr %p,i64 %offset
  %v = load i64,ptr %a,align 1
  ret i64 %v
}
define internal i64 @t.load32(ptr %p,i64 %offset) nounwind alwaysinline {
  %a = getelementptr i8,ptr %p,i64 %offset
  %v = load i32,ptr %a,align 1
  %w = zext i32 %v to i64
  ret i64 %w
}
define internal void @t.zero(ptr %p,i64 %bytes) nounwind noinline {
entry:
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%bytes
  br i1 %more,label %body,label %end
body:
  %a = getelementptr i8,ptr %p,i64 %i
  store volatile i8 0,ptr %a,align 1
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal i1 @t.clean(ptr %p,i64 %bytes) nounwind noinline {
entry:
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%bytes
  br i1 %more,label %body,label %end
body:
  %a = getelementptr i8,ptr %p,i64 %i
  %v = load volatile i8,ptr %a,align 1
  %zero = icmp eq i8 %v,0
  %next = add i64 %i,1
  br i1 %zero,label %loop,label %bad
end:
  ret i1 true
bad:
  ret i1 false
}
; Architecture-private register observations. The source language has no asm.
; This admission establishes bootstrap hardware geometry only, not the later
; typed owning gateway, syscall stack leases or complete caller retirement.
define internal i1 @t.bootstrap() nounwind noinline {
  %cs = call i16 asm sideeffect "movw %cs, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  %flags = call i64 asm sideeffect "pushfq; popq $0", "=r,~{dirflag},~{fpsr},~{flags},~{memory}"()
  %sp = call i64 asm sideeffect "movq %rsp, $0", "=r,~{dirflag},~{fpsr},~{flags}"()
  %ring = and i16 %cs,3
  %kernel = icmp eq i16 %ring,0
  %if = and i64 %flags,512
  %disabled = icmp eq i64 %if,0
  %low = icmp uge i64 %sp,1081344
  %high = icmp ult i64 %sp,1572864
  %a = and i1 %kernel,%disabled
  %b = and i1 %low,%high
  %ok = and i1 %a,%b
  ret i1 %ok
}

; Return the next *raw* boundary, including both reservations. Classifying and
; aligning each interval before merging reproduces the normalized-page rule:
; an unaligned raw boundary cannot be erased by premature RAM coalescing.
define internal i64 @t.next(ptr %boot,i64 %entries,i64 %cursor,i64 %reserved,i64 %resource,i64 %resourceEnd) nounwind noinline {
entry:
  %r = icmp ugt i64 %reserved,%cursor
  %first = select i1 %r,i64 %reserved,i64 -1
  %s = icmp ugt i64 %resource,%cursor
  %less = icmp ult i64 %resource,%first
  %take = and i1 %s,%less
  %second = select i1 %take,i64 %resource,i64 %first
  %e = icmp ugt i64 %resourceEnd,%cursor
  %el = icmp ult i64 %resourceEnd,%second
  %et = and i1 %e,%el
  %initial = select i1 %et,i64 %resourceEnd,i64 %second
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %minimum = phi i64 [%initial,%entry],[%chosenEnd,%body]
  %more = icmp ult i64 %i,%entries
  br i1 %more,label %body,label %end
body:
  %offset = mul i64 %i,20
  %at = add i64 %offset,720
  %base = call i64 @t.load64(ptr %boot,i64 %at)
  %la = add i64 %at,8
  %length = call i64 @t.load64(ptr %boot,i64 %la)
  %limit = add i64 %base,%length
  %above = icmp ugt i64 %base,%cursor
  %below = icmp ult i64 %base,%minimum
  %choose = and i1 %above,%below
  %chosenBase = select i1 %choose,i64 %base,i64 %minimum
  %endAbove = icmp ugt i64 %limit,%cursor
  %endBelow = icmp ult i64 %limit,%chosenBase
  %chooseEnd = and i1 %endAbove,%endBelow
  %chosenEnd = select i1 %chooseEnd,i64 %limit,i64 %chosenBase
  %next = add i64 %i,1
  br label %loop
end:
  ret i64 %minimum
}
define internal i1 @t.allocatable(ptr %boot,i64 %entries,i64 %start,i64 %end,i64 %reserved,i64 %resource,i64 %resourceEnd) nounwind noinline {
entry:
  %kernel = icmp ult i64 %start,%reserved
  %left = icmp ult i64 %start,%resourceEnd
  %right = icmp ugt i64 %end,%resource
  %overlap = and i1 %left,%right
  %blocked = or i1 %kernel,%overlap
  br i1 %blocked,label %bad,label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%advance]
  %found = phi i1 [false,%entry],[%now,%advance]
  %more = icmp ult i64 %i,%entries
  br i1 %more,label %body,label %endloop
body:
  %offset = mul i64 %i,20
  %at = add i64 %offset,720
  %base = call i64 @t.load64(ptr %boot,i64 %at)
  %la = add i64 %at,8
  %length = call i64 @t.load64(ptr %boot,i64 %la)
  %limit = add i64 %base,%length
  %before = icmp ule i64 %base,%start
  %after = icmp uge i64 %limit,%end
  %covers = and i1 %before,%after
  %ta = add i64 %at,16
  %type = call i64 @t.load32(ptr %boot,i64 %ta)
  %ram = icmp eq i64 %type,1
  %notram = xor i1 %ram,true
  %reservedClaim = and i1 %covers,%notram
  br i1 %reservedClaim,label %bad,label %advance
advance:
  %now = or i1 %found,%covers
  %next = add i64 %i,1
  br label %loop
endloop:
  ret i1 %found
bad:
  ret i1 false
}

; Complete constructor. BSS belongs to the linked-image reservation; the
; complete table/registry/token backing cannot become allocatable memory.
define ptr @topal_x86_frame_allocator_create_v1(ptr %boot) nounwind noinline {
entry:
  %hardware = call i1 @t.bootstrap()
  br i1 %hardware,label %once,label %refuse
once:
  %state = call i64 @t.word(ptr @topal_x86_frame_context_v1,i64 0)
  %fresh = icmp eq i64 %state,0
  br i1 %fresh,label %geometry,label %refuse
geometry:
  %address = ptrtoint ptr %boot to i64
  %nonzero = icmp ne i64 %address,0
  %inside = icmp ule i64 %address,16773120
  %end = ptrtoint ptr @topal_x86_kernel_reservation_end_v1 to i64
  %aboveBootstrap = icmp uge i64 %end,16777216
  %mapped = icmp ule i64 %end,1073741824
  %a = and i1 %nonzero,%inside
  %b = and i1 %aboveBootstrap,%mapped
  %valid = and i1 %a,%b
  br i1 %valid,label %handoff,label %failed
handoff:
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 0,i64 2)
  %setup = call i64 @t.load64(ptr %boot,i64 592)
  %extImage = call i64 @t.load32(ptr %boot,i64 192)
  %extSize = call i64 @t.load32(ptr %boot,i64 196)
  %resource = call i64 @t.load32(ptr %boot,i64 536)
  %resourceSize = call i64 @t.load32(ptr %boot,i64 540)
  %resourceEnd = add i64 %resource,%resourceSize
  %resNonzero = icmp ne i64 %resourceSize,0
  %resAfter = icmp uge i64 %resource,%end
  %resMapped = icmp ule i64 %resourceEnd,1073741824
  %noExtra = or i64 %setup,%extImage
  %noExt = or i64 %noExtra,%extSize
  %plain = icmp eq i64 %noExt,0
  %c = and i1 %resNonzero,%resAfter
  %d = and i1 %resMapped,%plain
  %handoffValid = and i1 %c,%d
  br i1 %handoffValid,label %count,label %failed
count:
  %cp = getelementptr i8,ptr %boot,i64 488
  %cv = load i8,ptr %cp,align 1
  %entries = zext i8 %cv to i64
  %has = icmp ne i64 %entries,0
  %bounded = icmp ule i64 %entries,128
  %table = and i1 %has,%bounded
  br i1 %table,label %validate,label %failed
validate:
  %i = phi i64 [0,%count],[%nextEntry,%entryNext]
  %more = icmp ult i64 %i,%entries
  br i1 %more,label %entryCheck,label %normalize
entryCheck:
  %offset = mul i64 %i,20
  %at = add i64 %offset,720
  %base = call i64 @t.load64(ptr %boot,i64 %at)
  %la = add i64 %at,8
  %length = call i64 @t.load64(ptr %boot,i64 %la)
  %limit = add i64 %base,%length
  %nonempty = icmp ne i64 %length,0
  %nowrap = icmp ugt i64 %limit,%base
  %proper = and i1 %nonempty,%nowrap
  %ta = add i64 %at,16
  %type = call i64 @t.load32(ptr %boot,i64 %ta)
  %ram = icmp eq i64 %type,1
  %outside = icmp ugt i64 %limit,1073741824
  %unsupported = and i1 %ram,%outside
  %supported = xor i1 %unsupported,true
  %entryValid = and i1 %proper,%supported
  br i1 %entryValid,label %entryNext,label %failed
entryNext:
  %nextEntry = add i64 %i,1
  br label %validate
normalize:
  br label %interval
interval:
  %cursor = phi i64 [0,%normalize],[%boundary,%advance]
  %n = phi i64 [0,%normalize],[%newN,%advance]
  %boundary = call i64 @t.next(ptr %boot,i64 %entries,i64 %cursor,i64 %end,i64 %resource,i64 %resourceEnd)
  %last = icmp eq i64 %boundary,-1
  br i1 %last,label %complete,label %classify
classify:
  %alloc = call i1 @t.allocatable(ptr %boot,i64 %entries,i64 %cursor,i64 %boundary,i64 %end,i64 %resource,i64 %resourceEnd)
  br i1 %alloc,label %align,label %skip
align:
  %rounded = add i64 %cursor,4095
  %overflow = icmp ult i64 %rounded,%cursor
  br i1 %overflow,label %failed,label %qualified
qualified:
  %lo = and i64 %rounded,-4096
  %hi = and i64 %boundary,-4096
  %whole = icmp ult i64 %lo,%hi
  br i1 %whole,label %window,label %skip
window:
  %fits = icmp ule i64 %hi,1073741824
  br i1 %fits,label %cells,label %failed
cells:
  %physical = lshr i64 %lo,12
  %stop = lshr i64 %hi,12
  br label %cellloop
cellloop:
  %frame = phi i64 [%physical,%cells],[%nextFrame,%write]
  %used = phi i64 [%n,%cells],[%nextN,%write]
  %remaining = icmp ult i64 %frame,%stop
  br i1 %remaining,label %capacity,label %added
capacity:
  %space = icmp ult i64 %used,262144
  br i1 %space,label %write,label %failed
write:
  %bytes = mul i64 %used,24
  %cellOffset = add i64 %bytes,128
  %cell = getelementptr i8,ptr @topal_x86_frame_table_v1,i64 %cellOffset
  call void @t.put(ptr %cell,i64 0,i64 0)
  ; All E820 type-1 claims have identical source-kind/provider provenance.
  call void @t.put(ptr %cell,i64 1,i64 1)
  call void @t.put(ptr %cell,i64 2,i64 %frame)
  %nextFrame = add i64 %frame,1
  %nextN = add i64 %used,1
  br label %cellloop
added:
  br label %advance
skip:
  br label %advance
advance:
  %newN = phi i64 [%used,%added],[%n,%skip]
  br label %interval
complete:
  %any = icmp ne i64 %n,0
  br i1 %any,label %publish,label %failed
publish:
  %registryBytes = mul i64 %n,64
  %tokenBytes = mul i64 %n,24
  call void @t.zero(ptr @topal_x86_frame_registry_v1,i64 %registryBytes)
  call void @t.zero(ptr @topal_x86_frame_tokens_v1,i64 %tokenBytes)
  call void @t.zero(ptr @topal_x86_frame_history_v1,i64 40)
  call void @t.zero(ptr @topal_x86_frame_private_v1,i64 65536)
  call void @t.zero(ptr @topal_x86_frame_stack_v1,i64 4096)
  call void @t.zero(ptr @topal_x86_frame_facts_v1,i64 24)
  call void @t.zero(ptr @topal_x86_frame_output_v1,i64 56)
  call void @t.zero(ptr @topal_x86_frame_details_v1,i64 32)
  call void @t.put(ptr @topal_x86_frame_table_v1,i64 0,i64 %n)
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 1,i64 1)
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 2,i64 1)
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 3,i64 %n)
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 4,i64 ptrtoint (ptr @topal_x86_frame_table_v1 to i64))
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 5,i64 ptrtoint (ptr @topal_x86_frame_snapshot_v1 to i64))
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 6,i64 ptrtoint (ptr @topal_x86_frame_registry_v1 to i64))
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 7,i64 ptrtoint (ptr @topal_x86_frame_history_v1 to i64))
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 8,i64 ptrtoint (ptr @topal_x86_frame_private_v1 to i64))
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 9,i64 ptrtoint (ptr @topal_x86_frame_stack_v1 to i64))
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 10,i64 ptrtoint (ptr @topal_x86_frame_facts_v1 to i64))
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 11,i64 ptrtoint (ptr @topal_x86_frame_output_v1 to i64))
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 12,i64 4096)
  call void @t.put(ptr @topal_x86_frame_handoff_v1,i64 0,i64 %address)
  call void @t.put(ptr @topal_x86_frame_handoff_v1,i64 1,i64 %end)
  call void @t.put(ptr @topal_x86_frame_handoff_v1,i64 2,i64 %resource)
  call void @t.put(ptr @topal_x86_frame_handoff_v1,i64 3,i64 %resourceEnd)
  call void @t.put(ptr @topal_x86_frame_handoff_v1,i64 4,i64 %n)
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 0,i64 1)
  ret ptr @topal_x86_frame_context_v1
failed:
  call void @t.put(ptr @topal_x86_frame_context_v1,i64 0,i64 3)
  ret ptr null
refuse:
  ret ptr null
}

define i1 @topal_frame_kernel_admit_v1(ptr %ctx,i64 %operation,i64 %count,i64 %alignment) nounwind noinline {
entry:
  %owned = icmp eq ptr %ctx,@topal_x86_frame_context_v1
  br i1 %owned,label %hardware,label %bad
hardware:
  %actual = call i1 @t.bootstrap()
  br i1 %actual,label %state,label %bad
state:
  %s = call i64 @t.word(ptr %ctx,i64 0)
  %issuer = call i64 @t.word(ptr %ctx,i64 1)
  %id = call i64 @t.word(ptr %ctx,i64 2)
  %n = call i64 @t.word(ptr %ctx,i64 3)
  %originalN = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 4)
  %live = icmp eq i64 %s,1
  %sameIssuer = icmp eq i64 %issuer,1
  %identity = icmp ne i64 %id,0
  %has = icmp ne i64 %n,0
  %capacity = icmp ule i64 %n,262144
  %sameN = icmp eq i64 %n,%originalN
  %a = and i1 %live,%sameIssuer
  %b = and i1 %identity,%has
  %c = and i1 %capacity,%sameN
  %ab = and i1 %a,%b
  %valid = and i1 %ab,%c
  br i1 %valid,label %pointers,label %bad
pointers:
  %p4 = call i64 @t.word(ptr %ctx,i64 4)
  %p5 = call i64 @t.word(ptr %ctx,i64 5)
  %p6 = call i64 @t.word(ptr %ctx,i64 6)
  %p7 = call i64 @t.word(ptr %ctx,i64 7)
  %p8 = call i64 @t.word(ptr %ctx,i64 8)
  %p9 = call i64 @t.word(ptr %ctx,i64 9)
  %p10 = call i64 @t.word(ptr %ctx,i64 10)
  %p11 = call i64 @t.word(ptr %ctx,i64 11)
  %p12 = call i64 @t.word(ptr %ctx,i64 12)
  %v4 = icmp eq i64 %p4,ptrtoint (ptr @topal_x86_frame_table_v1 to i64)
  %v5 = icmp eq i64 %p5,ptrtoint (ptr @topal_x86_frame_snapshot_v1 to i64)
  %v6 = icmp eq i64 %p6,ptrtoint (ptr @topal_x86_frame_registry_v1 to i64)
  %v7 = icmp eq i64 %p7,ptrtoint (ptr @topal_x86_frame_history_v1 to i64)
  %v8 = icmp eq i64 %p8,ptrtoint (ptr @topal_x86_frame_private_v1 to i64)
  %v9 = icmp eq i64 %p9,ptrtoint (ptr @topal_x86_frame_stack_v1 to i64)
  %v10 = icmp eq i64 %p10,ptrtoint (ptr @topal_x86_frame_facts_v1 to i64)
  %v11 = icmp eq i64 %p11,ptrtoint (ptr @topal_x86_frame_output_v1 to i64)
  %v12 = icmp eq i64 %p12,4096
  %v45 = and i1 %v4,%v5
  %v67 = and i1 %v6,%v7
  %v89 = and i1 %v8,%v9
  %v1011 = and i1 %v10,%v11
  %v4567 = and i1 %v45,%v67
  %v891011 = and i1 %v89,%v1011
  %allPointers = and i1 %v4567,%v891011
  %geometry = and i1 %allPointers,%v12
  br i1 %geometry,label %domains,label %bad
domains:
  %tableN = call i64 @t.word(ptr @topal_x86_frame_table_v1,i64 0)
  %sameTable = icmp eq i64 %tableN,%n
  %active = call i64 @t.word(ptr @topal_x86_frame_history_v1,i64 3)
  %poison = call i64 @t.word(ptr @topal_x86_frame_history_v1,i64 4)
  %busy = or i64 %active,%poison
  %idle = icmp eq i64 %busy,0
  %pin = load ptr,ptr @topal_frame_allocator_active_v1,align 8
  %sourcePin = load ptr,ptr @topal_x86_tail_boundary_active_v1,align 8
  %notPinned = icmp eq ptr %pin,null
  %notSourcePinned = icmp eq ptr %sourcePin,null
  %private = call i1 @t.clean(ptr @topal_x86_frame_private_v1,i64 65528)
  %stack = call i1 @t.clean(ptr @topal_x86_frame_stack_v1,i64 4096)
  %facts = call i1 @t.clean(ptr @topal_x86_frame_facts_v1,i64 24)
  %output = call i1 @t.clean(ptr @topal_x86_frame_output_v1,i64 56)
  %q0 = and i1 %sameTable,%idle
  %q1 = and i1 %notPinned,%notSourcePinned
  %q2 = and i1 %private,%stack
  %q3 = and i1 %facts,%output
  %q01 = and i1 %q0,%q1
  %q23 = and i1 %q2,%q3
  %clean = and i1 %q01,%q23
  br i1 %clean,label %operationCheck,label %bad
operationCheck:
  %known = icmp ule i64 %operation,2
  %query = icmp eq i64 %operation,0
  br i1 %known,label %request,label %bad
request:
  br i1 %query,label %end,label %budget
budget:
  %positive = icmp ne i64 %count,0
  %alignmentNonzero = icmp ne i64 %alignment,0
  %minus = sub i64 %alignment,1
  %mask = and i64 %alignment,%minus
  %power = icmp eq i64 %mask,0
  %frames = call i64 @t.word(ptr @topal_x86_frame_history_v1,i64 0)
  %loans = call i64 @t.word(ptr @topal_x86_frame_history_v1,i64 1)
  %generation = call i64 @t.word(ptr @topal_x86_frame_history_v1,i64 2)
  ; The generator checks these products using reconstructed source costs and
  ; maximum architectural N. Include the terminal step and virgin identity 2.
  %double = mul i64 %n,2
  %work = add i64 %double,4
  %frameCost = mul i64 %work,{{FRAMES_PER_STEP}}
  %loanCost = mul i64 %work,{{LOANS_PER_STEP}}
  %frameBudget = add i64 %frameCost,2
  %loanBudget = add i64 %loanCost,2
  %frameLimit = sub i64 -1,%frameBudget
  %loanLimit = sub i64 -1,%loanBudget
  %frameRoom = icmp ule i64 %frames,%frameLimit
  %loanRoom = icmp ule i64 %loans,%loanLimit
  %generationRoom = icmp ne i64 %generation,-1
  %noFrames = icmp eq i64 %frames,0
  %noLoans = icmp eq i64 %loans,0
  %noGeneration = icmp eq i64 %generation,0
  %noIds = and i1 %noFrames,%noLoans
  %virgin = and i1 %noIds,%noGeneration
  %validFrames = icmp uge i64 %frames,2
  %validLoans = icmp uge i64 %loans,2
  %validGeneration = icmp ne i64 %generation,0
  %validIds = and i1 %validFrames,%validLoans
  %old = and i1 %validIds,%validGeneration
  %validHistory = or i1 %virgin,%old
  %r0 = and i1 %positive,%alignmentNonzero
  %r1 = and i1 %power,%frameRoom
  %r2 = and i1 %loanRoom,%generationRoom
  %r01 = and i1 %r0,%r1
  %safeBudget = and i1 %r01,%r2
  %admitted = and i1 %safeBudget,%validHistory
  ret i1 %admitted
end:
  ret i1 true
bad:
  ret i1 false
}

define i1 @topal_frame_kernel_token_domain_v1(ptr %ctx,ptr %token,i1 %write) nounwind noinline {
entry:
  %owned = icmp eq ptr %ctx,@topal_x86_frame_context_v1
  br i1 %owned,label %domain,label %bad
domain:
  %n = call i64 @t.word(ptr %ctx,i64 3)
  %size = mul i64 %n,24
  %p = ptrtoint ptr %token to i64
  %base = ptrtoint ptr @topal_x86_frame_tokens_v1 to i64
  %offset = sub i64 %p,%base
  %lower = icmp uge i64 %p,%base
  %upper = icmp ult i64 %offset,%size
  %rem = urem i64 %offset,24
  %aligned = icmp eq i64 %rem,0
  %capacity = icmp ule i64 %n,262144
  %a = and i1 %lower,%upper
  %b = and i1 %aligned,%capacity
  %valid = and i1 %a,%b
  ret i1 %valid
bad:
  ret i1 false
}
define i1 @topal_frame_kernel_facts_domain_v1(ptr %ctx,ptr %facts,i64 %bytes) nounwind noinline {
  %owned = icmp eq ptr %ctx,@topal_x86_frame_context_v1
  %domain = icmp eq ptr %facts,@topal_x86_frame_details_v1
  %size = icmp eq i64 %bytes,32
  %a = and i1 %owned,%domain
  %valid = and i1 %a,%size
  ret i1 %valid
}

define i1 @topal_frame_kernel_zero_v1(ptr %ctx,i64 %physical,i64 %count) nounwind noinline {
entry:
  %owned = icmp eq ptr %ctx,@topal_x86_frame_context_v1
  br i1 %owned,label %hardware,label %bad
hardware:
  %boot = call i1 @t.bootstrap()
  br i1 %boot,label %check,label %bad
check:
  %state = call i64 @t.word(ptr %ctx,i64 0)
  %n = call i64 @t.word(ptr %ctx,i64 3)
  %id = call i64 @t.word(ptr %ctx,i64 2)
  %start = call i64 @t.word(ptr %ctx,i64 13)
  %pendingCount = call i64 @t.word(ptr %ctx,i64 14)
  %pendingId = call i64 @t.word(ptr %ctx,i64 15)
  %active = load ptr,ptr @topal_frame_allocator_active_v1,align 8
  %busy = icmp eq i64 %state,2
  %pinned = icmp eq ptr %active,%ctx
  %sameCount = icmp eq i64 %count,%pendingCount
  %sameId = icmp eq i64 %id,%pendingId
  %positive = icmp ne i64 %count,0
  %inside = icmp ult i64 %start,%n
  %capacity = icmp ule i64 %n,262144
  %left = sub i64 %n,%start
  %fits = icmp ule i64 %count,%left
  %physicalEnd = add i64 %physical,%count
  %nowrap = icmp ugt i64 %physicalEnd,%physical
  %mapped = icmp ule i64 %physicalEnd,262144
  %a = and i1 %busy,%pinned
  %b = and i1 %sameCount,%sameId
  %c = and i1 %positive,%inside
  %d = and i1 %capacity,%fits
  %e = and i1 %nowrap,%mapped
  %ab = and i1 %a,%b
  %cd = and i1 %c,%d
  %abcd = and i1 %ab,%cd
  %valid = and i1 %abcd,%e
  br i1 %valid,label %reservations,label %bad
reservations:
  %address = shl i64 %physical,12
  %limit = shl i64 %physicalEnd,12
  %imageEnd = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 1)
  %resource = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 2)
  %resourceEnd = call i64 @t.word(ptr @topal_x86_frame_handoff_v1,i64 3)
  %above = icmp uge i64 %address,%imageEnd
  %before = icmp ule i64 %limit,%resource
  %after = icmp uge i64 %address,%resourceEnd
  %disjoint = or i1 %before,%after
  %safe = and i1 %above,%disjoint
  br i1 %safe,label %loop,label %bad
loop:
  %i = phi i64 [0,%reservations],[%next,%advance]
  %more = icmp ult i64 %i,%count
  br i1 %more,label %cell,label %zero
cell:
  %index = add i64 %start,%i
  %offset = mul i64 %index,24
  %body = add i64 %offset,128
  %ptr = getelementptr i8,ptr @topal_x86_frame_table_v1,i64 %body
  %cellId = call i64 @t.word(ptr %ptr,i64 0)
  %tag = call i64 @t.word(ptr %ptr,i64 1)
  %frame = call i64 @t.word(ptr %ptr,i64 2)
  %expected = add i64 %physical,%i
  %identity = icmp eq i64 %cellId,%id
  %provenance = icmp eq i64 %tag,1
  %contiguous = icmp eq i64 %frame,%expected
  %f = and i1 %identity,%provenance
  %ok = and i1 %f,%contiguous
  br i1 %ok,label %advance,label %bad
advance:
  %next = add i64 %i,1
  br label %loop
zero:
  %ptrPhysical = inttoptr i64 %address to ptr
  %bytes = shl i64 %count,12
  call void @t.zero(ptr %ptrPhysical,i64 %bytes)
  ret i1 true
bad:
  ret i1 false
}

; Nonreturning component failure. This retires the actual source domains but
; deliberately grants NO owning-caller retirement/publication certificate.
; A qualified gateway must retire its complete stack and recovery state too.
define void @topal_frame_kernel_fatal_v1(ptr %ctx) nounwind noinline noreturn {
entry:
  %owned = icmp eq ptr %ctx,@topal_x86_frame_context_v1
  br i1 %owned,label %retire,label %halt
retire:
  call void @t.put(ptr %ctx,i64 0,i64 3)
  call void @t.put(ptr @topal_x86_frame_history_v1,i64 3,i64 0)
  call void @t.put(ptr @topal_x86_frame_history_v1,i64 4,i64 1)
  call void @t.zero(ptr @topal_x86_frame_private_v1,i64 65528)
  call void @t.zero(ptr @topal_x86_frame_stack_v1,i64 4096)
  call void @t.zero(ptr @topal_x86_frame_facts_v1,i64 24)
  call void @t.zero(ptr @topal_x86_frame_output_v1,i64 56)
  call void @t.zero(ptr @topal_x86_frame_details_v1,i64 32)
  store ptr null,ptr @topal_frame_allocator_active_v1,align 8
  store ptr null,ptr @topal_x86_tail_boundary_active_v1,align 8
  br label %halt
halt:
  call void @topal_x86_frame_target_halt_v1()
  unreachable
}
define void @topal_x86_frame_target_halt_v1() nounwind noinline noreturn {
entry:
  br label %halt
halt:
  call void asm sideeffect "cli; hlt", "~{dirflag},~{fpsr},~{flags},~{memory}"()
  br label %halt
}
