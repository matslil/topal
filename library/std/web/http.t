use language (
  version is v0.1
)

### Test whether a method used by the example has safe HTTP semantics.
pub safe-method? is fn (method : String) -> Boolean
  (method = "GET") or (method = "HEAD")

### Test whether a method used by the example has idempotent HTTP semantics.
pub idempotent-method? is fn (method : String) -> Boolean
  ((method = "GET") or (method = "HEAD")) or ((method = "PUT") or (method = "DELETE"))

### Select the controller operation for a supported method.
delete-operation is fn (matches : Boolean) -> String
  matches
    true then "delete"
    false then "unsupported"
delete-operation-callable is delete-operation

replace-operation is fn (matches : Boolean, next : String) -> String
  matches
    true then "replace"
    false then next
replace-operation-callable is replace-operation

read-operation is fn (matches : Boolean, next : String) -> String
  matches
    true then "read"
    false then next
read-operation-callable is read-operation

pub operation is fn (verb : String) -> String
  delete-result is delete-operation-callable (verb = "DELETE")
  replace-result is replace-operation-callable (verb = "PUT", delete-result)
  read-operation-callable (verb = "GET", replace-result)

### Test an explicit representation-version precondition.
pub version-matches? is fn (expected : Nat, current : Nat) -> Boolean
  expected = current

### Construct the design-0 response shape `(status, media type, body, version)`.
pub response is fn ((
  status : Nat,
  media-type : String,
  body : String,
  version : Nat
)) -> (Nat, String, String, Nat)
  (status, media-type, body, version)

### Construct a public RFC 9457-style problem response without internal detail.
pub problem is fn ((
  status : Nat,
  problem-type : String,
  title : String
)) -> (Nat, String, String, Nat)
  (status, "application/problem+json", problem-type concat ": " concat title, 0)
