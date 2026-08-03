# Sensitive values and leakage

Topal lets a programmer classify a value as sensitive when it contains a
password, private key, access token, or other information which should not leave
the application accidentally. Sensitivity is compiler-checked provenance, not
a different semantic type, an encryption mechanism, or a defense against
deliberately malicious application code.

`Sensitive` is a classifier and therefore appears on the right of `:` with an
initial capital letter:

```topal
password : Sensitive String
password is read-password

private-key : Sensitive PrivateKey
private-key is load-private-key configuration
```

`Sensitive String` remains semantically a `String`. The classification does not
change equality, select overloads, or create a new nominal type. It records that
the value directly contains or represents protected information.

## Direct-information propagation

Copying, moving, borrowing, or containing a sensitive value preserves its
classification:

```topal
password : Sensitive String is read-password
copy is password
record is ( credential is password )
```

Both `copy` and the reachable `credential` field remain sensitive. The same
rule applies to returning the value unchanged, destructuring it, selecting a
view, or placing it in a collection. Representation-derived operations such as
substring extraction, encoding, serialization, and formatting also preserve
sensitivity because their results still carry information taken directly from
the source:

```topal
prefix : Sensitive String is password substring ( 0, 3 )
encoded : Sensitive Bytes is utf8 encode password
message : Sensitive String is "Password: " + password
```

Renaming a binding or hiding a transformation behind a function cannot remove
this evidence. A function which returns directly represented information
classifies that result explicitly:

```topal
normalize-password is fn (
  password : Sensitive String
) -> Sensitive String
  normalize password
```

Returning a visibly sensitive value as an unclassified result is an error.
Sensitivity does not disappear merely because a new runtime object was
allocated.

## Quantitative leakage

A conclusion about sensitive information need not contain the source directly,
but it still reveals information. `Leakage` is a separate implementation
guarantee which gives a worst-case upper bound for each named sensitive source:

```topal
verify-password is fn (
  supplied : Sensitive String,
  expected : Sensitive String
) -> Boolean
  : Leakage (
      supplied <= 1[b],
      expected <= 1[b]
    )
  supplied == expected
```

`b` is the bit unit and `B` is the byte unit, so `1[B]` is definitionally
`8[b]`. A Boolean observation has at most two outcomes and can therefore reveal
at most one bit per invocation. `Leakage` bounds the complete observable
behavior rather than merely the returned representation.

The guarantee is relative to source identity. Entries for several sensitive
sources are independent upper bounds, not a claim that their sum is the exact
information in one output. Symbolic expressions may use static input sizes,
bounded repetition counts, and ordinary exact quantity arithmetic.

`Sensitive T` and `Leakage` answer different questions:

- `Sensitive T` says that a value directly carries protected information; and
- `Leakage ( source <= amount )` bounds what a computation may reveal about
  that source through every modeled observation.

A derived length, hash, comparison, or policy decision may therefore be
non-sensitive while still carrying leakage evidence. Standard-library and
user-defined operation contracts describe whether their results retain direct
sensitivity and what leakage their observations introduce. This avoids both
silently declassifying encodings and mechanically classifying every conclusion
as a directly sensitive value.

## Observable channels

The compiler analyzes all channels in the leakage model selected by the Topal
language or compiler variant. The standard model includes at least:

- returned values, errors, and their shapes;
- branch, decision-table, and dynamic-alternative selection when observable;
- task messages, effects, diagnostics, and external writes;
- output sizes and allocation behavior when externally observable;
- termination, nontermination, timeout, and cleanup behavior; and
- execution-time differences the compiler can derive from control flow,
  selected implementations, operations, and target evidence.

The source operation need not name a particular security technique. In the
`verify-password` example, ordinary `==` is valid only when the selected or
generated equality implementation satisfies the complete leakage requirement.
A short-circuit comparison which exposes length or matching-prefix information
through timing does not satisfy a one-bit bound. The compiler may select or
generate a suitable implementation; otherwise the declaration is rejected.
Constant-time execution is one possible implementation technique, not a
separate semantic operation and not by itself proof that other channels are
safe.

