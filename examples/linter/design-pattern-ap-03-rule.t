use language (
  version is v0.1,
  features is ( lint )
)
# The host supplies two bounded syntax facts: a state-named Boolean field and a
# state-named Union with eight or more alternatives. Either fact asks for an
# advisory. It cannot prove that a Boolean is wrong or that a large union is
# unmaintainable.
rule is fn static (boolean-state : Boolean, large-union : Boolean) -> Boolean
  (not boolean-state) and (not large-union)
