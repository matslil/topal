# String semantics

## Formal text

### TOPAL-STRING-EMPTY-001 — Empty construction

`empty String` shall construct the unique plain `String` whose preserved
Unicode scalar sequence has length zero. Construction shall not consult locale,
normalization, encoding, or external state.

### TOPAL-STRING-CONCAT-001 — Plain exact concatenation

For plain `String` values `a` and `b`, `a concat b` shall produce the
plain `String` whose preserved Unicode scalar sequence is the complete sequence
of `a` followed by the complete sequence of `b`. It shall not normalize,
reinterpret escapes, interpolate placeholders, or insert a separator.

Constraint-aware concatenation which retains shared normalization evidence is
applicable only when that evidence is implemented and selected. Its absence
shall not change the exact plain-string result defined here.

### TOPAL-STRING-LITERAL-COMPOSE-001 — Adjacent literal composition

Two or more adjacent string literal primaries in one expression shall compose
at construction into one plain `String` containing their preserved Unicode
scalar sequences in source order. This rule applies only while every composed
operand is a literal. A binding, function result, or other string-valued
expression requires the explicit `concat` operation.

### TOPAL-STRING-CHARACTER-COUNT-001 — User-perceived character count

For a plain `String` value `text`, `character-count text` shall produce the
finite nonnegative `Int` equal to the number of extended grapheme clusters in
the preserved Unicode sequence under the language context's pinned Unicode
segmentation data. Empty text has count zero. Canonically equivalent sequences
shall be segmented as preserved rather than normalized before counting.

### TOPAL-STRING-CHARACTER-AT-001 — Optional character indexing

For `text : String` and unchecked `index : Int`, `text character-at index`
counts extended grapheme clusters from zero under the same pinned segmentation
as `TOPAL-STRING-CHARACTER-COUNT-001`. A valid index returns `Some Character`
containing the complete preserved cluster. A negative index or one no less than
the character count returns `None` of `Optional Character`. The operation does
not normalize or index encoded bytes or Unicode scalar values.

### TOPAL-STRING-UPPER-001 — Universal uppercase mapping

For a plain `String` value `text`, `upper text` shall apply Unicode's complete,
locale-independent default uppercase mapping under the language context's
pinned Unicode data. It returns a plain `String`, may change the character
count, and shall neither inspect ambient locale nor add normalization evidence.

### TOPAL-STRING-LOWER-001 — Universal lowercase mapping

For a plain `String` value `text`, `lower text` shall apply Unicode's complete,
locale-independent default lowercase mapping under the language context's
pinned Unicode data. It returns a plain `String`, may change the character
count, and shall neither inspect ambient locale nor add normalization evidence.

### TOPAL-STRING-CASE-FOLD-001 — Universal full case folding

For a plain `String` value `text`, `case-fold text` shall apply Unicode's full,
locale-independent default case-folding mapping under the language context's
pinned Unicode data. It returns a plain `String`, may expand characters, and
shall neither inspect ambient locale nor add normalization evidence. This is a
caseless-comparison basis and is not defined as lowercase conversion.

### TOPAL-STRING-CANONICAL-EQUALITY-001 — Canonical equivalence

For plain String values `left` and `right`, `left canonically-equals right`
shall produce true exactly when their preserved Unicode scalar sequences are
canonically equivalent under the language context's pinned Unicode data. It
shall not mutate or normalize either operand. Ordinary String equality remains
exact preserved-sequence equality and is unaffected by this operation.

### TOPAL-STRING-CHARACTERS-COLLECT-001 — Character traversal collection

For a plain String `text`, `characters text` has classifier
`Generator Character Unit Unit` and traverses its preserved sequence as a
finite, ordered generator of complete extended grapheme clusters under the same
pinned segmentation as character counting. Each continuation accepts `Unit`;
after its last yield the generator returns `Unit`. Collecting
that unchanged traversal with `collect String` concatenates every yielded
Character in order and returns a plain String with exactly the original scalar
sequence. Empty input yields no Characters and collects to `empty String`.

### TOPAL-STRING-CHARACTERS-FOREACH-001 — Direct Character traversal

`characters text foreach { character } body` shall invoke `body` once for each
yielded Character in preserved order, with `character` scoped to that invocation.
After each body returns Unit, traversal resumes with Unit. Exhaustion returns
Unit, including for empty input. The binding does not escape the body and each
yielded value retains the Character constraint. A body producing a non-Unit
value is rejected rather than silently discarded.

