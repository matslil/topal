# Remaining fundamental design decisions

The semantic foundations define generic evidence, effects, execution, structured
tasks, scoped resources, typed error vocabularies, and foreign boundaries.
Several choices deliberately remain open because they affect grammar,
ergonomics, or the trusted implementation boundary rather than following
uniquely from Topal's existing principles.

## Recently settled foundations

The following questions from the initial audit are no longer open:

- [Constraints and capabilities](abstractions.md) are distinct kinds.
  Constraints retain a base type and limit its values through a predicate;
  capabilities make static interface and law promises.
- Constraint construction is object-first, as in
  `Integer constraint { value } ...`. The inferred anonymous function is the
  predicate.
- Chained classification proceeds from left to right. A function header such
  as `values : ( C : Sortable )` binds `C` with the subject kind supplied by
  `Sortable` and classifies `values` by the complete `C`. The longer
  `C : Type : Sortable` is valid but redundant.
- Function headers perform static type matching. Construction syntax can bind
  explicitly classified components such as `X : Type` and `Y : Type` from a
  tuple, and the function body is the implicit successful branch of the header
  match. Bare unbound names are not introduced.
- `_ : Kind` discards one construction parameter without removing it from the
  complete matched object. Matching and introspection see only declarations,
  structural views, fields, and capability evidence visible in their lexical
  scope.
- Open record patterns accept anonymous structural records with at least their
  stated visible fields. Nominal records participate only through an explicitly
  published structural view; `...` neither captures a row nor grants
  reconstruction authority.
- Evidence forgetting can satisfy a concrete classifier without changing a
  complete capture. A concrete header may use one canonical lossless conversion
  and exposes its destination type to the body; conversion never unifies
  repeated pattern bindings.
- Multi-component capabilities group their component objects into one component
  product. A matcher such as
  `C : Keyed (Key : Type, Value : Type)` obtains their identities from the
  canonical evidence for that exact `C`. The component classifiers remain
  explicit because bare unbound names are not introduced.
- Canonical capability evidence records exact ordinary operation identities.
  `( container : Indexed ) get index` invokes the certified `get` role instead
  of restarting unrestricted overload resolution. `Indexed get` statically
  names that role for the surrounding classified subject, allowing combinations
  such as `Indexed and ( Indexed get : OExec ( 1 ) )` without turning
  capabilities into namespaces or implementation containers.
- Returning a captured complete type such as `C` preserves the precise
  relationship between the input and result, including nominal identity,
  constraints, static sizes, and other parameters.
- Overload resolution is ordered like pattern matching. Declarations under one
  name in one namespace are tested in source order and the first applicable
  input header wins. Capability strength, conversion quality, and the expected
  result type do not reorder them. Optional diagnostics may identify surprising
  overlap or shadowing. An explicit call-site resource `Prefer` construction is
  the sole current ranking layer before source order; it never changes semantic
  applicability or invents missing evidence.
- Imported scopes remain qualified and do not merge overload sets. A qualified
  call selects its namespace before searching that namespace's declarations;
  scope aliases preserve the same order. Explicit cross-namespace overload-set
  composition has no current surface syntax.
- Capability evidence is coherent and claims are owner-scoped. Each canonical
  object-capability pair has at most one interpretation, claimed in the
  definition context of either the capability or satisfying object. Unrelated
  packages integrate them through an owned specialization rather than an orphan
  claim.
- An explicit owner claim suppresses derivation. Without one, exactly
  one derivation path may apply; competing derivations are an error which the
  capability or object owner resolves with an explicit claim. Import and
  discovery order never choose capability evidence.
- Universal case behavior and natural language are independent capabilities.
  `String` provides `CaseInsensitive` using Topal's fixed Unicode version;
  `Language T` separately supplies a static language identity. Language-specific
  case operations require both without changing canonical string equality.
- Programmers may declare law evidence. Soundly proved or exhaustively checked
  claims are verified; unresolved claims ordinarily become
  `trusted-unverified` evidence and emit `unverified-law` by default; refuted
  claims are errors. Sampling can refute but cannot verify a universal law.
  Evidence required for compiler safety or totality, including `Decreases`,
  must instead be verified.
- `lang disable-warning W` suppresses `W` for the next complete statement.
  `lang push-disable-warning W` and the matching
  `lang pop-disable-warning W` delimit a lexical suppression region. Suppression
  changes diagnostics but never the evidence's recorded trust status.
