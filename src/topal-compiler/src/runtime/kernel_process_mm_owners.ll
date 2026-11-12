; TK-DEC-284 / TOPAL-SYSTEMS-PROCESS-MM-001 private serialized MM owners.
; IDs: active PID/MM/generation, inactive PID/MM/generation, next MM identity.
; Tokens remain actual allocator handles; these scalar IDs grant no authority.
@topal_x86_process_mm_ids_v1 = global [7 x i64] zeroinitializer, align 8
@topal_x86_process_mm_inactive_token_v1 = global [3 x i64] zeroinitializer, align 8
@topal_x86_process_mm_inactive_state_v1 = global [4 x i64] zeroinitializer, align 8
@topal_x86_process_mm_prepared_owner_v1 = global [3 x i64] zeroinitializer, align 8
@topal_x86_process_mm_transition_v1 = global [3 x i64] zeroinitializer, align 8

define internal i64 @p.id(i64 %n) nounwind noinline noredzone {
  %v = call i64 @m.word(ptr @topal_x86_process_mm_ids_v1,i64 %n)
  ret i64 %v
}
define internal void @p.set_id(i64 %n,i64 %v) nounwind noinline noredzone {
  call void @m.put(ptr @topal_x86_process_mm_ids_v1,i64 %n,i64 %v)
  ret void
}
define internal void @p.copy(ptr %from,ptr %to,i64 %count) nounwind noinline noredzone {
entry:
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%count
  br i1 %more,label %body,label %end
body:
  %v = call i64 @m.word(ptr %from,i64 %i)
  call void @m.put(ptr %to,i64 %i,i64 %v)
  %actual = call i64 @m.word(ptr %to,i64 %i)
  %same = icmp eq i64 %v,%actual
  call void @pm.require(i1 %same)
  %next = add i64 %i,1
  br label %loop
end:
  ret void
}
define internal void @p.clear(ptr %to,i64 %count) nounwind noinline noredzone {
entry:
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%count
  br i1 %more,label %body,label %end
body:
  call void @m.put(ptr %to,i64 %i,i64 0)
  %next = add i64 %i,1
  br label %loop
end:
  call void @pm.zero(ptr %to,i64 %count)
  ret void
}
define internal ptr @p.table() nounwind noinline noredzone {
  %ctx = call ptr @m.context()
  %word = call i64 @m.word(ptr %ctx,i64 116)
  %mask = and i64 %word,4095
  %aligned = icmp eq i64 %mask,0
  %nonzero = icmp ne i64 %word,0
  %end = add i64 %word,{{TABLE_BYTES}}
  %nowrap = icmp uge i64 %end,%word
  %bounded = icmp ule i64 %end,1073741824
  %below = icmp ule i64 %end,4194304
  %above = icmp uge i64 %word,8388608
  %supervisor = or i1 %below,%above
  %a = and i1 %aligned,%nonzero
  %b = and i1 %nowrap,%bounded
  %ab = and i1 %a,%b
  %ok = and i1 %ab,%supervisor
  call void @pm.require(i1 %ok)
  %table = inttoptr i64 %word to ptr
  %live = call i64 @m.word(ptr %table,i64 0)
  %one = icmp eq i64 %live,1
  call void @pm.require(i1 %one)
  ret ptr %table
}
define internal void @p.bank(ptr %state,ptr %token) nounwind noinline noredzone {
entry:
  %live = call i64 @m.word(ptr %state,i64 0)
  %brk = call i64 @m.word(ptr %state,i64 1)
  %count = call i64 @m.word(ptr %state,i64 2)
  %base = call i64 @m.word(ptr %state,i64 3)
  %one = icmp eq i64 %live,1
  %low = icmp uge i64 %brk,6254592
  %high = icmp ule i64 %brk,8380416
  %bounded = icmp ule i64 %count,519
  %difference = sub i64 %brk,6254592
  %rounded = add i64 %difference,4095
  %expected = lshr i64 %rounded,12
  %same = icmp eq i64 %expected,%count
  %a = and i1 %one,%low
  %b = and i1 %high,%bounded
  %ab = and i1 %a,%b
  %ok = and i1 %ab,%same
  call void @pm.require(i1 %ok)
  %empty = icmp eq i64 %count,0
  br i1 %empty,label %zero,label %resolve
zero:
  call void @pm.zero(ptr %token,i64 3)
  %noBase = icmp eq i64 %base,0
  call void @pm.require(i1 %noBase)
  ret void
resolve:
  %actual = call i64 @m.resolve(ptr %token,i64 %count,i64 1)
  %equal = icmp eq i64 %actual,%base
  call void @pm.require(i1 %equal)
  ret void
}
define internal void @p.prime(i64 %op) nounwind noinline noredzone {
entry:
  %live = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 0)
  %empty = icmp eq i64 %live,0
  br i1 %empty,label %initialize,label %end
