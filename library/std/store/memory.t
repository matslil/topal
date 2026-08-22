use language (
  version is v0.1
)

# The design-0 reference store is an ordered List of `(identity, value)` pairs
# while keeping identity distinct from names and paths.

lookup-step is fn (
  (found : Optional String, requested : String),
  (identity : String, value : String)
) -> Optional String
  found
    Some present then found
    None then identity = requested
      true then Some value
      false then None String
lookup-step-callable is lookup-step

### Look up an object by stable identity rather than by a path or address.
pub lookup is fn (
  entries : List (String, String),
  requested : String
) -> Optional String
  entries fold (None String) { found, entry } lookup-step-callable ((found, requested), entry)

### Return the object count without exposing representation order.
pub object-count is fn (entries : List (String, String)) -> Nat
  entry-count entries

### Test whether provided consistency and durability ranks satisfy requirements.
pub guarantees-satisfy? is fn (
  (provided-consistency : Nat, provided-durability : Nat),
  (required-consistency : Nat, required-durability : Nat)
) -> Boolean
  (provided-consistency <= required-consistency) and (provided-durability >= required-durability)