- Effect dependencies use relational capabilities. `DependsOn` is binary;
  `Independent`, `Conflicts`, `Aliases`, and `MayAlias` each classify one
  identity list. `Aliases` proves one underlying resource, while `MayAlias`
  preserves uncertainty and is the safe default without independence evidence.
- Finite useful dynamic alternatives retain a sum of exact implementation
  evidence. Erasure keeps common capability guarantees and the union of possible
  effects, conservatively ordering unknown relationships.
- Foreign-language integration is postponed. Its retained boundary principles
  require sandboxed, validated, explicitly capability-restricted adapters, but
  the current language defines no foreign declaration syntax or ABI catalog.
- The [initial capability vocabulary](capabilities.md) now defines comparison,
  collection observation and construction, keyed association, combination, and
  function-law capabilities.
- Possible compiler-generated tests, symbolic proof tables, capability-law
  verification, and task/protocol proofs are recorded as
  [future work](FUTURE.md), not current language guarantees.
- `Result` has two explicit components:
  `Result ( Value, ErrorVocabularies )`. The second is one `ErrorCode` type, a
  product of them, or `()`. Products are flattened, deduplicated by nominal
  identity, and order-independent. No specially named error type is resolved
  from the surrounding scope. Visible generic bodies retain the complete
  vocabulary component symbolically in typed intermediate code.
- Composing several `Result` values always merges their error vocabularies. An
  anonymous composition permits at most one value-producing success; `Unit`
  and completion-only successes contribute no payload. When every
  value-producing input is bound, the binding names form the fields of an
  anonymous success record containing every value. Partially bound and
  multiply value-producing anonymous compositions are invalid.
- `Error.domain` identifies the stable reporting operation, subsystem, or
  abstraction, while the independently scoped `ErrorCode` value says what
  occurred. Several domains may use one code vocabulary. Code patterns qualify
  values through their defining scope; identifiers are not global.
- Capabilities are promises, never implementation containers. The bootstrap
  parser knows none of them; the selected Topal version supplies the atomic
  vocabulary, classified object kinds, and verification rules. Source code
  constructs new capability expressions only by combining existing capabilities
  with `and`, `or`, or static functions returning such combinations. Operations
  remain ordinary functions, and concrete overloads precede generic
  capability-constrained fallbacks when specialized behavior is desired.
- Effect expressions reuse capability-style classification and composition
  syntax while retaining effect semantics. A post-return classifier is an upper
  bound; function inputs may be classified for ordered specialization; `and`
  permits both effect sets; `or` retains distinguished implementation
  alternatives; and `Effects ()` is empty. Effect classifiers imply
  `Function`. Every resource parameter resolves to an existing visible identity
  rather than ambient authority.
- Declaration scopes behave as two semantic stages: complete declaration
  headers are collected before their definitions are checked. The compiler
  infers mutually recursive groups from the resulting call graph; initializer
  values still obey their construction dependencies. An optional function
  capability `Decreases ( Measures )` supplies pure well-founded measure
  expressions when termination inference needs guidance or an opaque,
  interface, or higher-order contract must publish the relationship.
- [Resource complexity guarantees](performance.md) are distinct from semantic
  capabilities but compose with them as classifiers. `OExec ( E )` and
  `OAlloc ( E )` express argument-dependent asymptotic upper bounds on abstract
  work and total dynamic allocation. `NoAlloc` separately promises exactly no
  allocation for any input. Direct call-site classification is a hard
  requirement. `Prefer ( Guarantees... )` supplies soft lexicographic selection
  goals before source order, while missing performance evidence still permits a
  semantically applicable fallback.
- Existential packages open through ordinary decision-table patterns. Hidden
  static components and their evidence are scoped to the selected action;
  discards retain unnamed evidence for dependent matching. An action result
  which still depends on a fresh identity is automatically existentially
  closed, while ordinary classification may forget the package to a weaker
  visible object.
- `with` is the immutable record-reconstruction keyword. It retains unspecified
  fields, revalidates invariants, and never mutates aliases of the original
  value. Persistent task-field replacement always uses a qualified left side,
  as in `@ count is @ count + amount`; an unqualified `is` always introduces a
  lexical binding.
- `match-first` is an ordered interaction decision table which initiates
  speculation-safe requests together and selects the first committed response;
  source order breaks logical ties. `match-all` waits for a labeled product of all
  responses and admits effectful requests under ordinary dependency analysis.
  Neither exposes pending-request or cancellation objects.
