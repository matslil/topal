use language (version is v0.1)
use library std (version is v0.1)

Pass is Boolean constraint { value } value = true
matches? is std pattern regex contains?

whole-text : Pass is Pass (matches? ("Topal", "^Topal$"))
start-only : Pass is Pass (matches? ("Topal language", "^Topal"))
end-only : Pass is Pass (matches? ("A Topal", "Topal$"))
anchored-rejection : Pass is Pass (not (matches? ("aTopal", "^Topal$")))
start-anchor-rejection : Pass is Pass (not (matches? ("A Topal", "^Topal")))
end-anchor-rejection : Pass is Pass (not (matches? ("Topal language", "Topal$")))
empty-expression : Pass is Pass (matches? ("Topal", ""))
substring-rejection : Pass is Pass (not (matches? ("Topal", "^Rust$")))

(whole-text, start-only, end-only, anchored-rejection,
 start-anchor-rejection, end-anchor-rejection, empty-expression,
 substring-rejection)
