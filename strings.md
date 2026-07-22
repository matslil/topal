# String and text model

This document records the provisional text model for Topal. It keeps ordinary
text as easy to handle as ASCII while the compiler and standard library account
for Unicode segmentation, casing, normalization algorithms, and encoding.

## Semantic types

`String` is immutable Unicode text which preserves the Unicode sequence supplied
to it. Its ordinary observable units are user-perceived characters; programs do
not need code-point indexing or encoding units during normal text processing.
Exact equality and encoding nevertheless preserve the distinction between
canonically equivalent sequences unless normalization is explicitly selected.

The core text-related types are:

```topal
Byte
Bytes
String
Character
Encoding
Encoded Encoding
```

- `Byte` is a unit of binary storage.
- `Bytes` is binary data with no implied text interpretation.
- `String` is zero or more user-perceived characters.
- `Character` is a `String` constrained to exactly one user-perceived character
  under Topal's Unicode version.
- `Encoded Encoding` is a byte representation governed by an encoding.

`Character` is therefore reusable constrained text rather than an unrelated
primitive:

```topal
Character is SingleCharacter String
```

A character may require several Unicode code points or encoded bytes, but
ordinary string operations do not expose that representation. Exact code-point
inspection belongs to specialized Unicode tooling rather than the fundamental
`String` interface.

## Source text

Topal source files use UTF-8 and invalid UTF-8 is a source error. Outside string
literal contents and comments, textual source tokens must be in Unicode
Normalization Form C (NFC). The compiler rejects non-normalized tokens instead
of silently creating spellings that differ from the source file.

String literal contents and comments preserve their Unicode sequences. Decoding
external text likewise preserves the decoded sequence rather than silently
normalizing it. Keywords remain ASCII. Unicode identifiers are allowed, with
diagnostics for mixed scripts and visually confusable names.

