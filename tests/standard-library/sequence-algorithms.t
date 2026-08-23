use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
take is std sequence take
drop is std sequence drop
split-at is std sequence split-at
retain is std sequence retain
reject is std sequence reject
unique is std sequence unique
even? is fn (value : Int) -> Boolean
  value % 2 = 0

values : List Int is Entry (1, Entry (2, Entry (3, Entry (4, Empty))))
duplicates : List Int is Entry (2, Entry (1, Entry (2, Entry (3, Entry (1, Empty)))))
prefix-two : List Int is Entry (1, Entry (2, Empty))
suffix-two : List Int is Entry (3, Entry (4, Empty))
prefix-three : List Int is Entry (1, Entry (2, Entry (3, Empty)))
only-four : List Int is Entry (4, Empty)
evens : List Int is Entry (2, Entry (4, Empty))
odds : List Int is Entry (1, Entry (3, Empty))
firsts : List Int is Entry (2, Entry (1, Entry (3, Empty)))

take-prefix : Pass is Pass ((take (values, 2)) = prefix-two)
take-clamps : Pass is Pass ((take (values, 20)) = values)
drop-prefix : Pass is Pass ((drop (values, 2)) = suffix-two)
drop-clamps : Pass is Pass ((drop (values, 20)) = (Empty Int))
split-list : Pass is Pass ((split-at (values, 3)) = (prefix-three, only-four))
take-text : Pass is Pass ((take ("Topal", 3)) = "Top")
drop-text : Pass is Pass ((drop ("Topal", 3)) = "al")
split-text : Pass is Pass ((split-at ("Topal", 2)) = ("To", "pal"))
stable-retain : Pass is Pass ((retain (values, even?)) = evens)
stable-reject : Pass is Pass ((reject (values, even?)) = odds)
first-occurrences : Pass is Pass ((unique duplicates) = firsts)

(take-prefix, take-clamps, drop-prefix, drop-clamps, split-list, take-text,
 drop-text, split-text, stable-retain, stable-reject, first-occurrences)
