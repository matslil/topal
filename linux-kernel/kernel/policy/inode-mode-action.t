use language (version is v0.1, features is (systems))
use library support (version is v0.1)

pub apply is fn ((region : KernelByteRegion, permissions : Nat, seconds : Nat)) -> Boolean
  support inode-metadata apply-mode (region, permissions, seconds)
