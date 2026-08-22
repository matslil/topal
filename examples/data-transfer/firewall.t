#!/usr/bin/env topal
use language (
  version is v0.1
)

# A source-level firewall pipeline. The packet bytes stay in their owning
# region; each layer passes structural spans and validates only the view it
# needs. Native adapters may substitute zero-copy regions without changing
# these policy decisions.
packet-span? is std data spans span?
overlap? is std data spans spans-overlap?
ipv4? is std network addresses ipv4?

ethernet-header is (0, 14)
ipv4-header is (14, 20)
transport-header is (34, 20)
packet-length is 96

ethernet-fits is packet-span? (0, 14, packet-length)
network-fits is packet-span? (14, 20, packet-length)
transport-fits is packet-span? (34, 20, packet-length)
headers-fit is (ethernet-fits and network-fits) and transport-fits
ethernet-network-distinct is not (overlap? (ethernet-header, ipv4-header))
network-transport-distinct is not (overlap? (ipv4-header, transport-header))
layers-do-not-alias is ethernet-network-distinct and network-transport-distinct
source-address-valid is ipv4? (192, 0, 2, 10)
destination-address-valid is ipv4? (198, 51, 100, 20)

Pass is Boolean constraint { value } value = true
headers-fit-test : Pass is Pass headers-fit
layers-do-not-alias-test : Pass is Pass layers-do-not-alias
source-address-test : Pass is Pass source-address-valid
destination-address-test : Pass is Pass destination-address-valid
truncated-packet-rejected : Pass is Pass (not (packet-span? (34, 20, 53)))
malformed-address-rejected : Pass is Pass (not (ipv4? (198, 51, 100, 256)))

(headers-fit-test, layers-do-not-alias-test, source-address-test,
 destination-address-test, truncated-packet-rejected,
 malformed-address-rejected)
