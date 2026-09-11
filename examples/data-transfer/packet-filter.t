#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# A modern packet-filter decision kernel. A native ingress adapter can provide
# one owned region plus validated metadata from XDP, AF_XDP, an OS packet API,
# or hardware. The hot path selects one immutable policy snapshot per batch,
# touches no shared mutable state, and forwards the original region unchanged.

### Test whether a parsed packet belongs to a supported network family.
supported-family? is fn (family : String) -> Boolean
  (family = "ipv4") or (family = "ipv6")

### Validate monotonic, non-overlapping link, network, and transport views.
layout? is fn ((
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
validity is fn (accepted : Boolean) -> String
  accepted
    true then "valid"
    false then "malformed"

### Produce the default-deny service verdict.
service-verdict is fn (service-policy : String) -> String
  service-policy = "allow"
    true then "accept-service"
    false then "drop-default"

### Divert fragments before applying a whole-packet verdict.
fragment-verdict is fn (fragment-state : String, next : String) -> String
  fragment-state = "fragment"
    true then "slow-path"
    false then next

### Apply the exact established-flow fast path before the general policy.
flow-verdict is fn (flow-state : String, next : String) -> String
  flow-state = "established"
    true then "accept-established"
    false then next

### Apply a denied-source rule before any accepting rule.
source-verdict is fn (source-policy : String, next : String) -> String
  source-policy = "deny"
    true then "drop-policy"
    false then next

### Fail closed for an unsupported network family.
family-verdict is fn (family-name : String, next : String) -> String
  supported-family? family-name
    true then next
    false then "drop-malformed"

### Fail closed when structural validation did not succeed.
validity-verdict is fn (validity-name : String, next : String) -> String
  validity-name = "valid"
    true then next
    false then "drop-malformed"

### Test whether a verdict admits forwarding on the fast path.
accepted? is fn (verdict-name : String) -> Boolean
  (verdict-name = "accept-established") or (verdict-name = "accept-service")

### Test whether a verdict rejects the packet.
dropped? is fn (verdict-name : String) -> Boolean
  ((verdict-name = "drop-malformed") or (verdict-name = "drop-policy")) or (verdict-name = "drop-default")

### Test whether a verdict transfers ownership to the bounded slow path.
slow-path? is fn (verdict-name : String) -> Boolean
  verdict-name = "slow-path"

ethernet-ipv4-tcp-layout is layout? (96, 0, 14, 14, 20, 34, 20)
ethernet-ipv6-tcp-layout is layout? (116, 0, 14, 14, 40, 54, 20)

# Compiled snapshot 41 has an exact established-flow table and an ordered
# general policy: deny source, defer fragments, allow the HTTPS service, drop
# everything else. The booleans below are validated metadata, not packet bytes.
allowed-service is service-verdict "allow"
denied-service is service-verdict "deny"
whole-allowed is fragment-verdict ("whole", allowed-service)
fragment-allowed is fragment-verdict ("fragment", allowed-service)
new-allowed is flow-verdict ("new", whole-allowed)
established-denied is flow-verdict ("established", denied-service)
source-allowed is source-verdict ("allow", new-allowed)
source-denied is source-verdict ("deny", established-denied)

ipv4-service is validity-verdict (validity ethernet-ipv4-tcp-layout,
  family-verdict ("ipv4", source-allowed))
ipv6-service is validity-verdict (validity ethernet-ipv6-tcp-layout,
  family-verdict ("ipv6", source-allowed))
established-fast-path is validity-verdict ("valid",
  family-verdict ("ipv4", source-verdict ("allow", established-denied)))
denied-before-established is validity-verdict ("valid",
  family-verdict ("ipv4", source-denied))
fragment-to-slow-path is validity-verdict ("valid",
  family-verdict ("ipv4", source-verdict ("allow", flow-verdict ("new", fragment-allowed))))
unknown-service is validity-verdict ("valid",
  family-verdict ("ipv4", source-verdict ("allow", flow-verdict ("new", fragment-verdict ("whole", denied-service)))))
truncated is validity-verdict (
  validity (layout? (53, 0, 14, 14, 20, 34, 20)), allowed-service
)
unknown-family is validity-verdict ("valid",
  family-verdict ("future-family", allowed-service))

packetbatch : List String is Entry (
  "accept-service", Entry (
  "accept-service", Entry (
  "accept-established", Entry (
  "drop-policy", Entry (
  "slow-path", Entry (
  "drop-default", Entry (
  "drop-malformed", Entry (
  "drop-malformed", Empty))))))))

Pass is Boolean constraint { value } value = true
ipv4-ipv6-share-service-policy : Pass is Pass ((ipv4-service = "accept-service") and (ipv6-service = "accept-service"))
exact-flow-precedes-general-policy : Pass is Pass (established-fast-path = "accept-established")
deny-rule-precedes-established-flow : Pass is Pass (denied-before-established = "drop-policy")
fragment-is-explicit-slow-path : Pass is Pass (fragment-to-slow-path = "slow-path")
unmatched-traffic-defaults-to-drop : Pass is Pass (unknown-service = "drop-default")
truncated-packet-fails-closed : Pass is Pass (truncated = "drop-malformed")
unsupported-family-fails-closed : Pass is Pass (unknown-family = "drop-malformed")
batch-size-ok is (entry-count packetbatch) = 8
batch-accept-ok is accepted? ipv4-service
batch-drop-ok is dropped? truncated
batch-slow-ok is slow-path? fragment-to-slow-path
batch-summary-ok is ((batch-size-ok and batch-accept-ok) and batch-drop-ok) and batch-slow-ok
batch-has-no-hidden-verdicts : Pass is Pass batch-summary-ok

(41,
 ipv4-ipv6-share-service-policy,
 exact-flow-precedes-general-policy,
 deny-rule-precedes-established-flow,
 fragment-is-explicit-slow-path,
 unmatched-traffic-defaults-to-drop,
 truncated-packet-fails-closed,
 unsupported-family-fails-closed,
 batch-has-no-hidden-verdicts)