### TOPAL-STRING-CHARACTERS-GENERATOR-001 — Named linear traversal

`characters text` may be bound as a value classified
`Generator Character Unit Unit`. Consuming that binding with `foreach` transfers
its linear continuation into the traversal; the same source binding is no
longer available afterward and cannot be consumed twice. Debugger history may
snapshot prior generator state for reverse inspection without making a second
source-level continuation.

### TOPAL-STRING-CHARACTERS-CLASSIFIER-001 — Explicit generator classification

The classifier `Generator Character Unit Unit` accepts the value constructed by
`characters text`. An explicitly classified binding shall retain the same
linear consumption behavior as an inferred binding. A value with a different
yield, resume, or return classifier does not satisfy this classifier.

### TOPAL-STRING-CHARACTERS-RESULT-001 — Generator function results

`Generator Character Unit Unit` is a valid ordinary function result classifier.
A function result may transfer the fresh continuation constructed by
`characters text` to its caller. The caller receives one linear generator value
which may be bound and consumed under `TOPAL-STRING-CHARACTERS-GENERATOR-001`.

### TOPAL-STRING-CHARACTERS-LINEAR-001 — Consumed generator rejection

After a named generator continuation is transferred into a consumer, any later
source use of that binding shall be rejected as an already-consumed generator,
not resolved as an ordinary value or silently restarted. Diagnostics shall
identify the consumed binding and may suggest constructing a fresh generator.

### TOPAL-STRING-CHARACTERS-PARAMETER-001 — Generator parameter transfer

`Generator Character Unit Unit` is a valid ordinary function parameter
classifier. Passing a named generator binding to such a parameter transfers its
linear continuation into the function scope; the caller binding is consumed.
The callee may traverse the parameter with `foreach`, and no source-level copy
of the continuation is created by argument evaluation or debugger history.

### TOPAL-STRING-CHARACTER-CLASSIFIER-001 — Character constraint

`Character` shall classify exactly those preserved `String` values whose count
under `TOPAL-STRING-CHARACTER-COUNT-001` is one. Classification shall retain
the complete original scalar sequence without normalization; the one character
may contain multiple Unicode scalar values or encoded bytes. Empty strings and
strings containing two or more characters shall fail classification.

### TOPAL-STRING-FROM-CHARACTER-001 — String construction from Character

`String character` shall accept a value classified as `Character` and return a
plain `String` with exactly the same preserved Unicode scalar sequence. It shall
not normalize, case-map, encode, or otherwise alter the character. A value not
classified as one user-perceived character is not an applicable operand.

### TOPAL-STRING-ENTRY-COUNT-001 — String sequence entry count

Because plain `String` provides `Sequence Character`, `entry-count text` shall
produce exactly the same finite nonnegative `Int` as `character-count text`
under `TOPAL-STRING-CHARACTER-COUNT-001`. It shall count user-perceived
characters, not Unicode scalar values, encoded bytes, or display columns.

### TOPAL-STRING-UTF8-BYTE-COUNT-001 — Prospective UTF-8 byte count

For a plain `String` value `text`, `text byte-count Utf8` shall produce the
finite nonnegative `Int` equal to the number of bytes in the standard UTF-8
encoding of its preserved Unicode scalar sequence. The operation observes a
prospective encoding boundary; it shall not attach an encoding to `text`,
normalize it, or count user-perceived characters or display columns.

### TOPAL-STRING-NORMALIZE-NFC-001 — Explicit NFC normalization

For a plain `String` value `text`, `text normalize NFC` shall produce the
Unicode Normalization Form C transformation of its preserved scalar sequence
under the language context's pinned Unicode data. The result shall be a plain
`String` in this implemented subset. The operation is explicit: constructing,
comparing, concatenating, counting, or encoding a plain String shall not invoke
it implicitly.
### TOPAL-STRING-EMPTY-PREDICATE-001 — String emptiness

For a plain `String` value `text`, `empty? text` shall produce `true` exactly
when its preserved Unicode scalar sequence is empty and `false` otherwise. It
shall agree with both `character-count text = 0` and `entry-count text = 0`
without normalizing, encoding, or otherwise transforming the value.

### TOPAL-STRING-NORMALIZE-NFD-001 — Explicit NFD normalization

For a plain `String` value `text`, `text normalize NFD` shall produce the
Unicode Normalization Form D transformation of its preserved scalar sequence
under the language context's pinned Unicode data. The result shall be a plain
`String` in this implemented subset. The operation is explicit and shall not
change the input binding or introduce normalization into other String
operations.
