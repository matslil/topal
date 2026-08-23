#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the finite directed-graph algorithm namespace.
pub revision is 1

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
