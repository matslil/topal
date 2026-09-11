use language (version is v0.1)
use library advent-of-code (version is v0.1)

Pass is Boolean constraint { value } value = true
fitting-region-count is advent-of-code packing fitting-region-count

description is packing"0:
#
#

2x1: 1
3x1: 2
4x1: 2
"packing

rotation-and-nonoverlap : Pass is Pass ((fitting-region-count description) = 2)

rotation-and-nonoverlap
