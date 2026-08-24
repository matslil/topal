#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the finite directed-graph algorithm namespace.
pub revision is 1

graph-breadth-first is fn (arguments : (String, List (String, String), List String)) -> List String
  graph-bfs arguments
graph-breadth-first-callable is graph-breadth-first
graph-depth-first is fn (arguments : (String, List (String, String), List String)) -> List String
  graph-dfs arguments
graph-depth-first-callable is graph-depth-first
graph-shortest is fn (arguments : (String, String, List (String, String), List String)) -> Optional (List String)
  graph-shortest-path arguments
graph-shortest-callable is graph-shortest
graph-topological is fn (arguments : (List (String, String), List String)) -> Optional (List String)
  graph-topological-sort arguments
graph-topological-callable is graph-topological
graph-components is fn (arguments : (List (String, String), List String)) -> List (List String)
  graph-weak-components arguments
graph-components-callable is graph-components
graph-weighted-shortest is fn (arguments : (String, String, List (String, String, Rational), List String)) -> Optional (List String, Rational)
  graph-weighted-shortest-path arguments
graph-weighted-shortest-callable is graph-weighted-shortest

append-unique is fn (values : List String, value : String) -> List String
  values contains-entry value
    true then values
    false then values append value
append-unique-callable is append-unique

propagate-edge is fn (
  reached : List String,
  (source : String, destination : String)
) -> List String
  reached contains-entry source
    true then append-unique-callable (reached, destination)
    false then reached
propagate-edge-callable is propagate-edge

propagate-once is fn (reached : List String, edges : List (String, String)) -> List String
  edges fold reached { selected, edge } propagate-edge-callable (selected, edge)
propagate-once-callable is propagate-once

### Return the start nodes and every reachable destination once.
pub reachable is fn (
  starts : List String,
  (edges : List (String, String), nodes : List String)
) -> List String
  nodes fold starts { reached, node } propagate-once-callable (reached, edges)
reachable-callable is reachable

### Test whether a destination is reachable from a start node, including itself.
pub reachable? is fn (
  start : String,
  (destination : String, edges : List (String, String), nodes : List String)
) -> Boolean
  starts : List String is one start
  (reachable-callable (starts, (edges, nodes))) contains-entry destination

### Visit reachable nodes in deterministic breadth-first order.
pub breadth-first is fn (
  start : String,
  (edges : List (String, String), nodes : List String)
) -> List String
  graph-breadth-first-callable (start, edges, nodes)

### Visit reachable nodes in deterministic depth-first order.
pub depth-first is fn (
  start : String,
  (edges : List (String, String), nodes : List String)
) -> List String
  graph-depth-first-callable (start, edges, nodes)

### Return a minimum-edge directed path, or absence when none exists.
pub shortest-path is fn (
  start : String,
  (destination : String, edges : List (String, String), nodes : List String)
) -> Optional (List String)
  graph-shortest-callable (start, destination, edges, nodes)

### Return a deterministic topological ordering, or absence for a cyclic graph.
pub topological-sort is fn (
  edges : List (String, String),
  nodes : List String
) -> Optional (List String)
  graph-topological-callable (edges, nodes)

### Return the weakly connected components in node order.
pub weak-components is fn (
  edges : List (String, String),
  nodes : List String
) -> List (List String)
  graph-components-callable (edges, nodes)

### Return a minimum-weight directed path and its exact total weight.
pub weighted-shortest-path is fn (
  start : String,
  (destination : String, edges : List (String, String, Rational), nodes : List String)
) -> Optional (List String, Rational)
  graph-weighted-shortest-callable (start, destination, edges, nodes)

### Count routes in a described finite DAG, with no mandatory intermediate nodes.
pub described-path-count is fn (description : String, (start : String, destination : String)) -> Int
  graph-described-path-count (description, start, destination, Empty String)

### Count routes in a described finite DAG that visit every required node.
pub described-required-path-count is fn (
  description : String,
  (start : String, destination : String, required : List String)
) -> Int
  graph-described-required-path-count (description, start, destination, required)
