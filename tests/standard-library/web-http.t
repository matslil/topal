use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
safe-method? is std web http safe-method?
idempotent-method? is std web http idempotent-method?

all-safe-methods : Pass is Pass (
  (((safe-method? "GET") and (safe-method? "HEAD")) and (safe-method? "OPTIONS")) and
  (safe-method? "TRACE")
)
unsafe-methods : Pass is Pass (
  (not (safe-method? "POST")) and (not (safe-method? "PUT"))
)
all-idempotent-methods : Pass is Pass (
  (((((idempotent-method? "GET") and (idempotent-method? "HEAD")) and
  (idempotent-method? "OPTIONS")) and (idempotent-method? "TRACE")) and
  (idempotent-method? "PUT")) and (idempotent-method? "DELETE")
)
nonidempotent-methods : Pass is Pass (
  (not (idempotent-method? "POST")) and (not (idempotent-method? "PATCH"))
)
method-names-are-case-sensitive : Pass is Pass (
  (not (safe-method? "get")) and (not (idempotent-method? "delete"))
)

(all-safe-methods, unsafe-methods, all-idempotent-methods,
 nonidempotent-methods, method-names-are-case-sensitive)