initialize:
  call void @pm.zero(ptr @topal_x86_frame_mm_state_v1,i64 4)
  call void @pm.zero(ptr @topal_x86_frame_mm_tokens_v1,i64 6)
  call void @m.validate(i64 0,i64 0)
  %switch = icmp eq i64 %op,11
  br i1 %switch,label %saved,label %current
saved:
  %table = call ptr @p.table()
  %old = call i64 @m.word(ptr %table,i64 1071)
  br label %check
current:
  %ctx = call ptr @m.context()
  %active = call i64 @m.word(ptr %ctx,i64 15)
  br label %check
check:
  %heap = phi i64 [%old,%saved],[%active,%current]
  %none = icmp eq i64 %heap,0
  call void @pm.require(i1 %none)
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 0,i64 1)
  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 1,i64 6254592)
  br label %end
end:
  ret void
}
define internal void @p.initialize_ids(i64 %op) nounwind noinline noredzone {
entry:
  %pid = call i64 @p.id(i64 0)
  %fresh = icmp eq i64 %pid,0
  br i1 %fresh,label %initialize,label %end
initialize:
  %switch = icmp eq i64 %op,11
  %exit = icmp eq i64 %op,12
  %needsOwner = or i1 %switch,%exit
  %allowed = xor i1 %needsOwner,true
  call void @pm.require(i1 %allowed)
  call void @pm.zero(ptr @topal_x86_process_mm_ids_v1,i64 7)
  call void @pm.zero(ptr @topal_x86_process_mm_inactive_state_v1,i64 4)
  call void @pm.zero(ptr @topal_x86_process_mm_inactive_token_v1,i64 3)
  %table = call ptr @p.table()
  %actual = call i64 @m.word(ptr %table,i64 7)
  %parent = icmp eq i64 %actual,1
  call void @pm.require(i1 %parent)
  call void @p.set_id(i64 0,i64 1)
  call void @p.set_id(i64 1,i64 1)
  call void @p.set_id(i64 6,i64 2)
  br label %end
end:
  ret void
}
define internal void @p.initial_parent() nounwind noinline noredzone {
entry:
  %pid = call i64 @p.id(i64 0)
  %fresh = icmp eq i64 %pid,0
  br i1 %fresh,label %initialize,label %end
initialize:
  ; The checked initial-process constructor owns PID one. This runs at its
  ; first empty-heap initialization, before descriptor import and before fork.
  ; Exec already retains a bound owner and therefore takes the unchanged arm.
  call void @pm.zero(ptr @topal_x86_process_mm_ids_v1,i64 7)
  call void @pm.zero(ptr @topal_x86_process_mm_inactive_state_v1,i64 4)
  call void @pm.zero(ptr @topal_x86_process_mm_inactive_token_v1,i64 3)
  call void @p.set_id(i64 0,i64 1)
  call void @p.set_id(i64 1,i64 1)
  call void @p.set_id(i64 6,i64 2)
  br label %end
end:
  ret void
}
define internal void @p.inactive() nounwind noinline noredzone {
entry:
  %pid = call i64 @p.id(i64 3)
  %none = icmp eq i64 %pid,0
  br i1 %none,label %empty,label %live
empty:
  %ids = getelementptr i64,ptr @topal_x86_process_mm_ids_v1,i64 3
  call void @pm.zero(ptr %ids,i64 3)
  call void @pm.zero(ptr @topal_x86_process_mm_inactive_state_v1,i64 4)
  call void @pm.zero(ptr @topal_x86_process_mm_inactive_token_v1,i64 3)
  ret void
live:
  %activePid = call i64 @p.id(i64 0)
  %different = icmp ne i64 %pid,%activePid
  %id = call i64 @p.id(i64 4)
  %activeId = call i64 @p.id(i64 1)
  %next = call i64 @p.id(i64 6)
  %nonzero = icmp ne i64 %id,0
  %issued = icmp ult i64 %id,%next
  %distinct = icmp ne i64 %id,%activeId
  %a = and i1 %different,%nonzero
  %b = and i1 %issued,%distinct
  %ok = and i1 %a,%b
  call void @pm.require(i1 %ok)
  call void @p.bank(ptr @topal_x86_process_mm_inactive_state_v1,ptr @topal_x86_process_mm_inactive_token_v1)
  call void @p.disjoint(ptr @topal_x86_frame_mm_state_v1,ptr @topal_x86_frame_mm_tokens_v1,ptr @topal_x86_process_mm_inactive_state_v1,ptr @topal_x86_process_mm_inactive_token_v1)
  ret void
}
define internal void @p.disjoint(ptr %a,ptr %at,ptr %b,ptr %bt) nounwind noinline noredzone {
entry:
  %ac = call i64 @m.word(ptr %a,i64 2)
  %bc = call i64 @m.word(ptr %b,i64 2)
  %ae = icmp eq i64 %ac,0
  %be = icmp eq i64 %bc,0
  %empty = or i1 %ae,%be
  br i1 %empty,label %end,label %check
check:
  %ab = call i64 @m.word(ptr %a,i64 3)
  %bb = call i64 @m.word(ptr %b,i64 3)
  %abytes = shl i64 %ac,12
  %bbytes = shl i64 %bc,12
  %aend = add i64 %ab,%abytes
  %bend = add i64 %bb,%bbytes
  %before = icmp ule i64 %aend,%bb
  %after = icmp ule i64 %bend,%ab
  %separate = or i1 %before,%after
  %aid = call i64 @m.word(ptr %at,i64 1)
  %bid = call i64 @m.word(ptr %bt,i64 1)
  %distinct = icmp ne i64 %aid,%bid
  %ok = and i1 %separate,%distinct
  call void @pm.require(i1 %ok)
  br label %end
end:
  ret void
}
define internal void @p.bind(i1 %switch) nounwind noinline noredzone {
  call void @p.bank(ptr @topal_x86_frame_mm_state_v1,ptr @topal_x86_frame_mm_tokens_v1)
  %candidate = getelementptr i64,ptr @topal_x86_frame_mm_tokens_v1,i64 3
  call void @pm.zero(ptr %candidate,i64 3)
  call void @p.inactive()
  %table = call ptr @p.table()
  %pid = call i64 @p.id(i64 0)
  %inactivePid = call i64 @p.id(i64 3)
  %id = call i64 @p.id(i64 1)
  %next = call i64 @p.id(i64 6)
  %nonzero = icmp ne i64 %id,0
  %issued = icmp ult i64 %id,%next
  %idOk = and i1 %nonzero,%issued
  call void @pm.require(i1 %idOk)
  %actualPid = call i64 @m.word(ptr %table,i64 7)
  %expectedPid = select i1 %switch,i64 %inactivePid,i64 %pid
  %samePid = icmp eq i64 %actualPid,%expectedPid
  call void @pm.require(i1 %samePid)
  %ctx = call ptr @m.context()
  %base = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 3)
  %count = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %inactiveBase = call i64 @m.word(ptr @topal_x86_process_mm_inactive_state_v1,i64 3)
  %heap = call i64 @m.word(ptr %ctx,i64 15)
  %expectedHeap = select i1 %switch,i64 %inactiveBase,i64 %base
  %sameHeap = icmp eq i64 %heap,%expectedHeap
  call void @pm.require(i1 %sameHeap)
  call void @m.validate(i64 %count,i64 %base)
  ret void
}
define internal i64 @p.issue() nounwind noinline noredzone {
  %next = call i64 @p.id(i64 6)
  %nonzero = icmp ne i64 %next,0
  %room = icmp ne i64 %next,-1
  %ok = and i1 %nonzero,%room
  call void @pm.require(i1 %ok)
  %after = add i64 %next,1
  call void @p.set_id(i64 6,i64 %after)
  ret i64 %next
}
define internal void @p.prepared_empty() nounwind noinline noredzone {
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_state_v1,i64 7)
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  call void @pm.zero(ptr @topal_x86_process_mm_prepared_owner_v1,i64 3)
  ret void
}
define internal void @p.receipt() nounwind noinline noredzone {
  call void @p.bank(ptr @topal_x86_process_mm_prepared_state_v1,ptr @topal_x86_process_mm_prepared_token_v1)
  %ctx = call ptr @m.context()
  %root = call i64 @m.word(ptr %ctx,i64 3)
  %ctxWord = ptrtoint ptr %ctx to i64
  %epoch = call i64 @m.word(ptr @topal_x86_frame_mm_epoch_v1,i64 0)
  %r = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 4)
  %c = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 5)
  %e = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 6)
  %sameR = icmp eq i64 %r,%root
  %sameC = icmp eq i64 %c,%ctxWord
  %sameE = icmp eq i64 %e,%epoch
  %a = and i1 %sameR,%sameC
  %ok = and i1 %a,%sameE
  call void @pm.require(i1 %ok)
  %pid = call i64 @p.id(i64 0)
  %id = call i64 @p.id(i64 1)
  %generation = call i64 @p.id(i64 2)
  %pPid = call i64 @m.word(ptr @topal_x86_process_mm_prepared_owner_v1,i64 0)
  %pId = call i64 @m.word(ptr @topal_x86_process_mm_prepared_owner_v1,i64 1)
  %pGeneration = call i64 @m.word(ptr @topal_x86_process_mm_prepared_owner_v1,i64 2)
  %sameP = icmp eq i64 %pid,%pPid
  %sameI = icmp eq i64 %id,%pId
  %sameG = icmp eq i64 %generation,%pGeneration
  %b = and i1 %sameP,%sameI
  %bound = and i1 %b,%sameG
  call void @pm.require(i1 %bound)
  ret void
}
define internal void @p.retire(ptr %state,ptr %token) nounwind noinline noredzone {
entry:
  call void @p.bank(ptr %state,ptr %token)
  %count = call i64 @m.word(ptr %state,i64 2)
  %base = call i64 @m.word(ptr %state,i64 3)
  %empty = icmp eq i64 %count,0
  br i1 %empty,label %clear,label %release
release:
  %unpinned = call i1 @topal_frame_allocator_pin_v1(ptr @topal_x86_frame_context_v1,ptr %token,i1 false)
  call void @pm.require(i1 %unpinned)
  call void @m.erase(i64 %base,i64 %count)
  %pointer = inttoptr i64 %base to ptr
  %words = shl i64 %count,9
  call void @pm.zero(ptr %pointer,i64 %words)
  %released = call i1 @topal_frame_allocator_release_v1(ptr @topal_x86_frame_context_v1,ptr %token)
  call void @pm.require(i1 %released)
  br label %clear
clear:
  call void @pm.zero(ptr %token,i64 3)
  call void @p.clear(ptr %state,i64 4)
  ret void
}
define internal i64 @p.prepare() nounwind noinline noredzone {
entry:
  %pid = call i64 @p.id(i64 0)
  %inactive = call i64 @p.id(i64 3)
  %parent = icmp eq i64 %pid,1
  %alone = icmp eq i64 %inactive,0
  %allowed = and i1 %parent,%alone
  call void @pm.require(i1 %allowed)
  call void @p.prepared_empty()
  %ok = call i1 @pm.prepare()
  br i1 %ok,label %remember,label %end
remember:
  call void @p.copy(ptr @topal_x86_process_mm_ids_v1,ptr @topal_x86_process_mm_prepared_owner_v1,i64 3)
  br label %end
end:
  %result = zext i1 %ok to i64
  ret i64 %result
}
define internal i64 @p.commit(i64 %child) nounwind noinline noredzone {
  call void @p.receipt()
  call void @p.disjoint(ptr @topal_x86_frame_mm_state_v1,ptr @topal_x86_frame_mm_tokens_v1,ptr @topal_x86_process_mm_prepared_state_v1,ptr @topal_x86_process_mm_prepared_token_v1)
  %table = call ptr @p.table()
  %pid = call i64 @p.id(i64 0)
  %parent = icmp eq i64 %pid,1
  call void @pm.require(i1 %parent)
  %inactivePid = call i64 @p.id(i64 3)
  %alone = icmp eq i64 %inactivePid,0
  call void @pm.require(i1 %alone)
  %header = call i64 @m.word(ptr %table,i64 1024)
  %unused = icmp eq i64 %header,0
  call void @pm.require(i1 %unused)
  %pending = call i64 @m.word(ptr %table,i64 1026)
  %same = icmp eq i64 %pending,%child
  %minimum = icmp uge i64 %child,2
  %maximum = icmp ule i64 %child,2147483647
  %range = and i1 %minimum,%maximum
  %valid = and i1 %same,%range
  call void @pm.require(i1 %valid)
  %oldCount = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %oldBase = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 3)
  %newCount = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 2)
  %newBase = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 3)
  %oldBreak = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 1)
  %newBreak = call i64 @m.word(ptr @topal_x86_process_mm_prepared_state_v1,i64 1)
  %sameCount = icmp eq i64 %oldCount,%newCount
  %sameBreak = icmp eq i64 %oldBreak,%newBreak
  %equal = and i1 %sameCount,%sameBreak
  call void @pm.require(i1 %equal)
  call void @pm.equal(i64 %oldBase,i64 %newBase,i64 %oldCount)
  %id = call i64 @p.issue()
  %generation = call i64 @p.id(i64 2)
  call void @m.put(ptr @topal_x86_process_mm_transition_v1,i64 0,i64 %child)
  call void @m.put(ptr @topal_x86_process_mm_transition_v1,i64 1,i64 %id)
  call void @m.put(ptr @topal_x86_process_mm_transition_v1,i64 2,i64 %generation)
  call void @m.publish(i64 %oldCount,i64 %newCount,i64 %newBase)
  %inactiveIds = getelementptr i64,ptr @topal_x86_process_mm_ids_v1,i64 3
  call void @p.copy(ptr @topal_x86_frame_mm_tokens_v1,ptr @topal_x86_process_mm_inactive_token_v1,i64 3)
  call void @p.copy(ptr @topal_x86_frame_mm_state_v1,ptr @topal_x86_process_mm_inactive_state_v1,i64 4)
  call void @p.copy(ptr @topal_x86_process_mm_ids_v1,ptr %inactiveIds,i64 3)
  call void @p.copy(ptr @topal_x86_process_mm_prepared_token_v1,ptr @topal_x86_frame_mm_tokens_v1,i64 3)
  call void @p.copy(ptr @topal_x86_process_mm_prepared_state_v1,ptr @topal_x86_frame_mm_state_v1,i64 4)
  call void @p.copy(ptr @topal_x86_process_mm_transition_v1,ptr @topal_x86_process_mm_ids_v1,i64 3)
  %ctx = call ptr @m.context()
  call void @m.put(ptr %ctx,i64 15,i64 %newBase)
  call void @p.clear(ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  call void @p.clear(ptr @topal_x86_process_mm_prepared_state_v1,i64 7)
  call void @p.clear(ptr @topal_x86_process_mm_prepared_owner_v1,i64 3)
  call void @p.clear(ptr @topal_x86_process_mm_transition_v1,i64 3)
  ret i64 1
}
define internal i64 @p.switch() nounwind noinline noredzone {
  call void @p.prepared_empty()
  %inactive = call i64 @p.id(i64 3)
  %live = icmp ne i64 %inactive,0
  call void @pm.require(i1 %live)
  %table = call ptr @p.table()
  %oldPid = call i64 @m.word(ptr %table,i64 1543)
  %sourcePid = call i64 @p.id(i64 0)
  %sameSource = icmp eq i64 %oldPid,%sourcePid
  call void @pm.require(i1 %sameSource)
  %oldCount = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %newCount = call i64 @m.word(ptr @topal_x86_process_mm_inactive_state_v1,i64 2)
  %newBase = call i64 @m.word(ptr @topal_x86_process_mm_inactive_state_v1,i64 3)
  call void @m.publish(i64 %oldCount,i64 %newCount,i64 %newBase)
  %inactiveIds = getelementptr i64,ptr @topal_x86_process_mm_ids_v1,i64 3
  call void @p.copy(ptr @topal_x86_frame_mm_tokens_v1,ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  call void @p.copy(ptr @topal_x86_process_mm_inactive_token_v1,ptr @topal_x86_frame_mm_tokens_v1,i64 3)
  call void @p.copy(ptr @topal_x86_process_mm_prepared_token_v1,ptr @topal_x86_process_mm_inactive_token_v1,i64 3)
  call void @p.copy(ptr @topal_x86_frame_mm_state_v1,ptr @topal_x86_process_mm_prepared_state_v1,i64 4)
  call void @p.copy(ptr @topal_x86_process_mm_inactive_state_v1,ptr @topal_x86_frame_mm_state_v1,i64 4)
  call void @p.copy(ptr @topal_x86_process_mm_prepared_state_v1,ptr @topal_x86_process_mm_inactive_state_v1,i64 4)
  call void @p.copy(ptr @topal_x86_process_mm_ids_v1,ptr @topal_x86_process_mm_transition_v1,i64 3)
  call void @p.copy(ptr %inactiveIds,ptr @topal_x86_process_mm_ids_v1,i64 3)
  call void @p.copy(ptr @topal_x86_process_mm_transition_v1,ptr %inactiveIds,i64 3)
  call void @p.clear(ptr @topal_x86_process_mm_prepared_token_v1,i64 3)
  call void @p.clear(ptr @topal_x86_process_mm_prepared_state_v1,i64 7)
  call void @p.clear(ptr @topal_x86_process_mm_transition_v1,i64 3)
  ret i64 1
}
define internal i64 @p.child_exit() nounwind noinline noredzone {
  call void @p.prepared_empty()
  %pid = call i64 @p.id(i64 0)
  %parent = call i64 @p.id(i64 3)
  %child = icmp uge i64 %pid,2
  %one = icmp eq i64 %parent,1
  %ok = and i1 %child,%one
  call void @pm.require(i1 %ok)
  %table = call ptr @p.table()
  %savedPid = call i64 @m.word(ptr %table,i64 1543)
  %matches = icmp eq i64 %savedPid,1
  call void @pm.require(i1 %matches)
  %oldCount = call i64 @m.word(ptr @topal_x86_frame_mm_state_v1,i64 2)
  %newCount = call i64 @m.word(ptr @topal_x86_process_mm_inactive_state_v1,i64 2)
  %newBase = call i64 @m.word(ptr @topal_x86_process_mm_inactive_state_v1,i64 3)
  call void @m.publish(i64 %oldCount,i64 %newCount,i64 %newBase)
  call void @p.retire(ptr @topal_x86_frame_mm_state_v1,ptr @topal_x86_frame_mm_tokens_v1)
  %inactiveIds = getelementptr i64,ptr @topal_x86_process_mm_ids_v1,i64 3
  call void @p.copy(ptr @topal_x86_process_mm_inactive_token_v1,ptr @topal_x86_frame_mm_tokens_v1,i64 3)
  call void @p.copy(ptr @topal_x86_process_mm_inactive_state_v1,ptr @topal_x86_frame_mm_state_v1,i64 4)
  call void @p.copy(ptr %inactiveIds,ptr @topal_x86_process_mm_ids_v1,i64 3)
  %ctx = call ptr @m.context()
  call void @m.put(ptr %ctx,i64 15,i64 %newBase)
  call void @p.clear(ptr @topal_x86_process_mm_inactive_token_v1,i64 3)
  call void @p.clear(ptr @topal_x86_process_mm_inactive_state_v1,i64 4)
  call void @p.clear(ptr %inactiveIds,i64 3)
  ret i64 1
}
define internal i64 @p.cancel_inactive() nounwind noinline noredzone {
entry:
  call void @p.prepared_empty()
  %pid = call i64 @p.id(i64 0)
  %parent = icmp eq i64 %pid,1
  call void @pm.require(i1 %parent)
  %inactive = call i64 @p.id(i64 3)
  %none = icmp eq i64 %inactive,0
  br i1 %none,label %end,label %cancel
cancel:
  %table = call ptr @p.table()
  %header = call i64 @m.word(ptr %table,i64 1024)
  %pending = call i64 @m.word(ptr %table,i64 1026)
  %live = icmp eq i64 %header,1
  %same = icmp eq i64 %pending,%inactive
  %valid = and i1 %live,%same
  call void @pm.require(i1 %valid)
  call void @p.retire(ptr @topal_x86_process_mm_inactive_state_v1,ptr @topal_x86_process_mm_inactive_token_v1)
  %ids = getelementptr i64,ptr @topal_x86_process_mm_ids_v1,i64 3
  call void @p.clear(ptr %ids,i64 3)
  br label %end
end:
  ret i64 1
}
define internal i64 @p.request(i64 %op,i64 %argument) nounwind noinline noredzone {
entry:
  %admitted = call i1 @topal_frame_kernel_admit_v1(ptr @topal_x86_frame_context_v1,i64 0,i64 0,i64 0)
  call void @pm.require(i1 %admitted)
  call void @pm.zero(ptr @topal_x86_frame_details_v1,i64 4)
  call void @pm.zero(ptr @topal_x86_process_mm_transition_v1,i64 3)
  %commit = icmp eq i64 %op,10
  %noArgument = icmp eq i64 %argument,0
  %format = or i1 %commit,%noArgument
  call void @pm.require(i1 %format)
  call void @p.prime(i64 %op)
  call void @p.initialize_ids(i64 %op)
  %switch = icmp eq i64 %op,11
  call void @p.bind(i1 %switch)
  switch i64 %op,label %bad [i64 9,label %prepare
    i64 10,label %create i64 11,label %handoff i64 12,label %exit i64 13,label %cancel]
prepare:
  %p = call i64 @p.prepare()
  ret i64 %p
create:
  %c = call i64 @p.commit(i64 %argument)
  ret i64 %c
handoff:
  %h = call i64 @p.switch()
  ret i64 %h
exit:
  %e = call i64 @p.child_exit()
  ret i64 %e
cancel:
  %r = call i64 @p.cancel_inactive()
  ret i64 %r
bad:
  call void @pm.require(i1 false)
  unreachable
}
define internal void @p.memory_entry(i64 %op) nounwind noinline noredzone {
entry:
  %pid = call i64 @p.id(i64 0)
  %unbound = icmp eq i64 %pid,0
  %cleanup = icmp eq i64 %op,6
  %skip = or i1 %unbound,%cleanup
  br i1 %skip,label %end,label %check
check:
  call void @pm.zero(ptr @topal_x86_process_mm_transition_v1,i64 3)
  call void @p.prepared_empty()
  ; The ordinary MM request has already bound its exact active context/base,
  ; complete heap and allocator handle. Descriptor initialization is a distinct
  ; startup lifetime, so early brk does not follow an unready descriptor table.
  call void @p.bank(ptr @topal_x86_frame_mm_state_v1,ptr @topal_x86_frame_mm_tokens_v1)
  call void @p.inactive()
  %ctx = call ptr @m.context()
  %phase = call i64 @m.word(ptr %ctx,i64 13)
  %ready = icmp uge i64 %phase,11
  br i1 %ready,label %descriptor,label %end
descriptor:
  call void @p.bind(i1 false)
  br label %end
end:
  ret void
}
define internal void @p.memory_return(i64 %op) nounwind noinline noredzone {
entry:
  %exec = icmp eq i64 %op,7
  %cleanup = icmp eq i64 %op,6
  br i1 %cleanup,label %retired,label %check
check:
  %pid = call i64 @p.id(i64 0)
  %bound = icmp ne i64 %pid,0
  %advance = and i1 %exec,%bound
  br i1 %advance,label %generation,label %end
generation:
  call void @pm.zero(ptr @topal_x86_frame_mm_state_v1,i64 4)
  call void @pm.zero(ptr @topal_x86_frame_mm_tokens_v1,i64 6)
  %old = call i64 @p.id(i64 2)
  %room = icmp ne i64 %old,-1
  call void @pm.require(i1 %room)
  %id = call i64 @p.issue()
  %nextGeneration = add i64 %old,1
  call void @p.set_id(i64 1,i64 %id)
  call void @p.set_id(i64 2,i64 %nextGeneration)
  br label %end
retired:
  call void @pm.zero(ptr @topal_x86_frame_mm_state_v1,i64 4)
  call void @pm.zero(ptr @topal_x86_frame_mm_tokens_v1,i64 6)
  call void @p.prepared_empty()
  call void @pm.zero(ptr @topal_x86_process_mm_inactive_state_v1,i64 4)
  call void @pm.zero(ptr @topal_x86_process_mm_inactive_token_v1,i64 3)
  call void @pm.zero(ptr @topal_x86_process_mm_transition_v1,i64 3)
  call void @p.clear(ptr @topal_x86_process_mm_ids_v1,i64 7)
  br label %end
end:
  ret void
}
