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

/// A generic type pattern whose parameters are replaced simultaneously.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypePattern {
    Parameter(String),
    Concrete(TypeIdentity),
    Tuple(Vec<Self>),
    Record(Vec<(String, Self)>),
}

impl TypePattern {
    /// Instantiate this pattern without discarding the substitution evidence.
    #[must_use]
    pub fn instantiate(&self, arguments: &BTreeMap<String, TypeIdentity>) -> Option<TypeIdentity> {
        match self {
            Self::Parameter(name) => arguments.get(name).cloned(),
            Self::Concrete(identity) => Some(identity.clone()),
            Self::Tuple(fields) => Some(TypeIdentity::Structural(StructuralType::Tuple(
                fields
                    .iter()
                    .map(|field| field.instantiate(arguments))
                    .collect::<Option<Vec<_>>>()?,
            ))),
            Self::Record(fields) => Some(TypeIdentity::Structural(StructuralType::Record(
                fields
                    .iter()
                    .map(|(label, field)| Some((label.clone(), field.instantiate(arguments)?)))
                    .collect::<Option<Vec<_>>>()?,
            ))),
        }
    }
}

/// Canonical evidence retained by one successful generic instantiation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstantiationEvidence {
    pub declaration: DeclarationIdentity,
    pub arguments: BTreeMap<String, TypeIdentity>,
    pub result: TypeIdentity,
}

/// Canonical proof that one subject provides an atomic capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityEvidence {
    pub capability: QualifiedName,
    pub subject: TypeIdentity,
    pub roles: BTreeMap<String, DeclarationIdentity>,
}

/// A coherence-checked collection of capability proofs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilitySet(BTreeMap<(QualifiedName, TypeIdentity), CapabilityEvidence>);

impl CapabilitySet {
    /// Insert evidence, accepting an identical proof and rejecting ambiguity.
    ///
    /// # Errors
    ///
    /// Returns an error when the same capability and subject have different roles.
    pub fn insert(&mut self, evidence: CapabilityEvidence) -> Result<(), &'static str> {
        let key = (evidence.capability.clone(), evidence.subject.clone());
        if let Some(existing) = self.0.get(&key) {
            return if existing == &evidence {
                Ok(())
            } else {
                Err("conflicting canonical capability evidence")
            };
        }
        self.0.insert(key, evidence);
        Ok(())
    }

    #[must_use]
    pub fn select(
        &self,
        capability: &QualifiedName,
        subject: &TypeIdentity,
    ) -> Option<&CapabilityEvidence> {
        self.0.get(&(capability.clone(), subject.clone()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InterfaceOperation {
    Function {
        inputs: Vec<TypeIdentity>,
        result: TypeIdentity,
    },
    Generator {
        inputs: Vec<TypeIdentity>,
        yielded: TypeIdentity,
        resumed: TypeIdentity,
        result: TypeIdentity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceShape {
    pub identity: DeclarationIdentity,
    pub operations: BTreeMap<String, InterfaceOperation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceImplementation {
    pub interface: InterfaceShape,
    pub operations: BTreeMap<String, DeclarationIdentity>,
}

impl InterfaceImplementation {
    /// Validate that an implementation supplies exactly the interface roles.
    ///
    /// # Errors
    ///
    /// Returns an error for a missing or additional operation role.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.interface.operations.keys().eq(self.operations.keys()) {
            Ok(())
        } else {
            Err("interface implementation roles do not match the interface")
        }
    }
}

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

    #[must_use]
    pub fn contains_all(&self, required: &Self) -> bool {
        required.0.is_subset(&self.0)
    }
}

/// A canonical effect row with an optional polymorphic tail variable.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectRow {
    pub known: EffectSet,
    pub tail: Option<String>,
}

impl EffectRow {
    #[must_use]
    pub fn compose(&self, other: &Self) -> Option<Self> {
        let tail = match (&self.tail, &other.tail) {
            (Some(left), Some(right)) if left != right => return None,
            (Some(tail), _) | (_, Some(tail)) => Some(tail.clone()),
            (None, None) => None,
        };
        Some(Self {
            known: self.known.union(&other.known),
            tail,
        })
    }

    #[must_use]
    pub fn is_contained_by(&self, allowed: &Self) -> bool {
        allowed.known.contains_all(&self.known)
            && (self.tail.is_none() || self.tail == allowed.tail)
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

    #[test]
    fn generic_patterns_preserve_exact_substitutions() {
        let pattern = TypePattern::Record(vec![
            ("first".into(), TypePattern::Parameter("T".into())),
            ("second".into(), TypePattern::Parameter("T".into())),
        ]);
        let nominal = TypeIdentity::Nominal {
            declaration: declaration("UserId", 2),
            parameters: Vec::new(),
        };
        let arguments = BTreeMap::from([("T".into(), nominal.clone())]);
        assert_eq!(
            pattern.instantiate(&arguments),
            Some(TypeIdentity::Structural(StructuralType::Record(vec![
                ("first".into(), nominal.clone()),
                ("second".into(), nominal),
            ])))
        );
        assert!(pattern.instantiate(&BTreeMap::new()).is_none());
    }

    #[test]
    fn capability_evidence_is_coherent_per_exact_subject() {
        let subject = TypeIdentity::Fundamental("Int");
        let capability = QualifiedName(vec!["lang".into(), "Equality".into()]);
        let evidence = CapabilityEvidence {
            capability: capability.clone(),
            subject: subject.clone(),
            roles: BTreeMap::from([("equal".into(), declaration("equal-int", 0))]),
        };
        let mut set = CapabilitySet::default();
        assert_eq!(set.insert(evidence.clone()), Ok(()));
        assert_eq!(set.insert(evidence.clone()), Ok(()));
        assert_eq!(set.select(&capability, &subject), Some(&evidence));

        let conflicting = CapabilityEvidence {
            roles: BTreeMap::from([("equal".into(), declaration("other-equal", 1))]),
            ..evidence
        };
        assert!(set.insert(conflicting).is_err());
    }

    #[test]
    fn interface_implementation_requires_exact_roles() {
        let shape = InterfaceShape {
            identity: declaration("Parser", 0),
            operations: BTreeMap::from([(
                "parse".into(),
                InterfaceOperation::Function {
                    inputs: vec![TypeIdentity::Fundamental("String")],
                    result: TypeIdentity::Fundamental("Boolean"),
                },
            )]),
        };
        let complete = InterfaceImplementation {
            interface: shape.clone(),
            operations: BTreeMap::from([("parse".into(), declaration("parse", 1))]),
        };
        assert_eq!(complete.validate(), Ok(()));
        assert!(
            InterfaceImplementation {
                interface: shape,
                operations: BTreeMap::new(),
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn effect_rows_compose_and_check_containment_canonically() {
        let read = QualifiedName(vec!["app".into(), "read".into()]);
        let write = QualifiedName(vec!["app".into(), "write".into()]);
        let left = EffectRow {
            known: EffectSet::from_effects([read.clone()]),
            tail: Some("E".into()),
        };
        let right = EffectRow {
            known: EffectSet::from_effects([write.clone()]),
            tail: Some("E".into()),
        };
        let composed = left.compose(&right).unwrap();
        assert_eq!(composed.known.iter().count(), 2);
        assert!(left.is_contained_by(&composed));
        assert!(right.is_contained_by(&composed));
        assert!(
            left.compose(&EffectRow {
                known: EffectSet::default(),
                tail: Some("Other".into()),
            })
            .is_none()
        );
    }
}
