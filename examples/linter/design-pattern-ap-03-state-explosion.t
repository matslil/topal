use language (version is v0.1)
# This intentionally demonstrates AP-03's limitation warning. Eight coupled
# alternatives make every transition table harder to maintain; independent
# dimensions should be considered before the machine grows further.
WorkflowState is Union
  Idle
  Armed
  Reading
  Validating
  Writing
  Retrying
  Recovering
  Failed
