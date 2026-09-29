use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArchitectureComponentKind {
    ExecutionArchitecture,
    AbiPlatform,
    Compute,
    Memory,
    Translation,
    Cache,
    Interconnect,
    TransferEngine,
    Channel,
    Device,
    Board,
}

impl ArchitectureComponentKind {
    const fn canonical(self) -> &'static str {
        match self {
            Self::ExecutionArchitecture => "execution-architecture",
            Self::AbiPlatform => "abi-platform",
            Self::Compute => "compute",
            Self::Memory => "memory",
            Self::Translation => "translation",
            Self::Cache => "cache",
            Self::Interconnect => "interconnect",
            Self::TransferEngine => "transfer-engine",
            Self::Channel => "channel",
            Self::Device => "device",
            Self::Board => "board",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchitectureComponent {
    pub identity: String,
    pub kind: ArchitectureComponentKind,
    pub definition: String,
    pub count: u64,
    pub properties: BTreeMap<String, String>,
    pub provenance: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchitectureConnection {
    pub identity: String,
    pub source: String,
    pub destination: String,
    pub kind: String,
    pub resources: BTreeSet<String>,
    pub provenance: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArchitectureCostDimension {
    Latency,
    InitiationInterval,
    Throughput,
    CodeSize,
    StaticData,
    Register,
    Stack,
    LocalMemory,
    PeakLive,
    TransferBytes,
    Energy,
}

impl ArchitectureCostDimension {
    const fn canonical(self) -> &'static str {
        match self {
            Self::Latency => "latency",
            Self::InitiationInterval => "initiation-interval",
            Self::Throughput => "throughput",
            Self::CodeSize => "code-size",
            Self::StaticData => "static-data",
            Self::Register => "register",
            Self::Stack => "stack",
            Self::LocalMemory => "local-memory",
            Self::PeakLive => "peak-live",
            Self::TransferBytes => "transfer-bytes",
            Self::Energy => "energy",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchitectureCostValue {
    Exact(u128),
    Interval { lower: u128, upper: u128 },
    Estimate { value: u128, error: u128 },
    Unknown,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchitectureCost {
    pub identity: String,
    pub subject: String,
    pub dimension: ArchitectureCostDimension,
    pub value: ArchitectureCostValue,
    pub unit: String,
    pub conditions: BTreeMap<String, String>,
    pub resources: BTreeSet<String>,
    pub provenance: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchitectureProvenance {
    pub identity: String,
    pub provider: String,
    pub source_class: String,
    pub revision: String,
    pub digest: String,
    pub assumptions: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchitectureModel {
    pub schema_revision: u64,
    pub language_revision: String,
    pub name: String,
    pub imports: BTreeMap<String, String>,
    pub features: BTreeMap<String, BTreeSet<String>>,
    pub components: Vec<ArchitectureComponent>,
    pub connections: Vec<ArchitectureConnection>,
    pub costs: Vec<ArchitectureCost>,
    pub provenance: Vec<ArchitectureProvenance>,
    pub qualifications: BTreeSet<String>,
}

impl ArchitectureModel {
    /// Validate the target-independent architecture schema.
    ///
    /// # Errors
    ///
    /// Returns the first ordered schema, reference, cost, or provenance error.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_revision != 1 || self.language_revision.is_empty() || self.name.is_empty() {
            return Err("architecture model requires supported revisions and a name");
        }
        let provenance = unique_identities(
            self.provenance.iter().map(|entry| entry.identity.as_str()),
            "architecture provenance identities must be nonempty and unique",
        )?;
        if self.provenance.iter().any(|entry| {
            entry.provider.is_empty()
                || entry.source_class.is_empty()
                || entry.revision.is_empty()
                || entry.digest.is_empty()
        }) {
            return Err("architecture provenance fields must be complete");
        }
        let components = unique_identities(
            self.components.iter().map(|entry| entry.identity.as_str()),
            "architecture component identities must be nonempty and unique",
        )?;
        if self.components.iter().any(|entry| {
            entry.count == 0
                || entry.definition.is_empty()
                || !provenance.contains(entry.provenance.as_str())
        }) {
            return Err("architecture components require count, definition, and provenance");
        }
        let connections = unique_identities(
            self.connections.iter().map(|entry| entry.identity.as_str()),
            "architecture connection identities must be nonempty and unique",
        )?;
        for connection in &self.connections {
            if !components.contains(connection.source.as_str())
                || !components.contains(connection.destination.as_str())
                || connection.kind.is_empty()
                || !provenance.contains(connection.provenance.as_str())
            {
                return Err("architecture connection has a dangling or incomplete reference");
            }
        }
        let resources = components
            .union(&connections)
            .copied()
            .collect::<BTreeSet<_>>();
        unique_identities(
            self.costs.iter().map(|entry| entry.identity.as_str()),
            "architecture cost identities must be nonempty and unique",
        )?;
        for cost in &self.costs {
            if !resources.contains(cost.subject.as_str())
                || cost.unit.is_empty()
                || !provenance.contains(cost.provenance.as_str())
                || !cost
                    .resources
                    .iter()
                    .all(|item| resources.contains(item.as_str()))
            {
                return Err("architecture cost has a dangling or incomplete reference");
            }
            if matches!(cost.value, ArchitectureCostValue::Interval { lower, upper } if lower > upper)
            {
                return Err("architecture cost interval has reversed bounds");
            }
        }
        for (feature, dependencies) in &self.features {
            if feature.is_empty()
                || dependencies.contains(feature)
                || !dependencies
                    .iter()
                    .all(|item| self.features.contains_key(item))
            {
                return Err("architecture feature dependency is invalid");
            }
        }
        Ok(())
    }

    /// Return the normative digest of a valid, canonically ordered model.
    ///
    /// # Errors
    ///
    /// Returns a validation error rather than hashing a partial model.
    pub fn canonical_sha256(&self) -> Result<String, &'static str> {
        self.validate()?;
        let mut fields = Vec::new();
        fields.push(format!("schema={}", self.schema_revision));
        fields.push(format!("language={}", self.language_revision));
        fields.push(format!("name={}", self.name));
        push_map(&mut fields, "import", &self.imports);
        for (feature, dependencies) in &self.features {
            fields.push(format!("feature={feature}:{}", join(dependencies)));
        }
        let mut components = self.components.iter().collect::<Vec<_>>();
        components.sort_by_key(|entry| &entry.identity);
        for entry in components {
            fields.push(format!(
                "component={}:{}:{}:{}:{}:{}",
                entry.identity,
                entry.kind.canonical(),
                entry.definition,
                entry.count,
                map_text(&entry.properties),
                entry.provenance
            ));
        }
        let mut connections = self.connections.iter().collect::<Vec<_>>();
        connections.sort_by_key(|entry| &entry.identity);
        for entry in connections {
            fields.push(format!(
                "connection={}:{}:{}:{}:{}:{}",
                entry.identity,
                entry.source,
                entry.destination,
                entry.kind,
                join(&entry.resources),
                entry.provenance
            ));
        }
        let mut costs = self.costs.iter().collect::<Vec<_>>();
        costs.sort_by_key(|entry| &entry.identity);
        for entry in costs {
            fields.push(format!(
                "cost={}:{}:{}:{}:{}:{}:{}:{}",
                entry.identity,
                entry.subject,
                entry.dimension.canonical(),
                cost_value_text(&entry.value),
                entry.unit,
                map_text(&entry.conditions),
                join(&entry.resources),
                entry.provenance
            ));
        }
        let mut provenance = self.provenance.iter().collect::<Vec<_>>();
        provenance.sort_by_key(|entry| &entry.identity);
        for entry in provenance {
            fields.push(format!(
                "provenance={}:{}:{}:{}:{}:{}",
                entry.identity,
                entry.provider,
                entry.source_class,
                entry.revision,
                entry.digest,
                join(&entry.assumptions)
            ));
        }
        fields.push(format!("qualification={}", join(&self.qualifications)));
        let mut hasher = Sha256::new();
        for field in fields {
            hasher.update(field.len().to_le_bytes());
            hasher.update(field.as_bytes());
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}

fn unique_identities<'a>(
    values: impl Iterator<Item = &'a str>,
    error: &'static str,
) -> Result<BTreeSet<&'a str>, &'static str> {
    let mut result = BTreeSet::new();
    for value in values {
        if value.is_empty() || !result.insert(value) {
            return Err(error);
        }
    }
    Ok(result)
}

fn join(values: &BTreeSet<String>) -> String {
    values.iter().cloned().collect::<Vec<_>>().join(",")
}

fn map_text(values: &BTreeMap<String, String>) -> String {
    values
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn cost_value_text(value: &ArchitectureCostValue) -> String {
    match value {
        ArchitectureCostValue::Exact(value) => format!("exact:{value}"),
        ArchitectureCostValue::Interval { lower, upper } => {
            format!("interval:{lower}:{upper}")
        }
        ArchitectureCostValue::Estimate { value, error } => {
            format!("estimate:{value}:{error}")
        }
        ArchitectureCostValue::Unknown => "unknown".into(),
        ArchitectureCostValue::Unsupported => "unsupported".into(),
    }
}

fn push_map(fields: &mut Vec<String>, prefix: &str, values: &BTreeMap<String, String>) {
    fields.extend(
        values
            .iter()
            .map(|(key, value)| format!("{prefix}={key}:{value}")),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> ArchitectureModel {
        ArchitectureModel {
            schema_revision: 1,
            language_revision: "v0.1".into(),
            name: "test".into(),
            imports: BTreeMap::new(),
            features: BTreeMap::from([("baseline".into(), BTreeSet::new())]),
            components: vec![ArchitectureComponent {
                identity: "cpu".into(),
                kind: ArchitectureComponentKind::Compute,
                definition: "generic".into(),
                count: 1,
                properties: BTreeMap::new(),
                provenance: "source".into(),
            }],
            connections: Vec::new(),
            costs: vec![ArchitectureCost {
                identity: "add-latency".into(),
                subject: "cpu".into(),
                dimension: ArchitectureCostDimension::Latency,
                value: ArchitectureCostValue::Interval { lower: 1, upper: 2 },
                unit: "cycle".into(),
                conditions: BTreeMap::new(),
                resources: BTreeSet::from(["cpu".into()]),
                provenance: "source".into(),
            }],
            provenance: vec![ArchitectureProvenance {
                identity: "source".into(),
                provider: "test".into(),
                source_class: "specification".into(),
                revision: "1".into(),
                digest: "00".repeat(32),
                assumptions: BTreeSet::new(),
            }],
            qualifications: BTreeSet::from(["schema".into()]),
        }
    }

    #[test]
    fn canonical_identity_ignores_input_vector_order() {
        let mut first = model();
        first.components.push(ArchitectureComponent {
            identity: "memory".into(),
            kind: ArchitectureComponentKind::Memory,
            definition: "ram".into(),
            count: 1,
            properties: BTreeMap::new(),
            provenance: "source".into(),
        });
        let mut second = first.clone();
        second.components.reverse();
        assert_eq!(first.canonical_sha256(), second.canonical_sha256());
    }

    #[test]
    fn rejects_dangling_connections_and_reversed_costs() {
        let mut dangling = model();
        dangling.connections.push(ArchitectureConnection {
            identity: "bad".into(),
            source: "cpu".into(),
            destination: "missing".into(),
            kind: "load-store".into(),
            resources: BTreeSet::new(),
            provenance: "source".into(),
        });
        assert!(dangling.validate().is_err());

        let mut reversed = model();
        reversed.costs[0].value = ArchitectureCostValue::Interval { lower: 3, upper: 2 };
        assert!(reversed.validate().is_err());
    }

    #[test]
    fn rejects_missing_feature_dependencies() {
        let mut invalid = model();
        invalid
            .features
            .insert("avx2".into(), BTreeSet::from(["avx".into()]));
        assert!(invalid.validate().is_err());
    }
}
