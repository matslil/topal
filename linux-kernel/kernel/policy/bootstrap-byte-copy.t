use language (version is v0.1, features is (systems))
use library support (version is v0.1)

# The root lends its existing owner; this entry constructs no capability.
pub apply is fn (region : KernelByteRegion) -> Unit
  copied : Boolean is support bytes copy-byte (region, 0, 1)
  ()
