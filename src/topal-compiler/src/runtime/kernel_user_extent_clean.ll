; Private ordinary-RAM predicate. Caller holds the complete stable buffer.
; Wide loads cover only complete eight-byte chunks; tails retain byte loads.
; No result or memory contents are cached across calls.
define internal i1 @t.clean(ptr %p,i64 %bytes) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  br label %words
words:
  %i = phi i64 [0,%entry],[%word_next,%word]
  %remaining = sub i64 %bytes,%i
  %wide = icmp uge i64 %remaining,8
  br i1 %wide,label %word,label %tail
word:
  %word_at = getelementptr i8,ptr %p,i64 %i
  %word_value = load volatile i64,ptr %word_at,align 1
  %word_zero = icmp eq i64 %word_value,0
  %word_next = add i64 %i,8
  br i1 %word_zero,label %words,label %bad
tail:
  %j = phi i64 [%i,%words],[%byte_next,%byte]
  %more = icmp ult i64 %j,%bytes
  br i1 %more,label %byte,label %end
byte:
  %byte_at = getelementptr i8,ptr %p,i64 %j
  %byte_value = load volatile i8,ptr %byte_at,align 1
  %byte_zero = icmp eq i8 %byte_value,0
  %byte_next = add i64 %j,1
  br i1 %byte_zero,label %tail,label %bad
end:
  ret i1 true
bad:
  ret i1 false
}
