impl ResourceTracker {
    /// Add one fresh ownership obligation.
    ///
    /// # Errors
    ///
    /// Returns an error when the resource identity already has an obligation.
    pub fn declare(
        &mut self,
        identity: ResourceIdentity,
        binding: impl Into<String>,
        lifetime: u64,
    ) -> Result<(), &'static str> {
        if self.resources.contains_key(&identity) {
            return Err("resource identity already has an ownership obligation");
        }
        self.declaration_order.push(identity.clone());
        self.resources.insert(
            identity,
            ResourceState::Owned {
                binding: binding.into(),
                lifetime,
            },
        );
        Ok(())
    }

    /// Move ownership to a new binding without duplicating the obligation.
    ///
    /// # Errors
    ///
    /// Returns an error after destruction or from a binding that is not the owner.
    pub fn move_to(
        &mut self,
        identity: &ResourceIdentity,
        from: &str,
        to: impl Into<String>,
    ) -> Result<(), &'static str> {
        let Some(ResourceState::Owned { binding, .. }) = self.resources.get_mut(identity) else {
            return Err("resource is not live");
        };
        if binding != from {
            return Err("move source does not own the resource");
        }
        *binding = to.into();
        Ok(())
    }

    /// Destroy live resources in reverse declaration order.
    #[must_use]
    pub fn destroy_all(&mut self) -> Vec<ResourceIdentity> {
        let mut destroyed = Vec::new();
        for identity in self.declaration_order.iter().rev() {
            if matches!(
                self.resources.get(identity),
                Some(ResourceState::Owned { .. })
            ) {
                self.resources
                    .insert(identity.clone(), ResourceState::Destroyed);
                destroyed.push(identity.clone());
            }
        }
        destroyed
    }

    #[must_use]
    pub fn state(&self, identity: &ResourceIdentity) -> Option<&ResourceState> {
        self.resources.get(identity)
    }
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