- `with-timeout` composes with a request, stream, `match-first`, or `match-all`.
  `match-first` permits only one surrounding group timeout; `match-all` may also time
  individual fields. Timeout is an observation failure, not proof that request
  effects did not occur, so outstanding effects remain in the dependency graph.
  Inline operands use parentheses and multiline operands use indentation, never
  both.
- Non-owning task monitoring uses `Weak TaskType`; there is no separate monitor
  operation. Weak promotion and task messaging observe different facts, so a
  successful promotion may still be followed by `task-terminated`. Endpoints
  remain restricted messaging authorities, while the final task result belongs
  exclusively to the owning instance's implicit join obligation.

## Surface grammar

The grammar must select compatible spellings for:

- type-construction patterns beyond homogeneous `Container Value`, including
  constructions such as `Map ( Key, Value )`;
- otherwise uninferable public error-vocabulary parameters and bounds.

These should be designed together so that indentation, recursive
classification, prefix application, and the zero-to-two operand rule remain
unambiguous. Function headers already bind generic type components through
static matching rather than through a separate generic-parameter list.

## Capability vocabulary and matching

Capability claim ownership and coherence are settled. The
[initial capability vocabulary](capabilities.md) selects the fundamental
comparison, collection, and function-law names. Libraries express formatting,
parsing, checked construction, and similar behavior with ordinary functions,
interfaces, and combinations of capabilities already supplied by the selected
language version. A genuinely new atomic promise requires a language-version
extension.

Multi-component capability matching and capability-combination syntax are
settled. New names bind combinations of existing promises; they do not declare
atomic capabilities or implementation bodies.

## Automated law proof

Programmer-authored law evidence and its trusted-unverified fallback are
settled. The [future verification design](FUTURE.md) still describes symbolic
proof tables, finite exhaustive verification, induction, and independently
checked proof certificates which can upgrade more claims to verified evidence.

## Effect annotations and handlers

Effects are inferred for ordinary implementations. An `Interface` contains
function and generator interaction shapes rather than the inferred effects of
one implementation. Applying it to a concrete context, packaged value, task, or
endpoint constructs implementation evidence whose compiled contract includes
the inferred effects. Callers retain that evidence whenever the selected
implementation is known.

Finite dynamic selection and conservative erasure are settled. Compiled Topal
artifacts normally retain typed generic intermediate code with symbolic types,
callback identities, effects, resource relationships, capabilities, layouts,
and implementation evidence. Final application compilation substitutes
concrete evidence and specializes useful alternatives; erased uses receive a
conservative shared implementation. The intermediate format is
compiler-versioned rather than a permanent machine ABI.

Consequently, visible generic bodies do not require source-level effect-row
parameters merely to defer compilation. Explicit effect upper bounds follow the
completed function type and otherwise-uninferable restrictions classify a
complete function input directly. Both use the same syntax as capability
classification; the classifier kind supplies their different meaning.

Task message result adaptation is settled. `Unit` handlers are events with no
reply. Every ordinary function handler that replies must return
`Result ( Completed, ApplicationErrors )` or
`Result ( Value, ApplicationErrors )`; plain reply types and function
`Result ( Unit, ApplicationErrors )` are invalid. A task generator's final
return must also be `Result`. An explicit interface must therefore declare
`Result` to permit a request or stream implementation through task messaging.
Task interaction failures extend the existing result's effective error-code
vocabulary through implementation evidence rather than adding a second wrapper
or changing direct implementations.

The only portable task-interaction code is `task-terminated`, in Topal's stable
task error domain. An endpoint always denotes an application-local task
instance which existed, so task messaging adds no separate unavailable,
admission, or transport errors. Remote services are represented by local
mirror tasks, which either report their interface's application errors or
terminate. A reply or final stream result accepted by the task runtime wins
over concurrent termination; if termination commits first, the interaction
returns `task-terminated`.

The initial design handles effects through application composition, tasks,
protocols, and constructed contexts. A future general handler construct should
be added only if concrete use cases cannot be expressed cleanly through those
boundaries. If added, its treatment of continuation linearity, task state, and
effect resource identities needs a separate design.

## Termination and external time

Topal exposes no general cancellation operation. Hard termination is an
ownership-authorized queued lifecycle transition. Once its handler returns,
suspended handlers execute no more programmer code and their continuations and
task state receive automatic cleanup. Every suspension point must consequently
leave state cleanly destructible.

