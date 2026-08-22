#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# A transport-independent REST API whose controller functions are embedded
# directly in an Interface implementation. An HTTP adapter validates message
# syntax and limits, selects the operation, and serializes these semantic
# responses; the controller itself needs neither a socket nor ambient authority.

response is std web http response
problem is std web http problem
operation is std web http operation
safe-method? is std web http safe-method?
idempotent-method? is std web http idempotent-method?
version-matches? is std web http version-matches?

read-result is fn (missing : Boolean) -> (Nat, String, String, Nat)
  missing
    true then problem (404, "urn:topal:todo:not-found", "Todo not found")
    false then response (200, "application/json", "todo", 7)
read-result-callable is read-result

stale-result is fn (
  stale-request : Boolean,
  current-result : (Nat, String, String, Nat)
) -> (Nat, String, String, Nat)
  stale-request
    true then problem (412, "urn:topal:todo:stale", "Version precondition failed")
    false then current-result
stale-result-callable is stale-result

invalid-result is fn (
  invalid-request : Boolean,
  valid-result : (Nat, String, String, Nat)
) -> (Nat, String, String, Nat)
  invalid-request
    true then problem (422, "urn:topal:todo:invalid", "Todo is invalid")
    false then valid-result
invalid-result-callable is invalid-result

TodoController is Interface
  read is fn (request : String) -> (Nat, String, String, Nat)
  replace is fn (request : String) -> (Nat, String, String, Nat)
  delete is fn (request : String) -> (Nat, String, String, Nat)

TodoController
  read is fn (request : String) -> (Nat, String, String, Nat)
    read-result-callable (request = "missing")
  replace is fn (request : String) -> (Nat, String, String, Nat)
    success is response (200, "application/json", request, 8)
    current-result is stale-result-callable (request = "stale", success)
    invalid-result-callable (request = "invalid", current-result)
  delete is fn (request : String) -> (Nat, String, String, Nat)
    stale-result-callable (request = "stale", response (204, "application/json", "", 8))

read-callable is read
replace-callable is replace
delete-callable is delete

# The adapter's routing result is data. It invokes only the selected embedded
# function and can reject unsupported methods before reading a request body.
get-operation is operation "GET"
put-operation is operation "PUT"
delete-operation is operation "DELETE"
post-operation is operation "POST"

found is read-callable "42"
missing is read-callable "missing"
replaced is replace-callable "updated todo"
invalid is replace-callable "invalid"
stale is replace-callable "stale"
deleted is delete-callable "42"
stale-delete is delete-callable "stale"
unsupported is problem (405, "urn:topal:http:method", "Method not allowed")

Pass is Boolean constraint { value } value = true
get-routes-to-read : Pass is Pass (get-operation = "read")
put-routes-to-replace : Pass is Pass (put-operation = "replace")
delete-routes-to-delete : Pass is Pass (delete-operation = "delete")
post-is-rejected : Pass is Pass ((post-operation = "unsupported") and (unsupported = (405, "application/problem+json", "urn:topal:http:method: Method not allowed", 0)))
get-is-safe-and-idempotent : Pass is Pass ((safe-method? "GET") and (idempotent-method? "GET"))
put-is-not-safe-but-idempotent : Pass is Pass ((not (safe-method? "PUT")) and (idempotent-method? "PUT"))
read-controller-returns-version : Pass is Pass (found = (200, "application/json", "todo", 7))
missing-resource-is-public-problem : Pass is Pass (missing = (404, "application/problem+json", "urn:topal:todo:not-found: Todo not found", 0))
valid-replacement-advances-version : Pass is Pass (replaced = (200, "application/json", "updated todo", 8))
invalid-body-is-unprocessable : Pass is Pass (invalid = (422, "application/problem+json", "urn:topal:todo:invalid: Todo is invalid", 0))
stale-replacement-is-precondition-failure : Pass is Pass (stale = (412, "application/problem+json", "urn:topal:todo:stale: Version precondition failed", 0))
delete-is-idempotent-controller-operation : Pass is Pass (deleted = (204, "application/json", "", 8))
stale-delete-is-precondition-failure : Pass is Pass (stale-delete = (412, "application/problem+json", "urn:topal:todo:stale: Version precondition failed", 0))
version-precondition-is-exact : Pass is Pass ((version-matches? (7, 7)) and (not (version-matches? (6, 7))))

(get-routes-to-read, put-routes-to-replace, delete-routes-to-delete,
 post-is-rejected, get-is-safe-and-idempotent,
 put-is-not-safe-but-idempotent, read-controller-returns-version,
 missing-resource-is-public-problem, valid-replacement-advances-version,
 invalid-body-is-unprocessable, stale-replacement-is-precondition-failure,
 delete-is-idempotent-controller-operation,
 stale-delete-is-precondition-failure, version-precondition-is-exact)
