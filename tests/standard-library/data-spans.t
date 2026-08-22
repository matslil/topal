use language (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
span? is std data spans span?
span-start is std data spans span-start
span-length is std data spans span-length
overlap? is std data spans spans-overlap?
gathered-length is std data spans gathered-length
no-spans : List (Nat, Nat) is Empty
scatter : List (Nat, Nat) is Entry ((0, 3), Entry ((8, 5), Entry ((20, 0), Empty)))

zero-length-at-start : Pass is Pass (span? (0, 0, 0))
zero-length-at-end : Pass is Pass (span? (8, 0, 8))
exact-bound : Pass is Pass (span? (3, 5, 8))
past-bound : Pass is Pass (not (span? (3, 6, 8)))
start-past-bound : Pass is Pass (not (span? (9, 0, 8)))
large-naturals-remain-exact : Pass is Pass (span? (100000000000000000000, 1, 100000000000000000001))
start-projection : Pass is Pass ((span-start (7, 11)) = 7)
length-projection : Pass is Pass ((span-length (7, 11)) = 11)
touching-spans-do-not-overlap : Pass is Pass (not (overlap? ((1, 3), (4, 2))))
separated-spans-do-not-overlap : Pass is Pass (not (overlap? ((1, 2), (4, 2))))
contained-spans-overlap : Pass is Pass (overlap? ((1, 8), (3, 2)))
overlap-is-symmetric : Pass is Pass ((overlap? ((2, 5), (4, 7))) = (overlap? ((4, 7), (2, 5))))
empty-left-does-not-overlap : Pass is Pass (not (overlap? ((2, 0), (2, 3))))
empty-right-does-not-overlap : Pass is Pass (not (overlap? ((2, 3), (2, 0))))
no-spans-have-zero-length : Pass is Pass ((gathered-length no-spans) = 0)
scatter-length-is-additive : Pass is Pass ((gathered-length scatter) = 8)

(zero-length-at-start, zero-length-at-end, exact-bound, past-bound,
 start-past-bound, large-naturals-remain-exact, start-projection,
 length-projection, touching-spans-do-not-overlap,
 separated-spans-do-not-overlap, contained-spans-overlap,
 overlap-is-symmetric, empty-left-does-not-overlap,
 empty-right-does-not-overlap, no-spans-have-zero-length,
 scatter-length-is-additive)
