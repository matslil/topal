use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
join is std text join
breadth-first is std graph breadth-first
depth-first is std graph depth-first
shortest-path is std graph shortest-path
topological-sort is std graph topological-sort
weak-components is std graph weak-components
weighted-shortest-path is std graph weighted-shortest-path
described-path-count is std graph described-path-count
described-required-path-count is std graph described-required-path-count

edges : List (String, String) is Entry (("a", "b"), Entry (("a", "c"), Entry (("b", "d"), Entry (("c", "d"), Empty))))
nodes : List String is Entry ("a", Entry ("b", Entry ("c", Entry ("d", Entry ("x", Empty)))))
weighted : List (String, String, Rational) is Entry (("a", "b", Rational 4), Entry (("a", "c", Rational 1), Entry (("c", "b", Rational 1), Entry (("b", "d", Rational 1), Entry (("c", "d", Rational 8), Empty)))))
cycle-edges : List (String, String) is Entry (("a", "b"), Entry (("b", "a"), Empty))
description is graph"start: a b
a: required
b: required out
required: out
"graph
required-nodes : List String is one "required"

path-is is fn (candidate : Optional (List String), expected : String) -> Boolean
  candidate
    Some path then (join (path, ",")) = expected
    None then false
path-absent? is fn (candidate : Optional (List String)) -> Boolean
  candidate
    Some path then false
    None then true
order-absent? is fn (candidate : Optional (List String)) -> Boolean
  candidate
    Some order then false
    None then true
weighted-payload-is is fn (path : List String, weight : Rational) -> Boolean
  _ is weight
  (join (path, ",")) = "a,c,b,d"
weighted-payload-is-callable is weighted-payload-is
weighted-is is fn (candidate : Optional (List String, Rational)) -> Boolean
  candidate
    Some payload then weighted-payload-is-callable payload
    None then false

breadth-order : Pass is Pass ((join (breadth-first ("a", (edges, nodes)), ",")) = "a,b,c,d")
depth-order : Pass is Pass ((join (depth-first ("a", (edges, nodes)), ",")) = "a,b,d,c")
minimum-edges : Pass is Pass (path-is (shortest-path ("a", ("d", edges, nodes)), "a,b,d"))
missing-path : Pass is Pass (path-absent? (shortest-path ("a", ("x", edges, nodes))))
topological-order : Pass is Pass (path-is (topological-sort (edges, nodes), "a,x,b,c,d"))
cycle-has-no-order : Pass is Pass (order-absent? (topological-sort (cycle-edges, nodes)))
component-count : Pass is Pass ((entry-count (weak-components (edges, nodes))) = 2)
exact-weight : Pass is Pass (weighted-is (weighted-shortest-path ("a", ("d", weighted, nodes))))
described-routes : Pass is Pass ((described-path-count (description, ("start", "out"))) = 3)
required-routes : Pass is Pass ((described-required-path-count (description, ("start", "out", required-nodes))) = 2)

(breadth-order, depth-order, minimum-edges, missing-path, topological-order,
 cycle-has-no-order, component-count, exact-weight, described-routes,
 required-routes)
