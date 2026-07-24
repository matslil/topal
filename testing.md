# Unit testing and structural path coverage

Topal unit tests may describe a function with a compact table of concrete
inputs, mocked dependency results, and expected results. The compiler checks
each row and verifies that the complete table covers every feasible structural
path through the function under test.

Testing facilities are a language feature. A source file activates them at the
point where its test declarations begin:

```topal
use lang feature testing
```

Activation introduces the testing vocabulary into the root scope from that
declaration to the next language selection or the end of the file. Names such
as `path-coverage`, `coverage`, `mock`, and `unused` are not reserved before
activation. A collision with a declaration already visible in the root scope
is reported at the feature declaration rather than silently shadowing either
meaning.

## Compact tables

A coverage table applies the function under test to `path-coverage`. Each
indented row has three parts:

```text
input arguments, ( ordered dependency interactions ) -> expected result
```

An interaction uses ordinary application followed by the existing `->`
separator:

```text
expected dependency call -> supplied return value
```

For example:

```topal
test-db is ( user-a, user-b )

eligible path-coverage
  ( test-db, user-a ), (
    test-db find user-a -> found,
    has-permission user-a -> true
  ) -> Ok true

  ( test-db, user-c ), (
    test-db find user-c -> not-found
  ) -> Ok false

  ( (), user-a ), () -> Error empty-db
```

`eligible` is the function under test. Its input is the first product in each
row. The second product is an ordered sequence of expected dependency calls and
the results supplied for them. The expression after the final `->` is the
expected result from `eligible`.

The empty product `()` says that the row expects no dependency call. A
single-input function may place its input directly before the comma; a
multi-input function uses the same positional or labeled product required by
an ordinary call.

The notation adds no general-purpose symbols. Parentheses and commas retain
their ordinary product meaning, while `->` already separates an input from an
output in algorithm syntax. Only `path-coverage` is introduced for this
facility, and it is introduced by the `testing` language feature.

The formatter keeps a short row on one line and expands only its interaction
product when necessary:

```topal
double path-coverage
  0, () -> 0
  1, () -> 2
  -1, () -> -2
  100, () -> 200
```

## Row meaning

Every row is one concrete unit-test scenario. The compiler:

1. applies the listed input values to the function under test;
2. replaces each declared dependency call with its supplied result;
3. checks that calls occur with the listed arguments and in listed order;
4. checks each supplied result against the dependency's declared output type,
   constraints, and protocol state;
5. executes the function and compares its result with the final cell; and
6. records the structural path exercised by the execution.

A row does not stand for a range of input values. Several rows may deliberately
exercise the same structural path. Boundary values, regressions, and important
domain examples remain useful even when they add no formal coverage. The
compiler does not warn merely because removing a row would preserve coverage.

The expected result is an ordinary expression evaluated in the testing
declaration's scope. It
may refer to bindings already available there, but it may not call the function
under test. This prevents a test from proving itself by using its implementation
as the expected result.

## Dependency interactions

Interactions are matched from left to right. A later call may refer directly to
the concrete value carried by an earlier supplied result:

```topal
load-current path-coverage
  absent-id, (
    accounts find absent-id -> Ok None
  ) -> Ok default-configuration

  present-id, (
    accounts find present-id -> Ok (Some account-a),
    configuration read (account-a settings-path) -> Ok configuration-a
  ) -> Ok configuration-a

  present-id, (
    accounts find present-id -> Ok (Some account-a),
    configuration read (account-a settings-path) -> Error unavailable
  ) -> Error unavailable
```

Here `account-a` is an ordinary value already available in the declaration
scope. The
first interaction supplies `Ok (Some account-a)`, and the following expected
call uses that same value. The interaction syntax does not give `is` any new
meaning or implicitly bind a name from a mocked result.

Two calls to the same dependency occur twice in the interaction product:

```topal
compare-current path-coverage
  sample-a, (
    samples read sample-a -> Ok 10,
    samples read sample-a -> Ok 12
  ) -> Ok Increased

  sample-b, (
    samples read sample-b -> Error unavailable
  ) -> Error unavailable
```

The interactions describe ordered call occurrences, not a map from algorithm
names to results. Except for collapsed loop repetitions described below, a row
fails if calls occur in another order, with other arguments, too often, or too
few times. A dependency which is not reached is simply absent from the
interaction product; no special sentinel is needed.

Calls made by repeated traversal of the same collapsed loop-body path do not
need repeated interaction entries. The first listed interaction supplies the
representative result, which the test mock reuses for later structurally
equivalent occurrences. Each reused result must still satisfy the dependency's
contract for the actual call arguments.

An author may list later occurrences explicitly when their distinct results are
important to the expected result or preserve a regression scenario. Explicit
entries take precedence over reuse. Calls from different branches within the
loop are never coalesced, even when they call the same dependency:

```topal
inspect-all path-coverage
  ( item-a, item-b, item-c ), (
    classify item-a -> ordinary
  ) -> Ok Completed
```

