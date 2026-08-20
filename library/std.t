#!/usr/bin/env topal
use language (
  version is v0.1
)

# The complete fundamental library is one flat `std` namespace. Functions are
# overloaded by their input classifiers; category directories are deliberately
# avoided so common fundamentals retain short, stable qualified names.

# Ordering.
### Return the smaller value. Equal operands preserve the left operand.
pub min is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    <= right then left
    otherwise right

### Return the larger value. Equal operands preserve the left operand.
pub max is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    >= right then left
    otherwise right

### Return both operands in ascending order, preserving left-biased ties.
pub min-max is fn (left : (Value : TotalOrder), right : Value) -> (Value, Value)
  left
    <= right then (left, right)
    otherwise (right, left)

# Optional.
### Test whether an Optional contains a value.
pub present? is fn (candidate : Optional (Value : Type)) -> Boolean
  candidate
    Some payload then true
    None then false

### Test whether an Optional is absent.
pub absent? is fn (candidate : Optional (Value : Type)) -> Boolean
  candidate
    Some payload then false
    None then true

### Transform a present Optional value and preserve absence.
pub map is fn (
  candidate : Optional (Input : Type),
  transformation : fn (Input) -> Output
) -> Optional Output
  candidate
    Some payload then Some (transformation payload)
    None then None Output

### Apply an Optional-returning transformation without nesting Optionals.
pub chain is fn (
  candidate : Optional (Input : Type),
  transformation : fn (Input) -> Optional Output
) -> Optional Output
  candidate
    Some payload then transformation payload
    None then None Output

optional-keep-when is fn (
  condition : Boolean,
  candidate : Optional (Value : Type)
) -> Optional Value
  condition
    true then candidate
    false then None Value
optional-keep-when-callable is optional-keep-when

### Retain a present Optional value only when it satisfies a predicate.
pub filter is fn (
  candidate : Optional (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Optional Value
  candidate
    Some payload then optional-keep-when-callable (predicate payload, candidate)
    None then None Value

### Return a present value or the eagerly evaluated fallback.
pub value-or is fn (candidate : Optional (Value : Type), fallback : Value) -> Value
  candidate
    Some payload then payload
    None then fallback

### Return the first present Optional, evaluating both arguments normally.
pub or-else is fn (
  candidate : Optional (Value : Type),
  fallback : Optional Value
) -> Optional Value
  candidate
    Some payload then candidate
    None then fallback

optional-zip-present is fn (
  left : (Left : Type),
  right : Optional (Right : Type)
) -> Optional (Left, Right)
  right
    Some right-value then Some (left, right-value)
    None then None (Left, Right)
optional-zip-present-callable is optional-zip-present

### Pair two present Optional values, or return absence if either is absent.
pub zip is fn (
  left : Optional (Left : Type),
  right : Optional (Right : Type)
) -> Optional (Left, Right)
  left
    Some left-value then optional-zip-present-callable (left-value, right)
    None then None (Left, Right)

### Remove one level of Optional nesting.
pub flatten is fn (candidate : Optional (Optional (Value : Type))) -> Optional Value
  candidate
    Some nested then nested
    None then None Value

# Result and Error.
### Test whether a Result contains a successful value.
pub ok? is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode))
) -> Boolean
  candidate
    Ok payload then true
    Error problem then false

### Test whether a Result contains an Error.
pub error? is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode))
) -> Boolean
  candidate
    Ok payload then false
    Error problem then true

### Transform a successful Result value while preserving its Error.
pub map is fn (
  candidate : Result ((Input : Type), (Codes : ErrorCode)),
  transformation : fn (Input) -> Output
) -> Result (Output, Codes)
  candidate
    Ok payload then transformation payload
    Error problem then problem

### Apply a Result-returning transformation and stop at the first Error.
pub chain is fn (
  candidate : Result ((Input : Type), (Codes : ErrorCode)),
  transformation : fn (Input) -> Result (Output, Codes)
) -> Result (Output, Codes)
  candidate
    Ok payload then transformation payload
    Error problem then problem

### Transform a Result's Error while preserving a successful value.
pub map-error is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode)),
  transformation : fn (Error) -> Error
) -> Result (Value, Codes)
  candidate
    Ok payload then payload
    Error problem then transformation problem

