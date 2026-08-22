use language (
  version is v0.1
)

queue-present-if is fn (accepted : Boolean, value : (Value : Type)) -> Optional Value
  accepted
    true then Some value
    false then None Value
queue-present-if-callable is queue-present-if

### Enqueue one complete message when the explicit queue bound permits it.
pub enqueue? is fn ((
  messages : List (Message : Type),
  message : Message,
  capacity : Nat
)) -> Optional List Message
  queue-present-if-callable (entry-count messages < capacity, messages append message)

### Dequeue one complete message together with the remaining queue.
pub dequeue is fn (messages : List (Message : Type)) -> Optional (Message, List Message)
  messages
    Empty then None (Message, List Message)
    Entry (message, remaining) then Some (message, remaining)

### Admit automatic retry only with explicit semantic evidence.
pub retry-safe? is fn (
  (idempotent : Boolean, deduplicated : Boolean, transactional : Boolean)
) -> Boolean
  (idempotent or deduplicated) or transactional

### Correlate a stable operation identity with one terminal observation.
pub completion is fn (
  operation : Nat,
  (kind : String, detail : String)
) -> (Nat, String, String)
  (operation, kind, detail)
