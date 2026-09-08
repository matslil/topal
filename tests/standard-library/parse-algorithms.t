use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
parse-int is std parse int
signed-integers is std parse signed-integers
unsigned-integers is std parse unsigned-integers
decimal-digits is std parse decimal-digits
decimal is std parse decimal
character-list is std parse character-list
character-string is std parse string
integer-rows is std parse integer-rows
vertical-integers is std parse vertical-integers
integer-pairs is std parse integer-pairs
integer-triples is std parse integer-triples

expected-signed : List Int is Entry (-12, Entry (7, Empty))
expected-unsigned : List Nat is Entry (12, Entry (7, Empty))
expected-digits : List Nat is Entry (9, Entry (0, Entry (5, Empty)))
parsed : Optional Int is Some -42
absent : Optional Int is None Int
expected-row-one : List Int is Entry (-1, Entry (2, Empty))
expected-row-two : List Int is Entry (3, Entry (4, Entry (5, Empty)))
expected-rows : List List Int is Entry (expected-row-one, Entry (expected-row-two, Empty))
expected-pairs : List (Int, Int) is one (-1, 2)
expected-triples : List (Int, Int, Int) is one (3, 4, 5)
expected-vertical : List Optional Int is Entry (Some 1, Entry (Some 23, Entry (Some 4, Empty)))
row-source is rows"a -1 2
no values
3 4 5
"rows
vertical-source is columns"12 
 34
+ *
"columns

strict : Pass is Pass ((parse-int "-42") = parsed)
malformed : Pass is Pass ((parse-int " 42") = absent)
signed-order : Pass is Pass ((signed-integers "x=-12 y=7") = expected-signed)
unsigned-order : Pass is Pass ((unsigned-integers "x=-12 y=7") = expected-unsigned)
digits : Pass is Pass ((decimal-digits "905") = expected-digits)
formatted : Pass is Pass ((decimal -120) = "-120")
unicode-characters : Pass is Pass ((entry-count (character-list "áb")) = 2)
unicode-round-trip : Pass is Pass ((character-string (character-list "áb")) = "áb")
rows-extracted : Pass is Pass ((integer-rows row-source) = expected-rows)
pairs-extracted : Pass is Pass ((integer-pairs row-source) = expected-pairs)
triples-extracted : Pass is Pass ((integer-triples row-source) = expected-triples)
vertical-extracted : Pass is Pass ((vertical-integers vertical-source) = expected-vertical)

(strict, malformed, signed-order, unsigned-order, digits, formatted,
 unicode-characters, unicode-round-trip, rows-extracted, pairs-extracted,
 triples-extracted, vertical-extracted)
