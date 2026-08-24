#!/usr/bin/env topal
use language (version is v0.1)
### Revision of the exact finite polyomino-packing namespace.
pub revision is 1
### Count rectangular regions that admit the requested free polyomino multiset.
pub fitting-region-count is fn (description : String) -> Int
  packing-described-fit-count description
