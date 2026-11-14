use language (version is v0.1, features is (systems))
use library support (version is v0.1)

# Qualification entry; the reusable memory operation lives in support.bytes.
pub apply is fn (region : KernelByteRegion) -> Unit
  observed is region byte count
  selected : Optional KernelByteRegion is region byte window (offset-bytes is 4, byte-count is 4, rights is read-write)
  selected
    Some window then {
      copied : Boolean is support bytes copy-byte (window, 0, 1)
      ()
    }
    None then ()
