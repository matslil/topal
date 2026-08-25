use language (
  version is v0.1
)

# An edge `(dependency, dependent)` says that changing dependency affects
# dependent. Repeated propagation is bounded by the number of graph units, so
# cycles terminate without giving the native host any selection policy.

append-unique is fn (values : List String, value : String) -> List String
  values contains-entry value
    true then values
    false then values append value

propagate-edge is fn (
  affected : List String,
  (dependency : String, dependent : String)
) -> List String
  affected contains-entry dependency
    true then append-unique (affected, dependent)
    false then affected

propagate-once is fn (
  affected : List String,
  edges : List (String, String)
) -> List String
  edges fold affected { selected, edge } propagate-edge (selected, edge)

### Return changed units followed by every reverse-transitive dependent once.
pub selected is fn (
  changed : List String,
  (edges : List (String, String), units : List String)
) -> List String
  units fold changed { affected, unit } propagate-once (affected, edges)

### Test whether one unit belongs to the reverse-transitive affected closure.
pub affected? is fn (
  candidate : String,
  (changed : List String, edges : List (String, String), units : List String)
) -> Boolean
  (selected (changed, (edges, units))) contains-entry candidate
