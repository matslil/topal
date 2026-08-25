# Function semantics

## Formal text

### TOPAL-FUNCTION-CAPABILITY-GENERIC-001 — Capability-constrained type variables

In a function header, `value : (T : Capability)` SHALL bind `T` to the exact
classifier of the accepted value and require that classifier to provide the
named capability. Every other parameter or result classified by `T` in the
same header SHALL use that identical classifier. Selection SHALL reject a call
whose later `T` argument has a different classifier even if both classifiers
independently provide the capability. A result classified by `T` SHALL retain
the bound classifier without erasing refinements or substituting a concrete
fallback type.

### TOPAL-FUNCTION-GENERIC-HEADER-001 — Generic header preservation

Source frontends SHALL preserve the complete structure of construction
patterns such as `Optional (Input : Type)` and higher-order classifiers such as
`fn (Input) -> Output`, including every nested type variable and relationship,
for later semantic matching. A frontend SHALL NOT truncate a function
classifier at `fn` or reinterpret its operand types as named declarations.

When a higher-order parameter's result introduces a related type variable
inside `Optional`, `List`, `Range`, `Result`, or a product, invocation through
a named function value SHALL bind that variable from the retained declaration
result. The binding SHALL be available as an ordinary type value inside the
invoked generic body and SHALL be substituted recursively into its result.
Constructing `Empty T` with such a bound type value SHALL produce an empty
`List T`; it SHALL NOT fall back to an unrelated element classifier.

### TOPAL-EXEC-COMPLETED-001 — Completion evidence

The identifier `Completed` in expression position shall evaluate to the
zero-data value classified by `Completed`. It shall be distinct from `Unit`:
returning it establishes a dependency on completion, while `Unit` establishes
no completion dependency. Test traces and debugger history shall expose the
construction of completion evidence before its binding or return.

### TOPAL-EXEC-COMPLETION-EFFECT-VALUE-001 — Completion and effect values

Returning `Completed` may be composed as data with an inert first-class Effect
expression. Constructing or returning the Effect value shall not perform it.

### TOPAL-EXEC-UNIT-EFFECT-VALUE-001 — Unit and effect values

`Unit` remains absence of a completion dependency when packaged beside an inert
Effect value; the static effect expression does not create work or completion.

### TOPAL-EXEC-TRAVERSAL-CONTROL-001 — Traversal elimination

`Continue state` and `Finish result` shall construct distinct traversal-control
values. A short-circuiting fold shall pass the payload of `Continue` to its next
invocation and shall immediately return the payload of `Finish` without
invoking its function for remaining entries. This initial executable fold
subset requires both payloads to satisfy the classifier of its initial state.
Traces and debugger history shall expose construction and early termination.

### TOPAL-FUNCTION-STATIC-NULLARY-001 — Static nullary function execution

A declaration `name is fn static () -> R` followed by one indented expression
shall introduce an immutable zero-parameter function named `name`. Applying
`name ()` shall evaluate its body in the declaration's lexical source context
and return that value exactly when it is classified by `R`. Declaration shall
not evaluate the body. Only bindings already visible at the declaration are
visible in its body; bindings declared later shall not be captured by calling
the function later. Each call shall evaluate the body once and shall expose
selection, entry, body decisions, and return in that order.

This initial executable function subset admits the explicit result classifiers
`Boolean`, `Int`, `Rational`, `String`, and `Unit`. Other headers, parameters,
multi-statement bodies, effects, and recursion remain unsupported until later
rules extend this subset.

### TOPAL-FUNCTION-STATIC-UNARY-001 — Static unary function execution

A declaration `name is fn static ( parameter : P ) -> R` followed by one
indented expression shall introduce an immutable one-parameter function. An
application `name argument` shall evaluate `argument` once in caller scope,
require its value to be classified by `P`, bind it to `parameter` only in a new
function scope containing the declaration's captured bindings, evaluate the
body once, and require the result to be classified by `R`.

Argument evaluation and validation shall precede function selection and entry.
Test traces shall expose parameter binding, selection, entry, body decisions,
and return in that semantic order. The supported parameter and result
classifiers are the same initial value classifiers admitted by
`TOPAL-FUNCTION-STATIC-NULLARY-001`.

### TOPAL-FUNCTION-EFFECT-BOUND-001 — Explicit effect upper bounds

A function header may place `:` and an effect expression after its complete
result type, on the same line or on a continued indented line. The expression
shall be retained as the function's allowed effect row. The implementation's
inferred row shall satisfy it under `TOPAL-EFFECT-CONTAIN-001`; declaration
shall fail rather than erase or widen an unsatisfied bound. Static function
views and tool traces shall retain the declared bound without performing its
effects.

