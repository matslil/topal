use language (version is v0.1, features is (systems))
use library policy (version is v0.1)

# Reuse the ordinary Unit implementation under an independently owned entry.
pub apply is fn (region : KernelByteRegion) -> Boolean
  copied : Unit is policy bootstrap-byte-copy apply region
  true
