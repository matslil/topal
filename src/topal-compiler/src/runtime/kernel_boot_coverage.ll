; TOPAL-SYSTEMS-KERNEL-IMAGE-BOOT-COVERAGE-001. Inactive table reads only.
@topal_x86_boot_coverage_v1 = global [64 x i64] zeroinitializer, align 8

define internal void @k.cover.fact(i64 %index,i64 %value,i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %at = getelementptr i64,ptr @topal_x86_boot_coverage_v1,i64 %index
  br i1 %save,label %store,label %compare
store:
  store volatile i64 %value,ptr %at,align 8
  ret void
compare:
  %old = load volatile i64,ptr %at,align 8
  %same = icmp eq i64 %old,%value
  call void @k.image.require(i1 %same)
  ret void
}

; Every load is bounded to the already validated whole bank, never to an
; arbitrary pointer copied from a table. No external USER leaf is followed.
define internal void @k.cover.page(i64 %base,i64 %address,i64 %cache) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %end = add i64 %base,{{BYTES}}
  br label %walk
walk:
  %table = phi i64 [%base,%0],[%next_table,%next]
  %shift = phi i64 [39,%0],[%next_shift,%next]
  %user = phi i1 [true,%0],[%u,%next]
  %nx = phi i1 [false,%0],[%n,%next]
  %low = icmp uge i64 %table,%base
  %last = add i64 %table,4096
  %high = icmp ule i64 %last,%end
  %offset = and i64 %table,4095
  %aligned = icmp eq i64 %offset,0
  %range = and i1 %low,%high
  %owned = and i1 %range,%aligned
  call void @k.image.require(i1 %owned)
  %index0 = lshr i64 %address,%shift
  %index = and i64 %index0,511
  %pointer = inttoptr i64 %table to ptr
  %entry = call i64 @t.word(ptr %pointer,i64 %index)
  %pw = and i64 %entry,3
  %available = icmp eq i64 %pw,3
  call void @k.image.require(i1 %available)
  %ub = and i64 %entry,4
  %eu = icmp ne i64 %ub,0
  %u = and i1 %user,%eu
  %en = icmp slt i64 %entry,0
  %n = or i1 %nx,%en
  %ps = and i64 %entry,128
  %large = icmp ne i64 %ps,0
  %small_level = icmp eq i64 %shift,12
  %large_level = icmp eq i64 %shift,21
  %allowed_large = or i1 %small_level,%large_level
  %nonlarge = xor i1 %large,true
  %format = or i1 %nonlarge,%allowed_large
  call void @k.image.require(i1 %format)
  %leaf = or i1 %small_level,%large
  br i1 %leaf,label %finish,label %next
next:
  %next_table = and i64 %entry,4503599627366400
  %next_shift = sub i64 %shift,9
  br label %walk
finish:
  %supervisor = xor i1 %u,true
  %rights = and i1 %supervisor,%n
  call void @k.image.require(i1 %rights)
  %unit = shl i64 1,%shift
  %mask = sub i64 %unit,1
  %clear = xor i64 %mask,-1
  %physical0 = and i64 %entry,4503599627366400
  %physical = and i64 %physical0,%clear
  %within = and i64 %address,%mask
  %translated = or i64 %physical,%within
  %identity = icmp eq i64 %translated,%address
  call void @k.image.require(i1 %identity)
  %pat_shift = select i1 %small_level,i64 7,i64 12
  %pat0 = lshr i64 %entry,%pat_shift
  %pat = and i64 %pat0,1
  %pat4 = shl i64 %pat,2
  %wtcd0 = lshr i64 %entry,3
  %wtcd = and i64 %wtcd0,3
  %actual_cache = or i64 %pat4,%wtcd
  %attributes = icmp eq i64 %actual_cache,%cache
  call void @k.image.require(i1 %attributes)
  ret void
}

