use language (version is v0.1)
# This is intentionally inadequate for a gate workflow that is expected to add
# inspection and fault states. A Boolean works today but hides the future state
# space and triggers the AP-03 structural advisory.
gate-state : Boolean is false
gate-state
  false then true
  true then false
