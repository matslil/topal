# Contracts and evidence

Topal revision `v0.2` adds relational contracts and a common evidence model.
The model separates facts needed for semantic correctness from facts about one
selected implementation. Revision `v0.1` retains its original function-header
syntax and meaning.

## Function clauses

A `v0.2` function header has this fixed order:

```text
fn ( parameters )
  requires input-predicate
  effects effect-upper-bound
  guarantees implementation-or-semantic-requirements
-> result : ResultClassifier
  ensures input-result-relation
```

Every clause is optional and occurs at most once. `requires`, `effects`, and
`guarantees` describe invocation of the function, so they occur after the
parameters and before the arrow. The optional result binding and `ensures`
describe the returned value, so they occur after the arrow. The result binding
exists only in `ensures`; it is not visible in the body. An unnamed result
remains valid when `ensures` is absent.

```topal
withdraw is fn (
  account : Account,
  amount : PositiveMoney
)
  requires ( amount <= account available )
  effects ( Write account )
  guarantees ( RetrySafe (
    identity is account id,
    equivalent is same-withdrawal
  ) )
-> result : Result ( Account, WithdrawalErrorCode )
  ensures ( withdrawal-relation account amount result )
```

Parameter properties remain with their parameter:

```topal
rewrite is fn (
  input : Buffer : Exclusive
)
-> Buffer
```

The clauses are contextual structural words in these positions. Capitalized
names such as `RetrySafe`, `ResourceBound`, and `Exclusive` are unqualified
objects supplied by `v0.2`; they are not keywords and cannot be shadowed in the
active root scope. Ordinary `:` classification outside a function header is
unchanged.

`v0.1` source may continue to put its effect/resource classifier after the
result as `-> Result : Classifier`. That spelling is rejected in `v0.2` so a
clause cannot appear to classify the result accidentally.

## Contract obligations

A `requires` predicate is pure and total and may refer to parameters, captured
static identities, and already verified facts. Every call must prove it. Topal
does not insert a hidden dynamic failure. Untrusted input is first validated by
an ordinary `Result`-producing constructor; its successful alternative carries
the proof.

An `ensures` predicate is pure and total. Every ordinary, error, or explicit
return proves the applicable relation over the inputs, result, and declared
effect trace. A verified relation is erased at execution. A diagnostic build
may reevaluate an executable predicate, but successful testing never creates
proof evidence and a failed diagnostic check reports a tool defect or violated
external assumption.

A stored nominal type or task may declare one invariant after its fields and
before its operations:

```topal
Account is type
  available : Money

  invariant value ( value available >= zero-money )
```

The explicit binding is visible only in the invariant. Construction establishes
the invariant and every public transition preserves it. A task invariant that
crosses suspension additionally needs protocol, version, or transaction
evidence.

## Evidence records

Semantic evidence proves a fact on which program meaning may rely, such as a
constraint, ownership fact, contract relation, or transaction law.
Implementation evidence proves that one selected implementation meets a bound,
progress class, layout, or specialization requirement. Removing implementation
evidence may make a hard selection unavailable, but never changes semantic
results or makes an unsafe operation safe.

Evidence retained across a public or opaque boundary records:

- its semantic or implementation kind;
- property and classified subject identities, including static parameters;
- `verified`, `trusted-unverified`, `externally-assumed`, or `refuted` status;
- checker derivation or provider identity;
- every assumption identity;
- the language revision; and
- an optional architecture-model identity.

Only `verified` evidence discharges memory, bounds, ownership, totality, race,
deadlock, protocol, or implementation obligations. Project policy may allow a
`trusted-unverified` ordinary semantic law, while retaining that status.
`externally-assumed` evidence discharges an obligation only when the consuming
application explicitly admits every recorded environmental assumption. A
refuted or mismatched record never applies.

Evidence status and `ImplementationEvidence` have no source constructor. A
programmer writes a property requirement; the checker, compiler, interpreter,
or checked provider produces the record. Generated source and foreign
translators have exactly the authority of handwritten source. An analysis or
qualification tool may inspect or independently validate a certificate, but
cannot upgrade a record by editing metadata.

## Standard property vocabulary

`v0.2` owns the verification rules for these unqualified constructors:

- `RetrySafe ( identity is I, equivalent is R )` relates repeated complete
  outcomes and effect traces for one stable operation identity;
- `AtomicCommit ( domains is D )` gives one all-or-nothing commit point for
  exactly `D`;
- `Durable ( store is S, failures is F )` survives the named failure model;
- `Compensates ( forward is F, relation is R )` states a checked trace
  relation and does not erase history;
- `Progress ( class is P )` constrains a concrete interaction implementation;
  and
- `Specialized ( static-inputs is S )` requires the selected compiler artifact
  to remove residual dispatch and structures caused solely by `S`.

Libraries may define relations reducible to contracts and effects. They cannot
give a new name optimizer semantics or create a new evidence status.

## Hard requirements and preferences

An unwrapped item in `guarantees` is hard. The selected implementation must
carry matching verified evidence, or the compiler/interpreter rejects it with
the property, subject, derived or unknown fact, assumptions, and candidates.
`Prefer` is lexicographic selection advice and may fall back to any
semantically valid implementation. An interpreter is therefore allowed to
reject `guarantees ( Progress ( class is LockFree ) )`, while accepting the
same semantic program when the property occurs only inside `Prefer`.

Target facts do not enter ordinary values or branches. Architecture evidence
uses the same typed matching seam, but its provider schema and physical
properties remain deferred to the architecture model.

## Actors and authority

| Entity | Who writes or uses it | Who establishes it |
| --- | --- | --- |
| Header clauses and owned invariants | Declaration owners and checked source generators | Checker verifies; runtime may diagnose |
| Relational semantic properties | Authors may require them and define pure relations | Checker or checked external adapter, with status retained |
| Progress, resource, layout, and specialization properties | Authors may require or prefer them | Concrete compiler/interpreter/runtime or checked artifact provider |
| Evidence status and implementation records | Read-only to source and tools | Checker/compiler or typed checked-provider interface |
| Architecture-dependent evidence | No current source constructor | Deferred architecture provider |

No provider interface is callable from ordinary Topal source. Imported text or
metadata remains a claim until the appropriate checker validates it.
