use language (
  version is v0.1
)

# A Span is represented in design-0 as `(start : Nat, length : Nat)`.

### Test whether a span fits within an enclosing bound.
pub span? is fn ((start : Nat, length : Nat, bound : Nat)) -> Boolean
  start + length <= bound

### Select the starting offset of a structural Span.
pub span-start is fn ((start : Nat, length : Nat)) -> Nat
  start

### Select the element count of a structural Span.
pub span-length is fn ((start : Nat, length : Nat)) -> Nat
  length

### Test whether two nonempty spans share at least one position.
pub spans-overlap? is fn (
  (left-start : Nat, left-length : Nat),
  (right-start : Nat, right-length : Nat)
) -> Boolean
  (left-length = 0) or (right-length = 0)
    true then false
    false then (left-start < (right-start + right-length)) and (right-start < (left-start + left-length))

### Return the total element count described by scatter/gather spans.
gathered-length-step is fn (total : Nat, (start : Nat, length : Nat)) -> Nat
  total + length
gathered-length-step-callable is gathered-length-step

pub gathered-length is fn (spans : List (Nat, Nat)) -> Nat
  spans fold 0 { total, span } gathered-length-step-callable (total, span)
