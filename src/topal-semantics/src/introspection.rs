//! Typed, static views of visible Topal language objects.

use std::collections::BTreeSet;

use crate::{DeclarationIdentity, EffectSet, LanguageVersion, QualifiedName, TypeIdentity};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticName(String);

impl StaticName {
    /// Construct a nonempty, authority-free static name.
    ///
    /// # Errors
    ///
    /// Rejects empty names and embedded null characters.
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.is_empty() || value.contains('\0') {
            Err("a static name must be nonempty and contain no null character")
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub type Label = StaticName;
pub type PathComponent = StaticName;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticPath(pub Vec<PathComponent>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeView {
    Primitive(StaticName),
    Tuple(ComponentStructure),
    Record(FieldStructure),
    Variant(ComponentStructure),
    Union(ComponentStructure),
    Refined(RefinementDescriptor),
    Function(FunctionSignature),
    Opaque(TypeIdentity),
    RecursiveReference(TypeIdentity),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    pub label: Option<Label>,
    pub type_identity: TypeIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentStructure(pub Vec<Component>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub label: Label,
    pub type_identity: TypeIdentity,
    pub depends_on: BTreeSet<usize>,
    pub evidence: Vec<QualifiedName>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldStructure(pub Vec<Field>);

impl FieldStructure {
    #[must_use]
    pub fn independent_fields(&self) -> Option<Vec<&Field>> {
        self.0
            .iter()
            .all(|field| field.depends_on.is_empty())
            .then(|| self.0.iter().collect())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefinementDescriptor {
    pub base: TypeIdentity,
    pub predicate: QualifiedName,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Staticness {
    Static,
    Runtime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionSignature {
    pub identity: DeclarationIdentity,
    pub inputs: Vec<TypeIdentity>,
    pub output: TypeIdentity,
    pub staticness: Staticness,
    pub effects: EffectSet,
    pub context_requirements: BTreeSet<QualifiedName>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VisibleObject {
    Type(TypeIdentity),
    Function(DeclarationIdentity),
    Scope(QualifiedName),
    Declaration(DeclarationIdentity),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeMember {
    pub name: StaticName,
    pub object: VisibleObject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeView {
    pub identity: QualifiedName,
    pub members: Vec<ScopeMember>,
}

impl ScopeView {
    /// Build a deterministic view containing only members visible at the site.
    ///
    /// # Errors
    ///
    /// Rejects duplicate visible names.
    pub fn visible(
        identity: QualifiedName,
        members: impl IntoIterator<Item = ScopeMember>,
    ) -> Result<Self, &'static str> {
        let mut members = members.into_iter().collect::<Vec<_>>();
        members.sort_by(|left, right| left.name.cmp(&right.name));
        if members.windows(2).any(|pair| pair[0].name == pair[1].name) {
            return Err("a scope view cannot contain duplicate visible names");
        }
        Ok(Self { identity, members })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationView {
    pub name: Option<StaticName>,
    pub canonical_path: Option<StaticPath>,
    pub documentation: Option<String>,
    pub license: Option<String>,
    pub copyrights: Vec<String>,
    pub language: LanguageContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageContext {
    pub language: StaticName,
    pub version: LanguageVersion,
    pub features: BTreeSet<QualifiedName>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ObjectRelation {
    SameObject,
    EquivalentType,
    Compatible,
    SameLayout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependent_fields_remain_lossless() {
        let integer = TypeIdentity::Fundamental("Int");
        let fields = FieldStructure(vec![
            Field {
                label: Label::new("start").unwrap(),
                type_identity: integer.clone(),
                depends_on: BTreeSet::new(),
                evidence: Vec::new(),
            },
            Field {
                label: Label::new("end").unwrap(),
                type_identity: integer,
                depends_on: BTreeSet::from([0]),
                evidence: vec![QualifiedName(vec!["greater-than-start".into()])],
            },
        ]);
        assert!(fields.independent_fields().is_none());
    }

    #[test]
    fn visible_scope_members_have_canonical_order() {
        let member = |name| ScopeMember {
            name: StaticName::new(name).unwrap(),
            object: VisibleObject::Type(TypeIdentity::Fundamental("Int")),
        };
        let view = ScopeView::visible(
            QualifiedName(vec!["example".into()]),
            [member("z"), member("a")],
        )
        .unwrap();
        assert_eq!(view.members[0].name.as_str(), "a");
        assert!(
            ScopeView::visible(QualifiedName(vec![]), [member("same"), member("same")]).is_err()
        );
    }
}
