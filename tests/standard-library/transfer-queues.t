use language (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
enqueue? is std transfer queues enqueue?
dequeue is std transfer queues dequeue
retry-safe? is std transfer queues retry-safe?
completion is std transfer queues completion
absent? is std absent?
value-or is std value-or

empty : List Int is Empty
one : List Int is Entry (7, Empty)
two : List Int is Entry (7, Entry (9, Empty))
remaining : List Int is Entry (9, Empty)
first-two : (Int, List Int) is (7, remaining)

zero-capacity-rejects : Pass is Pass (absent? (enqueue? (empty, 7, 0)))
room-appends-at-tail : Pass is Pass ((value-or ((enqueue? (one, 9, 2)), empty)) = two)
full-queue-rejects : Pass is Pass (absent? (enqueue? (one, 9, 1)))
empty-dequeue-is-absent : Pass is Pass (absent? (dequeue empty))
dequeue-preserves-fifo : Pass is Pass ((value-or ((dequeue two), (0, empty))) = first-two)
no-retry-evidence-rejects : Pass is Pass (not (retry-safe? (false, false, false)))
idempotence-admits-retry : Pass is Pass (retry-safe? (true, false, false))
deduplication-admits-retry : Pass is Pass (retry-safe? (false, true, false))
transaction-admits-retry : Pass is Pass (retry-safe? (false, false, true))
evidence-composition-admits-retry : Pass is Pass (retry-safe? (true, true, true))
completion-keeps-operation : Pass is Pass ((completion (42, ("ok", "written"))) = (42, "ok", "written"))

(zero-capacity-rejects, room-appends-at-tail, full-queue-rejects,
 empty-dequeue-is-absent, dequeue-preserves-fifo, no-retry-evidence-rejects,
 idempotence-admits-retry, deduplication-admits-retry,
 transaction-admits-retry, evidence-composition-admits-retry,
 completion-keeps-operation)
