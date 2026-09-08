#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the finite directed-graph algorithm namespace.
pub revision is 1

edge-source is fn ((source : String, destination : String)) -> String
  source
edge-destination is fn ((source : String, destination : String)) -> String
  destination

neighbor-step is fn ((node : String, neighbors : List String, edge : (String, String))) -> List String
  (edge-source edge) = node
    true then neighbors append (edge-destination edge)
    false then neighbors

neighbors-of is fn (node : String, edges : List (String, String)) -> List String
  empty-neighbors : List String is Empty
  edges fold empty-neighbors { neighbors, edge } neighbor-step (node, neighbors, edge)

traversal-visited is fn ((visited : List String, pending : List String)) -> List String
  visited
traversal-pending is fn ((visited : List String, pending : List String)) -> List String
  pending

bfs-neighbor is fn (state : (List String, List String), neighbor : String) -> (List String, List String)
  visited is traversal-visited state
  pending is traversal-pending state
  (visited contains-entry neighbor) or (pending contains-entry neighbor)
    true then state
    false then (visited, pending append neighbor)

bfs-node is fn ((edges : List (String, String), state : (List String, List String), node : String)) -> (List String, List String)
  visited is (traversal-visited state) append node
  initial is (visited, traversal-pending state)
  (neighbors-of (node, edges)) fold initial { next, neighbor } bfs-neighbor (next, neighbor)

bfs-optional-node is fn ((edges : List (String, String), state : (List String, List String), node : Optional String)) -> (List String, List String)
  node
    Some present then bfs-node (edges, state, present)
    None then state

bfs-once is fn (edges : List (String, String), state : (List String, List String)) -> (List String, List String)
  pending is traversal-pending state
  remaining is pending select-index (1 .. (entry-count pending))
  without-first is (traversal-visited state, remaining)
  bfs-optional-node (edges, without-first, first pending)

graph-breadth-first is fn (arguments : (String, List (String, String), List String)) -> List String
  start is fn ((value : String, edges : List (String, String), nodes : List String)) -> String
    value
  edges is fn ((value : String, edges : List (String, String), nodes : List String)) -> List (String, String)
    edges
  nodes is fn ((value : String, edges : List (String, String), nodes : List String)) -> List String
    nodes
  initial-visited : List String is Empty
  initial-pending : List String is one (start arguments)
  bounds is (nodes arguments) append (start arguments)
  final is bounds fold (initial-visited, initial-pending) { state, ignored } bfs-once (edges arguments, state)
  traversal-visited final

dfs-new-neighbor is fn (state : (List String, List String), neighbor : String) -> (List String, List String)
  visited is traversal-visited state
  selected is traversal-pending state
  (visited contains-entry neighbor) or (selected contains-entry neighbor)
    true then state
    false then (visited, selected append neighbor)

dfs-node is fn ((edges : List (String, String), remaining : List String, visited : List String, node : String)) -> (List String, List String)
  with-node is visited append node
  empty-selected : List String is Empty
  selected-state is (neighbors-of (node, edges)) fold (with-node, empty-selected) { state, neighbor } dfs-new-neighbor (state, neighbor)
  (traversal-visited selected-state, (traversal-pending selected-state) concat remaining)

dfs-unvisited is fn ((edges : List (String, String), remaining : List String, visited : List String, node : String)) -> (List String, List String)
  visited contains-entry node
    true then (visited, remaining)
    false then dfs-node (edges, remaining, visited, node)

dfs-optional-node is fn ((edges : List (String, String), remaining : List String, visited : List String, node : Optional String)) -> (List String, List String)
  node
    Some present then dfs-unvisited (edges, remaining, visited, present)
    None then (visited, remaining)

dfs-once is fn (edges : List (String, String), state : (List String, List String)) -> (List String, List String)
  pending is traversal-pending state
  remaining is pending select-index (1 .. (entry-count pending))
  dfs-optional-node (edges, remaining, traversal-visited state, first pending)

graph-depth-first is fn (arguments : (String, List (String, String), List String)) -> List String
  start is fn ((value : String, edges : List (String, String), nodes : List String)) -> String
    value
  edges is fn ((value : String, edges : List (String, String), nodes : List String)) -> List (String, String)
    edges
  nodes is fn ((value : String, edges : List (String, String), nodes : List String)) -> List String
    nodes
  initial-visited : List String is Empty
  initial-pending : List String is one (start arguments)
  bounds is (nodes arguments) concat (nodes arguments) append (start arguments)
  final is bounds fold (initial-visited, initial-pending) { state, ignored } dfs-once (edges arguments, state)
  traversal-visited final
