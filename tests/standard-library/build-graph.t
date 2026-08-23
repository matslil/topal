use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
selected is std build graph selected
affected? is std build graph affected?

changed : List String is Entry ("compile-core", Empty)
edges : List (String, String) is Entry (
  ("compile-core", "compile-app"),
  Entry (
    ("compile-app", "test-app"),
    Entry (("compile-other", "test-other"), Empty)
  )
)
cycle-edges : List (String, String) is Entry (
  ("a", "b"),
  Entry (("b", "a"), Entry (("b", "test-b"), Empty))
)
cycle-change : List String is Entry ("a", Empty)
units : List String is Entry ("compile-core", Entry ("compile-app", Entry ("test-app", Entry ("compile-other", Entry ("test-other", Empty)))))
cycle-units : List String is Entry ("a", Entry ("b", Entry ("test-b", Empty)))
no-units : List String is Empty

direct-dependent-selected : Pass is Pass (affected? ("compile-app", (changed, edges, units)))
indirect-test-selected : Pass is Pass (affected? ("test-app", (changed, edges, units)))
independent-build-skipped : Pass is Pass (not (affected? ("compile-other", (changed, edges, units))))
independent-test-skipped : Pass is Pass (not (affected? ("test-other", (changed, edges, units))))
changed-unit-selected : Pass is Pass (affected? ("compile-core", (changed, edges, units)))
cycle-terminates-and-selects-test : Pass is Pass (affected? ("test-b", (cycle-change, cycle-edges, cycle-units)))
selection-has-no-duplicates : Pass is Pass ((entry-count (selected (cycle-change, (cycle-edges, cycle-units)))) = 3)
zero-pass-keeps_changed_only : Pass is Pass ((selected (changed, (edges, no-units))) = changed)

(direct-dependent-selected, indirect-test-selected, independent-build-skipped,
 independent-test-skipped, changed-unit-selected,
 cycle-terminates-and-selects-test, selection-has-no-duplicates,
 zero-pass-keeps_changed_only)