define internal i64 @k.cover.extent(i64 %b,i64 %p,i64 %index,i64 %start,i64 %end,i64 %cache,i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %nonempty = icmp ult i64 %start,%end
  %bounded = icmp ule i64 %end,4294967296
  %valid = and i1 %nonempty,%bounded
  call void @k.image.require(i1 %valid)
  %lo = and i64 %start,-4096
  %sum = add i64 %end,4095
  %hi = and i64 %sum,-4096
  %length = sub i64 %hi,%lo
  %pages = lshr i64 %length,12
  %slot0 = mul i64 %index,4
  %slot = add i64 %slot0,16
  call void @k.cover.fact(i64 %slot,i64 %start,i1 %save)
  %s1 = add i64 %slot,1
  call void @k.cover.fact(i64 %s1,i64 %end,i1 %save)
  %s2 = add i64 %slot,2
  call void @k.cover.fact(i64 %s2,i64 %cache,i1 %save)
  %s3 = add i64 %slot,3
  call void @k.cover.fact(i64 %s3,i64 %pages,i1 %save)
  br label %walk
walk:
  %address = phi i64 [%lo,%0],[%next,%walk]
  call void @k.cover.page(i64 %b,i64 %address,i64 %cache)
  call void @k.cover.page(i64 %p,i64 %address,i64 %cache)
  %next = add i64 %address,4096
  %more = icmp ult i64 %next,%hi
  br i1 %more,label %walk,label %done
done:
  ret i64 %pages
}

define internal void @k.cover.verify(i64 %b,i64 %p,i1 %save) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
  %phase = call i64 @t.word(ptr @topal_x86_boot_coverage_v1,i64 0)
  %expected = select i1 %save,i64 0,i64 1
  %ordered = icmp eq i64 %phase,%expected
  call void @k.image.require(i1 %ordered)
  br label %roles
roles:
  %i = phi i64 [0,%0],[%next,%checked]
  %total = phi i64 [0,%0],[%sum,%checked]
  %fixed = icmp ult i64 %i,7
  br i1 %fixed,label %fixed_role,label %dynamic
fixed_role:
  %f = getelementptr [7 x [2 x i64]],ptr @k.cover.fixed,i64 0,i64 %i
  %fs = call i64 @t.word(ptr %f,i64 0)
  %fe = call i64 @t.word(ptr %f,i64 1)
  br label %check
dynamic:
  %inputs = icmp ult i64 %i,9
  br i1 %inputs,label %input,label %device
input:
  %params = icmp eq i64 %i,7
  %which = select i1 %params,i64 16,i64 17
  %start = call i64 @t.word(ptr @topal_x86_image_boot_observation_v1,i64 %which)
  %resource_end = call i64 @t.word(ptr @topal_x86_image_boot_observation_v1,i64 18)
  %params_end = add i64 %start,4096
  %end = select i1 %params,i64 %params_end,i64 %resource_end
  %direct = icmp ule i64 %end,1073741824
  call void @k.image.require(i1 %direct)
  br label %check
device:
  %device_i = sub i64 %i,9
  %d = getelementptr [2 x [2 x i64]],ptr @k.cover.mmio,i64 0,i64 %device_i
  %ds = call i64 @t.word(ptr %d,i64 0)
  %de = call i64 @t.word(ptr %d,i64 1)
  br label %check
check:
  %a = phi i64 [%fs,%fixed_role],[%start,%input],[%ds,%device]
  %z = phi i64 [%fe,%fixed_role],[%end,%input],[%de,%device]
  %cache = phi i64 [0,%fixed_role],[0,%input],[3,%device]
  %pages = call i64 @k.cover.extent(i64 %b,i64 %p,i64 %i,i64 %a,i64 %z,i64 %cache,i1 %save)
  %sum = add i64 %total,%pages
  %count_slot = add i64 %i,4
  call void @k.cover.fact(i64 %count_slot,i64 %pages,i1 %save)
  br label %checked
checked:
  %next = add i64 %i,1
  %more = icmp ult i64 %next,11
  br i1 %more,label %roles,label %done
done:
  call void @k.cover.fact(i64 1,i64 11,i1 %save)
  call void @k.cover.fact(i64 2,i64 %sum,i1 %save)
  call void @k.cover.fact(i64 3,i64 22,i1 %save)
  %completed = select i1 %save,i64 1,i64 2
  store volatile i64 %completed,ptr @topal_x86_boot_coverage_v1,align 8
  ret void
}
