//! Shared, deterministic semantic identities for every Topal source tool.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

/// Source-visible immutable language revision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LanguageVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub build: u64,
}

impl LanguageVersion {
    pub const DESIGN_0: Self = Self {
        major: 0,
        minor: 1,
        patch: 0,
        build: 0,
    };
}

impl Default for LanguageVersion {
    fn default() -> Self {
        Self::DESIGN_0
    }
}

impl fmt::Display for LanguageVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.patch == 0 && self.build == 0 {
            write!(formatter, "v{}.{}", self.major, self.minor)
        } else if self.build == 0 {
            write!(formatter, "v{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            write!(
                formatter,
                "v{}.{}.{}-{}",
                self.major, self.minor, self.patch, self.build
            )
        }
    }
}

impl FromStr for LanguageVersion {
    type Err = &'static str;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let body = text
            .strip_prefix('v')
            .ok_or("a language version begins with `v`")?;
        let (release, build) = body.split_once('-').unwrap_or((body, "0"));
        let components = release.split('.').collect::<Vec<_>>();
        if !(2..=3).contains(&components.len()) {
            return Err("a language version requires major and minor components");
        }
        Ok(Self {
            major: components[0].parse().map_err(|_| "invalid major version")?,
            minor: components[1].parse().map_err(|_| "invalid minor version")?,
            patch: components.get(2).map_or(Ok(0), |value| {
                value.parse().map_err(|_| "invalid patch version")
            })?,
            build: build.parse().map_err(|_| "invalid build version")?,
        })
    }
}

/// The closed object-kind vocabulary from `TOPAL-TYPE-KIND-001`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ObjectKind {
    Value,
    Type,
    Function,
    Predicate,
    Constraint,
    Capability,
    Interface,
    Pattern,
    Effect,
    Protocol,
    Scope,
    Module,
}

impl ObjectKind {
    /// Whether an object of `self` satisfies the requested kind classifier.
    #[must_use]
    pub fn satisfies(self, requested: Self) -> bool {
        self == requested || matches!((self, requested), (Self::Predicate, Self::Function))
    }
}

/// Canonical identity of a Topal type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeIdentity {
    Fundamental(&'static str),
    Structural(StructuralType),
    Nominal {
        declaration: DeclarationIdentity,
        parameters: Vec<Self>,
    },
}

/// Recursively exact structural type identity from `TOPAL-TYPE-ID-001`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StructuralType {
    Tuple(Vec<TypeIdentity>),
    Record(Vec<(String, TypeIdentity)>),
    Variant(Vec<TypeIdentity>),
    Union(Vec<(String, TypeIdentity)>),
}

/// Stable source declaration identity; aliases retain this identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeclarationIdentity {
    pub module: String,
    pub name: String,
    pub ordinal: usize,
}

/// A canonical qualified semantic name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QualifiedName(pub Vec<String>);

/// Conservative, canonically ordered effect row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectSet(BTreeSet<QualifiedName>);

impl EffectSet {
    #[must_use]
    pub fn from_effects(effects: impl IntoIterator<Item = QualifiedName>) -> Self {
        Self(effects.into_iter().collect())
    }

    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0).cloned().collect())
    }

    pub fn iter(&self) -> impl Iterator<Item = &QualifiedName> {
        self.0.iter()
    }
}

/// Stable relationship evidence retained by a typing derivation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relationship {
    pub name: QualifiedName,
    pub subjects: Vec<SemanticIdentity>,
}

/// Identity of an exact semantic object participating in relationships.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticIdentity {
    Declaration(DeclarationIdentity),
    Type(TypeIdentity),
    Value(String),
}

/// The reusable result of semantic analysis under `TOPAL-TYPE-JUDGE-001`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedObject {
    pub kind: ObjectKind,
    pub classifier: Option<TypeIdentity>,
    pub effects: EffectSet,
    pub relationships: Vec<Relationship>,
}

/// One declaration candidate in deterministic source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration<T> {
    pub identity: DeclarationIdentity,
    pub value: T,
}

/// Lexical declarations grouped by name while retaining source order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticScope<T> {
    declarations: BTreeMap<String, Vec<Declaration<T>>>,
}

impl<T> SemanticScope<T> {
    pub fn declare(&mut self, name: impl Into<String>, declaration: Declaration<T>) {
        self.declarations
            .entry(name.into())
            .or_default()
            .push(declaration);
    }

    #[must_use]
    pub fn candidates(&self, name: &str) -> &[Declaration<T>] {
        self.declarations.get(name).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(name: &str, ordinal: usize) -> DeclarationIdentity {
        DeclarationIdentity {
            module: "example".into(),
            name: name.into(),
            ordinal,
        }
    }

    #[test]
    fn predicate_is_the_only_subkind() {
        assert!(ObjectKind::Predicate.satisfies(ObjectKind::Function));
        assert!(!ObjectKind::Type.satisfies(ObjectKind::Value));
        assert!(!ObjectKind::Function.satisfies(ObjectKind::Predicate));
    }

    #[test]
    fn nominal_and_structural_identity_remain_distinct() {
        let shape =
            TypeIdentity::Structural(StructuralType::Tuple(vec![TypeIdentity::Fundamental(
                "Int",
            )]));
        let nominal = TypeIdentity::Nominal {
            declaration: declaration("Coordinate", 0),
            parameters: vec![shape.clone()],
        };
        assert_ne!(shape, nominal);
    }

    #[test]
    fn effect_union_is_canonical_and_idempotent() {
        let io = QualifiedName(vec!["lang".into(), "io".into()]);
        let clock = QualifiedName(vec!["lang".into(), "clock".into()]);
        let left = EffectSet::from_effects([io.clone(), clock.clone()]);
        let right = EffectSet::from_effects([io]);
        assert_eq!(left.union(&right), left);
        assert_eq!(left.iter().count(), 2);
    }

    #[test]
    fn declarations_retain_source_order() {
        let mut scope = SemanticScope::default();
        scope.declare(
            "choose",
            Declaration {
                identity: declaration("choose", 0),
                value: "first",
            },
        );
        scope.declare(
            "choose",
            Declaration {
                identity: declaration("choose", 1),
                value: "second",
            },
        );
        assert_eq!(scope.candidates("choose")[0].value, "first");
    }

    #[test]
    fn language_versions_expand_and_order_canonically() {
        assert_eq!("v0.1".parse(), Ok(LanguageVersion::DESIGN_0));
        assert_eq!(LanguageVersion::DESIGN_0.to_string(), "v0.1");
        assert!("v0.2".parse::<LanguageVersion>().unwrap() > LanguageVersion::DESIGN_0);
    }
}
