use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
blank? is std text blank?
canonical-equal? is std text canonical-equal?
caseless-equal? is std text caseless-equal?
nfc is std text nfc
nfd is std text nfd
reachable is std graph reachable
reachable? is std graph reachable?
factorial is std combinatorics factorial
subset-count is std combinatorics subset-count
mean is std statistics mean
lines is std text lines
words is std text words
join is std text join

edges : List (String, String) is Entry (("a", "b"), Entry (("b", "c"), Entry (("c", "a"), Entry (("x", "y"), Empty))))
nodes : List String is Entry ("a", Entry ("b", Entry ("c", Entry ("x", Entry ("y", Empty)))))
starts : List String is one "a"
four : List Int is Entry (1, Entry (2, Entry (3, Entry (4, Empty))))
rationals : List Rational is Entry (Rational (1, 2), Entry (Rational (3, 2), Empty))
none : List Int is Empty
two-and-half : Optional Rational is Some (Rational (5, 2))
one-rational : Optional Rational is Some (Rational 1)
no-mean : Optional Rational is None Rational
line-parts : List String is Entry ("one", Entry ("two", Empty))
word-parts : List String is Entry ("one", Entry ("two", Entry ("three", Empty)))
line-source is text"one
two
"text
actual-lines : List String is lines line-source
actual-words : List String is words " one  two  three "
actual-lines-joined is join (actual-lines, "|")
actual-words-joined is join (actual-words, "|")

empty-blank : Pass is Pass (blank? "")
unicode-blank : Pass is Pass (blank? "   ")
nonblank : Pass is Pass (not (blank? " x "))
canonical : Pass is Pass (canonical-equal? ("é", "é"))
caseless : Pass is Pass (caseless-equal? ("Straße", "STRASSE"))
normal-form-round-trip : Pass is Pass ((nfc (nfd "é")) = "é")
transitive-reachability : Pass is Pass (reachable? ("a", ("c", edges, nodes)))
isolated-component : Pass is Pass (not (reachable? ("a", ("y", edges, nodes))))
closure-order : Pass is Pass ((entry-count (reachable (starts, (edges, nodes)))) = 3)
zero-factorial : Pass is Pass ((factorial 0) = 1)
five-factorial : Pass is Pass ((factorial 5) = 120)
power-set-size : Pass is Pass ((subset-count four) = 16)
exact-mean : Pass is Pass ((mean four) = two-and-half)
rational-mean : Pass is Pass ((mean rationals) = one-rational)
empty-mean : Pass is Pass ((mean none) = no-mean)
line-splitting : Pass is Pass (actual-lines-joined = "one|two")
empty-lines : Pass is Pass ((lines "") = (Empty String))
unicode-words : Pass is Pass (actual-words-joined = "one|two|three")
empty-words : Pass is Pass ((words "") = (Empty String))
empty-join : Pass is Pass ((join ((Empty String), "--")) = "")
single-join : Pass is Pass ((join ((one "one"), "--")) = "one")
exact-join : Pass is Pass ((join (word-parts, "--")) = "one--two--three")

(empty-blank, unicode-blank, nonblank, canonical, caseless,
 normal-form-round-trip, transitive-reachability,
 isolated-component, closure-order, zero-factorial, five-factorial,
 power-set-size, exact-mean, rational-mean, empty-mean, line-splitting,
 empty-lines, unicode-words, empty-words, empty-join, single-join,
 exact-join)