path-visited is fn ((visited : List String, pending : List List String, found : List List String)) -> List String
  visited
path-pending is fn ((visited : List String, pending : List List String, found : List List String)) -> List List String
  pending
path-found is fn ((visited : List String, pending : List List String, found : List List String)) -> List List String
  found

path-complete? is fn (found : List List String) -> Boolean
  (entry-count found) > 0

path-last is fn (path : List String) -> Optional String
  length is entry-count path
  length = 0
    true then None String
    false then first (path select-index ((length - 1) .. length))

path-neighbor is fn ((path : List String, state : (List String, List List String, List List String), neighbor : String)) -> (List String, List List String, List List String)
  visited is path-visited state
  visited contains-entry neighbor
    true then state
    false then (visited append neighbor, (path-pending state) append (path append neighbor), path-found state)

path-expand is fn ((edges : List (String, String), path : List String, state : (List String, List List String, List List String), node : String)) -> (List String, List List String, List List String)
  (neighbors-of (node, edges)) fold state { next, neighbor } path-neighbor (path, next, neighbor)

path-node is fn ((destination : String, edges : List (String, String), path : List String, state : (List String, List List String, List List String), node : String)) -> (List String, List List String, List List String)
  node = destination
    true then (path-visited state, path-pending state, one path)
    false then path-expand (edges, path, state, node)

path-optional-node is fn ((destination : String, edges : List (String, String), path : List String, state : (List String, List List String, List List String), node : Optional String)) -> (List String, List List String, List List String)
  node
    Some present then path-node (destination, edges, path, state, present)
    None then state

path-optional-path is fn ((destination : String, edges : List (String, String), state : (List String, List List String, List List String), path : Optional (List String))) -> (List String, List List String, List List String)
  path
    Some present then path-optional-node (destination, edges, present, state, path-last present)
    None then state

path-once is fn ((destination : String, edges : List (String, String), state : (List String, List List String, List List String))) -> (List String, List List String, List List String)
  path-complete? (path-found state)
    true then state
    false then path-optional-path (destination, edges, (path-visited state, (path-pending state) select-index (1 .. (entry-count (path-pending state))), path-found state), first (path-pending state))

graph-shortest is fn (arguments : (String, String, List (String, String), List String)) -> Optional (List String)
  start is fn ((start : String, destination : String, edges : List (String, String), nodes : List String)) -> String
    start
  destination is fn ((start : String, destination : String, edges : List (String, String), nodes : List String)) -> String
    destination
  edges is fn ((start : String, destination : String, edges : List (String, String), nodes : List String)) -> List (String, String)
    edges
  nodes is fn ((start : String, destination : String, edges : List (String, String), nodes : List String)) -> List String
    nodes
  initial-visited : List String is one (start arguments)
  first-path : List String is one (start arguments)
  initial-pending : List List String is one first-path
  initial-found : List List String is Empty
  bounds is (nodes arguments) append (start arguments)
  target is destination arguments
  graph-edges is edges arguments
  final is bounds fold (initial-visited, initial-pending, initial-found) { state, ignored } path-once (target, graph-edges, state)
  first (path-found final)
incoming-step is fn ((remaining : List String, node : String, count : Nat, edge : (String, String))) -> Nat
  relevant is ((edge-destination edge) = node) and (remaining contains-entry (edge-source edge))
  relevant
    true then count + 1
    false then count

incoming-count is fn ((remaining : List String, node : String, edges : List (String, String))) -> Nat
  zero : Nat is Nat 0
  edges fold zero { count, edge } incoming-step (remaining, node, count, edge)

remove-node-step is fn ((removed : List String, sought : String, candidate : String)) -> List String
  candidate = sought
    true then removed
    false then removed append candidate

remove-node is fn (values : List String, sought : String) -> List String
  empty-values : List String is Empty
  values fold empty-values { removed, candidate } remove-node-step (removed, sought, candidate)

initial-ready-step is fn ((remaining : List String, edges : List (String, String), ready : List String, node : String)) -> List String
  (incoming-count (remaining, node, edges)) = 0
    true then ready append node
    false then ready

queued-order is fn ((order : List String, remaining : List String, ready : List String)) -> List String
  order
queued-remaining is fn ((order : List String, remaining : List String, ready : List String)) -> List String
  remaining
