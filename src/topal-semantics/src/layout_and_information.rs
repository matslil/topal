use std::collections::{BTreeMap, BTreeSet};

use crate::{EvidenceProducer, QualifiedName};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapeDimension {
    pub identity: String,
    pub extent: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Shape {
    pub dimensions: Vec<ShapeDimension>,
}

impl Shape {
    /// Validate unique semantic dimension identities.
    ///
    /// # Errors
    ///
    /// Returns an error when two dimensions have the same identity.
    pub fn validate(&self) -> Result<(), &'static str> {
        let identities = self
            .dimensions
            .iter()
            .map(|dimension| &dimension.identity)
            .collect::<BTreeSet<_>>();
        if identities.len() == self.dimensions.len() {
            Ok(())
        } else {
            Err("shape dimension identities must be unique")
        }
    }

    #[must_use]
    pub fn element_count(&self) -> Option<u128> {
        self.dimensions.iter().try_fold(1_u128, |count, dimension| {
            count.checked_mul(u128::from(dimension.extent))
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComponentOrganization {
    Interleaved,
    Separated,
    Blocked { extent: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositeLayout {
    pub shape: Shape,
    pub dimension_order: Vec<String>,
    pub strides: Option<BTreeMap<String, u128>>,
    pub blocking: BTreeMap<String, u64>,
    pub component_organization: ComponentOrganization,
    pub element_size: u128,
    pub alignment: u128,
}

impl CompositeLayout {
    /// Validate complete multidimensional representation policy.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid dimensions, sizes, alignment, blocking,
    /// strides, overlap, or arithmetic overflow.
    pub fn validate(&self) -> Result<(), &'static str> {
        self.shape.validate()?;
        if self.element_size == 0 || self.alignment == 0 {
            return Err("layout element size and alignment must be positive");
        }
        if matches!(
            self.component_organization,
            ComponentOrganization::Blocked { extent: 0 }
        ) || self.blocking.values().any(|extent| *extent == 0)
        {
            return Err("layout block extents must be positive");
        }
        let dimensions = self
            .shape
            .dimensions
            .iter()
            .map(|dimension| dimension.identity.clone())
            .collect::<BTreeSet<_>>();
        let order = self
            .dimension_order
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        if dimensions != order || order.len() != self.dimension_order.len() {
            return Err("dimension order must be a complete permutation");
        }
        if self
            .blocking
            .keys()
            .any(|identity| !dimensions.contains(identity))
        {
            return Err("blocking names an unknown dimension");
        }
        if let Some(strides) = &self.strides {
            if strides.keys().cloned().collect::<BTreeSet<_>>() != dimensions {
                return Err("explicit strides must cover every dimension");
            }
            if strides.values().any(|stride| *stride == 0) {
                return Err("explicit strides must be positive");
            }
            self.validate_explicit_strides(strides)?;
        } else if !self.element_size.is_multiple_of(self.alignment) {
            return Err("implicit element stride does not preserve alignment");
        }
        self.storage_size()
            .ok_or("layout storage-size arithmetic overflows")?;
        Ok(())
    }

    fn validate_explicit_strides(
        &self,
        strides: &BTreeMap<String, u128>,
    ) -> Result<(), &'static str> {
        let mut active = self
            .shape
            .dimensions
            .iter()
            .filter(|dimension| dimension.extent > 1)
            .map(|dimension| (strides[&dimension.identity], dimension.extent))
            .collect::<Vec<_>>();
        active.sort_unstable();
        let mut occupied_span = self.element_size;
        for (stride, extent) in active {
            if !stride.is_multiple_of(self.alignment) {
                return Err("explicit stride does not preserve alignment");
            }
            if stride < occupied_span {
                return Err("explicit strides may overlap semantic elements");
            }
            occupied_span = u128::from(extent - 1)
                .checked_mul(stride)
                .and_then(|additional| occupied_span.checked_add(additional))
                .ok_or("layout address arithmetic overflows")?;
        }
        Ok(())
    }

    fn offset_with_strides(
        &self,
        coordinates: &[u64],
        strides: &BTreeMap<String, u128>,
    ) -> Result<u128, &'static str> {
        self.shape.dimensions.iter().zip(coordinates).try_fold(
            0_u128,
            |offset, (dimension, coordinate)| {
                let term = u128::from(*coordinate)
                    .checked_mul(strides[&dimension.identity])
                    .ok_or("layout address arithmetic overflows")?;
                offset
                    .checked_add(term)
                    .ok_or("layout address arithmetic overflows")
            },
        )
    }

    /// Calculate an element offset in storage units.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid layout, an out-of-range coordinate, or
    /// address arithmetic overflow.
    pub fn offset(&self, coordinates: &[u64]) -> Result<u128, &'static str> {
        self.validate()?;
        if coordinates.len() != self.shape.dimensions.len()
            || coordinates
                .iter()
                .zip(&self.shape.dimensions)
                .any(|(coordinate, dimension)| *coordinate >= dimension.extent)
        {
            return Err("layout coordinate is outside its shape");
        }
        if let Some(strides) = &self.strides {
            return self.offset_with_strides(coordinates, strides);
        }
        let by_identity = self
            .shape
            .dimensions
            .iter()
            .zip(coordinates)
            .map(|(dimension, coordinate)| {
                (dimension.identity.as_str(), (*coordinate, dimension.extent))
            })
            .collect::<BTreeMap<_, _>>();
        let mut linear = 0_u128;
        for identity in &self.dimension_order {
            let (coordinate, extent) = by_identity[identity.as_str()];
            let block = self.blocking.get(identity).copied().unwrap_or(1);
            let outer_extent = extent.div_ceil(block);
            let outer = coordinate / block;
            linear = linear
                .checked_mul(u128::from(outer_extent))
                .and_then(|value| value.checked_add(u128::from(outer)))
                .ok_or("layout address arithmetic overflows")?;
        }
        for identity in &self.dimension_order {
            let (coordinate, extent) = by_identity[identity.as_str()];
            let block = self
                .blocking
                .get(identity)
                .copied()
                .unwrap_or(1)
                .min(extent.max(1));
            linear = linear
                .checked_mul(u128::from(block))
                .and_then(|value| value.checked_add(u128::from(coordinate % block)))
                .ok_or("layout address arithmetic overflows")?;
        }
        linear
            .checked_mul(self.element_size)
            .ok_or("layout address arithmetic overflows")
    }

    #[must_use]
    pub fn storage_size(&self) -> Option<u128> {
        if self
            .shape
            .dimensions
            .iter()
            .any(|dimension| dimension.extent == 0)
        {
            return Some(0);
        }
        if let Some(strides) = &self.strides {
            let maximum = self
                .shape
                .dimensions
                .iter()
                .try_fold(0_u128, |offset, dimension| {
                    u128::from(dimension.extent.saturating_sub(1))
                        .checked_mul(*strides.get(&dimension.identity)?)
                        .and_then(|term| offset.checked_add(term))
                })?;
            maximum.checked_add(self.element_size)
        } else {
            self.shape
                .dimensions
                .iter()
                .try_fold(1_u128, |count, dimension| {
                    let block = self
                        .blocking
                        .get(&dimension.identity)
                        .copied()
                        .unwrap_or(1)
                        .min(dimension.extent);
                    u128::from(dimension.extent.div_ceil(block))
                        .checked_mul(u128::from(block))
                        .and_then(|padded_extent| count.checked_mul(padded_extent))
                })?
                .checked_mul(self.element_size)
        }
    }

    #[must_use]
    pub fn zero_copy_compatible(&self, other: &Self) -> bool {
        self.shape == other.shape
            && self.element_size == other.element_size
            && self.dimension_order == other.dimension_order
            && self.strides == other.strides
            && self.blocking == other.blocking
            && self.component_organization == other.component_organization
            && self.alignment == other.alignment
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DuplicatePolicy {
    Reject,
    LastWins,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseEntry<T> {
    pub coordinates: Vec<u64>,
    pub value: T,
}

/// Validate and canonicalize sparse coordinates.
///
/// # Errors
///
/// Returns an error for an invalid shape, out-of-range coordinate, or a
/// duplicate forbidden by `duplicate_policy`.
pub fn canonical_sparse<T: Clone + Eq>(
    shape: &Shape,
    entries: impl IntoIterator<Item = SparseEntry<T>>,
    zero: &T,
    duplicate_policy: DuplicatePolicy,
) -> Result<Vec<SparseEntry<T>>, &'static str> {
    shape.validate()?;
    let mut canonical = BTreeMap::new();
    for entry in entries {
        if entry.coordinates.len() != shape.dimensions.len()
            || entry
                .coordinates
                .iter()
                .zip(&shape.dimensions)
                .any(|(coordinate, dimension)| *coordinate >= dimension.extent)
        {
            return Err("sparse coordinate is outside its shape");
        }
        if &entry.value == zero {
            continue;
        }
        if canonical.contains_key(&entry.coordinates) && duplicate_policy == DuplicatePolicy::Reject
        {
            return Err("sparse representation contains a duplicate coordinate");
        }
        canonical.insert(entry.coordinates, entry.value);
    }
    Ok(canonical
        .into_iter()
        .map(|(coordinates, value)| SparseEntry { coordinates, value })
        .collect())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InformationPolicy {
    pub identity: QualifiedName,
    pub labels: BTreeSet<String>,
    pub can_flow_to: BTreeSet<(String, String)>,
    pub join: BTreeMap<(String, String), String>,
    pub meet: BTreeMap<(String, String), String>,
}

impl InformationPolicy {
    /// Verify a finite lattice and its declared flow relation.
    ///
    /// # Errors
    ///
    /// Returns an error when labels, flow, join, or meet do not form the
    /// declared finite lattice.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.labels.is_empty() {
            return Err("information policy requires a label");
        }
        for label in &self.labels {
            if !self.can_flow_to.contains(&(label.clone(), label.clone())) {
                return Err("information flow relation is not reflexive");
            }
        }
        for (left, right) in &self.can_flow_to {
            if !self.labels.contains(left) || !self.labels.contains(right) {
                return Err("information flow relation names an unknown label");
            }
            if left != right && self.can_flow_to.contains(&(right.clone(), left.clone())) {
                return Err("information flow relation is not antisymmetric");
            }
        }
        for left in &self.labels {
            for middle in &self.labels {
                for right in &self.labels {
                    if self.can_flow_to.contains(&(left.clone(), middle.clone()))
                        && self.can_flow_to.contains(&(middle.clone(), right.clone()))
                        && !self.can_flow_to.contains(&(left.clone(), right.clone()))
                    {
                        return Err("information flow relation is not transitive");
                    }
                }
            }
        }
        for left in &self.labels {
            for right in &self.labels {
                let pair = (left.clone(), right.clone());
                let joined = self.join.get(&pair).ok_or("join is not total")?;
                let met = self.meet.get(&pair).ok_or("meet is not total")?;
                if !self.labels.contains(joined) || !self.labels.contains(met) {
                    return Err("join or meet returns an unknown label");
                }
                if !self.flows(left, joined)
                    || !self.flows(right, joined)
                    || !self.flows(met, left)
                    || !self.flows(met, right)
                {
                    return Err("join or meet does not bound both operands");
                }
                if self.labels.iter().any(|candidate| {
                    (self.flows(left, candidate) && self.flows(right, candidate))
                        && !self.flows(joined, candidate)
                }) {
                    return Err("join is not the least upper bound");
                }
                if self.labels.iter().any(|candidate| {
                    (self.flows(candidate, left) && self.flows(candidate, right))
                        && !self.flows(candidate, met)
                }) {
                    return Err("meet is not the greatest lower bound");
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn flows(&self, source: &str, target: &str) -> bool {
        self.can_flow_to
            .contains(&(source.to_owned(), target.to_owned()))
    }

    /// Return the declared join of two labels.
    ///
    /// # Errors
    ///
    /// Returns an error when the pair has no declared join.
    pub fn joined(&self, left: &str, right: &str) -> Result<String, &'static str> {
        self.join
            .get(&(left.to_owned(), right.to_owned()))
            .cloned()
            .ok_or("join is undefined for these labels")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InformationLabel {
    pub confidentiality: String,
    pub integrity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Labeled<T> {
    pub policy: QualifiedName,
    pub label: InformationLabel,
    pub value: T,
    pub provenance: BTreeSet<QualifiedName>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityKind {
    Declassify,
    Endorse,
}

#[derive(Debug, Eq, PartialEq)]
pub struct InformationAuthority {
    kind: AuthorityKind,
    policy: QualifiedName,
    source: String,
    target: String,
    purpose: QualifiedName,
    scope: QualifiedName,
}

/// Provision authorities only at a checked application boundary.
///
/// # Errors
///
/// Returns an error when `actor` lacks authority to provision the capability.
pub fn provision_information_authority(
    actor: EvidenceProducer,
    kind: AuthorityKind,
    policy: QualifiedName,
    source: impl Into<String>,
    target: impl Into<String>,
    purpose: QualifiedName,
    scope: QualifiedName,
) -> Result<InformationAuthority, &'static str> {
    if !matches!(
        actor,
        EvidenceProducer::Checker | EvidenceProducer::CheckedProvider
    ) {
        return Err("ordinary source cannot construct information-flow authority");
    }
    Ok(InformationAuthority {
        kind,
        policy,
        source: source.into(),
        target: target.into(),
        purpose,
        scope,
    })
}

/// Perform an authority-checked label transition while retaining provenance.
///
/// # Errors
///
/// Returns an error when the authority kind, policy, scope, or source label
/// does not match the value and requested transition.
pub fn relabel<T>(
    authority: &InformationAuthority,
    expected_kind: AuthorityKind,
    live_scope: &QualifiedName,
    mut value: Labeled<T>,
) -> Result<Labeled<T>, &'static str> {
    if authority.kind != expected_kind
        || authority.policy != value.policy
        || &authority.scope != live_scope
    {
        return Err("information-flow authority identity or scope does not match");
    }
    let current = match expected_kind {
        AuthorityKind::Declassify => &mut value.label.confidentiality,
        AuthorityKind::Endorse => &mut value.label.integrity,
    };
    if *current != authority.source {
        return Err("information-flow authority source label does not match");
    }
    current.clone_from(&authority.target);
    value.provenance.insert(authority.purpose.clone());
    Ok(value)
}

/// Apply an implicit control-flow label to a produced value.
///
/// # Errors
///
/// Returns an error when either program-counter join is undefined.
pub fn join_program_counter<T>(
    policy: &InformationPolicy,
    mut value: Labeled<T>,
    confidentiality_pc: &str,
    integrity_pc: &str,
) -> Result<Labeled<T>, &'static str> {
    value.label.confidentiality =
        policy.joined(&value.label.confidentiality, confidentiality_pc)?;
    value.label.integrity = policy.joined(&value.label.integrity, integrity_pc)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(value: &str) -> QualifiedName {
        QualifiedName(vec![value.into()])
    }

    fn two_label_policy() -> InformationPolicy {
        let labels: BTreeSet<String> = BTreeSet::from(["Public".into(), "Secret".into()]);
        let flow = BTreeSet::from([
            ("Public".into(), "Public".into()),
            ("Public".into(), "Secret".into()),
            ("Secret".into(), "Secret".into()),
        ]);
        let mut join = BTreeMap::new();
        let mut meet = BTreeMap::new();
        for left in &labels {
            for right in &labels {
                join.insert(
                    (left.clone(), right.clone()),
                    if left == "Secret" || right == "Secret" {
                        "Secret"
                    } else {
                        "Public"
                    }
                    .into(),
                );
                meet.insert(
                    (left.clone(), right.clone()),
                    if left == "Public" || right == "Public" {
                        "Public"
                    } else {
                        "Secret"
                    }
                    .into(),
                );
            }
        }
        InformationPolicy {
            identity: name("Medical"),
            labels,
            can_flow_to: flow,
            join,
            meet,
        }
    }

    #[test]
    fn blocked_layout_validates_and_rejects_out_of_range_coordinates() {
        let layout = CompositeLayout {
            shape: Shape {
                dimensions: vec![
                    ShapeDimension {
                        identity: "height".into(),
                        extent: 3,
                    },
                    ShapeDimension {
                        identity: "width".into(),
                        extent: 4,
                    },
                ],
            },
            dimension_order: vec!["height".into(), "width".into()],
            strides: None,
            blocking: BTreeMap::from([("height".into(), 2), ("width".into(), 2)]),
            component_organization: ComponentOrganization::Separated,
            element_size: 4,
            alignment: 4,
        };
        assert!(layout.validate().is_ok());
        assert_eq!(layout.storage_size(), Some(64));
        assert!(layout.offset(&[2, 3]).is_ok());
        assert!(layout.offset(&[3, 0]).is_err());
        assert!(layout.zero_copy_compatible(&layout));
    }

    #[test]
    fn explicit_stride_validation_is_bounded_and_accounts_for_element_extent() {
        let shape = Shape {
            dimensions: vec![
                ShapeDimension {
                    identity: "x".into(),
                    extent: 2,
                },
                ShapeDimension {
                    identity: "y".into(),
                    extent: 2,
                },
            ],
        };
        let overlapping = CompositeLayout {
            shape: shape.clone(),
            dimension_order: vec!["x".into(), "y".into()],
            strides: Some(BTreeMap::from([("x".into(), 4), ("y".into(), 4)])),
            blocking: BTreeMap::new(),
            component_organization: ComponentOrganization::Interleaved,
            element_size: 4,
            alignment: 4,
        };
        assert!(overlapping.validate().is_err());

        let large = CompositeLayout {
            shape: Shape {
                dimensions: vec![ShapeDimension {
                    identity: "x".into(),
                    extent: u64::MAX,
                }],
            },
            dimension_order: vec!["x".into()],
            strides: Some(BTreeMap::from([("x".into(), 8)])),
            blocking: BTreeMap::new(),
            component_organization: ComponentOrganization::Interleaved,
            element_size: 8,
            alignment: 8,
        };
        assert!(large.validate().is_ok());

        let empty = CompositeLayout {
            shape: Shape {
                dimensions: vec![ShapeDimension {
                    identity: "x".into(),
                    extent: 0,
                }],
            },
            dimension_order: vec!["x".into()],
            strides: None,
            blocking: BTreeMap::new(),
            component_organization: ComponentOrganization::Interleaved,
            element_size: 8,
            alignment: 8,
        };
        assert_eq!(empty.storage_size(), Some(0));
        assert!(empty.validate().is_ok());
    }

    #[test]
    fn sparse_representation_is_canonical_and_duplicate_checked() {
        let shape = Shape {
            dimensions: vec![ShapeDimension {
                identity: "x".into(),
                extent: 4,
            }],
        };
        let entries = vec![
            SparseEntry {
                coordinates: vec![2],
                value: 7,
            },
            SparseEntry {
                coordinates: vec![0],
                value: 0,
            },
            SparseEntry {
                coordinates: vec![1],
                value: 3,
            },
        ];
        let canonical = canonical_sparse(&shape, entries, &0, DuplicatePolicy::Reject).unwrap();
        assert_eq!(
            canonical
                .iter()
                .map(|entry| entry.coordinates[0])
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn policy_checks_lattice_implicit_flow_and_authority() {
        let policy = two_label_policy();
        policy.validate().unwrap();
        let scope = name("request");
        assert!(
            provision_information_authority(
                EvidenceProducer::SourceAuthor,
                AuthorityKind::Declassify,
                policy.identity.clone(),
                "Secret",
                "Public",
                name("audit"),
                scope.clone(),
            )
            .is_err()
        );
        let authority = provision_information_authority(
            EvidenceProducer::CheckedProvider,
            AuthorityKind::Declassify,
            policy.identity.clone(),
            "Secret",
            "Public",
            name("audit"),
            scope.clone(),
        )
        .unwrap();
        let value = Labeled {
            policy: policy.identity.clone(),
            label: InformationLabel {
                confidentiality: "Secret".into(),
                integrity: "Public".into(),
            },
            value: 5,
            provenance: BTreeSet::new(),
        };
        let public = relabel(&authority, AuthorityKind::Declassify, &scope, value).unwrap();
        assert_eq!(public.label.confidentiality, "Public");
        let tainted = join_program_counter(&policy, public, "Secret", "Public").unwrap();
        assert_eq!(tainted.label.confidentiality, "Secret");
    }
}
