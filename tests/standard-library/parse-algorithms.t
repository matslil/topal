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

expected-signed : List Int is Entry (-12, Entry (7, Empty))
expected-unsigned : List Nat is Entry (12, Entry (7, Empty))
expected-digits : List Nat is Entry (9, Entry (0, Entry (5, Empty)))
parsed : Optional Int is Some -42
absent : Optional Int is None Int

strict : Pass is Pass ((parse-int "-42") = parsed)
malformed : Pass is Pass ((parse-int " 42") = absent)
signed-order : Pass is Pass ((signed-integers "x=-12 y=7") = expected-signed)
unsigned-order : Pass is Pass ((unsigned-integers "x=-12 y=7") = expected-unsigned)
digits : Pass is Pass ((decimal-digits "905") = expected-digits)
formatted : Pass is Pass ((decimal -120) = "-120")
unicode-characters : Pass is Pass ((entry-count (character-list "áb")) = 2)
unicode-round-trip : Pass is Pass ((character-string (character-list "áb")) = "áb")

(strict, malformed, signed-order, unsigned-order, digits, formatted,
 unicode-characters, unicode-round-trip)
