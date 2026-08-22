use language (
  version is v0.1
)

octet? is fn (value : Nat) -> Boolean
  value <= 255
octet?-callable is octet?

### Validate an IPv4 address represented by four octets.
pub ipv4? is fn ((a : Nat, b : Nat, c : Nat, d : Nat)) -> Boolean
  (((octet?-callable a) and (octet?-callable b)) and (octet?-callable c)) and (octet?-callable d)

### Validate an IPv4 prefix length without treating it as IPv6.
pub ipv4-prefix? is fn (length : Nat) -> Boolean
  length <= 32

### Validate an IPv6 prefix length without treating it as IPv4.
pub ipv6-prefix? is fn (length : Nat) -> Boolean
  length <= 128

### Keep service identity distinct from a transport candidate.
pub candidate is fn ((
  service : String,
  transport : String,
  address : String
)) -> (String, String, String)
  (service, transport, address)