### Convert an Error to a successful value with a recovery function.
pub recover is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode)),
  recovery : fn (Error) -> Result (Value, Codes)
) -> Result (Value, Codes)
  candidate
    Ok payload then payload
    Error problem then recovery problem

### Return a successful Result value or the eagerly evaluated fallback.
pub value-or is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode)),
  fallback : Value
) -> Value
  candidate
    Ok payload then payload
    Error problem then fallback

### Return a successful Result or an eagerly evaluated fallback Result.
pub or-else is fn (
  candidate : Result ((Value : Type), (Codes : ErrorCode)),
  fallback : Result (Value, Codes)
) -> Result (Value, Codes)
  candidate
    Ok payload then payload
    Error problem then fallback

result-zip-ok is fn (
  left : (Left : Type),
  right : Result ((Right : Type), (Codes : ErrorCode))
) -> Result ((Left, Right), Codes)
  right
    Ok right-value then (left, right-value)
    Error problem then problem
result-zip-ok-callable is result-zip-ok

### Pair two successful Results, preserving the first Error encountered.
pub zip is fn (
  left : Result ((Left : Type), (Codes : ErrorCode)),
  right : Result ((Right : Type), Codes)
) -> Result ((Left, Right), Codes)
  left
    Ok left-value then result-zip-ok-callable (left-value, right)
    Error problem then problem

### Remove one level of Result nesting with the same Error vocabulary.
pub flatten is fn (
  candidate : Result (Result ((Value : Type), (Codes : ErrorCode)), Codes)
) -> Result (Value, Codes)
  candidate
    Ok nested then nested
    Error problem then problem

# Convex ranges.
### Return the inclusive lower bound of a Range.
pub lower-bound is fn (interval : Range (Value : TotalOrder)) -> Value
  range-lower interval

### Return the inclusive upper bound of a Range.
pub upper-bound is fn (interval : Range (Value : TotalOrder)) -> Value
  range-upper interval

### Return the inclusive lower and upper bounds of a Range.
pub bounds is fn (interval : Range (Value : TotalOrder)) -> (Value, Value)
  (range-lower interval, range-upper interval)

### Return the shared portion of two Ranges, or absence when disjoint.
pub intersection is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Range Value
  left and right

### Test whether two inclusive Ranges share at least one value.
pub overlaps? is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Boolean
  not (empty? (left and right))

range-min is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    <= right then left
    otherwise right
range-min-callable is range-min

range-max is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    >= right then left
    otherwise right
range-max-callable is range-max

### Return the smallest Range containing both input Ranges.
pub hull is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Range Value
  lower is range-min-callable (range-lower left, range-lower right)
  upper is range-max-callable (range-upper left, range-upper right)
  lower .. upper

### Test whether two Int Ranges touch without overlapping.
pub adjacent? is fn (left : Range Int, right : Range Int) -> Boolean
  ((range-upper left) + 1 = (range-lower right)) or ((range-upper right) + 1 = (range-lower left))

# Exact numbers.
### Return -1, 0, or 1 according to an Int's sign.
pub sign is fn (value : Int) -> Int
  value
    < 0 then -1
    > 0 then 1
    otherwise 0

### Return -1, 0, or 1 according to a Rational's sign.
pub sign is fn (value : Rational) -> Int
  value
    < 0 then -1
    > 0 then 1
    otherwise 0

### Return the exact nonnegative distance between two Int values.
pub distance is fn (left : Int, right : Int) -> Nat
  absolute (left - right)

### Return the exact nonnegative distance between two Rational values.
pub distance is fn (left : Rational, right : Rational) -> Rational
  absolute (left - right)

### Return the greatest common divisor as a nonnegative Nat; signs are ignored.
pub gcd is fn (left : Int, right : Int) -> Nat : Decreases (absolute right)
  right
    = 0 then absolute left
    otherwise gcd (right, left % right)

### Test whether an Int is evenly divisible by two.
pub even? is fn (value : Int) -> Boolean
  value % 2 = 0

### Test whether an Int is not evenly divisible by two.
pub odd? is fn (value : Int) -> Boolean
  value % 2 != 0

### Test exact divisibility. A zero divisor never divides a value.
pub divides? is fn (divisor : Int, dividend : Int) -> Boolean
  divisor
    = 0 then dividend = 0
    otherwise dividend % divisor = 0

### Return the exact reciprocal, or absence for zero.
pub reciprocal is fn (
  value : Rational
) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  1.0 / value

