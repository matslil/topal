# Generator semantics

### TOPAL-GENERATOR-ERROR-CODE-001 — Generator error-code vocabulary

The qualified namespace `lang generator` publishes the nominal enum type
`GeneratorErrorCode` with initial alternative `generator-closed`. This code
identity is independent of `Error.domain`. The domain shall be the lexical
namespace where the generator error occurs; generator identity and yield
position shall remain separate trace and source provenance. Only abandonment
of a live linear continuation supplies
`generator-closed`; ordinary source construction of the enum value does not
close or otherwise control a continuation.

For a built-in generator created and abandoned in the root namespace, the
intrinsic close signal therefore has `Error.domain = root`, while generator
provenance identifies `root.characters` separately. A handled close returns
Unit without exposing the intrinsic Error as the generator's final result.

### TOPAL-GENERATOR-DECLARATION-001 — Named generator declarations

A declaration `name is generator ( initial : Input )`, followed by `yields
Yield`, `resumes Resume`, and `-> Return` clauses, shall introduce a callable
resumable function with those four classifiers. Applying it binds the initial
operand and produces a fresh linear `Generator Yield Resume Return` value.

The first executable subset is conforming only in the root namespace, for one
`Character` or `String` input, one or more discarded `_ is yield value` statements, and a
final Unit expression, with `Character`, `Unit`, and `Unit` as the yield,
resume, and return classifiers respectively. Its generator provenance is the
declaration's qualified name. An intrinsic error raised while executing this
root declaration has `Error.domain = root`; that domain is independent of the
generator provenance.

The initial input classifier is independent of the three Generator directions.
A generator may therefore accept `String` while yielding `Character`, resuming
with Unit, and returning Unit.

The yield classifier is likewise independent. A custom `Generator String Unit
Unit` may suspend with complete String values; direct foreach binds each String
to its action parameter and resumes with Unit.

The final return may also be String. Traversing `Generator String Unit String`
shall deliver every yielded String and then produce the separately evaluated
final String.

### TOPAL-GENERATOR-BODY-STATEMENT-001 — Ordinary body statements

A generator continuation may execute ordinary binding and discarded-computation
statements between yields. Statements following a yield execute only after its
next successful resumption and before the next yield or final return.

### TOPAL-GENERATOR-FOREACH-001 — Custom Unit-resumed traversal

Direct foreach over a custom `Generator Character Unit Unit` shall observe its
yields in source order, invoke the Unit-returning action once for each value,
resume with Unit after every action, and produce the generator's final Unit.

### TOPAL-GENERATOR-LOCAL-BINDING-001 — Generator-local state

An ordinary binding in a generator body shall be evaluated in the generator's
local scope. Later yields may refer to that binding, and neither its name nor
its value becomes visible in the caller's scope.

### TOPAL-GENERATOR-EARLY-RETURN-001 — Return before first yield

A generator may reach its declared final return without yielding. Applying it
still produces a fresh linear generator value. Traversal shall invoke no action
and shall produce the generator's final return value.

### TOPAL-GENERATOR-FINAL-RETURN-001 — Distinct final value

The generator's final return classifier is independent of its yield classifier.
For `Generator Character Unit Character`, direct foreach shall invoke its action
for each yielded Character and then produce the distinct final Character.

### TOPAL-GENERATOR-SUSPEND-001 — Yield suspension ordering

Starting a generator shall execute only through its first yield or final return.
After a yield, statements following that yield shall not execute until the
consumer resumes the continuation. Each subsequent resumption repeats this
ordering through the next yield or final return.

### TOPAL-GENERATOR-RESUME-BINDING-001 — Successful resume binding

For a generator declaring `resumes Unit`, a successful foreach resumption shall
make Unit available as the successful value of the suspended yield expression.
A binding introduced by `name is yield value` becomes visible in the generator
scope only after that resumption and may supply the final Unit return.

### TOPAL-GENERATOR-CLOSE-001 — Abandoned custom continuation

Leaving a scope that owns a suspended custom generator shall consume and close
that continuation. The close signal's `Error.domain` shall be the lexical
namespace where closure occurs; the generator's qualified declaration identity
shall remain separate provenance. For the implemented root subset the domain is
`root`, independently of provenance such as `root.pause-once`.

### TOPAL-GENERATOR-CLOSE-HANDLER-001 — Close-result handling

When abandonment closes a yield whose result is bound, that binding shall
receive `Error(domain = lexical-namespace, code = generator-closed)`. Execution
shall continue after the yield so the generator may select an Error decision
rule and finish deliberate shutdown work. A path that observes this close error
shall not yield again.

### TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001 — Qualified close pattern

A Result decision may match the intrinsic close error specifically with
`Error ( code is lang generator generator-closed )`. This matches the nominal
code published by `lang generator`; it does not match by `Error.domain` or by
generator provenance.

### TOPAL-GENERATOR-FUNCTION-RESULT-001 — Returning a continuation

An ordinary function whose result classifier is a Generator may return a live
custom continuation. Function exit shall transfer that continuation without
closing it. The caller receives the same suspended local state and becomes its
single owner.

### TOPAL-GENERATOR-FUNCTION-PARAMETER-001 — Transferring a continuation

Passing a custom Generator as an ordinary function argument shall transfer its
single ownership into the matching parameter. The caller binding is consumed;
the callee receives the same suspended state and may traverse it or allow its
scope to close it.
