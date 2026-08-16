use language (
  version is v0.1,
  features is ( lint )
)
# Demonstrates an assistive Topal-authored rule. The host reports whether a task
# has private state and whether one of its message handlers explicitly updates it.
rule is fn static (has-state : Boolean, has-transition : Boolean) -> Boolean
  has-state
    true then has-transition
    otherwise true
