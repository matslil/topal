#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the described-route planning namespace.
pub revision is 1

edge-source is fn ((source : String, destination : String)) -> String
  source
edge-destination is fn ((source : String, destination : String)) -> String
  destination

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

append-unique is fn (values : List String, value : String) -> List String
  values contains-entry value
    true then values
    false then values append value

description-lines-values is fn ((values : List String, start : Nat, index : Nat)) -> List String
  values
description-lines-start is fn ((values : List String, start : Nat, index : Nat)) -> Nat
  start
description-lines-index is fn ((values : List String, start : Nat, index : Nat)) -> Nat
  index

description-optional-carriage-return? is fn (value : Optional Character) -> Boolean
  value
    Some previous then unicode-carriage-return-character previous
    None then false

description-previous-is-carriage-return? is fn ((text : String, start : Nat, index : Nat)) -> Boolean
  index > start
    true then description-optional-carriage-return? (first (collect (characters (text select-index ((index - 1) .. index)))))
    false then false

description-line-content-end is fn ((text : String, start : Nat, index : Nat)) -> Nat
  description-previous-is-carriage-return? (text, start, index)
    true then index - 1
    false then index

description-append-line is fn ((text : String, state : (List String, Nat, Nat))) -> (List String, Nat, Nat)
  index is description-lines-index state
  finish is description-line-content-end (text, description-lines-start state, index)
  ((description-lines-values state) append (text select-index ((description-lines-start state) .. finish)), index + 1, index + 1)

description-line-step is fn ((text : String, state : (List String, Nat, Nat), character : Character)) -> (List String, Nat, Nat)
  unicode-line-feed-character character
    true then description-append-line (text, state)
    false then (description-lines-values state, description-lines-start state, (description-lines-index state) + 1)

description-finish-lines is fn (text : String, state : (List String, Nat, Nat)) -> List String
  start is description-lines-start state
  length is description-lines-index state
  start < length
    true then (description-lines-values state) append (text select-index (start .. length))
    false then description-lines-values state

description-lines is fn (text : String) -> List String
  empty-lines : List String is Empty
  final is (collect (characters text)) fold (empty-lines, Nat 0, Nat 0) { state, character } description-line-step (text, state, character)
  description-finish-lines (text, final)

word-values is fn ((values : List String, start : Nat, index : Nat, inside? : Boolean)) -> List String
  values
word-start is fn ((values : List String, start : Nat, index : Nat, inside? : Boolean)) -> Nat
  start
word-index is fn ((values : List String, start : Nat, index : Nat, inside? : Boolean)) -> Nat
  index
word-inside? is fn ((values : List String, start : Nat, index : Nat, inside? : Boolean)) -> Boolean
  inside?

description-word-space is fn ((text : String, state : (List String, Nat, Nat, Boolean))) -> (List String, Nat, Nat, Boolean)
  index is word-index state
  word-inside? state
    true then ((word-values state) append (text select-index ((word-start state) .. index)), index + 1, index + 1, false)
    false then (word-values state, index + 1, index + 1, false)

description-word-content is fn (state : (List String, Nat, Nat, Boolean)) -> (List String, Nat, Nat, Boolean)
  index is word-index state
  word-inside? state
    true then (word-values state, word-start state, index + 1, true)
    false then (word-values state, index, index + 1, true)

description-word-step is fn ((text : String, state : (List String, Nat, Nat, Boolean), character : Character)) -> (List String, Nat, Nat, Boolean)
  unicode-whitespace-character character
    true then description-word-space (text, state)
    false then description-word-content state

description-finish-words is fn (text : String, state : (List String, Nat, Nat, Boolean)) -> List String
  word-inside? state
    true then (word-values state) append (text select-index ((word-start state) .. (word-index state)))
    false then word-values state

description-words is fn (text : String) -> List String
  empty-words : List String is Empty
  final is (collect (characters text)) fold (empty-words, Nat 0, Nat 0, false) { state, character } description-word-step (text, state, character)
  description-finish-words (text, final)

colon-index is fn ((index : Nat, found : List Nat)) -> Nat
  index
colon-found is fn ((index : Nat, found : List Nat)) -> List Nat
  found

colon-step is fn (state : (Nat, List Nat), character : Character) -> (Nat, List Nat)
  found is colon-found state
  (character = ":") and ((entry-count found) = 0)
    true then ((colon-index state) + 1, found append (colon-index state))
    false then ((colon-index state) + 1, found)

line-colon is fn (line : String) -> Optional Nat
  empty-found : List Nat is Empty
  final is (collect (characters line)) fold (Nat 0, empty-found) { state, character } colon-step (state, character)
  first (colon-found final)

append-destination-edge is fn ((source : String, edges : List (String, String), destination : String)) -> List (String, String)
  edges append (source, destination)

append-line-edges-at is fn ((line : String, edges : List (String, String), colon : Nat)) -> List (String, String)
  source is line select-index (0 .. colon)
  destinations is description-words (line select-index ((colon + 1) .. (entry-count line)))
  destinations fold edges { selected, destination } append-destination-edge (source, selected, destination)

append-line-edges is fn (edges : List (String, String), line : String) -> List (String, String)
  line-colon line
    Some colon then append-line-edges-at (line, edges, colon)
    None then edges

description-edges is fn (description : String) -> List (String, String)
  empty-edges : List (String, String) is Empty
  (description-lines description) fold empty-edges { edges, line } append-line-edges (edges, line)

