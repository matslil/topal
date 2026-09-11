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
transpose is std sequence transpose
even? is fn (value : Int) -> Boolean
  value % 2 = 0

values : List Int is Entry (1, Entry (2, Entry (3, Entry (4, Empty))))
none : List Int is Empty
one-value : List Int is one 1
duplicates : List Int is Entry (2, Entry (1, Entry (2, Entry (3, Entry (1, Empty)))))
prefix-two : List Int is Entry (1, Entry (2, Empty))
suffix-two : List Int is Entry (3, Entry (4, Empty))
prefix-three : List Int is Entry (1, Entry (2, Entry (3, Empty)))
only-four : List Int is Entry (4, Empty)
evens : List Int is Entry (2, Entry (4, Empty))
odds : List Int is Entry (1, Entry (3, Empty))
firsts : List Int is Entry (2, Entry (1, Entry (3, Empty)))
chunked : List List Int is Entry (prefix-two, Entry (suffix-two, Empty))
first-three-chunk : List Int is Entry (1, Entry (2, Entry (3, Empty)))
last-one-chunk : List Int is one 4
chunked-with-remainder : List List Int is Entry (first-three-chunk, Entry (last-one-chunk, Empty))
whole-chunk : List List Int is one values
window-one : List Int is Entry (1, Entry (2, Entry (3, Empty)))
window-two : List Int is Entry (2, Entry (3, Entry (4, Empty)))
windowed : List List Int is Entry (window-one, Entry (window-two, Empty))
no-windows : List List Int is Empty
whole-window : List List Int is one values
no-enumeration : List (Nat, Int) is Empty
enumerated-values : List (Nat, Int) is Entry ((0, 1), Entry ((1, 2), Entry ((2, 3), Entry ((3, 4), Empty))))
character-values : List Character is collect (characters "ab")
enumerated-characters : List (Nat, Character) is Entry ((0, "a"), Entry ((1, "b"), Empty))
natural-values : List Nat is Entry (Nat 2, Entry (Nat 4, Empty))
enumerated-naturals : List (Nat, Nat) is Entry ((0, 2), Entry ((1, 4), Empty))
run-input : List Int is Entry (1, Entry (1, Entry (2, Entry (2, Entry (1, Empty)))))
run-one : List Int is Entry (1, Entry (1, Empty))
run-two : List Int is Entry (2, Entry (2, Empty))
run-three : List Int is Entry (1, Empty)
runs : List List Int is Entry (run-one, Entry (run-two, Entry (run-three, Empty)))
letters : List String is Entry ("a", Entry ("b", Empty))
zipped : List (Int, String) is Entry ((1, "a"), Entry ((2, "b"), Empty))
no-pairs : List (Int, String) is Empty
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
short-row : List Int is Entry (5, Entry (6, Empty))
matrix : List List Int is Entry (values, Entry (short-row, Empty))
column-one : List Int is Entry (1, Entry (5, Empty))
column-two : List Int is Entry (2, Entry (6, Empty))
transposed : List List Int is Entry (column-one, Entry (column-two, Empty))
empty-row-matrix : List List Int is one none