queued-ready is fn ((order : List String, remaining : List String, ready : List String)) -> List String
  ready

append-new-ready is fn ((remaining : List String, edges : List (String, String), ready : List String, node : String)) -> List String
  eligible is (remaining contains-entry node) and (not (ready contains-entry node))
  zero-incoming is (incoming-count (remaining, node, edges)) = 0
  eligible and zero-incoming
    true then ready append node
    false then ready

queued-topological-node is fn ((nodes : List String, edges : List (String, String), state : (List String, List String, List String), node : String)) -> (List String, List String, List String)
  remaining is remove-node (queued-remaining state, node)
  pending is (queued-ready state) select-index (1 .. (entry-count (queued-ready state)))
  ready is nodes fold pending { selected, candidate } append-new-ready (remaining, edges, selected, candidate)
  ((queued-order state) append node, remaining, ready)

queued-topological-optional is fn ((nodes : List String, edges : List (String, String), state : (List String, List String, List String), node : Optional String)) -> (List String, List String, List String)
  node
    Some present then queued-topological-node (nodes, edges, state, present)
    None then state

queued-topological-once is fn ((nodes : List String, edges : List (String, String), state : (List String, List String, List String))) -> (List String, List String, List String)
  queued-topological-optional (nodes, edges, state, first (queued-ready state))

queued-topological-finish is fn (state : (List String, List String, List String)) -> Optional (List String)
  empty-list : List String is Empty
  (entry-count (queued-remaining state)) = 0
    true then Some (queued-order state)
    false then rest empty-list

graph-topological is fn (arguments : (List (String, String), List String)) -> Optional (List String)
  edges is fn ((edges : List (String, String), nodes : List String)) -> List (String, String)
    edges
  nodes is fn ((edges : List (String, String), nodes : List String)) -> List String
    nodes
  empty-order : List String is Empty
  empty-ready : List String is Empty
  graph-nodes is nodes arguments
  graph-edges is edges arguments
  initial-ready is graph-nodes fold empty-ready { ready, node } initial-ready-step (graph-nodes, graph-edges, ready, node)
  final is graph-nodes fold (empty-order, graph-nodes, initial-ready) { state, ignored } queued-topological-once (graph-nodes, graph-edges, state)
  queued-topological-finish final
weak-reverse-neighbor is fn ((node : String, neighbors : List String, source : String, destination : String)) -> List String
  destination = node
    true then neighbors append source
    false then neighbors

weak-neighbor-step is fn ((node : String, neighbors : List String, edge : (String, String))) -> List String
  source is edge-source edge
  destination is edge-destination edge
  source = node
    true then neighbors append destination
    false then weak-reverse-neighbor (node, neighbors, source, destination)

weak-neighbors is fn (node : String, edges : List (String, String)) -> List String
  empty-neighbors : List String is Empty
  edges fold empty-neighbors { neighbors, edge } weak-neighbor-step (node, neighbors, edge)

component-values is fn ((component : List String, visited : List String, pending : List String)) -> List String
  component
component-visited is fn ((component : List String, visited : List String, pending : List String)) -> List String
  visited
component-pending is fn ((component : List String, visited : List String, pending : List String)) -> List String
  pending

component-neighbor is fn (state : (List String, List String, List String), neighbor : String) -> (List String, List String, List String)
  visited is component-visited state
  pending is component-pending state
  (visited contains-entry neighbor) or (pending contains-entry neighbor)
    true then state
    false then (component-values state, visited, pending append neighbor)

component-node is fn ((edges : List (String, String), state : (List String, List String, List String), node : String)) -> (List String, List String, List String)
  visited is (component-visited state) append node
  initial is ((component-values state) append node, visited, component-pending state)
  (weak-neighbors (node, edges)) fold initial { next, neighbor } component-neighbor (next, neighbor)

component-optional-node is fn ((edges : List (String, String), state : (List String, List String, List String), node : Optional String)) -> (List String, List String, List String)
  node
    Some present then component-node (edges, state, present)
    None then state

component-once is fn (edges : List (String, String), state : (List String, List String, List String)) -> (List String, List String, List String)
  pending is component-pending state
  without-first is (component-values state, component-visited state, pending select-index (1 .. (entry-count pending)))
  component-optional-node (edges, without-first, first pending)

collect-component is fn ((nodes : List String, edges : List (String, String), visited : List String, start : String)) -> (List String, List String)
  empty-component : List String is Empty
  initial-pending : List String is one start
  final is nodes fold (empty-component, visited, initial-pending) { state, ignored } component-once (edges, state)
  (component-values final, component-visited final)

