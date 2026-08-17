#!/usr/bin/env topal
use language (
  version is v0.1
)

# Result success values remain unwrapped in Topal. These operations preserve a
# failure value unchanged unless an explicit Error transformation is supplied.
pub ok? is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode))
) -> Boolean
  candidate
    Ok payload then true
    Error problem then false

pub error? is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode))
) -> Boolean
  candidate
    Ok payload then false
    Error problem then true

pub map is fn (
  candidate : Result ((Input : Type), (Codes : ErrorCode)),
  transformation : fn (Input) -> Output
) -> Result (Output, Codes)
  candidate
    Ok payload then transformation payload
    Error problem then problem

pub chain is fn (
  candidate : Result ((Input : Type), (Codes : ErrorCode)),
  transformation : fn (Input) -> Result (Output, Codes)
) -> Result (Output, Codes)
  candidate
    Ok payload then transformation payload
    Error problem then problem

pub map-error is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode)),
  transformation : fn (Error) -> Error
) -> Result (Value, Codes)
  candidate
    Ok payload then payload
    Error problem then transformation problem

pub recover is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode)),
  recovery : fn (Error) -> Result (Value, Codes)
) -> Result (Value, Codes)
  candidate
    Ok payload then payload
    Error problem then recovery problem

pub value-or is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode)),
  fallback : Value
) -> Value
  candidate
    Ok payload then payload
    Error problem then fallback

pub or-else is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode)),
  fallback : Result (Value, Codes)
) -> Result (Value, Codes)
  candidate
    Ok payload then payload
    Error problem then fallback

zip-ok is fn (
  left : (Left : Type),
  right : Result ((Right : Type), (Codes : ErrorCode))
) -> Result ((Left, Right), Codes)
  right
    Ok right-value then (left, right-value)
    Error problem then problem
zip-ok-callable is zip-ok

pub zip is fn (
  left : Result ((Left : Type), (Codes : ErrorCode)),
  right : Result ((Right : Type), Codes)
) -> Result ((Left, Right), Codes)
  left
    Ok left-value then zip-ok-callable (left-value, right)
    Error problem then problem

pub flatten is fn (
  candidate : Result (Result ((Value : Type), (Codes : ErrorCode)), Codes)
) -> Result (Value, Codes)
  candidate
    Ok nested then nested
    Error problem then problem
