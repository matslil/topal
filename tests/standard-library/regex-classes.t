use language (version is v0.1)
use library std (version is v0.1)

Pass is Boolean constraint { value } value = true
matches? is std pattern regex contains?

class-range : Pass is Pass (matches? ("version b", "[a-c]"))
class-complement : Pass is Pass (matches? ("123x", "[^0-9]"))
decimal : Pass is Pass (matches? ("value ٤٢", "\d+"))
nondecimal : Pass is Pass (matches? ("٤x", "\D"))
whitespace : Pass is Pass (matches? ("Topal language", "Topal\slanguage"))
nonwhitespace : Pass is Pass (matches? (" x", "\S"))
word : Pass is Pass (matches? ("naïve", "\w+"))
nonword : Pass is Pass (matches? ("name!", "\W"))
decimal-complement-rejects : Pass is Pass (not (matches? ("٤٢", "^\D+$")))
whitespace-complement-rejects : Pass is Pass (not (matches? (" \n", "^\S+$")))
word-complement-rejects : Pass is Pass (not (matches? ("naïve", "^\W+$")))

(class-range, class-complement, decimal, nondecimal, whitespace,
 nonwhitespace, word, nonword, decimal-complement-rejects,
 whitespace-complement-rejects, word-complement-rejects)
