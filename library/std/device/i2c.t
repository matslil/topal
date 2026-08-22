use language (
  version is v0.1
)

### Validate an ordinary non-reserved seven-bit I2C target address.
pub i2c-seven-bit-address? is fn (address : Nat) -> Boolean
  (address >= 8) and (address <= 119)

### Construct a combined register-address write/read transaction description.
### Callers validate the address and controller-specific limits separately.
pub i2c-register-read is fn ((
  address : Nat,
  register : List Nat,
  read-length : Nat
)) -> (Nat, List Nat, Nat)
  (address, register, read-length)

### Test whether a combined transaction fits an explicit controller limit.
pub i2c-transfer-fits? is fn ((
  register-length : Nat,
  read-length : Nat,
  transfer-limit : Nat
)) -> Boolean
  register-length + read-length <= transfer-limit
