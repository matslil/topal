impl Analyzer {
    fn native_serialization_bytes(
        &self,
        version: LanguageVersion,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Vec<u8>, Diagnostic> {
        let mut types = Vec::new();
        let mut identities = BTreeMap::new();
        let (type_id, serialized) =
            self.native_serialized_value(value, environment, &mut types, &mut identities, true)?;
        let stream = SerializationStream {
            header: SerializationHeader {
                language_identity: "topal".into(),
                language_version: version,
                byte_order: StreamByteOrder::Little,
                streaming: false,
            },
            types,
            events: vec![SerializedEvent {
                type_id,
                value: serialized,
            }],
        };
        let bytes = serialize_native(&stream).map_err(|error| {
            source_diagnostic(
                &self.source,
                "E-SERIALIZATION",
                value.span,
                format!("native value cannot be serialized: {}", error.message),
            )
        })?;
        deserialize_native(&bytes, SerializationLimits::default()).map_err(|error| {
            source_diagnostic(
                &self.source,
                "E-SERIALIZATION",
                value.span,
                format!(
                    "compiler-generated native stream failed validation at {} byte {}: {}",
                    error.stage, error.offset, error.message
                ),
            )
        })?;
        Ok(bytes)
    }

    #[allow(clippy::too_many_lines)] // Supported schemas remain explicit beside their canonical values.
    fn native_serialized_value(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        types: &mut Vec<TypeDefinition>,
        identities: &mut BTreeMap<String, usize>,
        aggregate_allowed: bool,
    ) -> Result<(usize, SerializedValue), Diagnostic> {
        let (identity, definition, serialized) = match &value.value_type {
            CompilerType::Unit if matches!(value.kind, CompilerExpressionKind::Unit) => (
                "Unit".to_owned(),
                TypeDefinition::Unit {
                    identity: "Unit".into(),
                },
                SerializedValue::Unit,
            ),
            CompilerType::Boolean => {
                let CompilerExpressionKind::Boolean(boolean) = value.kind else {
                    return Err(unsupported(
                        &self.source,
                        value.span,
                        "native serialization of a dynamic Boolean",
                    ));
                };
                (
                    "Boolean".to_owned(),
                    TypeDefinition::Boolean {
                        identity: "Boolean".into(),
                    },
                    SerializedValue::Boolean(boolean),
                )
            }
            CompilerType::Int => {
                let integer = exact_int(value).ok_or_else(|| {
                    unsupported(
                        &self.source,
                        value.span,
                        "native serialization of a dynamic Int",
                    )
                })?;
                (
                    "Int".to_owned(),
                    TypeDefinition::Int {
                        identity: "Int".into(),
                        signed: true,
                        width_bits: 0,
                    },
                    SerializedValue::ArbitraryInt(integer),
                )
            }
            CompilerType::String => {
                let text = Self::known_string_value(value, environment).ok_or_else(|| {
                    unsupported(
                        &self.source,
                        value.span,
                        "native serialization of a dynamic String",
                    )
                })?;
                (
                    "String".to_owned(),
                    TypeDefinition::Text {
                        identity: "String".into(),
                    },
                    SerializedValue::Text(text),
                )
            }
            CompilerType::Tuple(_) if aggregate_allowed => {
                let CompilerExpressionKind::Tuple(fields) = &value.kind else {
                    return Err(unsupported(
                        &self.source,
                        value.span,
                        "native serialization of a dynamic Tuple",
                    ));
                };
                let mut components = Vec::with_capacity(fields.len());
                let mut encoded = Vec::with_capacity(fields.len());
                for field in fields {
                    let (id, field) =
                        self.native_serialized_value(field, environment, types, identities, false)?;
                    components.push(id);
                    encoded.push(field);
                }
                let identity = value.value_type.name();
                (
                    identity.clone(),
                    TypeDefinition::Tuple {
                        identity,
                        components,
                    },
                    SerializedValue::Product(encoded),
                )
            }
            CompilerType::Record(_) if aggregate_allowed => {
                let CompilerExpressionKind::Record(fields) = &value.kind else {
                    return Err(unsupported(
                        &self.source,
                        value.span,
                        "native serialization of a dynamic Record",
                    ));
                };
                let mut definitions = Vec::with_capacity(fields.len());
                let mut encoded = Vec::with_capacity(fields.len());
                for (label, field) in fields {
                    let (id, field) =
                        self.native_serialized_value(field, environment, types, identities, false)?;
                    definitions.push((label.clone(), id));
                    encoded.push(field);
                }
                (
                    "Record".to_owned(),
                    TypeDefinition::Record {
                        identity: "Record".into(),
                        fields: definitions,
                    },
                    SerializedValue::Product(encoded),
                )
            }
            _ => {
                return Err(source_diagnostic(
                    &self.source,
                    "E-COMPILER-UNSUPPORTED",
                    value.span,
                    format!(
                        "compiler increment does not support native serialization of {}",
                        value.value_type.name()
                    ),
                ));
            }
        };
        if let Some(id) = identities.get(&identity) {
            return Ok((*id, serialized));
        }
        let id = types.len();
        types.push(definition);
        identities.insert(identity, id);
        Ok((id, serialized))
    }

    #[allow(clippy::too_many_lines)] // The closed external-storage subset is validated in schema order.
    fn analyze_external_storage(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let [Expression::Identifier(operation), location] = items
            && self.source.slice(*operation) == "read"
            && let Expression::Identifier(location_name) = location
            && let Some(metadata) = self
                .external_locations
                .get(self.source.slice(*location_name))
                .cloned()
        {
            if matches!(
                metadata.location_type.layout.access.as_str(),
                "WriteOnly" | "Reserved"
            ) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-LAYOUT-NOT-READABLE",
                    span,
                    "location layout does not permit reads",
                ));
            }
            let location = self.analyze_expression(location, environment)?;
            require_same_type(
                &self.source,
                location.span,
                &CompilerType::ExternalLocation(Box::new(metadata.location_type.clone())),
                &location.value_type,
            )?;
            let value_type = external_layout_value_type(&metadata.location_type.layout)
                .ok_or_else(|| unsupported(&self.source, span, "runtime external layout family"))?;
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::ExternalLocationRead {
                    location: Box::new(location),
                    metadata,
                },
                value_type,
                int_range: None,
                rational_value: None,
                span,
            }));
        }
        if let [location, Expression::Identifier(operation), value] = items
            && self.source.slice(*operation) == "write"
            && let Expression::Identifier(location_name) = location
            && let Some(metadata) = self
                .external_locations
                .get(self.source.slice(*location_name))
                .cloned()
        {
            if matches!(
                metadata.location_type.layout.access.as_str(),
                "ReadOnly" | "Reserved"
            ) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-LAYOUT-NOT-WRITABLE",
                    span,
                    "location layout does not permit writes",
                ));
            }
            let location = self.analyze_expression(location, environment)?;
            require_same_type(
                &self.source,
                location.span,
                &CompilerType::ExternalLocation(Box::new(metadata.location_type.clone())),
                &location.value_type,
            )?;
            let value = self.analyze_expression(value, environment)?;
            let expected = external_layout_value_type(&metadata.location_type.layout)
                .ok_or_else(|| unsupported(&self.source, span, "runtime external layout family"))?;
            require_same_type(&self.source, value.span, &expected, &value.value_type)?;
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::ExternalLocationWrite {
                    location: Box::new(location),
                    value: Box::new(value),
                    metadata,
                },
                value_type: CompilerType::Unit,
                int_range: None,
                rational_value: None,
                span,
            }));
        }

        if let [
            attributes,
            Expression::Identifier(constructor),
            semantic @ ..,
        ] = items
            && self.source.slice(*constructor) == "Layout"
            && !semantic.is_empty()
        {
            let layout = self.analyze_external_layout(attributes, semantic, span)?;
            return Ok(Some(external_metadata_expression(
                CompilerExternalMetadata::Layout(layout),
                span,
            )));
        }
        if let [Expression::Identifier(constructor), attributes] = items
            && self.source.slice(*constructor) == "AddressRange"
        {
            let fields = external_record_fields(&self.source, attributes, "AddressRange")?;
            require_external_fields(
                &self.source,
                span,
                &fields,
                &["caching", "medium", "minimum-access-size"],
                "AddressRange",
            )?;
            let caching = external_identifier_field(&self.source, &fields, "caching")?;
            let minimum_access_size_bits =
                external_size_field(&self.source, &fields, "minimum-access-size")?;
            let medium = external_identifier_field(&self.source, &fields, "medium")?;
            if !matches!(caching.as_str(), "Cached" | "Uncached")
                || !matches!(medium.as_str(), "Memory" | "MMIO")
                || minimum_access_size_bits == 0
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "closed AddressRange attribute values",
                ));
            }
            return Ok(Some(external_metadata_expression(
                CompilerExternalMetadata::AddressRangeType(CompilerAddressRangeType {
                    identity: String::new(),
                    caching,
                    minimum_access_size_bits,
                    medium,
                }),
                span,
            )));
        }
        if let [Expression::Identifier(constructor), attributes] = items
            && self.source.slice(*constructor) == "AddressOffset"
        {
            let fields = external_record_fields(&self.source, attributes, "AddressOffset")?;
            require_external_fields(
                &self.source,
                span,
                &fields,
                &["alignment", "range"],
                "AddressOffset",
            )?;
            let range_name = external_identifier_field(&self.source, &fields, "range")?;
            let Some(CompilerExternalMetadata::AddressRange(range)) =
                self.external_metadata.get(&range_name)
            else {
                return Err(unsupported(
                    &self.source,
                    span,
                    "AddressOffset without one retained AddressRange value",
                ));
            };
            let alignment = external_nat_field(&self.source, &fields, "alignment")?;
            let alignment_bytes = u64::try_from(alignment)
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| unsupported(&self.source, span, "positive byte alignment"))?;
            return Ok(Some(external_metadata_expression(
                CompilerExternalMetadata::AddressOffsetType(CompilerAddressOffsetType {
                    identity: String::new(),
                    range: range.clone(),
                    alignment_bytes,
                }),
                span,
            )));
        }
        if let [
            Expression::Identifier(constructor),
            Expression::Identifier(layout_name),
        ] = items
            && self.source.slice(*constructor) == "Location"
        {
            let layout_name = self.source.slice(*layout_name);
            let Some(CompilerExternalMetadata::Layout(layout)) =
                self.external_metadata.get(layout_name)
            else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-LOCATION-LAYOUT",
                    span,
                    "Location requires an explicit Layout value",
                ));
            };
            return Ok(Some(external_metadata_expression(
                CompilerExternalMetadata::LocationType(CompilerLocationType {
                    identity: String::new(),
                    layout: layout.clone(),
                }),
                span,
            )));
        }

        let [Expression::Identifier(name), argument] = items else {
            return Ok(None);
        };
        let name = self.source.slice(*name);
        let Some(metadata) = self.external_metadata.get(name).cloned() else {
            return Ok(None);
        };
        match metadata {
            CompilerExternalMetadata::Layout(layout) => {
                let Expression::Integer(literal) = argument else {
                    return Err(unsupported(
                        &self.source,
                        argument.span(),
                        "dynamic external-layout conversion",
                    ));
                };
                if !matches!(layout.family, CompilerExternalLayoutFamily::UnsignedNat) {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "runtime external layout family",
                    ));
                }
                let integer = parse_integer(self.source.slice(*literal)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NUMERIC-LITERAL",
                        *literal,
                        "invalid integer literal",
                    )
                })?;
                let limit = BigInt::from(1_u8) << layout.storage_size_bits;
                if integer < BigInt::from(0) || integer >= limit {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-LAYOUT-NOT-REPRESENTABLE",
                        *literal,
                        "integer is not representable by the selected layout",
                    ));
                }
                let value = CompilerExpression {
                    kind: CompilerExpressionKind::Int(integer.clone()),
                    value_type: CompilerType::Nat,
                    int_range: Some(IntRange::exact(integer)),
                    rational_value: None,
                    span: argument.span(),
                };
                let value_type = external_layout_value_type(&layout)
                    .expect("closed runtime layout has a semantic value type");
                Ok(Some(CompilerExpression {
                    kind: CompilerExpressionKind::ExternalLayoutCoerce {
                        layout,
                        value: Box::new(value),
                    },
                    value_type,
                    int_range: None,
                    rational_value: None,
                    span,
                }))
            }
            CompilerExternalMetadata::AddressRangeType(range_type) => {
                let (lower, upper) = external_inclusive_range(&self.source, argument)?;
                if lower < BigInt::from(0) || upper < lower {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ADDRESS-RANGE",
                        argument.span(),
                        "AddressRange requires an ordered nonnegative Nat range",
                    ));
                }
                Ok(Some(external_metadata_expression(
                    CompilerExternalMetadata::AddressRange(CompilerAddressRange {
                        identity: String::new(),
                        range_type,
                        lower,
                        upper,
                    }),
                    span,
                )))
            }
            CompilerExternalMetadata::AddressOffsetType(offset_type) => {
                let Expression::Integer(literal) = argument else {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ADDRESS-OFFSET",
                        argument.span(),
                        "AddressOffset requires a Nat byte offset",
                    ));
                };
                let offset = parse_integer(self.source.slice(*literal)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-ADDRESS-OFFSET",
                        *literal,
                        "AddressOffset requires a Nat byte offset",
                    )
                })?;
                if offset < BigInt::from(0)
                    || &offset % offset_type.alignment_bytes != BigInt::from(0_u8)
                {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ADDRESS-OFFSET-ALIGNMENT",
                        *literal,
                        "address offset does not satisfy its byte alignment",
                    ));
                }
                if offset > offset_type.range.upper.clone() - &offset_type.range.lower {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ADDRESS-OFFSET-RANGE",
                        *literal,
                        "address offset lies outside its associated range",
                    ));
                }
                Ok(Some(external_metadata_expression(
                    CompilerExternalMetadata::AddressOffset(CompilerAddressOffset {
                        identity: String::new(),
                        offset_type,
                        offset,
                    }),
                    span,
                )))
            }
            CompilerExternalMetadata::LocationType(location_type) => {
                let Expression::Identifier(offset_name) = argument else {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-LOCATION-OFFSET",
                        argument.span(),
                        "a location requires an AddressOffset value",
                    ));
                };
                let Some(CompilerExternalMetadata::AddressOffset(offset)) = self
                    .external_metadata
                    .get(self.source.slice(*offset_name))
                    .cloned()
                else {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-LOCATION-OFFSET",
                        argument.span(),
                        "a location requires an AddressOffset value",
                    ));
                };
                validate_compiler_location(&self.source, argument.span(), &location_type, &offset)?;
                let location = CompilerLocation {
                    location_type: location_type.clone(),
                    offset,
                };
                Ok(Some(CompilerExpression {
                    kind: CompilerExpressionKind::ExternalLocationConstruct(location),
                    value_type: CompilerType::ExternalLocation(Box::new(location_type)),
                    int_range: None,
                    rational_value: None,
                    span,
                }))
            }
            CompilerExternalMetadata::AddressRange(_)
            | CompilerExternalMetadata::AddressOffset(_) => Err(unsupported(
                &self.source,
                span,
                "application of an external-storage value",
            )),
        }
    }

    #[allow(clippy::too_many_lines)] // Every admitted layout family keeps its closed schema visible.
    fn analyze_external_layout(
        &self,
        attributes: &Expression,
        semantic: &[Expression],
        span: Span,
    ) -> Result<CompilerExternalLayout, Diagnostic> {
        let fields = external_record_fields(&self.source, attributes, "Layout")?;
        let semantic_span = Span::new(
            semantic
                .first()
                .expect("checked nonempty semantic")
                .span()
                .start,
            semantic
                .last()
                .expect("checked nonempty semantic")
                .span()
                .end,
        );
        let semantic_name = compact_classifier(self.source.slice(semantic_span));
        let mut layout = CompilerExternalLayout {
            identity: String::new(),
            semantic: semantic_name.clone(),
            storage_size_bits: 0,
            encoding: None,
            endian: None,
            access: "ReadWrite".into(),
            alignment_bytes: 1,
            family: CompilerExternalLayoutFamily::UnsignedNat,
        };
        match semantic_name.as_str() {
            "Nat" => {
                require_external_fields(
                    &self.source,
                    span,
                    &fields,
                    &["access", "encoding", "endian", "storage-size"],
                    "Layout Nat",
                )?;
                layout.storage_size_bits =
                    external_size_field(&self.source, &fields, "storage-size")?;
                layout.encoding = Some(external_identifier_field(
                    &self.source,
                    &fields,
                    "encoding",
                )?);
                layout.endian = Some(external_identifier_field(&self.source, &fields, "endian")?);
                layout.access = external_identifier_field(&self.source, &fields, "access")?;
                layout.alignment_bytes = 4;
                if layout.storage_size_bits != 32
                    || layout.encoding.as_deref() != Some("UnsignedBinary")
                    || layout.endian.as_deref() != Some("Little")
                    || layout.access != "ReadWrite"
                {
                    return Err(unsupported(&self.source, span, "closed UInt32LE layout"));
                }
            }
            "String" => {
                require_external_fields(
                    &self.source,
                    span,
                    &fields,
                    &["encoding", "length", "storage-size", "termination"],
                    "Layout String",
                )?;
                layout.storage_size_bits =
                    external_size_field(&self.source, &fields, "storage-size")?;
                layout.encoding = Some(external_identifier_field(
                    &self.source,
                    &fields,
                    "encoding",
                )?);
                let length = external_identifier_field(&self.source, &fields, "length")?;
                let termination = external_identifier_field(&self.source, &fields, "termination")?;
                layout.family = CompilerExternalLayoutFamily::Utf8Text;
                if layout.storage_size_bits != 64
                    || layout.encoding.as_deref() != Some("Utf8")
                    || length != "NoLength"
                    || termination != "NoTerminator"
                {
                    return Err(unsupported(&self.source, span, "closed Utf8Text layout"));
                }
            }
            "OptionalNat" => {
                require_external_fields(
                    &self.source,
                    span,
                    &fields,
                    &[
                        "encoding",
                        "payload-placement",
                        "storage-size",
                        "tag-layout",
                        "tags",
                    ],
                    "Layout Optional Nat",
                )?;
                let tag_layout = external_identifier_field(&self.source, &fields, "tag-layout")?;
                self.require_external_nat_layout(&tag_layout, span)?;
                let tags_expression = fields.get("tags").expect("required tags field");
                let tags = external_record_fields(&self.source, tags_expression, "layout tags")?;
                require_external_fields(
                    &self.source,
                    tags_expression.span(),
                    &tags,
                    &["none", "some"],
                    "layout tags",
                )?;
                let none = external_nat_field(&self.source, &tags, "none")?;
                let some = external_nat_field(&self.source, &tags, "some")?;
                let none = u64::try_from(none)
                    .map_err(|_| unsupported(&self.source, span, "finite layout tag"))?;
                let some = u64::try_from(some)
                    .map_err(|_| unsupported(&self.source, span, "finite layout tag"))?;
                let payload_placement =
                    external_identifier_field(&self.source, &fields, "payload-placement")?;
                layout.storage_size_bits =
                    external_size_field(&self.source, &fields, "storage-size")?;
                layout.encoding = Some(external_identifier_field(
                    &self.source,
                    &fields,
                    "encoding",
                )?);
                layout.alignment_bytes = 4;
                layout.family = CompilerExternalLayoutFamily::TaggedOptionalNat {
                    tag_layout,
                    tags: vec![("none".into(), none), ("some".into(), some)],
                    payload_placement: payload_placement.clone(),
                };
                if layout.storage_size_bits != 64
                    || layout.encoding.as_deref() != Some("Tagged")
                    || payload_placement != "AfterTag"
                    || !matches!(layout.family, CompilerExternalLayoutFamily::TaggedOptionalNat { ref tags, .. }
                        if tags == &[("none".into(), 0), ("some".into(), 1)])
                {
                    return Err(unsupported(&self.source, span, "closed MaybeNatLayout"));
                }
            }
            "Array2Nat" => {
                require_external_fields(
                    &self.source,
                    span,
                    &fields,
                    &["element-layout", "storage-size", "stride"],
                    "Layout Array 2 Nat",
                )?;
                let element_layout =
                    external_identifier_field(&self.source, &fields, "element-layout")?;
                self.require_external_nat_layout(&element_layout, span)?;
                let stride_bits = external_size_field(&self.source, &fields, "stride")?;
                layout.storage_size_bits =
                    external_size_field(&self.source, &fields, "storage-size")?;
                layout.alignment_bytes = 4;
                layout.family = CompilerExternalLayoutFamily::NatArray {
                    count: 2,
                    element_layout,
                    stride_bits,
                };
                if layout.storage_size_bits != 64 || stride_bits != 32 {
                    return Err(unsupported(&self.source, span, "closed PairArrayLayout"));
                }
            }
            _ if matches!(semantic, [Expression::Product { .. }]) => {
                require_external_fields(
                    &self.source,
                    span,
                    &fields,
                    &["packing"],
                    "product Layout",
                )?;
                let packing = external_identifier_field(&self.source, &fields, "packing")?;
                let [
                    Expression::Product {
                        fields: components, ..
                    },
                ] = semantic
                else {
                    unreachable!("guard established one product semantic")
                };
                let mut retained = Vec::new();
                for component in components {
                    let Some(label) = component.label else {
                        return Err(unsupported(&self.source, span, "positional product layout"));
                    };
                    let Expression::Identifier(layout_name) = &component.value else {
                        return Err(unsupported(
                            &self.source,
                            span,
                            "expanded product layout field",
                        ));
                    };
                    let layout_name = self.source.slice(*layout_name).to_owned();
                    self.require_external_nat_layout(&layout_name, component.value.span())?;
                    retained.push((self.source.slice(label).to_owned(), layout_name));
                }
                layout.semantic = "(first : Nat, second : Nat)".into();
                layout.storage_size_bits = u64::try_from(retained.len()).unwrap_or(u64::MAX) * 32;
                layout.alignment_bytes = 4;
                layout.family = CompilerExternalLayoutFamily::Product {
                    fields: retained,
                    packing: packing.clone(),
                };
                if packing != "Natural"
                    || layout.storage_size_bits != 64
                    || !matches!(
                        &layout.family,
                        CompilerExternalLayoutFamily::Product { fields, .. }
                            if fields.iter().map(|(label, _)| label.as_str()).eq(["first", "second"])
                    )
                {
                    return Err(unsupported(&self.source, span, "closed HeaderLayout"));
                }
            }
            _ => {
                return Err(unsupported(
                    &self.source,
                    span,
                    "external layout semantic family",
                ));
            }
        }
        Ok(layout)
    }

    fn require_external_nat_layout(&self, name: &str, span: Span) -> Result<(), Diagnostic> {
        if matches!(
            self.external_metadata.get(name),
            Some(CompilerExternalMetadata::Layout(CompilerExternalLayout {
                family: CompilerExternalLayoutFamily::UnsignedNat,
                ..
            }))
        ) {
            Ok(())
        } else {
            Err(unsupported(
                &self.source,
                span,
                "external Nat component layout",
            ))
        }
    }
}