append-edge-nodes is fn (nodes : List String, edge : (String, String)) -> List String
  with-source is append-unique (nodes, edge-source edge)
  append-unique (with-source, edge-destination edge)

description-nodes is fn (edges : List (String, String)) -> List String
  empty-nodes : List String is Empty
  edges fold empty-nodes { nodes, edge } append-edge-nodes (nodes, edge)

route-node is fn ((node : String, seen : List String, count : Int)) -> String
  node
route-seen is fn ((node : String, seen : List String, count : Int)) -> List String
  seen
route-count is fn ((node : String, seen : List String, count : Int)) -> Int
  count

canonical-seen-step is fn ((visited : List String, node : String, selected : List String, required : String)) -> List String
  present is (visited contains-entry required) or (node = required)
  present
    true then selected append required
    false then selected

canonical-seen is fn ((required : List String, visited : List String, node : String)) -> List String
  empty-seen : List String is Empty
  required fold empty-seen { selected, required-node } canonical-seen-step (visited, node, selected, required-node)

route-update-values is fn ((values : List (String, List String, Int), updated? : Boolean)) -> List (String, List String, Int)
  values
route-updated? is fn ((values : List (String, List String, Int), updated? : Boolean)) -> Boolean
  updated?

string-list-equal? is fn (left : List String, right : List String) -> Boolean
  ((entry-count left) = (entry-count right)) and (left contains-sequence right)

update-route-step is fn ((replacement : (String, List String, Int), state : (List (String, List String, Int), Boolean), candidate : (String, List String, Int))) -> (List (String, List String, Int), Boolean)
  same is ((route-node candidate) = (route-node replacement)) and (string-list-equal? (route-seen candidate, route-seen replacement))
  same
    true then ((route-update-values state) append (route-node candidate, route-seen candidate, (route-count candidate) + (route-count replacement)), true)
    false then ((route-update-values state) append candidate, route-updated? state)

finish-route-update is fn (state : (List (String, List String, Int), Boolean), replacement : (String, List String, Int)) -> List (String, List String, Int)
  route-updated? state
    true then route-update-values state
    false then (route-update-values state) append replacement

add-route is fn (values : List (String, List String, Int), replacement : (String, List String, Int)) -> List (String, List String, Int)
  empty-values : List (String, List String, Int) is Empty
  final is values fold (empty-values, false) { state, candidate } update-route-step (replacement, state, candidate)
  finish-route-update (final, replacement)

propagate-route-destination is fn ((required : List String, route : (String, List String, Int), values : List (String, List String, Int), destination : String)) -> List (String, List String, Int)
  seen is canonical-seen (required, route-seen route, destination)
  add-route (values, (destination, seen, route-count route))

propagate-route-edge is fn ((required : List String, route : (String, List String, Int), values : List (String, List String, Int), edge : (String, String))) -> List (String, List String, Int)
  (edge-source edge) = (route-node route)
    true then propagate-route-destination (required, route, values, edge-destination edge)
    false then values

propagate-route is fn ((required : List String, edges : List (String, String), values : List (String, List String, Int), route : (String, List String, Int))) -> List (String, List String, Int)
  edges fold values { updated, edge } propagate-route-edge (required, route, updated, edge)

select-node-route is fn ((node : String, selected : List (String, List String, Int), route : (String, List String, Int))) -> List (String, List String, Int)
  (route-node route) = node
    true then selected append route
    false then selected

routes-for-node is fn (values : List (String, List String, Int), node : String) -> List (String, List String, Int)
  empty-values : List (String, List String, Int) is Empty
  values fold empty-values { selected, route } select-node-route (node, selected, route)

propagate-node-routes is fn ((required : List String, edges : List (String, String), values : List (String, List String, Int), node : String)) -> List (String, List String, Int)
  routes is routes-for-node (values, node)
  routes fold values { updated, route } propagate-route (required, edges, updated, route)

unique-required-step is fn (selected : List String, node : String) -> List String
  selected contains-entry node
    true then selected
    false then selected append node

unique-required is fn (required : List String) -> List String
  empty-required : List String is Empty
  required fold empty-required { selected, node } unique-required-step (selected, node)

topological-values is fn (order : Optional (List String)) -> List String
  order
    Some values then values
    None then Empty String

route-total-step is fn ((destination : String, required : List String, total : Int, route : (String, List String, Int))) -> Int
  accepted is ((route-node route) = destination) and (string-list-equal? (route-seen route, required))
  accepted
    true then total + (route-count route)
    false then total

described-route-count is fn ((description : String, start : String, destination : String, required : List String)) -> Int
  edges is description-edges description
  nodes is description-nodes edges
  required-once is unique-required required
  initial-seen is canonical-seen (required-once, Empty String, start)
  initial-route is (start, initial-seen, 1)
  initial-values : List (String, List String, Int) is one initial-route
  order is topological-values (graph-topological (edges, nodes))
  final is order fold initial-values { values, node } propagate-node-routes (required-once, edges, values, node)
  final fold 0 { total, route } route-total-step (destination, required-once, total, route)

### Count routes in a described finite DAG, with no mandatory intermediate nodes.
pub described-path-count is fn (description : String, (start : String, destination : String)) -> Int
  described-route-count (description, start, destination, Empty String)

### Count routes in a described finite DAG that visit every required node.
pub described-required-path-count is fn (
  description : String,
  (start : String, destination : String, required : List String)
) -> Int
  described-route-count (description, start, destination, required)

