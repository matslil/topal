use language (version is v0.1, features is (systems))

outside-score is fn (region : KernelByteRegion) -> Nat
  size is region byte count
  region byte load (offset-bytes is size)
    Some value then 0
    None then 1

huge-score is fn (region : KernelByteRegion) -> Nat
  region byte load (offset-bytes is 18446744073709551616)
    Some value then 0
    None then 1

byte-score is fn (region : KernelByteRegion) -> Nat
  oversized-byte is region byte store (offset-bytes is 0, value is 256)
  oversized-byte
    true then 0
    false then 1

wrapped-score is fn (region : KernelByteRegion) -> Nat
  region byte window (offset-bytes is 18446744073709551615, byte-count is 1, rights is read-only)
    Some view then 0
    None then 1

end-score is fn (region : KernelByteRegion) -> Nat
  size is region byte count
  region byte window (offset-bytes is size, byte-count is 0, rights is read-only)
    Some view then empty-view-score view
    None then 0

empty-view-score is fn (view : KernelByteRegion) -> Nat
  view byte load (offset-bytes is 0)
    Some byte then 0
    None then 1

widen-score is fn (region : KernelByteRegion) -> Nat
  size is region byte count
  region byte window (offset-bytes is size, byte-count is 0, rights is read-only)
    Some view then widened-view-score view
    None then 0

widened-view-score is fn (view : KernelByteRegion) -> Nat
  view byte window (offset-bytes is 0, byte-count is 0, rights is read-write)
    Some widened then 0
    None then 1

readonly-score is fn (region : KernelByteRegion) -> Nat
  size is region byte count
  region byte window (offset-bytes is 0, byte-count is size, rights is read-only)
    Some view then blocked-store-score view
    None then 0

blocked-store-score is fn (view : KernelByteRegion) -> Nat
  stored is view byte store (offset-bytes is 0, value is 9)
  stored
    true then 0
    false then 1

pub apply is fn (region : KernelByteRegion) -> Unit
  outside is outside-score region
  huge is huge-score region
  oversized-byte is byte-score region
  wrapped is wrapped-score region
  end is end-score region
  widen is widen-score region
  readonly is readonly-score region
  score is outside + huge + oversized-byte + wrapped + end + widen + readonly
  recorded is region byte store (offset-bytes is 0, value is score)
  ()