take-zero : Pass is Pass ((take (values, 0)) = none)
take-prefix : Pass is Pass ((take (values, 2)) = prefix-two)
take-clamps : Pass is Pass ((take (values, 20)) = values)
drop-zero : Pass is Pass ((drop (values, 0)) = values)
drop-prefix : Pass is Pass ((drop (values, 2)) = suffix-two)
drop-clamps : Pass is Pass ((drop (values, 20)) = none)
split-start : Pass is Pass ((split-at (values, 0)) = (none, values))
split-list : Pass is Pass ((split-at (values, 3)) = (prefix-three, only-four))
take-empty-text : Pass is Pass ((take ("Topal", 0)) = "")
take-text : Pass is Pass ((take ("Topal", 3)) = "Top")
drop-empty-text : Pass is Pass ((drop ("Topal", 0)) = "Topal")
drop-text : Pass is Pass ((drop ("Topal", 3)) = "al")
split-text : Pass is Pass ((split-at ("Topal", 2)) = ("To", "pal"))
empty-retain : Pass is Pass ((retain (none, even?)) = none)
stable-retain : Pass is Pass ((retain (values, even?)) = evens)
empty-reject : Pass is Pass ((reject (none, even?)) = none)
stable-reject : Pass is Pass ((reject (values, even?)) = odds)
empty-unique : Pass is Pass ((unique none) = none)
first-occurrences : Pass is Pass ((unique duplicates) = firsts)
first-index : Pass is Pass (index-is (index-of (duplicates, 1), 1))
last-index : Pass is Pass (index-is (last-index-of (duplicates, 2), 2))
absent-index : Pass is Pass (index-absent? (index-of (duplicates, 9)))
empty-last-index : Pass is Pass (index-absent? (last-index-of (none, 9)))
empty-left-rotation : Pass is Pass ((rotate-left (none, 20)) = none)
zero-left-rotation : Pass is Pass ((rotate-left (values, 0)) = values)
left-rotation : Pass is Pass ((rotate-left (values, 2)) = left-rotated)
empty-right-rotation : Pass is Pass ((rotate-right (none, 20)) = none)
zero-right-rotation : Pass is Pass ((rotate-right (values, 0)) = values)
right-rotation-wraps : Pass is Pass ((rotate-right (values, 5)) = right-rotated)
empty-chunks : Pass is Pass ((chunks (none, 2)) = no-windows)
fixed-chunks : Pass is Pass ((chunks (values, 2)) = chunked)
remainder-chunk : Pass is Pass ((chunks (values, 3)) = chunked-with-remainder)
oversized-chunk : Pass is Pass ((chunks (values, 8)) = whole-chunk)
empty-windows : Pass is Pass ((windows (none, 2)) = no-windows)
sliding-windows : Pass is Pass ((windows (values, 3)) = windowed)
whole-list-window : Pass is Pass ((windows (values, 4)) = whole-window)
oversized-windows : Pass is Pass ((windows (values, 8)) = no-windows)
empty-enumeration : Pass is Pass ((enumerate none) = no-enumeration)
indexed-entries : Pass is Pass ((enumerate values) = enumerated-values)
character-entries : Pass is Pass ((enumerate character-values) = enumerated-characters)
natural-entries : Pass is Pass ((enumerate natural-values) = enumerated-naturals)
natural-index : Pass is Pass (index-is (index-of (natural-values, Nat 4), 1))
empty-groups : Pass is Pass ((group-runs none) = no-windows)
adjacent-groups : Pass is Pass ((group-runs run-input) = runs)
empty-left-pairs : Pass is Pass ((zip-pairs (none, letters)) = no-pairs)
shortest-pairs : Pass is Pass ((zip-pairs (values, letters)) = zipped)
empty-range-materialization : Pass is Pass ((range-values (2 .. 2)) = none)
open-range-materialization : Pass is Pass ((range-values (1 <..= 4)) = range-list)
range-materialization : Pass is Pass ((range-values (2 ..= 4)) = range-list)
empty-transpose : Pass is Pass ((transpose no-windows) = no-windows)
empty-row-transpose : Pass is Pass ((transpose empty-row-matrix) = no-windows)
shortest-transpose : Pass is Pass ((transpose matrix) = transposed)

(take-zero, take-prefix, take-clamps, drop-zero, drop-prefix, drop-clamps,
 split-start, split-list, take-empty-text, take-text, drop-empty-text,
 drop-text, split-text, empty-retain, stable-retain, empty-reject,
 stable-reject, empty-unique, first-occurrences, first-index, last-index,
 absent-index, empty-last-index, empty-left-rotation, zero-left-rotation,
 left-rotation, empty-right-rotation, zero-right-rotation,
 right-rotation-wraps, empty-chunks, fixed-chunks, remainder-chunk,
 oversized-chunk, empty-windows, sliding-windows, whole-list-window,
 oversized-windows, empty-enumeration, indexed-entries, character-entries,
 natural-entries, natural-index, empty-groups,
 adjacent-groups, empty-left-pairs, shortest-pairs,
 empty-range-materialization, open-range-materialization,
 range-materialization, empty-transpose, empty-row-transpose,
 shortest-transpose)