String-literal delimiters are described in [the syntax sketch](syntax.md#string-literals).
Ordinary and tagged raw literals have the same `String` semantics: their source
contents are preserved without escape processing or normalization. Literal tags
are NFC source syntax and must match exactly; they are not part of the resulting
string.

The Unicode version used for normalization, character segmentation, properties,
and case operations is fixed by the language or compiler version and is
available as build metadata for reproducibility.

Protocols, signatures, tests, and lossless byte-for-byte round trips retain the
original `Encoded` value. A decoded `String` preserves its Unicode sequence, but
decoding and re-encoding need not preserve invalid, redundant, or otherwise
encoding-specific byte representations.

## Normalization constraints

Normalization is a constraint on `String`, not an invariant of every string:

```topal
Normalized NFC String
Normalized NFD String
```

Applying a normalized type to dynamic input validates the existing sequence and
fails when it is not already in the requested form. It does not silently change
the value. Explicit normalization transforms the sequence and establishes the
corresponding evidence:

```topal
normalized is text normalize NFC
normalized : Normalized NFC String
```

This separates two boundary policies. Validation rejects noncanonical input
when a protocol requires it, while normalization safely accepts canonically
equivalent text when the application does not care which sequence arrived.

Normalization forms are not generally closed under exact concatenation. Even
two individually normalized strings can require composition or canonical
reordering where they meet. Standard concatenation maintains a shared
normalization constraint by repairing the join:

```text
Normalized F String concatenate Normalized F String
  -> Normalized F String
```

For example:

```topal
left : Normalized NFC String
right : Normalized NFC String
combined is left concatenate right
combined : Normalized NFC String
```

The repair may compose or reorder Unicode code points, but it preserves the
characters visible through the ordinary string interface. This transformation
is part of maintaining the explicitly selected normalization constraint.

If either operand lacks matching normalization evidence, concatenation preserves
the operands' exact sequences and returns plain `String`. The compiler may still
retain a normalization constraint when it can prove the particular join needs
no repair. A plain result can always be normalized explicitly afterward:

```topal
combined is ( left concatenate right ) normalize NFC
```

## Fundamental operations

The minimal semantic structure of `String` consists of empty construction,
construction from one `Character`, concatenation, and character traversal:

```topal
empty String
String character
left concatenate right
characters text
```

Plain concatenation preserves both supplied Unicode sequences. When both
operands carry the same normalization constraint, the constraint-aware behavior
described above repairs their join and retains that evidence.

Counting, selection, equality, and comparison remain standard vocabulary
because their laws are useful and implementations can provide them more
efficiently than user-defined traversal:

```topal
character-count text
text character-at index
text select character-range
left = right
left compare ( right , collation )
```

Character segmentation is provided by the `String` interface rather than
exposed as code-point manipulation. Normalization and casing are standard
vocabulary because correct implementations require the language's Unicode data
and may inspect or transform the complete string:

```topal
upper text
lower text
case-fold text
```

The unqualified operations use Unicode's locale-independent default casing from
Topal's fixed Unicode version. This is the deterministic default when the text's
language is unknown; it never depends on the operating system, process, or
user's ambient locale. Locale-sensitive casing names its policy explicitly:

```topal
text upper Turkish
text lower Lithuanian
```

Casing returns `String`, not `Character`, because a mapping can depend on
context or change the number of characters. `case-fold` is the ordinary basis
for caseless matching rather than lowercasing. When their input carries a
normalization constraint, these standard transformations normalize their result
to the same form and retain the evidence. With plain `String` input they return
plain `String` and do not add normalization implicitly.

## Counting

`String` has no unqualified `length`. Its ordinary semantic count is named
after its unit, while encoded size names an encoding:

```topal
character-count text
text byte-count Utf8
```

The second operation asks how many bytes `text` would occupy if encoded as
UTF-8. A `String` has no encoding even when a program intends to encode it at a
later boundary. Once encoding has produced an `Encoded Utf8` value, its encoded
byte sequence has an unambiguous byte count:

```topal
encoded is text encode Utf8
size is byte-count encoded
```

Displayed glyphs and terminal columns depend on fonts, locale, and rendering
context and are separate operations. Other containers use the same
`<unit>-count` vocabulary with their canonical unit:

```topal
entry-count array
entry-count map
entry-count set
byte-count bytes
```

Conceptually these names can be built on one generic counting abstraction:

```topal
value count Entries
text count Characters
text count ( Bytes Utf8 )
```

## Traversal, indexing, and selection

Character traversal produces a finite generator of `Character`:

```topal
character-generator is characters text

character-generator foreach { character }
  process character
```

Normal text processing should prefer traversal to repeated numeric indexing.
Character indexing returns `Optional Character` when given an unchecked numeric
index:

```topal
text character-at index
```

Finding a character may require scanning from a known boundary; Topal does not
promise constant-time access merely because the operation accepts an index.
Repeated access can use an index tied to the particular string:

```topal
index : CharacterIndex text
```

Such an index proves that it belongs to `text`, is on a character boundary, and
is in range. String selection likewise uses character boundaries and cannot
split a character accidentally. The result remains `String`; selection and
normalization evidence may be retained when their constraints remain true, as
described in [the range model](ranges.md).

## Equality and human-language comparison

Plain equality compares the preserved Unicode sequences. Canonically equivalent
but differently expressed strings are therefore not necessarily equal:

```topal
left = right
```

Caseless and human-language comparisons request the appropriate operation or
policy explicitly:

```topal
case-fold text
left canonically-equals right
left compare ( right , collation )
```

## Encodings and storage formats

String encodings describe stored or transmitted bytes, not different semantic
string types. They are analogous to endianness and integer width: properties of
an external or requested representation. An encoding may additionally promise
a Unicode normalization form at that boundary. The initial vocabulary is:

```text
Utf8       Utf8Nfc       Utf8Nfd
Utf16      Utf16Nfc      Utf16Nfd
Utf32      Utf32Nfc      Utf32Nfd
Ascii
```

The hyphenated descriptive names are UTF8, UTF8-NFC, UTF8-NFD, UTF16,
UTF16-NFC, UTF16-NFD, UTF32, UTF32-NFC, UTF32-NFD, and ASCII. The identifiers
above follow Topal's ordinary identifier spelling.

Encoding with an NFC or NFD variant transforms the string to that form before
emitting bytes. Decoding validates both the character encoding and the declared
normalization form, producing corresponding normalization evidence on success.
The unqualified UTF variants preserve the semantic string's existing Unicode
sequence and add no normalization promise.

UTF-16 and UTF-32 layouts additionally specify endianness as an independent
layout attribute rather than multiplying the encoding vocabulary with `LE` and
`BE` variants.

```topal
text encode Utf8
text encode Utf16
decode encoded
```

Decoding untrusted bytes returns an explicit error:

```topal
bytes decode Utf8
  Ok text then use text
  Error problem then report problem
```

Encoding may fail when the target encoding cannot represent every character.
UTF-8 can represent every valid `String`; ASCII cannot:

```topal
text encode Ascii
  Ok encoded then write encoded
  Error character then report character
```

ASCII text and ASCII-encoded storage remain separate concepts:

```topal
AsciiText is AsciiCharacters String
Encoded Ascii
```

The first is text constrained to the ASCII repertoire. The second is a specific
byte representation. ASCII text is already in every Unicode normalization form,
so that evidence may be derived without transforming it.

## Storage size and representation freedom

`storage-size` includes payload, headers, alignment, indexes, and overhead of a
specified representation. It is distinct from semantic counts and encoded size:

```topal
text byte-count Utf8
text storage-size representation
```

No source operation may rely on `String` using UTF-8 or on a particular Unicode
segmentation internally. The compiler may select compact ASCII storage, UTF-8,
ropes, shared slices, interned data, or another representation when observable
behavior is preserved.

Representation becomes observable only at an explicit boundary:

```topal
text encode encoding
value store layout
value storage-size layout
```

This is the same design principle used for integer endianness and hardware
layouts: programmers state boundary requirements while the compiler chooses the
internal representation.

## Provisional examples

```topal
greeting is String "Hej världen"
first is greeting character-at 0
count is character-count greeting
utf8-size is greeting byte-count Utf8
shout is upper greeting
```

```topal
identifier : CamelCase String
```

```topal
encoded is text encode Utf16LE
```

The exact call signatures may evolve with the rest of the syntax. The stable
design intent is that ordinary text exposes characters rather than code-point
machinery, exact Unicode sequences remain available when wanted, normalization
is explicit and compositional, and storage formats remain boundary concerns.