If all three iterations take the same body path, only the first `classify`
interaction is required for coverage and `ordinary` is reused. If `item-b`
takes another branch, that branch's `classify item-b` interaction must appear
in the row which covers it. Calls outside a collapsed repetition and separate
call sites always remain distinct ordered interactions.

## Structural paths

Coverage begins with full feasible path coverage over the function's control
flow. Loops and structural recursion are then collapsed so that repetition
count does not create infinitely many paths.

For every loop, the compiler distinguishes:

- the loop is not entered; and
- the loop is entered one or more times.

It additionally distinguishes every feasible path through one abstract
iteration and every distinct exit:

- normal loop completion;
- early return;
- error propagation;
- cancellation or another declared control exit.

It does not distinguish one iteration from two or a thousand iterations merely
because their counts differ. For structural recursion, the corresponding
distinction is the base case versus one or more recursive steps.

For example:

```topal
contains is fn ( values : List Integer, wanted : Integer ) -> Boolean
  values each value
    value = wanted then return true

  false
```

has three collapsed structural paths:

1. the loop is not entered;
2. the loop is entered and returns early after a match; and
3. the loop is entered and completes without a match.

A complete table is:

```topal
contains path-coverage
  ( (), 3 ), () -> false
  ( ( 1, 3, 5 ), 3 ), () -> true
  ( ( 1, 5, 7 ), 3 ), () -> false
```

The compiler does not require every list length or every possible match
position. Authors may add such cases when they are useful.

Branches inside a loop remain part of structural coverage. If an iteration can
take negative, zero, and positive alternatives, each feasible alternative must
be exercised. One concrete row may satisfy several obligations over several
iterations, provided its complete execution remains valid.

## Completeness

The compiler constructs an acyclic abstraction of the function's control-flow
graph by replacing each cyclic region with its skipped and entered forms. It
then derives every feasible structural path through that abstraction. A table
is complete exactly when every derived path is exercised by at least one valid
row.

This automatically covers every result region introduced by a control-flow
decision. Variation within branchless computation does not create more paths:

```topal
double is fn ( value : Integer ) -> Integer
  value * 2
```

`double` has one structural path. One correct row is sufficient for certified
coverage, although a useful table will often contain several values:

```topal
double path-coverage
  0, () -> 0
  1, () -> 2
  -1, () -> -2
  100, () -> 200
```

Coverage certification says that all structural paths are represented. It does
not prove that an algorithm is extensionally correct for every possible input.
Each row remains a concrete test, and additional examples guard calculations,
boundaries, and regressions which control-flow coverage alone cannot establish.

## Feasibility and unreachable paths

Only feasible paths require rows. The compiler uses input types, constraints,
dependency contracts, and accumulated branch conditions to prove a path
unreachable. A branch forbidden by a dependency's public result constraint
therefore does not create a test obligation.

Mocked results are otherwise allowed to be any values admitted by the
dependency contract. The current implementation of a mocked dependency may
happen to produce a narrower set, but isolated unit tests cannot rely on an
undeclared property of that implementation.

The coverage checker reports one of three states for a candidate path:

- **covered** — a valid row exercises it;
- **unreachable** — the compiler has proof that no permitted scenario reaches
  it; or
- **unresolved** — the compiler can neither find a covering row nor prove it
  unreachable.

Both an uncovered feasible path and an unresolved path prevent certification.
The compiler never treats failure to find a witness as proof of
unreachability.

When possible, a missing-path diagnostic includes a concrete or symbolic
counterexample:

```text
The table for `eligible` is incomplete.

Missing structural path:
  `users find id` returns `Error problem`
  `eligible` propagates `Error problem`

Example obligation:
  id: UserId
  users find id: Error unavailable
```

## Coverage reports

`coverage` exposes the compiler's static report to test tooling:

```topal
report is coverage eligible
```

Conceptually, the report contains the function identity, covered paths,
unreachable paths with proofs, unresolved obligations, and the rows which
cover each path. These are static diagnostic objects and do not add reflection
metadata to the executable.

The normal compilation diagnostic is condensed:

```text
`eligible`: complete structural coverage
  table rows: 6
  covered paths: 4
  unreachable paths: 1
```

Detailed reporting may show that several rows exercise the same path, but does
not characterize those rows as redundant.

## Scope and initial restrictions

Applying `path-coverage` may inspect the implementation of a function only
where ordinary visibility permits that implementation to be tested. It does
not reveal private control flow through a published interface to an unrelated
package.

The initial coverage checker supports functions for which:

- calls are statically resolved;
- all effects crossing the unit boundary are represented by declared
  dependencies;
- dependency interactions form a finite sequence after equivalent loop
  repetitions are coalesced;
- loop and recursion regions have compiler-recognized structure; and
- feasibility conditions belong to constraint theories supported by the
  compiler.

If those requirements are not met, ordinary example rows may still run as
tests, but the compiler cannot certify exhaustive structural coverage. The
diagnostic identifies the unsupported control-flow or constraint operation
rather than silently weakening the requested guarantee.