### TOPAL-FUNCTION-PACKAGED-OPERAND-001 — Packaged operand patterns

Either syntactic operand may be a parenthesized field package whose declarations
use `name : Type`. A field may append `default expression`; defaults fill only
omitted labeled associations and SHALL NOT make the complete syntactic operand
optional. Invocation SHALL reject unknown or duplicate labels, require every
nondefaulted field, validate every supplied or defaulted value against its
declared classifier, and bind all field names in the one invocation scope.
Parameter names SHALL be unique across all packaged and unpackaged operands.

### TOPAL-FUNCTION-STATIC-BINARY-001 — Static binary infix function execution

A declaration `name is fn static ( left : L, right : R ) -> O` shall introduce
a two-operand function. An infix application `left-value name right-value`
shall evaluate the left and right operands once in source order, validate them
against `L` and `R`, and bind them to `left` and `right` in declaration order
within the function scope defined by
`TOPAL-FUNCTION-STATIC-UNARY-001`.

Parameter names shall be distinct. Argument validation shall complete before
function selection and entry; a failed shape, arity, or classifier check shall
not enter the function. Test traces shall expose each successful binding in
declaration order before selection, entry, body decisions, and return.

### TOPAL-FUNCTION-BLOCK-001 — Function block execution

An executable function body may contain one or more equally indented statements.
They shall execute from top to bottom in a fresh function scope according to
the block and sequencing rules of the language design. Each non-final statement
shall either introduce a binding, explicitly discard its value, or evaluate to
`Unit`; the final statement's value is the function result and shall satisfy the
declared result classifier.

Bindings introduced by the block shall become visible only to later statements
in the same invocation and shall not escape it. Test traces and debugger history
shall expose their creation and subsequent resolution between function entry
and return.

### TOPAL-FUNCTION-RETURN-001 — Explicit function return

A statement `return expression` shall evaluate `expression` once and complete
the nearest enclosing function immediately with that value. Statements after
the return in the same function invocation shall not be evaluated. The returned
value shall satisfy the function's declared result classifier through the same
validation used for an implicit final-expression result.

`return` outside a function body shall be rejected. Test traces and debugger
history shall expose the explicit-return decision before the common function
return event and shall contain no decisions from skipped statements.

### TOPAL-FUNCTION-ORDINARY-001 — Ordinary runtime function execution

A declaration `name is fn ( parameters ) -> R` shall introduce an ordinary
runtime function without the static-evaluation guarantee. Nullary, unary, and
positional-product parameters, lexical capture, block execution, explicit
return, argument validation, and result validation shall otherwise follow the
corresponding executable function rules above.

Runtime application may select either an ordinary or static function; using a
static function in a runtime context forgets only its static guarantee. Test
traces shall distinguish ordinary declarations, argument bindings, entry, and
return with this rule identity.

### TOPAL-FUNCTION-CALL-CHAIN-001 — Nested function calls

While evaluating a function body, an application may select another visible
function declaration using the same argument and result rules as a root call.
An ordinary function may call an ordinary or static function. A static function
may call only a static function, preserving its static-evaluation guarantee.

For an acyclic call chain, each nested call shall receive a fresh invocation
scope and return its validated result to the caller expression. Test traces and
debugger history shall nest selection, entry, body decisions, and return in
call order without flattening or hiding the callee.

### TOPAL-FUNCTION-LOCAL-SCOPE-001 — Invocation-local shadowing

A function invocation shall create a lexical scope nested inside its captured
declaration scope. A parameter or body declaration may shadow a captured outer
name, while two declarations in the same invocation scope remain an error.
Resolution after the shadowing declaration shall select the local name.

Completing the invocation shall discard its local declarations without changing
the captured or caller bindings. Test traces and debugger history shall expose
local creation and resolution while snapshots after return retain the outer
binding.

### TOPAL-FUNCTION-OVERLOAD-001 — Ordered typed overload selection

Multiple function declarations in one scope may share a name when their input
classifier sequence or staticness differs. Parameter names and result
classifiers shall not distinguish otherwise identical overloads. The overload
set shall preserve source declaration order.

An application shall evaluate its argument once, restrict candidates to the
staticness required by the call context, and select the first candidate in
source order whose complete input header accepts that value. The result context
shall not change selection. Test traces and debugger history shall identify the
selected input signature separately from function entry.

### TOPAL-FUNCTION-RECURSION-INT-001 — Proven decreasing Int recursion

