; Closed target callbacks establish the actual entry, physical-access, table and
; backing owners. Callback names/word results are not source capability APIs.
declare i64 @topal_x86_user_extent_begin_v1()
declare i64 @topal_x86_user_extent_root_v1(i64)
declare ptr @topal_x86_user_extent_table_v1(i64,i64,i64,i64)
declare i64 @topal_x86_user_extent_backing_v1(i64,i64,i64,i64,i64)
declare void @topal_x86_user_extent_end_v1(i64,i64)
declare void @topal_x86_user_extent_fatal_v1() noreturn

; Private result protocol: 0 complete/no-touch-empty, 1 range, 2 absent,
; 3 permission, 4 unsupported. Broken trusted invariants never return.
define dso_local i64 @topal_x86_owned_user_extent_v1(i64 %start,i64 %length,i64 %write) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %end = add i64 %start,%length
  %wrap = icmp ult i64 %end,%start
  br i1 %wrap,label %range,label %empty_check
empty_check:
  %empty = icmp eq i64 %length,0
  br i1 %empty,label %empty_result,label %user_range
user_range:
  %low = icmp ult i64 %start,140737488355328
  %high = icmp ule i64 %end,140737488355328
  %user = and i1 %low,%high
  br i1 %user,label %direction,label %range
direction:
  %known = icmp ule i64 %write,1
  br i1 %known,label %open,label %fatal
open:
  %identity = call i64 @topal_x86_user_extent_begin_v1()
  %live = icmp ne i64 %identity,0
  br i1 %live,label %root,label %unsupported_mode
root:
  %root_word = call i64 @topal_x86_user_extent_root_v1(i64 %identity)
  %first = and i64 %start,-4096
  %last_byte = sub i64 %end,1
  %last = and i64 %last_byte,-4096
  %output = icmp eq i64 %write,1
  %required = select i1 %output,i64 7,i64 5
  br label %page
page:
  %cursor = phi i64 [%first,%root],[%next_page,%advance]
  br label %table
table:
  %physical = phi i64 [%root_word,%page],[%next_table,%descend]
  %level = phi i64 [4,%page],[%next_level,%descend]
  %alignment_bits = and i64 %physical,4095
  %aligned = icmp eq i64 %alignment_bits,0
  %in_window = icmp ule i64 %physical,1073737728
  %table_range = and i1 %aligned,%in_window
  br i1 %table_range,label %table_owner,label %fatal
table_owner:
  %storage = call ptr @topal_x86_user_extent_table_v1(i64 %identity,i64 %root_word,i64 %physical,i64 %level)
  %owned = icmp ne ptr %storage,null
  br i1 %owned,label %fetch,label %fatal
fetch:
  ; Do not speculatively fetch an unchecked next-table candidate. The target
  ; callback must return storage for the complete live owned table page.
  call void asm sideeffect "lfence", "~{memory},~{dirflag},~{fpsr},~{flags}"()
  %above = sub i64 %level,1
  %scale = mul i64 %above,9
  %shift = add i64 %scale,12
  %index_word = lshr i64 %cursor,%shift
  %index = and i64 %index_word,511
  %at = getelementptr i64,ptr %storage,i64 %index
  %word = load volatile i64,ptr %at,align 8
  %present_bit = and i64 %word,1
  %present = icmp ne i64 %present_bit,0
  br i1 %present,label %format,label %absent
format:
  ; Only the qualified address, NX and P/RW/US/A/D bits are admitted. In
  ; particular a huge-leaf/PAT or unqualified software/key format is not a table.
  %extra = and i64 %word,9218868437227409304
  %supported = icmp eq i64 %extra,0
  br i1 %supported,label %permission,label %unsupported_entry
permission:
  %grants = and i64 %word,%required
  %permitted = icmp eq i64 %grants,%required
  br i1 %permitted,label %leaf_check,label %denied
leaf_check:
  %next_table = and i64 %word,4503599627366400
  %leaf = icmp eq i64 %level,1
  br i1 %leaf,label %backing,label %descend
descend:
  %next_level = sub i64 %level,1
  br label %table
backing:
  %backing_result = call i64 @topal_x86_user_extent_backing_v1(i64 %identity,i64 %root_word,i64 %cursor,i64 %next_table,i64 %write)
  %has_owner = icmp eq i64 %backing_result,0
  br i1 %has_owner,label %covered,label %backing_failure
backing_failure:
  %owner_denied = icmp eq i64 %backing_result,1
  br i1 %owner_denied,label %denied,label %fatal
covered:
  %complete = icmp eq i64 %cursor,%last
  br i1 %complete,label %accepted,label %advance
advance:
  %next_page = add i64 %cursor,4096
  br label %page
accepted:
  br label %close
absent:
  br label %close
denied:
  br label %close
unsupported_entry:
  br label %close
close:
  %result = phi i64 [0,%accepted],[2,%absent],[3,%denied],[4,%unsupported_entry]
  call void @topal_x86_user_extent_end_v1(i64 %identity,i64 %root_word)
  ret i64 %result
empty_result:
  ret i64 0
range:
  ret i64 1
unsupported_mode:
  ret i64 4
fatal:
  call void @topal_x86_user_extent_fatal_v1()
  unreachable
}