### Sum a finite List of Int values exactly; an empty List yields zero.
pub sum is fn (values : List Int) -> Int
  values fold 0 { total, value } total + value

### Sum a finite List of Rational values exactly; an empty List yields zero.
pub sum is fn (values : List Rational) -> Rational
  values fold 0.0 { total, value } total + value

### Multiply a finite List of Int values exactly; an empty List yields one.
pub product is fn (values : List Int) -> Int
  values fold 1 { total, value } total * value

### Multiply a finite List of Rational values exactly; an empty List yields one.
pub product is fn (values : List Rational) -> Rational
  values fold 1.0 { total, value } total * value

# Unicode text.
### Normalize text to Unicode NFC.
pub nfc is fn (text : String) -> String
  text normalize NFC

### Normalize text to Unicode NFD.
pub nfd is fn (text : String) -> String
  text normalize NFD

### Compare text after canonical normalization; case remains significant.
pub canonical-equal is fn (left : String, right : String) -> Boolean
  left canonically-equals right

### Compare text using Unicode default caseless matching, without locale policy.
pub caseless-equal is fn (left : String, right : String) -> Boolean
  (case-fold left) = (case-fold right)

### Test whether text begins with an exact String prefix.
pub starts-with? is fn (text : String, prefix : String) -> Boolean
  string-starts-with (text, prefix)

### Test whether text ends with an exact String suffix.
pub ends-with? is fn (text : String, suffix : String) -> Boolean
  string-ends-with (text, suffix)

### Test whether text contains an exact String fragment.
pub contains? is fn (text : String, fragment : String) -> Boolean
  string-contains (text, fragment)

### Remove Unicode whitespace from both ends of text, not from its interior.
pub trim is fn (text : String) -> String
  string-trim text

### Replace every non-overlapping exact occurrence; an empty target is rejected.
pub replace-all is fn (
  (text : String, pattern : String, replacement : String)
) -> String
  string-replace-all (text, pattern, replacement)

### Concatenate text with itself count times; zero yields the empty String.
pub repeat is fn (text : String, count : Nat) -> String
  string-repeat (text, count)

# Finite Lists.
### Test whether any List entry satisfies a predicate, stopping at the first match.
pub any? is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Boolean
  values fold false { found, value } found or (predicate value)

### Test whether every List entry satisfies a predicate, stopping at the first failure.
pub all? is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Boolean
  values fold true { accepted, value } accepted and (predicate value)

### Test whether no List entry satisfies a predicate, stopping at the first match.
pub none? is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Boolean
  not (values fold false { found, value } found or (predicate value))

increment-if is fn (count : Int, accepted : Boolean) -> Int
  accepted
    true then count + 1
    false then count
increment-if-callable is increment-if

### Count List entries satisfying a predicate.
pub count-where is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Int
  values fold 0 { count, value } increment-if-callable (count, predicate value)

present-if is fn (accepted : Boolean, value : (Value : Type)) -> Optional Value
  accepted
    true then Some value
    false then None Value
present-if-callable is present-if

find-step is fn (
  (found : Optional (Value : Type), accepted : Boolean, value : Value)
) -> Optional Value
  found
    Some payload then found
    None then present-if-callable (accepted, value)
find-step-callable is find-step

### Return the first List entry satisfying a predicate, or absence.
pub find is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Optional Value
  values fold (None Value) { found, value } find-step-callable (found, predicate value, value)

filter-map-step is fn (
  collected : List (Output : Type),
  candidate : Optional Output
) -> List Output
  candidate
    Some value then collected append value
    None then collected
filter-map-step-callable is filter-map-step

### Transform List entries and retain only present Optional results.
pub filter-map is fn (
  values : List (Input : Type),
  transformation : fn (Input) -> Optional Output
) -> List Output
  values fold (Empty Output) { collected, value } filter-map-step-callable (collected, transformation value)

### Transform each List entry to a List and concatenate the results in order.
pub flat-map is fn (
  values : List (Input : Type),
  transformation : fn (Input) -> List Output
) -> List Output
  values fold (Empty Output) { collected, value } collected concat (transformation value)

# Lazy generators.
### Yield consecutive Int values forever from initial; consumers must bound traversal.
pub count-from is fn (initial : Int) -> Generator Int Unit Unit
  initial iterate ({ value } value + 1)
