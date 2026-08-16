use language (
  version is v0.1,
  features is ( lint )
)
# Demonstrates the lint language variant and a static rule entry point. The
# bootstrap checker validates this module before future Topal rule execution.
rule is fn static () -> Unit
  ()
