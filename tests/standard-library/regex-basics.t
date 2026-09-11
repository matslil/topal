use language (version is v0.1)
use library std (version is v0.1)

Pass is Boolean constraint { value } value = true
matches? is std pattern regex contains?

literal-substring : Pass is Pass (matches? ("a Topal program", "Topal"))
dot : Pass is Pass (matches? ("Topal", "T.pal"))
dot-rejects-line-feed : Pass is Pass (not (matches? ("a\nb", "^a.b$")))
scalar-dot : Pass is Pass (matches? ("👩‍🔬", "^...$"))
alternative : Pass is Pass (matches? ("dogs", "(cat|dog)s?"))
alternative-rejection : Pass is Pass (not (matches? ("birds", "^(cat|dog)s?$")))
quoted-metacharacter : Pass is Pass (matches? ("a+b", "a\+b"))

(literal-substring, dot, dot-rejects-line-feed, scalar-dot, alternative,
 alternative-rejection, quoted-metacharacter)
