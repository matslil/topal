use language (version is v0.1)
use library std (version is v0.1)

Pass is Boolean constraint { value } value = true
matches? is std pattern regex contains?

whole-text : Pass is Pass (matches? ("Topal", "^Topal$"))
anchored-rejection : Pass is Pass (not (matches? ("aTopal", "^Topal$")))
empty-expression : Pass is Pass (matches? ("Topal", ""))
substring-rejection : Pass is Pass (not (matches? ("Topal", "^Rust$")))

(whole-text, anchored-rejection, empty-expression, substring-rejection)
