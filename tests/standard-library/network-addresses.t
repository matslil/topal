use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
ipv4? is std network addresses ipv4?
ipv4-prefix? is std network addresses ipv4-prefix?
ipv6-prefix? is std network addresses ipv6-prefix?
candidate is std network addresses candidate

all-zero-ipv4 : Pass is Pass (ipv4? (0, 0, 0, 0))
all-maximum-ipv4 : Pass is Pass (ipv4? (255, 255, 255, 255))
first-octet-overflow : Pass is Pass (not (ipv4? (256, 0, 0, 0)))
middle-octet-overflow : Pass is Pass (not (ipv4? (192, 0, 256, 1)))
ipv4-zero-prefix : Pass is Pass (ipv4-prefix? 0)
ipv4-full-prefix : Pass is Pass (ipv4-prefix? 32)
ipv4-prefix-overflow : Pass is Pass (not (ipv4-prefix? 33))
ipv6-zero-prefix : Pass is Pass (ipv6-prefix? 0)
ipv6-full-prefix : Pass is Pass (ipv6-prefix? 128)
ipv6-prefix-overflow : Pass is Pass (not (ipv6-prefix? 129))
families-retain-distinct-bounds : Pass is Pass ((not (ipv4-prefix? 64)) and (ipv6-prefix? 64))
candidate-preserves-service : Pass is Pass ((candidate ("dns", "udp", "192.0.2.1:53")) = ("dns", "udp", "192.0.2.1:53"))
same-service-allows-ipv6 : Pass is Pass ((candidate ("dns", "udp", "[2001:db8::1]:53")) = ("dns", "udp", "[2001:db8::1]:53"))

(all-zero-ipv4, all-maximum-ipv4, first-octet-overflow,
 middle-octet-overflow, ipv4-zero-prefix, ipv4-full-prefix,
 ipv4-prefix-overflow, ipv6-zero-prefix, ipv6-full-prefix,
 ipv6-prefix-overflow, families-retain-distinct-bounds,
 candidate-preserves-service, same-service-allows-ipv6)
