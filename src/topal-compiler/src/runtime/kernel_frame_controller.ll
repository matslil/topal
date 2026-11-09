; Provider-private x86-64 controller. No source address/authority constructors.
; Context words: state,issuer,next-id,N,table,snapshot,registry,history,private,
; source-stack,facts,output,granule,pending-start,pending-count,pending-id.
; Registry records: issuer,id,start,count,alignment,provenance,pins,live.
; Tokens are owned 24-byte private values: issuer,id,registry-index.
; All domains/geometry/context must be admitted BEFORE their first dereference.
declare i1 @topal_frame_kernel_admit_v1(ptr, i64, i64, i64)
declare i1 @topal_frame_kernel_token_domain_v1(ptr, ptr, i1)
declare i1 @topal_frame_kernel_facts_domain_v1(ptr, ptr, i64)
declare i1 @topal_frame_kernel_zero_v1(ptr, i64, i64)
declare void @topal_frame_kernel_fatal_v1(ptr) noreturn
declare i64 @topal_tail_owner_invoke_v1(ptr,i64,ptr,ptr,ptr,ptr,ptr)
@topal_frame_allocator_active_v1 = global ptr null, align 8

define internal i64 @f.word(ptr %p, i64 %index) nounwind alwaysinline {
  %a = getelementptr i64, ptr %p, i64 %index
  %v = load i64, ptr %a, align 8
  ret i64 %v
}
define internal void @f.put(ptr %p, i64 %index, i64 %value) nounwind alwaysinline {
  %a = getelementptr i64, ptr %p, i64 %index
  store i64 %value, ptr %a, align 8
  ret void
}
define internal ptr @f.pointer(ptr %ctx, i64 %index) nounwind alwaysinline {
  %v = call i64 @f.word(ptr %ctx, i64 %index)
  %p = inttoptr i64 %v to ptr
  ret ptr %p
}
define internal ptr @f.cell(ptr %table, i64 %index) nounwind alwaysinline {
  %body = getelementptr i8, ptr %table, i64 128
  %offset = mul i64 %index, 24
  %p = getelementptr i8, ptr %body, i64 %offset
  ret ptr %p
}
define internal ptr @f.record(ptr %registry, i64 %index) nounwind alwaysinline {
  %offset = mul i64 %index, 64
  %p = getelementptr i8, ptr %registry, i64 %offset
  ret ptr %p
}
define internal void @f.zero(ptr %p, i64 %words) nounwind noinline {
entry:
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%words
  br i1 %more,label %body,label %end
body:
  call void @f.put(ptr %p,i64 %i,i64 0)
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal void @f.copy(ptr %to, ptr %from, i64 %words) nounwind noinline {
entry:
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%words
  br i1 %more,label %body,label %end
body:
  %v = call i64 @f.word(ptr %from,i64 %i)
  call void @f.put(ptr %to,i64 %i,i64 %v)
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal void @f.retire_query(ptr %ctx) nounwind noinline {
  %facts = call ptr @f.pointer(ptr %ctx,i64 10)
  %output = call ptr @f.pointer(ptr %ctx,i64 11)
  call void @f.zero(ptr %facts,i64 3)
  call void @f.zero(ptr %output,i64 7)
  call void @f.put(ptr %ctx,i64 13,i64 0)
  call void @f.put(ptr %ctx,i64 14,i64 0)
  call void @f.put(ptr %ctx,i64 15,i64 0)
  store ptr null,ptr @topal_frame_allocator_active_v1,align 8
  ret void
}
define internal void @f.fatal(ptr %ctx) nounwind noinline noreturn {
  call void @f.put(ptr %ctx,i64 0,i64 3)
  %history = call ptr @f.pointer(ptr %ctx,i64 7)
  call void @f.put(ptr %history,i64 3,i64 0)
  call void @f.put(ptr %history,i64 4,i64 1)
  ; Preserve metadata, pending issuance and history. The target fatal boundary
  ; owns complete source/caller cleanup and cannot resume this allocator.
  call void @topal_frame_kernel_fatal_v1(ptr %ctx)
  unreachable
}
define void @topal_tail_boundary_fatal_v1() nounwind noinline noreturn {
  %ctx = load ptr,ptr @topal_frame_allocator_active_v1,align 8
  call void @f.fatal(ptr %ctx)
  unreachable
}

; Complete canonical natural extraction, including high limbs and padding.
define internal i64 @f.result(ptr %ctx, ptr %output) nounwind noinline {
entry:
  %sign = call i64 @f.word(ptr %output,i64 0)
  %length = call i64 @f.word(ptr %output,i64 1)
  %low = call i64 @f.word(ptr %output,i64 2)
  %high0 = call i64 @f.word(ptr %output,i64 3)
  %high1 = call i64 @f.word(ptr %output,i64 4)
  %high2 = call i64 @f.word(ptr %output,i64 5)
  %kind = call i64 @f.word(ptr %output,i64 6)
  %natural = icmp eq i64 %sign,0
  %short = icmp ule i64 %length,2
  %tag = icmp eq i64 %kind,2
  %h0 = or i64 %high0,%high1
  %h1 = or i64 %h0,%high2
  %padding = icmp eq i64 %h1,0
  %a = and i1 %natural,%short
  %b = and i1 %tag,%padding
  %valid = and i1 %a,%b
  br i1 %valid,label %canonical,label %bad
canonical:
  %empty = icmp eq i64 %length,0
  br i1 %empty,label %zero,label %nonzero
zero:
  %iszero = icmp eq i64 %low,0
  br i1 %iszero,label %end,label %bad
nonzero:
  %one = icmp eq i64 %length,1
  %upper = lshr i64 %low,32
  %lower = and i64 %low,4294967295
  %upperzero = icmp eq i64 %upper,0
  %lowernonzero = icmp ne i64 %lower,0
  %validone = and i1 %upperzero,%lowernonzero
  %validtwo = icmp ne i64 %upper,0
  %ok = select i1 %one,i1 %validone,i1 %validtwo
  br i1 %ok,label %end,label %bad
end:
  ret i64 %low
bad:
  call void @f.fatal(ptr %ctx)
  unreachable
}

define internal i64 @f.invoke(ptr %ctx,i64 %count,i64 %alignment,i64 %identity,i64 %phase,i64 %start,i64 %provenance) nounwind noinline {
entry:
  %n = call i64 @f.word(ptr %ctx,i64 3)
  %table = call ptr @f.pointer(ptr %ctx,i64 4)
  %snapshot = call ptr @f.pointer(ptr %ctx,i64 5)
  %history = call ptr @f.pointer(ptr %ctx,i64 7)
  %private = call ptr @f.pointer(ptr %ctx,i64 8)
  %stack = call ptr @f.pointer(ptr %ctx,i64 9)
  %facts = call ptr @f.pointer(ptr %ctx,i64 10)
  %output = call ptr @f.pointer(ptr %ctx,i64 11)
  call void @f.put(ptr %ctx,i64 0,i64 2)
  %minus = sub i64 %alignment,1
  call void @f.put(ptr %table,i64 1,i64 %count)
  call void @f.put(ptr %table,i64 2,i64 %minus)
  call void @f.put(ptr %table,i64 3,i64 %identity)
  call void @f.put(ptr %table,i64 4,i64 %provenance)
  call void @f.put(ptr %table,i64 5,i64 %phase)
  call void @f.put(ptr %table,i64 6,i64 %start)
  call void @f.put(ptr %table,i64 7,i64 %start)
  call void @f.put(ptr %table,i64 8,i64 0)
  call void @f.put(ptr %table,i64 9,i64 0)
  call void @f.put(ptr %table,i64 10,i64 0)
  call void @f.put(ptr %table,i64 11,i64 0)
  %cellwords = mul i64 %n,3
  %words = add i64 %cellwords,16
  call void @f.copy(ptr %snapshot,ptr %table,i64 %words)
  %double = mul i64 %n,2
  %work = add i64 %double,3
  call void @f.put(ptr %facts,i64 0,i64 %work)
  call void @f.put(ptr %facts,i64 1,i64 0)
  call void @f.put(ptr %facts,i64 2,i64 0)
  %bodybytes = mul i64 %n,24
  %bytes = add i64 %bodybytes,128
  store ptr %ctx,ptr @topal_frame_allocator_active_v1,align 8
  %status = call i64 @topal_tail_owner_invoke_v1(ptr %table,i64 %bytes,ptr %facts,ptr %history,ptr %private,ptr %stack,ptr %output)
  %normal = icmp eq i64 %status,0
  br i1 %normal,label %result,label %bad
result:
  %value = call i64 @f.result(ptr %ctx,ptr %output)
  %protocol = icmp eq i64 %value,-1
  %written = call i64 @f.word(ptr %table,i64 11)
  %same = icmp eq i64 %written,%value
  %notprotocol = xor i1 %protocol,true
  %validresult = and i1 %notprotocol,%same
  br i1 %validresult,label %end,label %bad
end:
  ret i64 %value
bad:
  call void @f.fatal(ptr %ctx)
  unreachable
}

; Compare every metadata word outside the explicit query-control prefix. The
; snapshot is comparison-only; it is never published or used to restore state.
define internal i1 @f.delta(ptr %ctx,i64 %start,i64 %count,i64 %oldid,i64 %newid) nounwind noinline {
entry:
  %n = call i64 @f.word(ptr %ctx,i64 3)
  %table = call ptr @f.pointer(ptr %ctx,i64 4)
  %snapshot = call ptr @f.pointer(ptr %ctx,i64 5)
  %body = mul i64 %n,3
  %words = add i64 %body,16
  %end = add i64 %start,%count
  br label %header
header:
  %h = phi i64 [0,%entry],[%hn,%hnext]
  %hmore = icmp ult i64 %h,16
  br i1 %hmore,label %hcheck,label %cells
hcheck:
  %after = icmp uge i64 %h,5
  %before = icmp ule i64 %h,11
  %control = and i1 %after,%before
  br i1 %control,label %hnext,label %hequal
hequal:
  %hv = call i64 @f.word(ptr %table,i64 %h)
  %hs = call i64 @f.word(ptr %snapshot,i64 %h)
  %heq = icmp eq i64 %hv,%hs
  br i1 %heq,label %hnext,label %bad
hnext:
  %hn = add i64 %h,1
  br label %header
cells:
  %i = phi i64 [0,%header],[%next,%advance]
  %more = icmp ult i64 %i,%n
  br i1 %more,label %cell,label %done
cell:
  %now = call ptr @f.cell(ptr %table,i64 %i)
  %was = call ptr @f.cell(ptr %snapshot,i64 %i)
  %id = call i64 @f.word(ptr %was,i64 0)
  %ge = icmp uge i64 %i,%start
  %lt = icmp ult i64 %i,%end
  %selected = and i1 %ge,%lt
  %eligible = icmp eq i64 %id,%oldid
  %validold = select i1 %selected,i1 %eligible,i1 true
  %expected = select i1 %selected,i64 %newid,i64 %id
  %actual = call i64 @f.word(ptr %now,i64 0)
  %sameid = icmp eq i64 %actual,%expected
  %p = call i64 @f.word(ptr %was,i64 1)
  %q = call i64 @f.word(ptr %now,i64 1)
  %phys = call i64 @f.word(ptr %was,i64 2)
  %newphys = call i64 @f.word(ptr %now,i64 2)
  %samep = icmp eq i64 %p,%q
  %samephys = icmp eq i64 %phys,%newphys
  %a = and i1 %validold,%sameid
  %b = and i1 %samep,%samephys
  %ok = and i1 %a,%b
  br i1 %ok,label %advance,label %bad
advance:
  %next = add i64 %i,1
  br label %cells
done:
  %phase = call i64 @f.word(ptr %table,i64 5)
  %complete = icmp eq i64 %phase,4
  ret i1 %complete
bad:
  ret i1 false
}

; Validate the entire selected span independently of the source's run state.
define internal i1 @f.span(ptr %ctx,i64 %start,i64 %count,i64 %alignment,i64 %identity,i64 %provenance) nounwind noinline {
entry:
  %n = call i64 @f.word(ptr %ctx,i64 3)
  %inside = icmp ult i64 %start,%n
  %left = sub i64 %n,%start
  %fits = icmp ule i64 %count,%left
  %positive = icmp ne i64 %count,0
  %divisor = icmp ne i64 %alignment,0
  %a = and i1 %inside,%fits
  %ab = and i1 %a,%positive
  %b = and i1 %ab,%divisor
  br i1 %b,label %first,label %bad
first:
  %snapshot = call ptr @f.pointer(ptr %ctx,i64 5)
  %base = call ptr @f.cell(ptr %snapshot,i64 %start)
  %phys = call i64 @f.word(ptr %base,i64 2)
  %tag = call i64 @f.word(ptr %base,i64 1)
  %nonzero = icmp ne i64 %tag,0
  %matching = icmp eq i64 %tag,%provenance
  %remainder = urem i64 %phys,%alignment
  %aligned = icmp eq i64 %remainder,0
  %c = and i1 %nonzero,%matching
  %d = and i1 %c,%aligned
  br i1 %d,label %loop,label %bad
loop:
  %i = phi i64 [0,%first],[%next,%advance]
  %more = icmp ult i64 %i,%count
  br i1 %more,label %check,label %end
check:
  %index = add i64 %start,%i
  %cell = call ptr @f.cell(ptr %snapshot,i64 %index)
  %id = call i64 @f.word(ptr %cell,i64 0)
  %p = call i64 @f.word(ptr %cell,i64 1)
  %frame = call i64 @f.word(ptr %cell,i64 2)
  %wanted = add i64 %phys,%i
  %nowrap = icmp uge i64 %wanted,%phys
  %sameid = icmp eq i64 %id,%identity
  %samep = icmp eq i64 %p,%provenance
  %sameframe = icmp eq i64 %frame,%wanted
  %e = and i1 %nowrap,%sameid
  %f = and i1 %samep,%sameframe
  %ok = and i1 %e,%f
  br i1 %ok,label %advance,label %bad
advance:
  %next = add i64 %i,1
  br label %loop
end:
  ret i1 true
bad:
  ret i1 false
}

; Boolean availability validation does not choose or publish an allocation.
define internal i1 @f.available(ptr %ctx,i64 %count,i64 %alignment) nounwind noinline {
entry:
  %n = call i64 @f.word(ptr %ctx,i64 3)
  %snapshot = call ptr @f.pointer(ptr %ctx,i64 5)
  br label %loop
loop:
  %cursor = phi i64 [%n,%entry],[%index,%advance]
  %suffix = phi i64 [0,%entry],[%length,%advance]
  %nextphys = phi i64 [0,%entry],[%phys,%advance]
  %nexttag = phi i64 [0,%entry],[%tag,%advance]
  %more = icmp ne i64 %cursor,0
  br i1 %more,label %check,label %end
check:
  %index = sub i64 %cursor,1
  %cell = call ptr @f.cell(ptr %snapshot,i64 %index)
  %id = call i64 @f.word(ptr %cell,i64 0)
  %tag = call i64 @f.word(ptr %cell,i64 1)
  %phys = call i64 @f.word(ptr %cell,i64 2)
  %free = icmp eq i64 %id,0
  %eligible = icmp ne i64 %tag,0
  %usable = and i1 %free,%eligible
  %following = add i64 %phys,1
  %nowrap = icmp ne i64 %following,0
  %contiguous = icmp eq i64 %following,%nextphys
  %same = icmp eq i64 %tag,%nexttag
  %a = and i1 %nowrap,%contiguous
  %extends = and i1 %a,%same
  %plus = add i64 %suffix,1
  %candidate = select i1 %extends,i64 %plus,i64 1
  %length = select i1 %usable,i64 %candidate,i64 0
  %remainder = urem i64 %phys,%alignment
  %aligned = icmp eq i64 %remainder,0
  %enough = icmp uge i64 %length,%count
  %fit = and i1 %aligned,%enough
  br i1 %fit,label %found,label %advance
advance:
  br label %loop
found:
  ret i1 true
end:
  ret i1 false
}

define internal ptr @f.token(ptr %ctx,ptr %token) nounwind noinline {
entry:
  %issuer = call i64 @f.word(ptr %ctx,i64 1)
  %n = call i64 @f.word(ptr %ctx,i64 3)
  %owner = call i64 @f.word(ptr %token,i64 0)
  %id = call i64 @f.word(ptr %token,i64 1)
  %slot = call i64 @f.word(ptr %token,i64 2)
  %same = icmp eq i64 %owner,%issuer
  %nonzero = icmp ne i64 %id,0
  %inside = icmp ult i64 %slot,%n
  %a = and i1 %same,%nonzero
  %ok = and i1 %a,%inside
  br i1 %ok,label %record,label %bad
record:
  %registry = call ptr @f.pointer(ptr %ctx,i64 6)
  %r = call ptr @f.record(ptr %registry,i64 %slot)
  %ri = call i64 @f.word(ptr %r,i64 0)
  %rd = call i64 @f.word(ptr %r,i64 1)
  %live = call i64 @f.word(ptr %r,i64 7)
  %iok = icmp eq i64 %ri,%issuer
  %dok = icmp eq i64 %rd,%id
  %active = icmp eq i64 %live,1
  %b = and i1 %iok,%dok
  %valid = and i1 %b,%active
  br i1 %valid,label %end,label %bad
end:
  ret ptr %r
bad:
  ret ptr null
}

define i1 @topal_frame_allocator_allocate_v1(ptr %ctx,i64 %count,i64 %alignment,ptr %token) nounwind noinline {
entry:
  %admitted = call i1 @topal_frame_kernel_admit_v1(ptr %ctx,i64 1,i64 %count,i64 %alignment)
  br i1 %admitted,label %domain,label %refuse
domain:
  %owned = call i1 @topal_frame_kernel_token_domain_v1(ptr %ctx,ptr %token,i1 true)
  br i1 %owned,label %request,label %refuse
request:
  %state = call i64 @f.word(ptr %ctx,i64 0)
  %live = icmp eq i64 %state,1
  %positive = icmp ne i64 %count,0
  %nonzero = icmp ne i64 %alignment,0
  %minus = sub i64 %alignment,1
  %mask = and i64 %alignment,%minus
  %power = icmp eq i64 %mask,0
  %id = call i64 @f.word(ptr %ctx,i64 2)
  %hasid = icmp ne i64 %id,0
  %room = icmp ult i64 %id,-1
  %t0 = call i64 @f.word(ptr %token,i64 0)
  %t1 = call i64 @f.word(ptr %token,i64 1)
  %t2 = call i64 @f.word(ptr %token,i64 2)
  %t01 = or i64 %t0,%t1
  %t012 = or i64 %t01,%t2
  %fresh = icmp eq i64 %t012,0
  %a = and i1 %live,%positive
  %b = and i1 %nonzero,%power
  %c = and i1 %hasid,%room
  %d = and i1 %a,%b
  %e = and i1 %c,%fresh
  %valid = and i1 %d,%e
  br i1 %valid,label %slots,label %refuse
slots:
  %n = call i64 @f.word(ptr %ctx,i64 3)
  %registry = call ptr @f.pointer(ptr %ctx,i64 6)
  br label %slotloop
slotloop:
  %slot = phi i64 [0,%slots],[%nextslot,%slotnext]
  %inside = icmp ult i64 %slot,%n
  br i1 %inside,label %slotcheck,label %refuse
slotcheck:
  %r = call ptr @f.record(ptr %registry,i64 %slot)
  %active = call i64 @f.word(ptr %r,i64 7)
  %free = icmp eq i64 %active,0
  br i1 %free,label %invoke,label %slotnext
slotnext:
  %nextslot = add i64 %slot,1
  br label %slotloop
invoke:
  %result = call i64 @f.invoke(ptr %ctx,i64 %count,i64 %alignment,i64 %id,i64 0,i64 0,i64 0)
  %absent = icmp eq i64 %result,0
  br i1 %absent,label %exhausted,label %selected
exhausted:
  %untouched = call i1 @f.delta(ptr %ctx,i64 0,i64 0,i64 0,i64 0)
  %available = call i1 @f.available(ptr %ctx,i64 %count,i64 %alignment)
  %unavailable = xor i1 %available,true
  %correct = and i1 %untouched,%unavailable
  br i1 %correct,label %nofit,label %bad
nofit:
  call void @f.retire_query(ptr %ctx)
  call void @f.put(ptr %ctx,i64 0,i64 1)
  ret i1 false
selected:
  %start = sub i64 %result,1
  %inrange = icmp ult i64 %start,%n
  br i1 %inrange,label %snapshot,label %bad
snapshot:
  %copy = call ptr @f.pointer(ptr %ctx,i64 5)
  %first = call ptr @f.cell(ptr %copy,i64 %start)
  %provenance = call i64 @f.word(ptr %first,i64 1)
  %physical = call i64 @f.word(ptr %first,i64 2)
  %span = call i1 @f.span(ptr %ctx,i64 %start,i64 %count,i64 %alignment,i64 0,i64 %provenance)
  %delta = call i1 @f.delta(ptr %ctx,i64 %start,i64 %count,i64 0,i64 %id)
  %safe = and i1 %span,%delta
  br i1 %safe,label %zero,label %bad
zero:
  call void @f.put(ptr %ctx,i64 13,i64 %start)
  call void @f.put(ptr %ctx,i64 14,i64 %count)
  call void @f.put(ptr %ctx,i64 15,i64 %id)
  %zeroed = call i1 @topal_frame_kernel_zero_v1(ptr %ctx,i64 %physical,i64 %count)
  br i1 %zeroed,label %publish,label %bad
publish:
  %issuer = call i64 @f.word(ptr %ctx,i64 1)
  call void @f.put(ptr %r,i64 0,i64 %issuer)
  call void @f.put(ptr %r,i64 1,i64 %id)
  call void @f.put(ptr %r,i64 2,i64 %start)
  call void @f.put(ptr %r,i64 3,i64 %count)
  call void @f.put(ptr %r,i64 4,i64 %alignment)
  call void @f.put(ptr %r,i64 5,i64 %provenance)
  call void @f.put(ptr %r,i64 6,i64 0)
  call void @f.put(ptr %r,i64 7,i64 1)
  call void @f.put(ptr %token,i64 0,i64 %issuer)
  call void @f.put(ptr %token,i64 1,i64 %id)
  call void @f.put(ptr %token,i64 2,i64 %slot)
  %nextid = add i64 %id,1
  call void @f.put(ptr %ctx,i64 2,i64 %nextid)
  call void @f.retire_query(ptr %ctx)
  call void @f.put(ptr %ctx,i64 0,i64 1)
  ret i1 true
bad:
  call void @f.fatal(ptr %ctx)
  unreachable
refuse:
  ret i1 false
}

define i1 @topal_frame_allocator_release_v1(ptr %ctx,ptr %token) nounwind noinline {
entry:
  %admitted = call i1 @topal_frame_kernel_admit_v1(ptr %ctx,i64 0,i64 0,i64 0)
  br i1 %admitted,label %domain,label %refuse
domain:
  %owned = call i1 @topal_frame_kernel_token_domain_v1(ptr %ctx,ptr %token,i1 true)
  br i1 %owned,label %lookup,label %refuse
lookup:
  %state = call i64 @f.word(ptr %ctx,i64 0)
  %live = icmp eq i64 %state,1
  br i1 %live,label %validate_token,label %refuse
validate_token:
  %r = call ptr @f.token(ptr %ctx,ptr %token)
  %found = icmp ne ptr %r,null
  br i1 %found,label %record,label %refuse
record:
  %pins = call i64 @f.word(ptr %r,i64 6)
  %unmapped = icmp eq i64 %pins,0
  %count = call i64 @f.word(ptr %r,i64 3)
  %alignment = call i64 @f.word(ptr %r,i64 4)
  %id = call i64 @f.word(ptr %r,i64 1)
  %start = call i64 @f.word(ptr %r,i64 2)
  %provenance = call i64 @f.word(ptr %r,i64 5)
  br i1 %unmapped,label %admit,label %refuse
admit:
  %request = call i1 @topal_frame_kernel_admit_v1(ptr %ctx,i64 2,i64 %count,i64 %alignment)
  br i1 %request,label %invoke,label %refuse
invoke:
  %result = call i64 @f.invoke(ptr %ctx,i64 %count,i64 %alignment,i64 %id,i64 2,i64 %start,i64 %provenance)
  %expected = add i64 %start,1
  %same = icmp eq i64 %result,%expected
  %span = call i1 @f.span(ptr %ctx,i64 %start,i64 %count,i64 %alignment,i64 %id,i64 %provenance)
  %delta = call i1 @f.delta(ptr %ctx,i64 %start,i64 %count,i64 %id,i64 0)
  %a = and i1 %same,%span
  %valid = and i1 %a,%delta
  br i1 %valid,label %consume,label %bad
consume:
  call void @f.zero(ptr %r,i64 8)
  call void @f.zero(ptr %token,i64 3)
  call void @f.retire_query(ptr %ctx)
  call void @f.put(ptr %ctx,i64 0,i64 1)
  ret i1 true
bad:
  call void @f.fatal(ptr %ctx)
  unreachable
refuse:
  ret i1 false
}

define i1 @topal_frame_allocator_pin_v1(ptr %ctx,ptr %token,i1 %acquire) nounwind noinline {
entry:
  %admitted = call i1 @topal_frame_kernel_admit_v1(ptr %ctx,i64 0,i64 0,i64 0)
  br i1 %admitted,label %domain,label %refuse
domain:
  %owned = call i1 @topal_frame_kernel_token_domain_v1(ptr %ctx,ptr %token,i1 false)
  br i1 %owned,label %lookup,label %refuse
lookup:
  %state = call i64 @f.word(ptr %ctx,i64 0)
  %live = icmp eq i64 %state,1
  br i1 %live,label %validate_token,label %refuse
validate_token:
  %r = call ptr @f.token(ptr %ctx,ptr %token)
  %found = icmp ne ptr %r,null
  br i1 %found,label %record,label %refuse
record:
  %pins = call i64 @f.word(ptr %r,i64 6)
  %room = icmp ne i64 %pins,-1
  %nonzero = icmp ne i64 %pins,0
  %allowed = select i1 %acquire,i1 %room,i1 %nonzero
  %plus = add i64 %pins,1
  %minus = sub i64 %pins,1
  %next = select i1 %acquire,i64 %plus,i64 %minus
  br i1 %allowed,label %end,label %refuse
end:
  call void @f.put(ptr %r,i64 6,i64 %next)
  ret i1 true
refuse:
  ret i1 false
}

; Private provider facts for mapping/construction, never a source address or
; ownership constructor. The opaque token remains live and is not consumed.
define i1 @topal_frame_allocator_resolve_v1(ptr %ctx,ptr %token,ptr %facts) nounwind noinline {
entry:
  %admitted = call i1 @topal_frame_kernel_admit_v1(ptr %ctx,i64 0,i64 0,i64 0)
  br i1 %admitted,label %domains,label %refuse
domains:
  %owned = call i1 @topal_frame_kernel_token_domain_v1(ptr %ctx,ptr %token,i1 false)
  %output = call i1 @topal_frame_kernel_facts_domain_v1(ptr %ctx,ptr %facts,i64 32)
  %both = and i1 %owned,%output
  br i1 %both,label %lookup,label %refuse
lookup:
  %state = call i64 @f.word(ptr %ctx,i64 0)
  %live = icmp eq i64 %state,1
  br i1 %live,label %validate_token,label %refuse
validate_token:
  %r = call ptr @f.token(ptr %ctx,ptr %token)
  %found = icmp ne ptr %r,null
  br i1 %found,label %record,label %refuse
record:
  %start = call i64 @f.word(ptr %r,i64 2)
  %count = call i64 @f.word(ptr %r,i64 3)
  %alignment = call i64 @f.word(ptr %r,i64 4)
  %provenance = call i64 @f.word(ptr %r,i64 5)
  %table = call ptr @f.pointer(ptr %ctx,i64 4)
  %cell = call ptr @f.cell(ptr %table,i64 %start)
  %physical = call i64 @f.word(ptr %cell,i64 2)
  call void @f.put(ptr %facts,i64 0,i64 %physical)
  call void @f.put(ptr %facts,i64 1,i64 %count)
  call void @f.put(ptr %facts,i64 2,i64 %alignment)
  call void @f.put(ptr %facts,i64 3,i64 %provenance)
  ret i1 true
refuse:
  ret i1 false
}
