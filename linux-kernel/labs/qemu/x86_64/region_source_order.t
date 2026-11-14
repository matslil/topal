use language (version is v0.1, features is (systems))

read-before-write is fn (region : KernelByteRegion) -> Nat
  region byte load (offset-bytes is 1)
    Some value then value
    None then 0

change-source is fn (region : KernelByteRegion) -> Nat
  stored is region byte store (offset-bytes is 1, value is 7)
  0

pub apply is fn (region : KernelByteRegion) -> Unit
  stored is region byte store (value is read-before-write region, offset-bytes is change-source region)
  ()