A unary `Int` function whose complete body is a decision table over its
parameter is proven terminating by this initial rule when its first rule is
`<= bound then base`, where `bound` is an `Int` literal, its second rule is
`otherwise recursive-action`, the base
contains no self-call, and every self-call in the recursive action passes
`parameter - step`, where `step` satisfies
`TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001`. Values above the guarded bound
strictly decrease toward it; values at or below the bound take the base.

Only a function satisfying this structural proof shall execute a recursive edge
under this rule. Test traces and debugger history shall expose proof acceptance
at declaration and every recursive descent before nested function entry. Cycles
without an implemented termination proof remain rejected.

### TOPAL-FUNCTION-RECURSION-EUCLIDEAN-001 — Proven Euclidean recursion

A binary `Int` function explicitly measured by the absolute value of its
second parameter is proven terminating when it returns a nonrecursive base for
a zero second parameter and every recursive edge replaces `(left, right)` with
`(right, left % right)`. Euclidean modulo supplies a nonnegative remainder
strictly below `absolute right`, so the measure decreases on every recursive
edge. Traces and debugger history SHALL identify this proof at declaration and
descent.

### TOPAL-FUNCTION-RECURSION-INT-INCREASING-001 — Proven increasing Int recursion

The dual structural rule proves a unary `Int` function terminating when its
first decision rule is `>= bound then base` for an `Int` literal bound, its
second rule is `otherwise recursive-action`, the base contains no self-call,
and every self-call passes `parameter + step`, where `step` satisfies
`TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001`. Values below the bound strictly
increase toward it; values at or above the bound take the base.

Test traces and debugger history shall distinguish this increasing proof from
the decreasing proof at declaration and on every recursive descent. Other
recursive steps remain rejected until another termination rule proves them.

### TOPAL-FUNCTION-RECURSION-NAT-001 — Proven range-preserving decreasing Nat recursion

A unary `Nat` function is proven terminating when its complete body is a
decision table over its parameter with `<= bound then base` followed by
`otherwise recursive-action`, where `bound` is a nonnegative integer literal,
the base contains no self-call, and every self-call passes `parameter - step`
for a positive integer literal satisfying `step <= bound + 1`. This bound makes
the smallest possible recursive argument nonnegative while every step strictly
decreases toward the inclusive base matcher.

Test traces and debugger history shall expose this proof separately from the
more general signed-`Int` recursion rules. Other decreasing `Nat` forms require
a proof that every recursive argument remains nonnegative.

### TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001 — Proven increasing Nat recursion

A unary `Nat` function is proven terminating when its complete body uses an
inclusive `>= bound` base matcher followed by `otherwise recursive-action`, the
base contains no self-call, and every self-call passes `parameter + step` for a
positive integer literal step. Addition preserves nonnegativity; overshooting
the bound is permitted because the inclusive matcher stops the next entry.

### TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001 — Proven mutual decreasing Nat recursion

A closed cycle of unary `Nat` functions is proven terminating when every member
uses the shape of `TOPAL-FUNCTION-RECURSION-NAT-001`, every recursive action
calls the same next cycle member with a bound-preserving decrement, and the final member
calls the first. Each bound shall be a nonnegative integer literal. The complete
cycle shall be established before any recursive edge executes; an isolated or
overshooting candidate remains unproven.

### TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001 — Proven mutual increasing Nat recursion

A closed cycle of unary `Nat` functions is proven terminating when every member
uses an inclusive `>= bound` base matcher and every recursive action calls the
same next member with `parameter + step` for a positive integer literal step.
The final member shall call the first. Every edge preserves nonnegativity and
strictly progresses; the complete cycle shall be proven before execution.

### TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001 — Proven mutual decreasing Int recursion

An initial mutual-recursion rule proves a closed cycle of two or more unary
`Int` functions when every member's complete body is a decision table over its
parameter with `<= bound then base` followed by `otherwise next-call`. Each
`bound` shall be an `Int` literal, each base action shall contain no call to that
member's next cycle function, and each next call shall pass `parameter - step`
with a step satisfying `TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001`. The final member shall call the first, so
every edge in the closed cycle strictly decreases before control returns to the
same function.

A conforming implementation shall not accept an isolated candidate edge as
proof. It shall establish the complete closed cycle and reject execution when
any participating edge does not satisfy the rule. Test traces and debugger
history shall distinguish candidate edges, completed cycle proof, and recursive
descent.

### TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001 — Proven mutual increasing Int recursion

The dual mutual-recursion rule proves a closed cycle of two or more unary `Int`
functions when every member uses `>= bound then base` followed by
`otherwise next-call`, each bound is an `Int` literal, each base contains no
call to the next member, and every next call passes `parameter + step` with a
step satisfying `TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001`.
Every edge must use this increasing rule; mixing increasing and decreasing
candidate edges shall not prove a cycle.

