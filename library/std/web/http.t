use language (
  version is v0.1
)

### Test whether an HTTP method has safe semantics under RFC 9110.
pub safe-method? is fn (method : String) -> Boolean
  (((method = "GET") or (method = "HEAD")) or (method = "OPTIONS")) or (method = "TRACE")

### Test whether an HTTP method has idempotent semantics under RFC 9110.
pub idempotent-method? is fn (method : String) -> Boolean
  (safe-method? method) or ((method = "PUT") or (method = "DELETE"))
