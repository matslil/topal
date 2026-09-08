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

### Retain a present Optional value only when it satisfies a predicate.
pub filter is fn (
  candidate : Optional (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Optional Value
  candidate
    Some payload then optional-keep-when (predicate payload, candidate)
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

### Pair two present Optional values, or return absence if either is absent.
pub zip is fn (
  left : Optional (Left : Type),
  right : Optional (Right : Type)
) -> Optional (Left, Right)
  left
    Some left-value then optional-zip-present (left-value, right)
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

### Pair two successful Results, preserving the first Error encountered.
pub zip is fn (
  left : Result ((Left : Type), (Codes : ErrorCode)),
  right : Result ((Right : Type), Codes)
) -> Result ((Left, Right), Codes)
  left
    Ok left-value then result-zip-ok (left-value, right)
    Error problem then problem

### Remove one level of Result nesting with the same Error vocabulary.
pub flatten is fn (
  candidate : Result (Result ((Value : Type), (Codes : ErrorCode)), Codes)
) -> Result (Value, Codes)
  candidate
    Ok nested then nested
    Error problem then problem

# Convex ranges.
### Return the lower endpoint of a Range.
pub lower-bound is fn (interval : Range (Value : TotalOrder)) -> Value
  range-lower interval

### Return the upper endpoint of a Range.
pub upper-bound is fn (interval : Range (Value : TotalOrder)) -> Value
  range-upper interval

### Test whether a Range includes its lower endpoint.
pub lower-inclusive? is fn (interval : Range (Value : TotalOrder)) -> Boolean
  range-lower-inclusive? interval

### Test whether a Range includes its upper endpoint.
pub upper-inclusive? is fn (interval : Range (Value : TotalOrder)) -> Boolean
  range-upper-inclusive? interval


### Return the lower and upper endpoints of a Range.
pub bounds is fn (interval : Range (Value : TotalOrder)) -> (Value, Value)
  (range-lower interval, range-upper interval)

### Return the shared portion of two Ranges, or absence when disjoint.
pub intersection is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Range Value
  left and right

### Test whether two Ranges share at least one value.
pub overlaps? is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Boolean
  not (empty? (left and right))

range-min is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    <= right then left
    otherwise right

range-max is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    >= right then left
    otherwise right

range-hull-lower-inclusive is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Boolean
  left-lower is range-lower left
  right-lower is range-lower right
  left-inclusive is lower-inclusive? left
  right-inclusive is lower-inclusive? right
  left-lower
    < right-lower then left-inclusive
    > right-lower then right-inclusive
    otherwise left-inclusive or right-inclusive

range-hull-upper-inclusive is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Boolean
  left-upper is range-upper left
  right-upper is range-upper right
  left-inclusive is upper-inclusive? left
  right-inclusive is upper-inclusive? right
  left-upper
    > right-upper then left-inclusive
    < right-upper then right-inclusive
    otherwise left-inclusive or right-inclusive

### Return the smallest Range containing both input Ranges.
pub hull is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Range Value
  lower is range-min (range-lower left, range-lower right)
  upper is range-max (range-upper left, range-upper right)
  include-lower is range-hull-lower-inclusive (left, right)
  include-upper is range-hull-upper-inclusive (left, right)
  inclusivity is (include-lower, include-upper)
  inclusivity
    = (true, true) then lower ..= upper
    = (true, false) then lower .. upper
    = (false, true) then lower <..= upper
    otherwise lower <.. upper

range-first-int is fn (interval : Range Int) -> Int
  lower-inclusive? interval
    true then range-lower interval
    false then (range-lower interval) + 1

range-last-int is fn (interval : Range Int) -> Int
  upper-inclusive? interval
    true then range-upper interval
    false then (range-upper interval) - 1

interval-lower is fn ((lower : Int, upper : Int)) -> Int
  lower
interval-upper is fn ((lower : Int, upper : Int)) -> Int
  upper

interval-before? is fn (left : (Int, Int), right : (Int, Int)) -> Boolean
  left-lower is interval-lower left
  right-lower is interval-lower right
  left-lower < right-lower
    true then true
    false then (left-lower = right-lower) and ((interval-upper left) < (interval-upper right))

increment-interval-bound is fn (count : Nat, accepted : Boolean) -> Nat
  accepted
    true then count + 1
    false then count

interval-lower-bound is fn (values : List (Int, Int), sought : (Int, Int)) -> Nat
  zero : Nat is Nat 0
  values fold zero { count, candidate } increment-interval-bound (count, interval-before? (candidate, sought))

insert-interval is fn (candidate : (Int, Int), values : List (Int, Int)) -> List (Int, Int)
  boundary is interval-lower-bound (values, candidate)
  length is entry-count values
  (values select-index (0 .. boundary)) concat (one candidate) concat (values select-index (boundary .. length))

sort-interval-step is fn (sorted : List (Int, Int), candidate : (Int, Int)) -> List (Int, Int)
  insert-interval (candidate, sorted)

sort-intervals is fn (values : List (Int, Int)) -> List (Int, Int)
  empty-intervals : List (Int, Int) is Empty
  values fold empty-intervals { sorted, candidate } sort-interval-step (sorted, candidate)

last-interval is fn (values : List (Int, Int)) -> Optional (Int, Int)
  length is entry-count values
  length = 0
    true then None (Int, Int)
    false then first (values select-index ((length - 1) .. length))

replace-last-interval is fn (values : List (Int, Int), replacement : (Int, Int)) -> List (Int, Int)
  length is entry-count values
  (values select-index (0 .. (length - 1))) append replacement

merge-interval-present is fn ((merged : List (Int, Int), previous : (Int, Int), candidate : (Int, Int))) -> List (Int, Int)
  lower is interval-lower candidate
  upper is interval-upper candidate
  previous-upper is interval-upper previous
  lower <= (previous-upper + 1)
    true then replace-last-interval (merged, (interval-lower previous, range-max (previous-upper, upper)))
    false then merged append candidate

merge-interval-step is fn (merged : List (Int, Int), candidate : (Int, Int)) -> List (Int, Int)
  last-interval merged
    Some previous then merge-interval-present (merged, previous, candidate)
    None then merged append candidate

valid-interval-step is fn (selected : List (Int, Int), candidate : (Int, Int)) -> List (Int, Int)
  (interval-lower candidate) <= (interval-upper candidate)
    true then selected append candidate
    false then selected

coalesce-intervals is fn (intervals : List (Int, Int)) -> List (Int, Int)
  empty-intervals : List (Int, Int) is Empty
  valid is intervals fold empty-intervals { selected, candidate } valid-interval-step (selected, candidate)
  (sort-intervals valid) fold empty-intervals { merged, candidate } merge-interval-step (merged, candidate)

normalize-range-step is fn (selected : List (Int, Int), interval : Range Int) -> List (Int, Int)
  candidate is (range-first-int interval, range-last-int interval)
  valid-interval-step (selected, candidate)

ranges-to-intervals is fn (intervals : List (Range Int)) -> List (Int, Int)
  empty-intervals : List (Int, Int) is Empty
  intervals fold empty-intervals { selected, interval } normalize-range-step (selected, interval)

interval-range-step is fn (selected : List (Range Int), interval : (Int, Int)) -> List (Range Int)
  selected append ((interval-lower interval) ..= (interval-upper interval))

intervals-to-ranges is fn (intervals : List (Int, Int)) -> List (Range Int)
  empty-ranges : List (Range Int) is Empty
  intervals fold empty-ranges { selected, interval } interval-range-step (selected, interval)

### Normalize and merge overlapping or adjacent finite Int Ranges.
pub coalesce is fn (intervals : List (Range Int)) -> List (Range Int)
  intervals-to-ranges (coalesce-intervals (ranges-to-intervals intervals))

### Normalize closed endpoint pairs as inclusive Int intervals.
pub coalesce is fn (intervals : List (Int, Int)) -> List (Int, Int)
  coalesce-intervals intervals

### Test whether two Int Ranges touch without overlapping.
pub adjacent? is fn (left : Range Int, right : Range Int) -> Boolean
  left-first is range-first-int left
  left-last is range-last-int left
  right-first is range-first-int right
  right-last is range-last-int right
  (left-last + 1 = right-first) or (right-last + 1 = left-first)

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
  text-characters is collect (characters text)
  prefix-characters is collect (characters prefix)
  length is entry-count prefix-characters
  (text-characters select-index (0 .. length)) = prefix-characters

### Test whether text ends with an exact String suffix.
pub ends-with? is fn (text : String, suffix : String) -> Boolean
  text-characters is collect (characters text)
  suffix-characters is collect (characters suffix)
  text-length is entry-count text-characters
  suffix-length is entry-count suffix-characters
  suffix-length > text-length
    true then false
    false then (text-characters select-index ((text-length - suffix-length) .. text-length)) = suffix-characters

### Test whether text contains an exact String fragment.
pub contains? is fn (text : String, fragment : String) -> Boolean
  (collect (characters text)) contains-sequence (collect (characters fragment))

trim-leading-count is fn ((count : Nat, leading? : Boolean)) -> Nat
  count
trim-leading-active? is fn ((count : Nat, leading? : Boolean)) -> Boolean
  leading?

trim-leading-step is fn (state : (Nat, Boolean), character : Character) -> (Nat, Boolean)
  active? is trim-leading-active? state
  whitespace? is unicode-whitespace-character character
  active? and whitespace?
    true then ((trim-leading-count state) + 1, true)
    false then (trim-leading-count state, false)

trim-boundary is fn (values : List Character) -> Nat
  zero : Nat is Nat 0
  final is values fold (zero, true) { state, character } trim-leading-step (state, character)
  trim-leading-count final

### Remove Unicode whitespace from both ends of text, not from its interior.
pub trim is fn (text : String) -> String
  values is collect (characters text)
  leading is trim-boundary values
  trailing is trim-boundary ((collect (characters text)) reverse)
  text select-index (leading .. ((entry-count values) - trailing))

ReplacePattern is String constraint { pattern } (entry-count (collect (characters pattern))) > 0

replace-find-step is fn ((text : List Character, pattern : List Character, indexes : List Nat, index : Nat)) -> List Nat
  length is entry-count pattern
  (text select-index (index .. (index + length))) = pattern
    true then indexes append index
    false then indexes

replace-find is fn (text : String, pattern : String) -> List Nat
  checked : ReplacePattern is ReplacePattern pattern
  _ is checked
  text-characters is collect (characters text)
  pattern-characters is collect (characters pattern)
  pattern-length is entry-count pattern-characters
  text-length is entry-count text-characters
  candidates is collect (0 iterate ({ index } index + 1) take-while ({ index } index + pattern-length <= text-length))
  indexes : List Nat is Empty
  candidates fold indexes { found, index } replace-find-step (text-characters, pattern-characters, found, index)

replace-parts is fn ((parts : List String, start : Nat)) -> List String
  parts
replace-start is fn ((parts : List String, start : Nat)) -> Nat
  start

replace-split-step is fn ((text : String, pattern-length : Nat, state : (List String, Nat), index : Nat)) -> (List String, Nat)
  start is replace-start state
  index < start
    true then state
    false then ((replace-parts state) append (text select-index (start .. index)), index + pattern-length)

replace-split is fn (text : String, pattern : String) -> List String
  indexes is replace-find (text, pattern)
  pattern-length is entry-count (collect (characters pattern))
  final is indexes fold ((Empty String), Nat 0) { state, index } replace-split-step (text, pattern-length, state, index)
  (replace-parts final) append (text select-index ((replace-start final) .. (entry-count text)))

replace-joined is fn ((text : String, first? : Boolean)) -> String
  text
replace-first? is fn ((text : String, first? : Boolean)) -> Boolean
  first?

replace-join-step is fn ((replacement : String, state : (String, Boolean), part : String)) -> (String, Boolean)
  replace-first? state
    true then (part, false)
    false then ((replace-joined state) concat replacement concat part, false)

replace-source is fn ((text : String, pattern : String, replacement : String)) -> String
  final is (replace-split (text, pattern)) fold ("", true) { state, part } replace-join-step (replacement, state, part)
  replace-joined final

### Replace every non-overlapping exact occurrence; an empty target is rejected.
pub replace-all is fn (
  (text : String, pattern : String, replacement : String)
) -> String
  replace-source (text, pattern, replacement)

### Concatenate text with itself count times; zero yields the empty String.
repeat-step is fn ((repeated : String, text : String, index : Nat)) -> String
  _ is index
  repeated concat text

pub repeat is fn (text : String, count : Nat) -> String
  indexes is collect (0 iterate ({ index } index + 1) take-while ({ index } index < count))
  indexes fold "" { repeated, index } repeat-step (repeated, text, index)

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

### Count List entries satisfying a predicate.
pub count-where is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Int
  values fold 0 { count, value } increment-if (count, predicate value)

present-if is fn (accepted : Boolean, value : (Value : Type)) -> Optional Value
  accepted
    true then Some value
    false then None Value

find-step is fn (
  (found : Optional (Value : Type), accepted : Boolean, value : Value)
) -> Optional Value
  found
    Some payload then found
    None then present-if (accepted, value)

### Return the first List entry satisfying a predicate, or absence.
pub find is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Optional Value
  values fold (None Value) { found, value } find-step (found, predicate value, value)

filter-map-step is fn (
  collected : List (Output : Type),
  candidate : Optional Output
) -> List Output
  candidate
    Some value then collected append value
    None then collected

### Transform List entries and retain only present Optional results.
pub filter-map is fn (
  values : List (Input : Type),
  transformation : fn (Input) -> Optional Output
) -> List Output
  values fold (Empty Output) { collected, value } filter-map-step (collected, transformation value)

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
