# Model extensible workflow state as an algebraic union

Use a named `Union` when a workflow has several meaningful modes or is likely
to acquire them. A transition then makes each mode visible and an exhaustive
decision exposes every location that must consider a new alternative.

The warning is deliberately advisory. A Boolean is still the clearest choice
for a stable binary fact, and a large state union can be the honest model when
its alternatives are intrinsically coupled. The rule recognizes only a
state-named Boolean field and a state-named union with at least eight
alternatives; it does not prove workflow behavior, reachability, or runtime
cost.