An implementation shall establish the complete closed cycle before recursive
descent and shall distinguish increasing candidates, completed cycle proof,
and descent in test traces and debugger history.

### TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001 — Strict literal progress

For the implemented bounded `Int` recursion proofs, `step` shall be an `Int`
literal whose exact value is greater than zero. A decreasing edge shall
subtract it and an increasing edge shall add it. Zero, a negative literal, a
runtime value, or the opposite operation shall not establish progress under
these rules. Overshooting the bound is permitted because the inclusive base
matcher selects the base action on the next entry.

### TOPAL-FUNCTION-RECURSION-ALL-CALLS-001 — Every recursive edge progresses

When one recursive action contains multiple calls returning to the same active
function, every such call shall independently satisfy the applicable
termination rule. The action is not proven merely because one recursive branch
progresses. Calls may occur within product fields or nested application
operands, and their results may be combined after each call returns.

Test traces and debugger history shall retain every proven recursive descent in
ordinary evaluation order.

For a mutual-cycle member, one action may contain multiple calls to its next
member within product fields or nested application operands. Every discovered
cycle call shall name the same next member and independently satisfy the
cycle's direction and progress rule. A different target or one invalid edge
shall prevent the member from contributing a proven cycle edge.

### TOPAL-FUNCTION-RECURSION-OVERLOAD-IDENTITY-001 — Overload-specific call graph nodes

Each selected function overload shall be a distinct node in recursion and call
graph analysis. A call from one overload to another overload with the same name
but a different complete input signature shall be an ordinary call edge, not a
recursive return to the active overload. Returning to the same selected input
signature shall remain recursive and require applicable termination evidence.

Test traces and debugger history shall expose overload selection before entry so
the two identities and their execution order remain observable.

### TOPAL-FUNCTION-NESTED-001 — Nested lexical function declaration

A function body may declare another function in its invocation scope. The
nested declaration shall capture the bindings visible at that statement,
including outer parameters and earlier body bindings, and may be called by
later statements in the same invocation. Each invocation shall construct its
own nested function and capture.

The nested name shall not escape the enclosing invocation. Test traces and
debugger history shall expose nested declaration after outer entry, followed by
the nested call's argument bindings, entry, body decisions, and return before
the outer function returns.

### TOPAL-FUNCTION-FORWARD-DECLARATION-001 — Complete header visibility

Within one declaration scope, a function body may reference a function declared
later in source order when the later declaration has a complete explicit input
and result classification. The later function shall be available when the
earlier body executes after both declarations have completed.

This visibility shall apply only to function declarations in the same scope. It
shall not make an ordinary initializer value available before that initializer
has executed. Test traces and debugger history shall identify the later
function's selection and entry inside the earlier function's call frame.
### TOPAL-FUNCTION-ANONYMOUS-001 — Contextual inferred anonymous functions

A braced parameter pattern followed by an expression or indented body shall
construct an anonymous function only where the surrounding application selects
one function type. Invocation shall bind its parameter patterns to the supplied
arguments and resolve other names from the lexical environment captured when
the function was constructed. Each invocation establishes its own return scope.

The constructed function is an ordinary value and may be bound before being
passed to another application. Resolving such a binding shall retain the same
captured lexical environment, parameter arity, and invocation semantics as a
function written directly at that contextual call site.

A bound anonymous function may be applied directly. A unary function accepts
its single operand directly. A function with several parameter components
accepts one positional product containing those components and binds them in
source order; an arity mismatch shall be rejected before entering its body.

### TOPAL-FUNCTION-CALLABLE-VALUE-001 — Symbolic callable values

A symbolic callable in value position shall produce a function value retaining
that exact callable identity. Applying a binary callable value shall accept one
two-field positional product and evaluate the corresponding operation. The
symbol `-` additionally accepts one direct numeric operand as negation. Binding
and later applying the value shall not restart name-based overload lookup.

### TOPAL-FUNCTION-VALUE-001 — Named function values

Resolving a declared function without applying it shall produce a function
value retaining its declaration identity and complete ordered overload set.
The value may be bound under another name and applied there; application shall
perform ordinary argument evaluation, overload selection, static-context,
termination, entry, and result checks against the retained declarations. It
shall not resolve unrelated declarations that happen to use the new name.
A function body that refers to an earlier function declaration in its defining
context SHALL retain that named function value without requiring an explicit
alias binding. A private retained function SHALL NOT thereby become a published
member of an enclosing module.
