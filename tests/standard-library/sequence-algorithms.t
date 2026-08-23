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
index-of is std sequence index-of
last-index-of is std sequence last-index-of
rotate-left is std sequence rotate-left
rotate-right is std sequence rotate-right
chunks is std sequence chunks
windows is std sequence windows
enumerate is std sequence enumerate
group-runs is std sequence group-runs
zip-pairs is std sequence zip
range-values is std sequence values
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
chunked : List List Int is Entry (prefix-two, Entry (suffix-two, Empty))
window-one : List Int is Entry (1, Entry (2, Entry (3, Empty)))
window-two : List Int is Entry (2, Entry (3, Entry (4, Empty)))
windowed : List List Int is Entry (window-one, Entry (window-two, Empty))
no-windows : List List Int is Empty
enumerated-values : List (Nat, Int) is Entry ((0, 1), Entry ((1, 2), Entry ((2, 3), Entry ((3, 4), Empty))))
run-input : List Int is Entry (1, Entry (1, Entry (2, Entry (2, Entry (1, Empty)))))
run-one : List Int is Entry (1, Entry (1, Empty))
run-two : List Int is Entry (2, Entry (2, Empty))
run-three : List Int is Entry (1, Empty)
runs : List List Int is Entry (run-one, Entry (run-two, Entry (run-three, Empty)))
letters : List String is Entry ("a", Entry ("b", Empty))
zipped : List (Int, String) is Entry ((1, "a"), Entry ((2, "b"), Empty))
range-list : List Int is Entry (2, Entry (3, Entry (4, Empty)))
index-is is fn (candidate : Optional Nat, expected : Int) -> Boolean
  candidate
    Some index then index = expected
    None then false
index-absent? is fn (candidate : Optional Nat) -> Boolean
  candidate
    Some index then false
    None then true
left-rotated : List Int is Entry (3, Entry (4, Entry (1, Entry (2, Empty))))
right-rotated : List Int is Entry (4, Entry (1, Entry (2, Entry (3, Empty))))

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
first-index : Pass is Pass (index-is (index-of (duplicates, 1), 1))
last-index : Pass is Pass (index-is (last-index-of (duplicates, 2), 2))
absent-index : Pass is Pass (index-absent? (index-of (duplicates, 9)))
left-rotation : Pass is Pass ((rotate-left (values, 2)) = left-rotated)
right-rotation-wraps : Pass is Pass ((rotate-right (values, 5)) = right-rotated)
fixed-chunks : Pass is Pass ((chunks (values, 2)) = chunked)
sliding-windows : Pass is Pass ((windows (values, 3)) = windowed)
oversized-windows : Pass is Pass ((windows (values, 8)) = no-windows)
indexed-entries : Pass is Pass ((enumerate values) = enumerated-values)
adjacent-groups : Pass is Pass ((group-runs run-input) = runs)
shortest-pairs : Pass is Pass ((zip-pairs (values, letters)) = zipped)
range-materialization : Pass is Pass ((range-values (2 ..= 4)) = range-list)

(take-prefix, take-clamps, drop-prefix, drop-clamps, split-list, take-text,
 drop-text, split-text, stable-retain, stable-reject, first-occurrences,
 first-index, last-index, absent-index, left-rotation, right-rotation-wraps,
 fixed-chunks, sliding-windows, oversized-windows, indexed-entries,
 adjacent-groups, shortest-pairs, range-materialization)
