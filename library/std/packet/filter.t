use language (
  version is v0.1
)

# Portable packet-filter policy. Strings represent stable boundary vocabulary
# in design-0; a later nominal-data revision may expose equivalent enums.

### Test whether a parsed packet belongs to a supported network family.
pub supported-family? is fn (family : String) -> Boolean
  (family = "ipv4") or (family = "ipv6")

### Validate monotonic, non-overlapping link, network, and transport views.
pub layout? is fn ((
  packet-length : Nat,
  link-start : Nat,
  link-length : Nat,
  network-start : Nat,
  network-length : Nat,
  transport-start : Nat,
  transport-length : Nat
)) -> Boolean
  link-end is link-start + link-length
  network-end is network-start + network-length
  transport-end is transport-start + transport-length
  ((link-end <= network-start) and (network-end <= transport-start)) and (transport-end <= packet-length)

### Convert structural validation evidence to stable decision vocabulary.
pub validity is fn (accepted : Boolean) -> String
  accepted
    true then "valid"
    false then "malformed"

### Produce the default-deny service verdict.
pub service-verdict is fn (service-policy : String) -> String
  service-policy = "allow"
    true then "accept-service"
    false then "drop-default"

### Divert fragments before applying a whole-packet verdict.
pub fragment-verdict is fn (fragment-state : String, next : String) -> String
  fragment-state = "fragment"
    true then "slow-path"
    false then next

### Apply the exact established-flow fast path before the general policy.
pub flow-verdict is fn (flow-state : String, next : String) -> String
  flow-state = "established"
    true then "accept-established"
    false then next

### Apply a denied-source rule before any accepting rule.
pub source-verdict is fn (source-policy : String, next : String) -> String
  source-policy = "deny"
    true then "drop-policy"
    false then next

### Fail closed for an unsupported network family.
pub family-verdict is fn (family-name : String, next : String) -> String
  supported-family? family-name
    true then next
    false then "drop-malformed"

### Fail closed when structural validation did not succeed.
pub validity-verdict is fn (validity-name : String, next : String) -> String
  validity-name = "valid"
    true then next
    false then "drop-malformed"

### Test whether a verdict admits forwarding on the fast path.
pub accepted? is fn (verdict-name : String) -> Boolean
  (verdict-name = "accept-established") or (verdict-name = "accept-service")

### Test whether a verdict rejects the packet.
pub dropped? is fn (verdict-name : String) -> Boolean
  ((verdict-name = "drop-malformed") or (verdict-name = "drop-policy")) or (verdict-name = "drop-default")

### Test whether a verdict transfers ownership to the bounded slow path.
pub slow-path? is fn (verdict-name : String) -> Boolean
  verdict-name = "slow-path"
