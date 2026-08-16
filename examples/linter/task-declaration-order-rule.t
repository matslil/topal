use language (
  version is v0.1,
  features is ( lint )
)
# Demonstrates a contained Topal-authored lint rule. The host supplies adjacent
# read-only task declaration phases as Int values; true preserves their order.
rule is fn static (previous : Int, current : Int) -> Boolean
  previous <= current
