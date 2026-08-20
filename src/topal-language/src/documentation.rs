use topal_syntax::{DocumentedDeclaration, DocumentedParameter};

fn entry(name: &str, kind: &str, syntax: &str, documentation: &str) -> DocumentedDeclaration {
    DocumentedDeclaration {
        name: format!("lang {name}"),
        kind: kind.into(),
        syntax: syntax.into(),
        documentation: Some(documentation.into()),
        parameters: Vec::<DocumentedParameter>::new(),
    }
}

/// Return documentation for public built-in identifiers in `lang`.
#[must_use]
pub fn lang_documentation() -> Vec<DocumentedDeclaration> {
    [
        ("Unit", "type", "lang Unit", "The completion type containing only `()`. It carries no payload."),
        ("Boolean", "type", "lang Boolean", "Truth values `true` and `false`, used by decisions and predicates."),
        ("Int", "type", "lang Int", "Arbitrary-precision signed integers. Integer operations do not overflow."),
        ("Nat", "type", "lang Nat", "Arbitrary-precision nonnegative integers."),
        ("Rational", "type", "lang Rational", "Exact rational numbers, not floating-point approximations."),
        ("String", "type", "lang String", "Immutable Unicode text. Characters and encoded bytes are distinct."),
        ("Character", "type", "lang Character", "One Unicode scalar value rather than a grapheme cluster."),
        ("Type", "type", "lang Type", "The static classifier of type objects."),
        ("Optional", "type", "lang Optional Value", "Either `Some payload` or typed absence `None Value`."),
        ("Result", "type", "lang Result (Value, Codes)", "Either `Ok payload` or an `Error` with an admitted code vocabulary."),
        ("List", "type", "lang List Value", "An immutable finite ordered sequence."),
        ("Range", "type", "lang Range Value", "An inclusive pair of ordered bounds; not by itself proof of finite discrete traversal."),
        ("Array", "type", "lang Array (Count, Value)", "A fixed-size homogeneous collection whose count is part of its type."),
        ("Map", "type", "lang Map (Key, Value)", "A finite key-to-value association. Iteration order is not implied."),
        ("Set", "type", "lang Set Value", "A finite collection of unique values. Iteration order is not implied."),
        ("Bag", "type", "lang Bag Value", "A finite multiset retaining each value's multiplicity."),
        ("Generator", "type", "lang Generator (Yielded, Resumed, Result)", "A resumable computation. Exhaustion, final result, and close are distinct."),
        ("Error", "type", "lang Error Codes", "Structured failure evidence with a declared code vocabulary and provenance."),
        ("Layout", "type", "lang Layout Value", "A checked external representation description; it grants no storage access."),
        ("Location", "type", "lang Location Value", "A checked addressed value governed by layout and access evidence."),
        ("Identity", "type", "lang Identity", "A stable opaque identity for a statically inspectable object."),
        ("DeclarationView", "type", "lang DeclarationView", "Visible declaration metadata, including optional documentation."),
        ("FunctionView", "type", "lang FunctionView", "A static view of a function's inputs, output, effects, and identity."),
        ("ScopeView", "type", "lang ScopeView", "A deterministic view containing only visible members."),
        ("LanguageContext", "type", "lang LanguageContext", "The selected language version and feature set."),
        ("context", "function", "lang context", "Return the current static language context."),
        ("version", "binding", "lang version", "The selected Topal language version."),
        ("identity", "function", "lang identity value", "Return a statically known object's stable identity. Runtime-only values are rejected."),
        ("view", "function", "lang view value", "Return a semantic structural view without exposing private representation."),
        ("declaration", "function", "lang declaration value", "Return visible declaration metadata, including source documentation."),
        ("public-members", "function", "lang public-members scope", "Return published visible members. Private members remain hidden transitively."),
        ("same-object", "operator", "left lang same-object right", "Test whether two static views denote exactly the same object."),
        ("equivalent-type", "operator", "left lang equivalent-type right", "Test semantic type equivalence, which is stronger than convertibility."),
        ("compatible-with", "operator", "left lang compatible-with right", "Test the defined compatibility relation between two static types."),
        ("same-layout", "operator", "left lang same-layout right", "Test exact layout equivalence; compatible types can have different layouts."),
        ("serialize", "function", "version (lang serialize) value", "Serialize a supported value canonically for a selected language version."),
        ("deserialize", "function", "lang deserialize stream", "Validate and reconstruct a value from a native serialization stream."),
    ]
    .into_iter()
    .map(|(name, kind, syntax, documentation)| entry(name, kind, syntax, documentation))
    .collect()
}