`terminate-cleanly` is the cooperative alternative available as the terminal
expression of a message handler returning `Unit` or
`Result ( Completed, TerminationErrorCode )`. It rejects queued and new
requests with `task-terminated`, discards their `Unit` events, allows
already-suspended handlers to finish, and then performs the ordinary lifecycle
handler and cleanup. The `Result ( Completed, TerminationErrorCode )` form
retains the initiating session and replies after termination; the `Unit` form
does not wait.

Generator abandonment is settled independently of a general cancellation
surface. `yield` has effective type
`Result ( ResumeValue, GeneratorErrorCode )`; abandoning its linear
continuation supplies the language-defined `generator-closed` code. The
generator may perform explicit shutdown work but cannot yield again on that
path. Returning or propagating the close signal ends the generator, after
which automatic cleanup runs. The owning scope waits for shutdown and cleanup
and retains their failures. Consumers have no explicit generator-cancellation
operation.

Short standard-library delays may pause the current task for bounded hardware
waiting. Ordinary interaction timeouts use
`5[s] with-timeout ( message-interface call )` inline or an unparenthesized
indented right operand. The relative monotonic quantity is converted to a
hidden absolute deadline and registered with a compiler-created
application-local timeout server. A hidden timeout ID prevents sequential and
cancellation races.

`with-timeout` accepts a reply-bearing request, stream message call,
`match-first`, or `match-all`. A request or structured group has one timed wait. A
stream starts a fresh interval while awaiting its first yield or final return
and after each consumer resume while awaiting the next yield or final return.
No timer runs while the consumer handles a yielded value. Individual
`match-first` alternatives cannot be timed; `match-all` may time individual fields.

The construction merges `TimeoutErrorCode` into an existing result or wraps a
non-`Result` wait value in `Result`. It never cancels underlying requests. A
timeout discards their later response values but preserves their outstanding
effects and therefore cannot be interpreted as proof that an operation failed
or did not commit. A stream timeout abandons its continuation through the
existing `generator-closed` path; values already yielded remain observed. The
direct `timeout-error timeout-occurred` failure uses the `lang with-timeout`
domain, so the same code returned under the handler's domain remains
distinguishable.

The `testing` feature supplies a qualified `testing advance-time` function and
a virtual monotonic clock. Advancement processes intermediate deadlines and
their dependency-ready work deterministically, making timeout and short-delay
tests independent of real elapsed time. Civil time remains separate.

## Resource moves and non-owning capabilities

Resource acquisition uses ordinary success-continuation binding, as in
`file-system open-file path { file }`, and Topal defines no generic
`with-resource` operation. Acquisition and cleanup contribute only completion
and their error vocabularies to the enclosing anonymous `Result` composition;
the continuation contributes its success value. Returning a resource from the
continuation, directly or inside another value, is an ordinary explicit escape.
Existing ownership analysis moves it into the receiving scope when the old
binding is no longer used, or uses safe sharing when other references remain.
Destructor responsibility and possible failure follow the final reference. No
transfer keyword, linear capability, scoped-owner wrapper, or public ownership
type is added. Policy-specific library helpers remain ordinary functions.

Non-owning resource back references and task-instance monitors use the
language-defined `Weak` capability-backed construction. A `Weak T` does not
keep `T` alive. Access
atomically returns `Result ( T, WeakErrorCode )` with `weak-unavailable` when
the target cannot be retained. Applying a weak value to a block retains one
ordinary `T` for the complete block; that value may escape through the ordinary
move rules. Weak references expose no counts or collection timing. Weak task
promotion does not acquire the final join result, and messaging may still
report `task-terminated`. Task endpoints remain distinct restricted messaging
authorities rather than weak values.

## Deferred foreign integration

Foreign-language integration is future work rather than a remaining initial
language decision. The portable safety principles remain documented so a later
design does not introduce ambient authority, unchecked values, or raw Topal
references. ABI families, primitive ABI mappings, callbacks, error conventions,
linkage metadata, and all related surface syntax are deliberately unspecified.

## Resource complexity

The initial resource complexity model is settled in
[resource complexity guarantees](performance.md). Core collection names still
do not imply a representation or complexity class. Semantic capabilities such
as `Indexed` may be combined with applicable performance evidence to name
stronger classifiers such as `RandomAccess`, while callers may forget that
evidence and retain the semantic capability.

Elapsed-time guarantees, peak live-memory bounds, and platform-specific
resource dimensions remain future refinements.
