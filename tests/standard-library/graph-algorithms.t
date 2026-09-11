use language (
  version is v0.1
)
use library std (
  version is v0.1
)
use library advent-of-code (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
join is std text join
reachable is std graph reachable
breadth-first is std graph breadth-first
depth-first is std graph depth-first
shortest-path is std graph shortest-path
topological-sort is std graph topological-sort
weak-components is std graph weak-components
weighted-shortest-path is std graph weighted-shortest-path
described-path-count is advent-of-code graph described-path-count
described-required-path-count is advent-of-code graph described-required-path-count

edges : List (String, String) is Entry (("a", "b"), Entry (("a", "c"), Entry (("b", "d"), Entry (("c", "d"), Empty))))
nodes : List String is Entry ("a", Entry ("b", Entry ("c", Entry ("d", Entry ("x", Empty)))))
weighted : List (String, String, Rational) is Entry (("a", "b", Rational 4), Entry (("a", "c", Rational 1), Entry (("c", "b", Rational 1), Entry (("b", "d", Rational 1), Entry (("c", "d", Rational 8), Empty)))))
cycle-edges : List (String, String) is Entry (("a", "b"), Entry (("b", "a"), Empty))
empty-edges : List (String, String) is Empty
empty-nodes : List String is Empty
duplicate-starts : List String is Entry ("a", Entry ("a", Empty))
reachable-a : List String is Entry ("a", Entry ("b", Entry ("c", Entry ("d", Empty))))
no-weighted-edges : List (String, String, Rational) is Empty
description is graph"start: a b
a: required
b: required out
required: out
"graph
required-nodes : List String is one "required"
duplicate-required-nodes : List String is Entry ("required", Entry ("required", Empty))

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
weighted-is is fn (candidate : Optional (List String, Rational)) -> Boolean
  candidate
    Some payload then weighted-payload-is payload
    None then false

duplicate-starts-are-unique : Pass is Pass ((reachable (duplicate-starts, (edges, nodes))) = reachable-a)
start-reaches-itself : Pass is Pass (path-is (shortest-path ("a", ("a", empty-edges, empty-nodes)), "a"))
breadth-order : Pass is Pass ((join (breadth-first ("a", (edges, nodes)), ",")) = "a,b,c,d")
isolated-breadth : Pass is Pass ((join (breadth-first ("x", (empty-edges, nodes)), ",")) = "x")
depth-order : Pass is Pass ((join (depth-first ("a", (edges, nodes)), ",")) = "a,b,d,c")
minimum-edges : Pass is Pass (path-is (shortest-path ("a", ("d", edges, nodes)), "a,b,d"))
missing-path : Pass is Pass (path-absent? (shortest-path ("a", ("x", edges, nodes))))
topological-order : Pass is Pass (path-is (topological-sort (edges, nodes), "a,x,b,c,d"))
cycle-has-no-order : Pass is Pass (order-absent? (topological-sort (cycle-edges, nodes)))
empty-topological-order : Pass is Pass (path-is (topological-sort (empty-edges, empty-nodes), ""))
component-count : Pass is Pass ((entry-count (weak-components (edges, nodes))) = 2)
exact-weight : Pass is Pass (weighted-is (weighted-shortest-path ("a", ("d", weighted, nodes))))
missing-weighted-path : Pass is Pass (not (weighted-is (weighted-shortest-path ("a", ("d", no-weighted-edges, nodes)))))
described-routes : Pass is Pass ((described-path-count (description, ("start", "out"))) = 3)
required-routes : Pass is Pass ((described-required-path-count (description, ("start", "out", required-nodes))) = 2)
no-described-route : Pass is Pass ((described-path-count ("start: middle", ("start", "out"))) = 0)
duplicate-requirements-count-once : Pass is Pass (
  (described-required-path-count (
    description,
    ("start", "out", duplicate-required-nodes)
  )) = 2
)

(duplicate-starts-are-unique, start-reaches-itself, breadth-order,
 isolated-breadth, depth-order, minimum-edges, missing-path,
 topological-order, cycle-has-no-order, empty-topological-order,
 component-count, exact-weight, missing-weighted-path, described-routes,
 required-routes, no-described-route, duplicate-requirements-count-once)
