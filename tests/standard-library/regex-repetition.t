use language (version is v0.1)
use library std (version is v0.1)

Pass is Boolean constraint { value } value = true
matches? is std pattern regex contains?

optional : Pass is Pass (matches? ("color", "colou?r"))
zero-or-more : Pass is Pass (matches? ("ac", "ab*c"))
one-or-more : Pass is Pass (not (matches? ("ac", "ab+c")))
exact-repeat : Pass is Pass (matches? ("abbc", "ab{2}c"))
bounded-repeat : Pass is Pass (matches? ("abbbc", "ab{2,4}c"))
unbounded-repeat : Pass is Pass (matches? ("abbbbbc", "ab{2,}c"))

(optional, zero-or-more, one-or-more, exact-repeat, bounded-repeat,
 unbounded-repeat)