Channels outside the selected model are grey zones and are ignored by default.
These may include target-specific cache behavior, speculative execution, power
analysis, undocumented hardware behavior, or scheduler details for which the
variant supplies no analysis. Verification consequently always means verified
against a named, versioned leakage model rather than proof against every
physical side channel.

## Conservative composition

Visible typed intermediate code retains sensitive-source identities, leakage
expressions, implementation evidence, and the model under which bounds were
checked. The compiler composes bounds conservatively:

- sequential observations of one source add;
- bounded repetition multiplies the body's bound by the iteration bound;
- unbounded repeated observation is unbounded;
- alternatives include both their observable results and information revealed
  by which alternative was selected;
- calls substitute caller source identities and measures into callee evidence;
- records and other combined observations retain all contributing bounds; and
- erased, opaque, or unknown implementation evidence has unbounded or
  indeterminate leakage unless a usable guarantee remains.

Repeated and adaptive calls accumulate; a one-bit oracle does not become free
merely because every individual call has the same contract. Optimizations and
implementation selection must preserve the bound. A `Leakage` classification
is a hard applicability requirement, so source order breaks ties only among
implementations which already satisfy it.

## Verification and acceptance policy

For a required bound, leakage analysis has three modeled outcomes:

- **verified**: the calculated upper bound satisfies the requirement;
- **disproved**: the calculated upper bound exceeds it; or
- **indeterminate**: the compiler recognizes a relevant modeled dependency but
  cannot calculate a sufficient bound.

The default policy accepts verified evidence and reports a compile error for
both disproved and indeterminate requirements. Unmodeled grey zones do not by
themselves make the result indeterminate.

A compiler option may accept indeterminate analysis in a best-effort build. It
records `trusted-unverified` leakage evidence and emits a diagnostic unless that
diagnostic is explicitly suppressed. Accepting a known violation is a separate,
more severe option and must never be implied by accepting indeterminate
evidence. Artifacts retain violated status when such an exceptional build is
permitted; they do not falsely publish the requirement as satisfied.

A language or compiler variant may select a stricter or broader leakage model
and its default acceptance policy. Compiled contracts record the model identity
and version, verification status, and deliberately excluded channel classes so
downstream compilation can reproduce or strengthen the decision.

## Application boundaries

The compiler distinguishes local computation from operations which can send
information outside the application. Such boundaries include files, sockets,
child processes, exported interactions, and externally collected diagnostics.
A boundary parameter must explicitly accept direct sensitive information:

```topal
send-credential is fn (
  connection : Connection,
  credential : Sensitive String
) -> Result ( Completed, BoundaryErrorCode )
  protocol-send ( connection, credential )
```

This classification authorizes the direct transfer; it does not promise
encryption, redaction, access control, secure erasure, or a small leakage bound.
Those properties require their own contracts. An unclassified boundary rejects
a sensitive value:

```topal
password : Sensitive String is read-password
print password # Compile error: print does not accept sensitive information.
```

The check follows sensitive information through records, collections, errors,
closures, and other containers. A boundary or enclosing scope may additionally
require a quantitative `Leakage` bound for conclusions derived from the source.

## Diagnostics and generated support

Compiler-generated diagnostics must not print sensitive contents. They may
identify the binding, type, source location, leakage status, model, and rejected
boundary. Tracing, logging, panic presentation, core dumps, and similar
generated support omit or redact sensitive contents unless their external
operation explicitly accepts them and satisfies any applicable leakage
requirement.

This system is intended to prevent mistakes and provide useful quantitative
checking. It does not replace cryptography, platform memory protection, secret
storage, review of explicitly sensitive boundaries, or analysis of side
channels outside the selected leakage model.