components-values is fn ((components : List List String, visited : List String)) -> List List String
  components
components-visited is fn ((components : List List String, visited : List String)) -> List String
  visited

append-component is fn (state : (List List String, List String), found : (List String, List String)) -> (List List String, List String)
  component is fn ((component : List String, visited : List String)) -> List String
    component
  visited is fn ((component : List String, visited : List String)) -> List String
    visited
  ((components-values state) append (component found), visited found)

components-step is fn ((nodes : List String, edges : List (String, String), state : (List List String, List String), start : String)) -> (List List String, List String)
  visited is components-visited state
  visited contains-entry start
    true then state
    false then append-component (state, collect-component (nodes, edges, visited, start))

graph-components is fn (arguments : (List (String, String), List String)) -> List List String
  edges is fn ((edges : List (String, String), nodes : List String)) -> List (String, String)
    edges
  nodes is fn ((edges : List (String, String), nodes : List String)) -> List String
    nodes
  graph-nodes is nodes arguments
  empty-components : List List String is Empty
  empty-visited : List String is Empty
  final is graph-nodes fold (empty-components, empty-visited) { state, start } components-step (graph-nodes, edges arguments, state, start)
  components-values final
weighted-source is fn ((source : String, destination : String, weight : Rational)) -> String
  source
weighted-destination is fn ((source : String, destination : String, weight : Rational)) -> String
  destination
weighted-weight is fn ((source : String, destination : String, weight : Rational)) -> Rational
  weight

NonnegativeWeight is Rational constraint { weight } weight >= (Rational 0)

validate-weighted-edge is fn (validated : List (String, String, Rational), edge : (String, String, Rational)) -> List (String, String, Rational)
  checked : NonnegativeWeight is NonnegativeWeight (weighted-weight edge)
  _ is checked
  validated append edge

validate-weighted-edges is fn (edges : List (String, String, Rational)) -> List (String, String, Rational)
  empty-edges : List (String, String, Rational) is Empty
  edges fold empty-edges { validated, edge } validate-weighted-edge (validated, edge)

distance-node is fn ((node : String, weight : Rational, path : List String)) -> String
  node
distance-weight is fn ((node : String, weight : Rational, path : List String)) -> Rational
  weight
distance-path is fn ((node : String, weight : Rational, path : List String)) -> List String
  path

select-distance is fn ((sought : String, selected : List (String, Rational, List String), candidate : (String, Rational, List String))) -> List (String, Rational, List String)
  (distance-node candidate) = sought
    true then selected append candidate
    false then selected

distance-for is fn (distances : List (String, Rational, List String), sought : String) -> List (String, Rational, List String)
  empty-distances : List (String, Rational, List String) is Empty
  distances fold empty-distances { selected, candidate } select-distance (sought, selected, candidate)

replace-distance-step is fn ((replacement : (String, Rational, List String), selected : List (String, Rational, List String), candidate : (String, Rational, List String))) -> List (String, Rational, List String)
  (distance-node candidate) = (distance-node replacement)
    true then selected append replacement
    false then selected append candidate

replace-distance is fn (distances : List (String, Rational, List String), replacement : (String, Rational, List String)) -> List (String, Rational, List String)
  empty-distances : List (String, Rational, List String) is Empty
  distances fold empty-distances { selected, candidate } replace-distance-step (replacement, selected, candidate)

relax-existing is fn ((distances : List (String, Rational, List String), replacement : (String, Rational, List String), existing : (String, Rational, List String))) -> List (String, Rational, List String)
  (distance-weight replacement) < (distance-weight existing)
    true then replace-distance (distances, replacement)
    false then distances

relax-optional-existing is fn ((distances : List (String, Rational, List String), replacement : (String, Rational, List String), existing : Optional (String, Rational, List String))) -> List (String, Rational, List String)
  existing
    Some present then relax-existing (distances, replacement, present)
    None then distances append replacement

relax-from is fn ((distances : List (String, Rational, List String), edge : (String, String, Rational), source : (String, Rational, List String))) -> List (String, Rational, List String)
  destination is weighted-destination edge
  replacement is (destination, (distance-weight source) + (weighted-weight edge), (distance-path source) append destination)
  relax-optional-existing (distances, replacement, first (distance-for (distances, destination)))

