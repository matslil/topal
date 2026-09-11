use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
min is std min
max is std max
min-max is std min-max
lower-bound is std lower-bound
upper-bound is std upper-bound
lower-inclusive? is std lower-inclusive?
upper-inclusive? is std upper-inclusive?
bounds is std bounds
intersection is std intersection
overlaps? is std overlaps?
hull is std hull
coalesce is std coalesce
adjacent? is std adjacent?
nfc is std nfc
canonical-equal is std canonical-equal
caseless-equal is std caseless-equal
starts-with? is std starts-with?
ends-with? is std ends-with?
contains? is std contains?
trim is std trim
replace-all is std replace-all
repeat is std repeat

range-signature is fn (interval : Range Int) -> (Int, Int, Boolean, Boolean)
  (lower-bound interval, upper-bound interval,
   lower-inclusive? interval, upper-inclusive? interval)
append-range-signature is fn (
  signatures : List (Int, Int, Boolean, Boolean),
  interval : Range Int
) -> List (Int, Int, Boolean, Boolean)
  signatures append (range-signature interval)
ranges-signature is fn (values : List Range Int) -> List (Int, Int, Boolean, Boolean)
  empty-signatures : List (Int, Int, Boolean, Boolean) is Empty
  values fold empty-signatures { signatures, interval } append-range-signature (signatures, interval)

range-values : List Range Int is Entry (
  1 ..= 2,
  Entry (3 ..= 4, Entry (8 <..= 9, Entry (6 ..= 5, Empty)))
)
coalesced-ranges : List Range Int is Entry (1 ..= 4, Entry (9 ..= 9, Empty))
interval-values : List (Int, Int) is Entry (
  (3, 4),
  Entry ((1, 2), Entry ((8, 7), Empty))
)
coalesced-intervals : List (Int, Int) is one (1, 4)

minimum-left : Pass is Pass ((min (2, 4)) = 2)
minimum-right : Pass is Pass ((min (4, 2)) = 2)
maximum-left : Pass is Pass ((max (4.0, 2.0)) = (Rational 4))
maximum-right : Pass is Pass ((max (2.0, 4.0)) = (Rational 4))
ordered-pair : Pass is Pass ((min-max (4, 2)) = (2, 4))
equal-pair : Pass is Pass ((min-max (2, 2)) = (2, 2))
lower-observation : Pass is Pass ((lower-bound (-2 <..= 5)) = -2)
upper-observation : Pass is Pass ((upper-bound (-2 <..= 5)) = 5)
closed-lower : Pass is Pass (lower-inclusive? (-2 .. 5))
open-lower : Pass is Pass (not (lower-inclusive? (-2 <.. 5)))
closed-upper : Pass is Pass (upper-inclusive? (-2 <..= 5))
open-upper : Pass is Pass (not (upper-inclusive? (-2 .. 5)))
paired-bounds : Pass is Pass ((bounds (-2 .. 5)) = (-2, 5))
shared-intersection : Pass is Pass (
  (range-signature (intersection (0 .. 8, 4 .. 12))) = (4, 8, true, false)
)
empty-intersection : Pass is Pass (empty? (intersection (0 .. 2, 3 .. 5)))
closed-overlap : Pass is Pass (overlaps? (0 ..= 3, 3 ..= 5))
disjoint-ranges : Pass is Pass (not (overlaps? (0 .. 2, 3 .. 5)))
closed-hull : Pass is Pass (
  (range-signature (hull (0 ..= 2, 1 ..= 3))) = (0, 3, true, true)
)
open-upper-hull : Pass is Pass (
  (range-signature (hull (0 .. 2, 1 .. 3))) = (0, 3, true, false)
)
open-lower-hull : Pass is Pass (
  (range-signature (hull (0 <..= 2, 1 ..= 3))) = (0, 3, false, true)
)
open-hull : Pass is Pass (
  (range-signature (hull (0 <.. 2, 1 <.. 3))) = (0, 3, false, false)
)
coalesced-range-values : Pass is Pass (
  (ranges-signature (coalesce range-values)) =
    (ranges-signature coalesced-ranges)
)
coalesced-interval-values : Pass is Pass (
  (coalesce interval-values) = coalesced-intervals
)
closed-adjacency : Pass is Pass (adjacent? (0 ..= 2, 3 ..= 5))
open-gap : Pass is Pass (not (adjacent? (0 .. 2, 3 .. 5)))

normal-form : Pass is Pass ((nfc "é") = "é")
canonical-case-sensitive : Pass is Pass (not (canonical-equal ("é", "e")))
caseless-match : Pass is Pass (caseless-equal ("Straße", "STRASSE"))
caseless-rejection : Pass is Pass (not (caseless-equal ("Topal", "Rust")))
empty-prefix : Pass is Pass (starts-with? ("Topal", ""))
oversized-prefix : Pass is Pass (not (starts-with? ("Topal", "Topal language")))
empty-suffix : Pass is Pass (ends-with? ("Topal", ""))
oversized-suffix : Pass is Pass (not (ends-with? ("Topal", "A Topal")))
empty-fragment : Pass is Pass (contains? ("Topal", ""))
absent-fragment : Pass is Pass (not (contains? ("Topal", "Rust")))
empty-trim : Pass is Pass ((trim "") = "")
all-whitespace-trim : Pass is Pass ((trim "   ") = "")
interior-whitespace : Pass is Pass ((trim "a b") = "a b")
nonoverlapping-replacement : Pass is Pass ((replace-all ("aaaa", "aa", "x")) = "xx")
absent-replacement : Pass is Pass ((replace-all ("text", "missing", "x")) = "text")
zero-repeat : Pass is Pass ((repeat ("ab", 0)) = "")

(minimum-left, minimum-right, maximum-left, maximum-right, ordered-pair,
 equal-pair, lower-observation, upper-observation, closed-lower, open-lower,
 closed-upper, open-upper, paired-bounds, shared-intersection,
 empty-intersection, closed-overlap, disjoint-ranges, closed-hull,
 open-upper-hull, open-lower-hull, open-hull, coalesced-range-values,
 coalesced-interval-values, closed-adjacency, open-gap, normal-form,
 canonical-case-sensitive, caseless-match, caseless-rejection, empty-prefix,
 oversized-prefix, empty-suffix, oversized-suffix, empty-fragment,
 absent-fragment, empty-trim, all-whitespace-trim, interior-whitespace,
 nonoverlapping-replacement, absent-replacement, zero-repeat)
