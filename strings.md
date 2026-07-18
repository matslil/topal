# String and text model

This document records the provisional text model for Topal. It separates text
semantics from storage representation so Unicode is natural in source code and
programs without making algorithms depend on a particular encoding.

## Semantic types

`String` is an immutable sequence of Unicode scalar values. It is a semantic
text type, not an array of bytes, UTF-8 code units, or characters. The compiler
may represent different strings differently as long as their observable text
semantics are preserved.

The core text-related types are:

```topal
Byte
Bytes
Scalar
String
Character
Encoding
Encoded Encoding
```

- `Byte` is a unit of binary storage.
- `Bytes` is binary data with no implied text interpretation.
- `Scalar` is one Unicode scalar value.
- `String` is zero or more Unicode scalar values.
- `Character` is a `String` constrained to exactly one extended grapheme
  cluster: one user-perceived character under Topal's Unicode version.
- `Encoded Encoding` is a byte representation governed by an encoding.

`Character` is therefore reusable constrained text rather than an unrelated
primitive:

```topal
Character is SingleGrapheme String
```

A character may contain multiple scalars and bytes. A scalar may be only one
part of a character, such as a combining mark.

## Source text

Topal source files use UTF-8 and invalid UTF-8 is a source error. Outside string
literals and comments, textual source tokens must be in Unicode Normalization
Form C (NFC). The compiler rejects non-normalized tokens instead of silently
creating spellings that differ from the source file.

String literal contents and comments preserve their scalar sequences. They are
not normalized, because exact text can matter for protocols, signatures, tests,
and round trips. Keywords remain ASCII. Unicode identifiers are allowed, with
diagnostics for mixed scripts and visually confusable names.

The Unicode version used for normalization, character segmentation, properties,
and case operations is fixed by the language or compiler version and is
available as build metadata for reproducibility.

## Counting vocabulary

Counting always names the kind of unit being counted. `String` deliberately has
no unqualified `length`, because bytes, scalars, characters, and display cells
are different measurements:

```topal
text byte-count Utf8
text scalar-count
text char-count
```

`char-count` counts extended grapheme clusters. `byte-count` requires an
encoding because semantic text has no inherent byte representation. Displayed
glyphs and terminal columns depend on fonts, locale, and rendering context and
are separate operations.

Other containers use the same `<unit>-count` vocabulary with their canonical
unit:

```topal
array entry-count
map entry-count
set entry-count
bytes byte-count
```

An array entry is an element, a map entry is a key/value association, and a set
entry is a member. Types with more than one meaningful view use a more specific
unit rather than pretending they have one universal length.

Conceptually these names can be built on one generic counting abstraction:

```topal
value count Entries
text count Scalars
text count Characters
text count ( Bytes Utf8 )
```

The shorter names are ordinary algorithms or aliases derived from this model,
not unrelated language primitives.

## Storage size

`storage-size` includes payload, headers, alignment, indexes, and other overhead
of a specified storage representation. It is distinct from semantic counts and
encoded size:

```topal
text byte-count Utf8             # bytes in the UTF-8 encoding
text storage-size representation # bytes occupied by that representation
```

Physical size is not an intrinsic property of a semantic value. It may depend
on the compiler target, allocator, sharing, interning, and chosen layout.
Consequently `storage-size` must either name a storage layout or be explicitly
an implementation-dependent diagnostic query. It must not constrain ordinary
compiler optimizations accidentally.

Shared storage also needs an explicit accounting policy: a query must state
whether shared allocations are counted once for a complete object graph or once
per reference. This policy remains an implementation-design question; the
semantic distinction between `byte-count` and `storage-size` does not.

## Traversal and indexing

Text exposes explicit views:

```topal
text bytes Utf8
text scalars
text characters
```

The latter two produce finite generators of `Scalar` and `Character`. Normal
text processing should prefer traversal to repeated numeric indexing.

Character indexing addresses grapheme clusters:

```topal
text character-at index
```

It returns `Optional Character` when given an unchecked numeric index. Finding
the third character may require scanning from a known boundary; Topal does not
promise constant-time access merely because the operation accepts an index.

Repeated access can use an index or cursor tied to the particular string and
unit:

```topal
index : CharacterIndex text
```

Such an index proves that it belongs to `text`, lies on a character boundary,
and is in range. Scalar, character, and encoded-byte indices are distinct. A
string slice similarly requires boundaries for its chosen unit and cannot split
a character accidentally. The result remains an ordinary `String`; slice and
selection provenance may be retained as constraints for checking and
optimization as described in [the range model](ranges.md).

## Equality and normalization

Plain string equality compares exact scalar sequences:

```topal
left = right
```

Canonically equivalent spellings are not silently made equal. Normalization and
human-language comparisons are explicit:

```topal
text normalize NFC
text normalize NFD
left canonically-equals right
left case-fold
left compare ( right , collation )
```

This preserves exact data while allowing callers to request the equivalence
appropriate to their domain.

## Encodings and storage formats

UTF-8, UTF-16, ASCII, and legacy character encodings describe stored or
transmitted bytes, not different semantic string types. They are analogous to
endianness and integer width: properties of an external or requested
representation.

```topal
text encode Utf8
text encode Utf16LE
encoded decode
```

Decoding untrusted bytes returns an explicit error:

```topal
bytes decode Utf8
  Ok text then use text
  Error problem then report problem
```

Encoding may fail when the target encoding cannot represent every scalar. UTF-8
can represent every valid `String`; ASCII cannot:

```topal
text encode Ascii
  Ok encoded then write encoded
  Error scalar then report scalar
```

ASCII text and ASCII-encoded storage remain separate concepts:

```topal
AsciiText is AsciiCharacters String
Encoded Ascii
```

The first is Unicode text constrained to the ASCII repertoire. The second is a
specific byte representation.

## Representation freedom

No source operation may rely on `String` using UTF-8 internally. The compiler is
free to select UTF-8, ASCII-compatible compact storage, ropes, shared slices,
interned data, or another representation when behavior is preserved.

In particular, every ASCII byte sequence is also the identical UTF-8 byte
sequence. The compiler may store a string proven to contain only ASCII using an
ASCII-optimized representation even when it will later be encoded as UTF-8. It
may also change representations across operations or compilation targets.

Representation becomes observable only at an explicit boundary such as:

```topal
text encode encoding
value store layout
value storage-size layout
```

This is the same design principle used for integer endianness and hardware
layouts: programmers state boundary requirements, while the compiler chooses
the internal representation.

## Provisional examples

```topal
greeting is String "Hej världen"
first is greeting character-at 0
characters is greeting char-count
utf8-size is greeting byte-count Utf8
```

```topal
identifier : CamelCase String
```

```topal
encoded is text encode Utf16LE
```

```topal
text characters
  character then process character
```

The exact call signatures may evolve with the rest of the syntax. The stable
design intent is that text units are explicit, Unicode character behavior is
natural, and storage formats remain boundary concerns rather than leaking into
semantic algorithms.
