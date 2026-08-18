#!/usr/bin/env topal
use language (
  version is v0.1
)

# Generic Optional operations preserve the payload type established by the
# Optional value. They deliberately provide no forced extraction operation.
pub present? is fn (candidate : Optional (Value : Type)) -> Boolean
  candidate
    Some payload then true
    None then false

pub absent? is fn (candidate : Optional (Value : Type)) -> Boolean
  candidate
    Some payload then false
    None then true

pub map is fn (
  candidate : Optional (Input : Type),
  transformation : fn (Input) -> Output
) -> Optional Output
  candidate
    Some payload then Some (transformation payload)
    None then None Output

pub chain is fn (
  candidate : Optional (Input : Type),
  transformation : fn (Input) -> Optional Output
) -> Optional Output
  candidate
    Some payload then transformation payload
    None then None Output

keep-when is fn (
  condition : Boolean,
  candidate : Optional (Value : Type)
) -> Optional Value
  condition
    true then candidate
    false then None Value
keep-when-callable is keep-when

pub filter is fn (
  candidate : Optional (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Optional Value
  candidate
    Some payload then keep-when-callable (predicate payload, candidate)
    None then None Value

pub value-or is fn (candidate : Optional (Value : Type), fallback : Value) -> Value
  candidate
    Some payload then payload
    None then fallback

pub or-else is fn (
  candidate : Optional (Value : Type),
  fallback : Optional Value
) -> Optional Value
  candidate
    Some payload then candidate
    None then fallback

zip-present is fn (
  left : (Left : Type),
  right : Optional (Right : Type)
) -> Optional (Left, Right)
  right
    Some right-value then Some (left, right-value)
    None then None (Left, Right)
zip-present-callable is zip-present

pub zip is fn (
  left : Optional (Left : Type),
  right : Optional (Right : Type)
) -> Optional (Left, Right)
  left
    Some left-value then zip-present-callable (left-value, right)
    None then None (Left, Right)

pub flatten is fn (candidate : Optional (Optional (Value : Type))) -> Optional Value
  candidate
    Some nested then nested
    None then None Value
