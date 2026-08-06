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
