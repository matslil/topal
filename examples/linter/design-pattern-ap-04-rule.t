use language (
  version is v0.1,
  features is ( lint )
)
# The host supplies bounded syntax facts for a protocol-named Boolean and a
# protocol-named Union. A union model is still limited: it cannot establish
# typestate without affine, state-indexed capabilities.
rule is fn static (boolean-protocol : Boolean, model-protocol : Boolean) -> Boolean
  (not boolean-protocol) and (not model-protocol)
