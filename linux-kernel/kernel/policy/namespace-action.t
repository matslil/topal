use language (version is v0.1, features is (systems))
use library support (version is v0.1)

pub apply is fn ((region : KernelByteRegion, operation : Nat, seconds : Nat)) -> Boolean
  support namespace-metadata apply (region, operation, seconds)