relax-source is fn ((distances : List (String, Rational, List String), edge : (String, String, Rational), source : Optional (String, Rational, List String))) -> List (String, Rational, List String)
  source
    Some present then relax-from (distances, edge, present)
    None then distances

relax-edge is fn (distances : List (String, Rational, List String), edge : (String, String, Rational)) -> List (String, Rational, List String)
  relax-source (distances, edge, first (distance-for (distances, weighted-source edge)))

relax-round is fn (edges : List (String, String, Rational), distances : List (String, Rational, List String)) -> List (String, Rational, List String)
  edges fold distances { current, edge } relax-edge (current, edge)

weighted-result is fn (entry : Optional (String, Rational, List String)) -> Optional (List String, Rational)
  empty-results : List (List String, Rational) is Empty
  entry
    Some present then Some (distance-path present, distance-weight present)
    None then first empty-results

graph-weighted-shortest is fn (arguments : (String, String, List (String, String, Rational), List String)) -> Optional (List String, Rational)
  start is fn ((start : String, destination : String, edges : List (String, String, Rational), nodes : List String)) -> String
    start
  destination is fn ((start : String, destination : String, edges : List (String, String, Rational), nodes : List String)) -> String
    destination
  edges is fn ((start : String, destination : String, edges : List (String, String, Rational), nodes : List String)) -> List (String, String, Rational)
    edges
  nodes is fn ((start : String, destination : String, edges : List (String, String, Rational), nodes : List String)) -> List String
    nodes
  initial-path : List String is one (start arguments)
  initial-entry is (start arguments, Rational 0, initial-path)
  initial-distances : List (String, Rational, List String) is one initial-entry
  graph-edges is validate-weighted-edges (edges arguments)
  final is (nodes arguments) fold initial-distances { distances, ignored } relax-round (graph-edges, distances)
  weighted-result (first (distance-for (final, destination arguments)))

append-unique is fn (values : List String, value : String) -> List String
  values contains-entry value
    true then values
    false then values append value

propagate-edge is fn (
  reached : List String,
  (source : String, destination : String)
) -> List String
  reached contains-entry source
    true then append-unique (reached, destination)
    false then reached

propagate-once is fn (reached : List String, edges : List (String, String)) -> List String
  edges fold reached { selected, edge } propagate-edge (selected, edge)

### Return the start nodes and every reachable destination once.
pub reachable is fn (
  starts : List String,
  (edges : List (String, String), nodes : List String)
) -> List String
  nodes fold starts { reached, node } propagate-once (reached, edges)

### Test whether a destination is reachable from a start node, including itself.
pub reachable? is fn (
  start : String,
  (destination : String, edges : List (String, String), nodes : List String)
) -> Boolean
  starts : List String is one start
  (reachable (starts, (edges, nodes))) contains-entry destination

### Visit reachable nodes in deterministic breadth-first order.
pub breadth-first is fn (
  start : String,
  (edges : List (String, String), nodes : List String)
) -> List String
  graph-breadth-first (start, edges, nodes)

### Visit reachable nodes in deterministic depth-first order.
pub depth-first is fn (
  start : String,
  (edges : List (String, String), nodes : List String)
) -> List String
  graph-depth-first (start, edges, nodes)

### Return a minimum-edge directed path, or absence when none exists.
pub shortest-path is fn (
  start : String,
  (destination : String, edges : List (String, String), nodes : List String)
) -> Optional (List String)
  graph-shortest (start, destination, edges, nodes)

### Return a deterministic topological ordering, or absence for a cyclic graph.
pub topological-sort is fn (
  edges : List (String, String),
  nodes : List String
) -> Optional (List String)
  graph-topological (edges, nodes)

### Return the weakly connected components in node order.
pub weak-components is fn (
  edges : List (String, String),
  nodes : List String
) -> List (List String)
  graph-components (edges, nodes)

### Return a minimum-weight directed path and its exact total weight.
pub weighted-shortest-path is fn (
  start : String,
  (destination : String, edges : List (String, String, Rational), nodes : List String)
) -> Optional (List String, Rational)
  graph-weighted-shortest (start, destination, edges, nodes)

### Count routes in a described finite DAG, with no mandatory intermediate nodes.
pub described-path-count is fn (description : String, (start : String, destination : String)) -> Int
  graph-described-path-count (description, start, destination, Empty String)

### Count routes in a described finite DAG that visit every required node.
pub described-required-path-count is fn (
  description : String,
  (start : String, destination : String, required : List String)
) -> Int
  graph-described-required-path-count (description, start, destination, required)
