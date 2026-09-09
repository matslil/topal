use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
matches? is std pattern regex contains?

literal-substring : Pass is Pass (matches? ("a Topal program", "Topal"))
dot : Pass is Pass (matches? ("Topal", "T.pal"))
scalar-dot : Pass is Pass (matches? ("👩‍🔬", "^...$"))
class-range : Pass is Pass (matches? ("version b", "[a-c]"))
class-complement : Pass is Pass (matches? ("123x", "[^0-9]"))
decimal : Pass is Pass (matches? ("value ٤٢", "\d+"))
nondecimal : Pass is Pass (matches? ("٤x", "\D"))
whitespace : Pass is Pass (matches? ("Topal language", "Topal\slanguage"))
word : Pass is Pass (matches? ("naïve", "\w+"))
alternative : Pass is Pass (matches? ("dogs", "(cat|dog)s?"))
optional : Pass is Pass (matches? ("color", "colou?r"))
zero-or-more : Pass is Pass (matches? ("ac", "ab*c"))
one-or-more : Pass is Pass (not (matches? ("ac", "ab+c")))
exact-repeat : Pass is Pass (matches? ("abbc", "ab{2}c"))
bounded-repeat : Pass is Pass (matches? ("abbbc", "ab{2,4}c"))
unbounded-repeat : Pass is Pass (matches? ("abbbbbc", "ab{2,}c"))
whole-text : Pass is Pass (matches? ("Topal", "^Topal$"))
anchored-rejection : Pass is Pass (not (matches? ("aTopal", "^Topal$")))
empty-expression : Pass is Pass (matches? ("Topal", ""))
quoted-metacharacter : Pass is Pass (matches? ("a+b", "a\+b"))

(literal-substring, dot, scalar-dot, class-range, class-complement, decimal, nondecimal,
 whitespace, word, alternative, optional, zero-or-more, one-or-more,
 exact-repeat, bounded-repeat, unbounded-repeat, whole-text,
 anchored-rejection, empty-expression, quoted-metacharacter)
