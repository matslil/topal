use language (version is v0.1)
# The selector is a semantic untrusted fact, never an execution capability.
pub matches-selector? is fn ((number : Nat, expected : Nat)) -> Boolean
  number = expected
