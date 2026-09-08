use language (version is v0.1)
use library std (version is v0.1)

Pass is Boolean constraint { value } value = true
fitting-region-count is std packing fitting-region-count

description is packing"0:
##

2x1: 1
1x1: 1
"packing

one-region-fits : Pass is Pass ((fitting-region-count description) = 1)

one-region-fits
