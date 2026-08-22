use language (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
address? is std device i2c i2c-seven-bit-address?
register-read is std device i2c i2c-register-read
transfer-fits? is std device i2c i2c-transfer-fits?

empty-register : List Nat is Empty
register : List Nat is Entry (16, Entry (32, Empty))
expected-read : (Nat, List Nat, Nat) is (72, register, 4)

reserved-low-address-rejects : Pass is Pass (not (address? 7))
first-ordinary-address : Pass is Pass (address? 8)
last-ordinary-address : Pass is Pass (address? 119)
reserved-high-address-rejects : Pass is Pass (not (address? 120))
seven-bit-overflow-rejects : Pass is Pass (not (address? 128))
empty-transfer-fits-zero : Pass is Pass (transfer-fits? (0, 0, 0))
exact-controller-limit : Pass is Pass (transfer-fits? (2, 4, 6))
past-controller-limit : Pass is Pass (not (transfer-fits? (2, 5, 6)))
descriptor-preserves_transaction : Pass is Pass ((register-read (72, register, 4)) = expected-read)
empty-register-is-preserved : Pass is Pass ((register-read (72, empty-register, 0)) = (72, empty-register, 0))

(reserved-low-address-rejects, first-ordinary-address,
 last-ordinary-address, reserved-high-address-rejects,
 seven-bit-overflow-rejects, empty-transfer-fits-zero,
 exact-controller-limit, past-controller-limit,
 descriptor-preserves_transaction, empty-register-is-preserved)
