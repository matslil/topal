use language (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
lookup is std store memory lookup
object-count is std store memory object-count
guarantees-satisfy? is std store memory guarantees-satisfy?
absent? is std absent?
value-or is std value-or

empty-store : List (String, String) is Empty
store : List (String, String) is Entry (("stable-a", "first"), Entry (("stable-b", "second"), Empty))
duplicate-store : List (String, String) is Entry (("stable-a", "first"), Entry (("stable-a", "later"), Empty))

empty-lookup-is-absent : Pass is Pass (absent? (lookup (empty-store, "stable-a")))
unknown-identity-is-absent : Pass is Pass (absent? (lookup (store, "missing")))
identity-finds-value : Pass is Pass ((value-or ((lookup (store, "stable-b")), "missing")) = "second")
first-identity-wins : Pass is Pass ((value-or ((lookup (duplicate-store, "stable-a")), "missing")) = "first")
empty-count : Pass is Pass ((object-count empty-store) = 0)
stored-count : Pass is Pass ((object-count store) = 2)
exact-guarantees-satisfy : Pass is Pass (guarantees-satisfy? ((2, 2), (2, 2)))
stronger-durability-satisfies : Pass is Pass (guarantees-satisfy? ((2, 3), (2, 2)))
insufficient-durability-rejects : Pass is Pass (not (guarantees-satisfy? ((2, 1), (2, 2))))
weaker-consistency-rejects : Pass is Pass (not (guarantees-satisfy? ((3, 2), (2, 2))))

(empty-lookup-is-absent, unknown-identity-is-absent, identity-finds-value,
 first-identity-wins, empty-count, stored-count, exact-guarantees-satisfy,
 stronger-durability-satisfies, insufficient-durability-rejects,
 weaker-consistency-rejects)
