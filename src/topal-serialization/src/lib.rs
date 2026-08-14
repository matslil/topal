//! Canonical, authority-free Topal native serialization protocol.

use std::fmt;

use num_bigint::{BigInt, Sign};
use topal_semantics::LanguageVersion;
use topal_source::is_nfc;

const MAGIC: &[u8; 8] = b"TOPALSER";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamByteOrder {
    Little,
    Big,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
    pub language_identity: String,
    pub language_version: LanguageVersion,
    pub byte_order: StreamByteOrder,
    pub streaming: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeDefinition {
    Unit {
        identity: String,
    },
    Boolean {
        identity: String,
    },
    Int {
        identity: String,
        signed: bool,
        width_bits: u64,
    },
    Text {
        identity: String,
    },
    Tuple {
        identity: String,
        components: Vec<usize>,
    },
    Record {
        identity: String,
        fields: Vec<(String, usize)>,
    },
    Variant {
        identity: String,
        alternatives: Vec<(String, usize)>,
    },
    Sequence {
        identity: String,
        element: usize,
    },
    ObjectDescription {
        identity: String,
        kind: u8,
        schema_payload: Vec<u8>,
    },
}

impl TypeDefinition {
    fn identity(&self) -> &str {
        match self {
            Self::Unit { identity }
            | Self::Boolean { identity }
            | Self::Int { identity, .. }
            | Self::Text { identity }
            | Self::Tuple { identity, .. }
            | Self::Record { identity, .. }
            | Self::Variant { identity, .. }
            | Self::Sequence { identity, .. }
            | Self::ObjectDescription { identity, .. } => identity,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SerializedValue {
    Unit,
    Boolean(bool),
    Int(i128),
    ArbitraryInt(BigInt),
    Text(String),
    Bytes(Vec<u8>),
    Product(Vec<Self>),
    Variant {
        alternative: usize,
        value: Box<Self>,
    },
    Sequence(Vec<Self>),
    ObjectDescription(Vec<Self>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub type_id: usize,
    pub value: SerializedValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    pub header: Header,
    pub types: Vec<TypeDefinition>,
    pub events: Vec<Event>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Malformed,
    Unsupported,
    ResourceLimit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolError {
    pub kind: ErrorKind,
    pub stage: &'static str,
    pub offset: usize,
    pub message: &'static str,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at byte {}: {}",
            self.stage, self.offset, self.message
        )
    }
}

impl std::error::Error for ProtocolError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub types: usize,
    pub events: usize,
    pub frame_bytes: usize,
    pub text_bytes: usize,
    pub nesting_depth: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            types: 4_096,
            events: 1_000_000,
            frame_bytes: 16 * 1_024 * 1_024,
            text_bytes: 1_024 * 1_024,
            nesting_depth: 256,
        }
    }
}

/// Serialize a validated stream into canonical protocol 1.0 bytes.
///
/// # Errors
///
/// Returns a source-stage protocol error when types or values are invalid.
pub fn serialize(stream: &Stream) -> Result<Vec<u8>, ProtocolError> {
    validate_types(&stream.types, 0)?;
    let mut output = MAGIC.to_vec();
    put_uvarint(1, &mut output);
    put_uvarint(0, &mut output);
    put_text(&stream.header.language_identity, &mut output)?;
    for component in [
        stream.header.language_version.major,
        stream.header.language_version.minor,
        stream.header.language_version.patch,
        stream.header.language_version.build,
    ] {
        put_uvarint(component, &mut output);
    }
    output.push(match stream.header.byte_order {
        StreamByteOrder::Little => 0,
        StreamByteOrder::Big => 1,
    });
    put_uvarint(0, &mut output);
    put_uvarint(stream.types.len() as u64, &mut output);
    put_uvarint(
        if stream.header.streaming {
            u64::MAX
        } else {
            stream.events.len() as u64
        },
        &mut output,
    );
    for definition in &stream.types {
        encode_type(definition, &mut output)?;
    }
    for event in &stream.events {
        let definition = stream.types.get(event.type_id).ok_or_else(|| {
            error(
                ErrorKind::Malformed,
                "event",
                output.len(),
                "event references an unknown type",
            )
        })?;
        let mut frame = Vec::new();
        put_uvarint(event.type_id as u64, &mut frame);
        encode_value(
            &event.value,
            definition,
            &stream.types,
            stream.header.byte_order,
            &mut frame,
        )?;
        put_uvarint(frame.len() as u64, &mut output);
        output.extend(frame);
    }
    if stream.header.streaming {
        output.push(0);
    }
    Ok(output)
}

/// Validate and deserialize one complete protocol 1.0 stream.
///
/// # Errors
///
/// Returns a deterministic stage, byte offset, and error category before
/// exposing an invalid event or exceeding a configured resource limit.
#[allow(clippy::too_many_lines)] // The protocol stages remain in normative wire order.
pub fn deserialize(bytes: &[u8], limits: Limits) -> Result<Stream, ProtocolError> {
    let mut reader = Reader { bytes, offset: 0 };
    if reader.take(8, "header")? != MAGIC {
        return Err(reader.failure(ErrorKind::Malformed, "header", "invalid stream magic"));
    }
    let major = reader.uvarint("header")?;
    let minor = reader.uvarint("header")?;
    if major != 1 || minor != 0 {
        return Err(reader.failure(
            ErrorKind::Unsupported,
            "header",
            "unsupported protocol version",
        ));
    }
    let language_identity = reader.text("header", limits.text_bytes)?;
    let language_version = LanguageVersion {
        major: reader.uvarint("header")?,
        minor: reader.uvarint("header")?,
        patch: reader.uvarint("header")?,
        build: reader.uvarint("header")?,
    };
    if language_version != LanguageVersion::DESIGN_0 {
        return Err(reader.failure(
            ErrorKind::Unsupported,
            "header",
            "unsupported language version",
        ));
    }
    let byte_order = match reader.byte("header")? {
        0 => StreamByteOrder::Little,
        1 => StreamByteOrder::Big,
        _ => return Err(reader.failure(ErrorKind::Malformed, "header", "invalid byte order")),
    };
    if reader.uvarint("header")? != 0 {
        return Err(reader.failure(ErrorKind::Unsupported, "header", "unknown header flags"));
    }
    let type_count = reader.count("header", limits.types)?;
    let declared_events = reader.uvarint("header")?;
    let streaming = declared_events == u64::MAX;
    let event_count = if streaming {
        0
    } else {
        usize::try_from(declared_events).map_err(|_| {
            reader.failure(
                ErrorKind::ResourceLimit,
                "header",
                "event count exceeds host limits",
            )
        })?
    };
    if !streaming && event_count > limits.events {
        return Err(reader.failure(
            ErrorKind::ResourceLimit,
            "header",
            "declared event count exceeds configured limit",
        ));
    }
    let mut types = Vec::with_capacity(type_count);
    for _ in 0..type_count {
        types.push(reader.type_definition(&types, limits.text_bytes)?);
    }
    validate_types(&types, reader.offset)?;
    let mut events = Vec::with_capacity(event_count);
    loop {
        if (!streaming && events.len() == event_count)
            || (streaming && reader.bytes.get(reader.offset) == Some(&0))
        {
            if streaming {
                reader.offset += 1;
            }
            break;
        }
        if events.len() >= limits.events {
            return Err(reader.failure(
                ErrorKind::ResourceLimit,
                "event",
                "event count exceeds configured limit",
            ));
        }
        let frame_length = reader.count("event", limits.frame_bytes)?;
        let frame_offset = reader.offset;
        let frame = reader.take(frame_length, "event")?;
        let mut frame_reader = Reader {
            bytes: frame,
            offset: 0,
        };
        let type_id = frame_reader.count("event", types.len().saturating_sub(1))?;
        let definition = types.get(type_id).ok_or_else(|| {
            error(
                ErrorKind::Malformed,
                "event",
                frame_offset,
                "event references an unknown type",
            )
        })?;
        let value = frame_reader.value(
            definition,
            &types,
            byte_order,
            limits.text_bytes,
            0,
            limits.nesting_depth,
        )?;
        if frame_reader.offset != frame.len() {
            return Err(error(
                ErrorKind::Malformed,
                "event",
                frame_offset + frame_reader.offset,
                "event has trailing bytes",
            ));
        }
        events.push(Event { type_id, value });
    }
    if reader.offset != bytes.len() {
        return Err(reader.failure(ErrorKind::Malformed, "stream", "trailing stream bytes"));
    }
    Ok(Stream {
        header: Header {
            language_identity,
            language_version,
            byte_order,
            streaming,
        },
        types,
        events,
    })
}

fn encode_type(definition: &TypeDefinition, output: &mut Vec<u8>) -> Result<(), ProtocolError> {
    put_text(definition.identity(), output)?;
    let mut payload = Vec::new();
    let kind = match definition {
        TypeDefinition::Unit { .. } => 0,
        TypeDefinition::Boolean { .. } => 1,
        TypeDefinition::Int {
            signed, width_bits, ..
        } => {
            payload.push(u8::from(*signed));
            put_uvarint(*width_bits, &mut payload);
            2
        }
        TypeDefinition::Text { .. } => {
            payload.push(0);
            4
        }
        TypeDefinition::Tuple { components, .. } => {
            put_ids(components, &mut payload);
            6
        }
        TypeDefinition::Record { fields, .. } => {
            put_uvarint(fields.len() as u64, &mut payload);
            for (label, id) in fields {
                put_text(label, &mut payload)?;
                put_uvarint(*id as u64, &mut payload);
            }
            7
        }
        TypeDefinition::Variant { alternatives, .. } => {
            put_uvarint(alternatives.len() as u64, &mut payload);
            for (tag, (label, id)) in alternatives.iter().enumerate() {
                put_uvarint(tag as u64, &mut payload);
                put_text(label, &mut payload)?;
                put_uvarint(*id as u64, &mut payload);
            }
            8
        }
        TypeDefinition::Sequence { element, .. } => {
            put_uvarint(*element as u64, &mut payload);
            10
        }
        TypeDefinition::ObjectDescription {
            kind,
            schema_payload,
            ..
        } => {
            if *kind > 16 || matches!(*kind, 0 | 1 | 2 | 4 | 6 | 7 | 8 | 10) {
                return Err(error(
                    ErrorKind::Malformed,
                    "type table",
                    output.len(),
                    "invalid described type kind",
                ));
            }
            payload.extend(schema_payload);
            *kind
        }
    };
    output.push(kind);
    put_uvarint(payload.len() as u64, output);
    output.extend(payload);
    Ok(())
}

#[allow(clippy::too_many_lines)] // Every protocol value kind remains visibly exhaustive.
fn encode_value(
    value: &SerializedValue,
    definition: &TypeDefinition,
    types: &[TypeDefinition],
    order: StreamByteOrder,
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    match (value, definition) {
        (SerializedValue::Unit, TypeDefinition::Unit { .. }) => Ok(()),
        (SerializedValue::Boolean(value), TypeDefinition::Boolean { .. }) => {
            output.push(u8::from(*value));
            Ok(())
        }
        (SerializedValue::ArbitraryInt(value), TypeDefinition::Int { width_bits: 0, .. }) => {
            let (sign, mut magnitude) = value.to_bytes_be();
            if sign == Sign::NoSign {
                magnitude.clear();
            }
            output.push(match sign {
                Sign::Minus => 1,
                Sign::NoSign | Sign::Plus => 0,
            });
            put_uvarint(magnitude.len() as u64, output);
            output.extend(magnitude);
            Ok(())
        }
        (
            SerializedValue::Int(value),
            TypeDefinition::Int {
                signed, width_bits, ..
            },
        ) => {
            if *width_bits == 0 {
                return Err(error(
                    ErrorKind::Malformed,
                    "value",
                    0,
                    "arbitrary integer requires an arbitrary-precision value",
                ));
            }
            let width = usize::try_from(width_bits / 8).map_err(|_| {
                error(
                    ErrorKind::Malformed,
                    "value",
                    0,
                    "integer width is too large",
                )
            })?;
            if width == 0 || width > 16 || !width_bits.is_multiple_of(8) {
                return Err(error(
                    ErrorKind::Malformed,
                    "value",
                    0,
                    "invalid integer width",
                ));
            }
            let fits = if *signed {
                if *width_bits == 128 {
                    true
                } else {
                    let bound = 1_i128 << (*width_bits - 1);
                    *value >= -bound && *value < bound
                }
            } else if *value < 0 {
                false
            } else if *width_bits == 128 {
                true
            } else {
                *value < (1_i128 << *width_bits)
            };
            if !fits {
                return Err(error(
                    ErrorKind::Malformed,
                    "value",
                    0,
                    "integer is outside its declared width",
                ));
            }
            let bytes = match order {
                StreamByteOrder::Little => value.to_le_bytes(),
                StreamByteOrder::Big => value.to_be_bytes(),
            };
            match order {
                StreamByteOrder::Little => output.extend_from_slice(&bytes[..width]),
                StreamByteOrder::Big => output.extend_from_slice(&bytes[16 - width..]),
            }
            Ok(())
        }
        (SerializedValue::Text(text), TypeDefinition::Text { .. }) => put_text(text, output),
        (SerializedValue::Product(values), TypeDefinition::Tuple { components, .. }) => {
            encode_components(values, components, types, order, output)
        }
        (SerializedValue::Product(values), TypeDefinition::Record { fields, .. }) => {
            let components = fields.iter().map(|(_, id)| *id).collect::<Vec<_>>();
            encode_components(values, &components, types, order, output)
        }
        (
            SerializedValue::Variant { alternative, value },
            TypeDefinition::Variant { alternatives, .. },
        ) => {
            let Some((_, type_id)) = alternatives.get(*alternative) else {
                return Err(error(
                    ErrorKind::Malformed,
                    "value",
                    0,
                    "invalid variant tag",
                ));
            };
            put_uvarint(*alternative as u64, output);
            encode_value(value, &types[*type_id], types, order, output)
        }
        (SerializedValue::Sequence(values), TypeDefinition::Sequence { element, .. }) => {
            put_uvarint(values.len() as u64, output);
            for value in values {
                encode_value(value, &types[*element], types, order, output)?;
            }
            Ok(())
        }
        (
            SerializedValue::ObjectDescription(values),
            TypeDefinition::ObjectDescription {
                kind,
                schema_payload,
                ..
            },
        ) => encode_described_value(values, *kind, schema_payload, types, order, output),
        _ => Err(error(
            ErrorKind::Malformed,
            "value",
            0,
            "value does not match its type",
        )),
    }
}

fn encode_components(
    values: &[SerializedValue],
    ids: &[usize],
    types: &[TypeDefinition],
    order: StreamByteOrder,
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    if values.len() != ids.len() {
        return Err(error(
            ErrorKind::Malformed,
            "value",
            0,
            "product field count mismatch",
        ));
    }
    for (value, id) in values.iter().zip(ids) {
        encode_value(value, &types[*id], types, order, output)?;
    }
    Ok(())
}

fn encode_described_value(
    values: &[SerializedValue],
    kind: u8,
    schema: &[u8],
    types: &[TypeDefinition],
    order: StreamByteOrder,
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    let references = described_references(kind, schema, types.len(), usize::MAX, 0)?;
    match kind {
        3 | 13..=15 => encode_components(values, &references, types, order, output),
        5 => match values {
            [SerializedValue::Bytes(bytes)] => {
                put_uvarint(bytes.len() as u64, output);
                output.extend(bytes);
                Ok(())
            }
            _ => Err(error(
                ErrorKind::Malformed,
                "value",
                0,
                "Bytes description requires one byte value",
            )),
        },
        9 => match values {
            [SerializedValue::Variant { alternative, value }] => {
                let Some(type_id) = references.get(*alternative) else {
                    return Err(error(
                        ErrorKind::Malformed,
                        "value",
                        0,
                        "invalid union alternative",
                    ));
                };
                put_uvarint(*alternative as u64, output);
                encode_value(value, &types[*type_id], types, order, output)
            }
            _ => Err(error(
                ErrorKind::Malformed,
                "value",
                0,
                "Union description requires one alternative",
            )),
        },
        11 => match values {
            [SerializedValue::Sequence(entries)] => {
                put_uvarint(entries.len() as u64, output);
                for entry in entries {
                    encode_value(entry, &types[references[0]], types, order, output)?;
                }
                Ok(())
            }
            _ => Err(error(
                ErrorKind::Malformed,
                "value",
                0,
                "Set description requires one sequence",
            )),
        },
        12 => match values {
            [SerializedValue::Sequence(entries)] => {
                put_uvarint(entries.len() as u64, output);
                for entry in entries {
                    let SerializedValue::Product(pair) = entry else {
                        return Err(error(
                            ErrorKind::Malformed,
                            "value",
                            0,
                            "Map entry requires a key-value product",
                        ));
                    };
                    encode_components(pair, &references, types, order, output)?;
                }
                Ok(())
            }
            _ => Err(error(
                ErrorKind::Malformed,
                "value",
                0,
                "Map description requires one sequence",
            )),
        },
        16 => Err(error(
            ErrorKind::Unsupported,
            "value",
            0,
            "a bare recursive description has no finite value",
        )),
        _ => Err(error(
            ErrorKind::Malformed,
            "value",
            0,
            "unknown described value kind",
        )),
    }
}

fn validate_types(types: &[TypeDefinition], offset: usize) -> Result<(), ProtocolError> {
    let mut identities = std::collections::BTreeSet::new();
    for (index, definition) in types.iter().enumerate() {
        if !identities.insert(definition.identity()) {
            return Err(error(
                ErrorKind::Malformed,
                "type table",
                offset,
                "duplicate type identity",
            ));
        }
        let references = match definition {
            TypeDefinition::Tuple { components, .. } => components.clone(),
            TypeDefinition::Record { fields, .. } => fields.iter().map(|(_, id)| *id).collect(),
            TypeDefinition::Variant { alternatives, .. } => {
                alternatives.iter().map(|(_, id)| *id).collect()
            }
            TypeDefinition::Sequence { element, .. } => vec![*element],
            _ => Vec::new(),
        };
        if references.iter().any(|reference| *reference >= index) {
            return Err(error(
                ErrorKind::Malformed,
                "type table",
                offset,
                "type reference is not earlier in the table",
            ));
        }
        match definition {
            TypeDefinition::Int { width_bits, .. }
                if *width_bits != 0 && !width_bits.is_multiple_of(8) =>
            {
                return Err(error(
                    ErrorKind::Malformed,
                    "type table",
                    offset,
                    "fixed integer width is not a positive multiple of eight",
                ));
            }
            TypeDefinition::Record { fields, .. } => {
                validate_labels(fields.iter().map(|(label, _)| label), offset)?;
            }
            TypeDefinition::Variant { alternatives, .. } => {
                validate_labels(alternatives.iter().map(|(label, _)| label), offset)?;
            }
            TypeDefinition::ObjectDescription {
                kind,
                schema_payload,
                ..
            } => validate_described_schema(*kind, schema_payload, index, usize::MAX, offset)?,
            _ => {}
        }
    }
    Ok(())
}

fn validate_labels<'a>(
    labels: impl IntoIterator<Item = &'a String>,
    offset: usize,
) -> Result<(), ProtocolError> {
    let mut unique = std::collections::BTreeSet::new();
    if labels.into_iter().all(|label| unique.insert(label)) {
        Ok(())
    } else {
        Err(error(
            ErrorKind::Malformed,
            "type table",
            offset,
            "duplicate field or alternative label",
        ))
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize, stage: &'static str) -> Result<&'a [u8], ProtocolError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| self.failure(ErrorKind::Malformed, stage, "length overflow"))?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| self.failure(ErrorKind::Malformed, stage, "premature end of stream"))?;
        self.offset = end;
        Ok(value)
    }

    fn byte(&mut self, stage: &'static str) -> Result<u8, ProtocolError> {
        Ok(self.take(1, stage)?[0])
    }

    fn uvarint(&mut self, stage: &'static str) -> Result<u64, ProtocolError> {
        let start = self.offset;
        let mut value = 0_u64;
        for index in 0..10 {
            let byte = self.byte(stage)?;
            if index == 9 && byte > 1 {
                return Err(self.failure(ErrorKind::Malformed, stage, "varint overflow"));
            }
            value |= u64::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                if index > 0 && byte == 0 {
                    return Err(error(
                        ErrorKind::Malformed,
                        stage,
                        start,
                        "nonminimal varint",
                    ));
                }
                return Ok(value);
            }
        }
        Err(error(
            ErrorKind::Malformed,
            stage,
            start,
            "unterminated varint",
        ))
    }

    fn count(&mut self, stage: &'static str, maximum: usize) -> Result<usize, ProtocolError> {
        let value = self.uvarint(stage)?;
        let count = usize::try_from(value).map_err(|_| {
            self.failure(ErrorKind::ResourceLimit, stage, "count exceeds host limits")
        })?;
        if count > maximum {
            return Err(self.failure(
                ErrorKind::ResourceLimit,
                stage,
                "declared count exceeds configured limit",
            ));
        }
        Ok(count)
    }

    fn text(&mut self, stage: &'static str, maximum: usize) -> Result<String, ProtocolError> {
        let length = self.count(stage, maximum)?;
        let start = self.offset;
        let bytes = self.take(length, stage)?;
        let text = std::str::from_utf8(bytes)
            .map_err(|_| error(ErrorKind::Malformed, stage, start, "invalid UTF-8"))?;
        if text.contains('\0') || !is_nfc(text) {
            return Err(error(
                ErrorKind::Malformed,
                stage,
                start,
                "text is not valid NFC source text",
            ));
        }
        Ok(text.to_owned())
    }

    #[allow(clippy::too_many_lines)] // Kind decoding follows the stable numeric registry.
    fn type_definition(
        &mut self,
        previous: &[TypeDefinition],
        text_limit: usize,
    ) -> Result<TypeDefinition, ProtocolError> {
        let identity = self.text("type table", text_limit)?;
        let kind = self.byte("type table")?;
        let length = self.count("type table", usize::MAX)?;
        let payload_offset = self.offset;
        let payload = self.take(length, "type table")?;
        let mut reader = Self {
            bytes: payload,
            offset: 0,
        };
        let definition = match kind {
            0 => TypeDefinition::Unit { identity },
            1 => TypeDefinition::Boolean { identity },
            2 => TypeDefinition::Int {
                identity,
                signed: match reader.byte("type table")? {
                    0 => false,
                    1 => true,
                    _ => {
                        return Err(error(
                            ErrorKind::Malformed,
                            "type table",
                            payload_offset,
                            "invalid signedness Boolean",
                        ));
                    }
                },
                width_bits: reader.uvarint("type table")?,
            },
            4 => {
                if reader.byte("type table")? != 0 {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "type table",
                        payload_offset,
                        "unsupported text normalization",
                    ));
                }
                TypeDefinition::Text { identity }
            }
            6 => TypeDefinition::Tuple {
                identity,
                components: reader.ids(previous.len())?,
            },
            7 => {
                let count = reader.count("type table", reader.bytes.len())?;
                let mut fields = Vec::with_capacity(count);
                for _ in 0..count {
                    fields.push((
                        reader.text("type table", text_limit)?,
                        reader.count("type table", previous.len().saturating_sub(1))?,
                    ));
                }
                TypeDefinition::Record { identity, fields }
            }
            8 => {
                let count = reader.count("type table", reader.bytes.len())?;
                let mut alternatives = Vec::with_capacity(count);
                for expected in 0..count {
                    if reader.count("type table", count)? != expected {
                        return Err(error(
                            ErrorKind::Malformed,
                            "type table",
                            payload_offset + reader.offset,
                            "variant tags are not dense",
                        ));
                    }
                    alternatives.push((
                        reader.text("type table", text_limit)?,
                        reader.count("type table", previous.len().saturating_sub(1))?,
                    ));
                }
                TypeDefinition::Variant {
                    identity,
                    alternatives,
                }
            }
            10 => TypeDefinition::Sequence {
                identity,
                element: reader.count("type table", previous.len().saturating_sub(1))?,
            },
            3 | 5 | 9 | 11..=16 => {
                validate_described_schema(
                    kind,
                    payload,
                    previous.len(),
                    text_limit,
                    payload_offset,
                )?;
                reader.offset = payload.len();
                TypeDefinition::ObjectDescription {
                    identity,
                    kind,
                    schema_payload: payload.to_vec(),
                }
            }
            _ => {
                return Err(error(
                    ErrorKind::Malformed,
                    "type table",
                    payload_offset,
                    "unknown protocol 1.0 type kind",
                ));
            }
        };
        if reader.offset != payload.len() {
            return Err(error(
                ErrorKind::Malformed,
                "type table",
                payload_offset + reader.offset,
                "type payload has trailing bytes",
            ));
        }
        Ok(definition)
    }

    fn ids(&mut self, prior: usize) -> Result<Vec<usize>, ProtocolError> {
        let count = self.count("type table", self.bytes.len())?;
        (0..count)
            .map(|_| self.count("type table", prior.saturating_sub(1)))
            .collect()
    }

    #[allow(clippy::too_many_lines)] // Each wire kind remains explicit at the validation boundary.
    fn value(
        &mut self,
        definition: &TypeDefinition,
        types: &[TypeDefinition],
        order: StreamByteOrder,
        text_limit: usize,
        depth: usize,
        depth_limit: usize,
    ) -> Result<SerializedValue, ProtocolError> {
        if depth > depth_limit {
            return Err(self.failure(
                ErrorKind::ResourceLimit,
                "value",
                "value nesting exceeds configured limit",
            ));
        }
        match definition {
            TypeDefinition::Unit { .. } => Ok(SerializedValue::Unit),
            TypeDefinition::Boolean { .. } => match self.byte("value")? {
                0 => Ok(SerializedValue::Boolean(false)),
                1 => Ok(SerializedValue::Boolean(true)),
                _ => Err(self.failure(ErrorKind::Malformed, "value", "invalid Boolean")),
            },
            TypeDefinition::Int {
                signed, width_bits, ..
            } => {
                if *width_bits == 0 {
                    let sign = match self.byte("value")? {
                        0 => Sign::Plus,
                        1 if *signed => Sign::Minus,
                        1 => {
                            return Err(self.failure(
                                ErrorKind::Malformed,
                                "value",
                                "unsigned arbitrary integer has a negative sign",
                            ));
                        }
                        _ => {
                            return Err(self.failure(
                                ErrorKind::Malformed,
                                "value",
                                "invalid arbitrary integer sign",
                            ));
                        }
                    };
                    let length =
                        self.count("value", self.bytes.len().saturating_sub(self.offset))?;
                    let magnitude = self.take(length, "value")?;
                    if magnitude.first() == Some(&0) {
                        return Err(self.failure(
                            ErrorKind::Malformed,
                            "value",
                            "arbitrary integer magnitude is not minimal",
                        ));
                    }
                    if magnitude.is_empty() && sign == Sign::Minus {
                        return Err(self.failure(
                            ErrorKind::Malformed,
                            "value",
                            "negative zero is not canonical",
                        ));
                    }
                    return Ok(SerializedValue::ArbitraryInt(if magnitude.is_empty() {
                        BigInt::from(0)
                    } else {
                        BigInt::from_bytes_be(sign, magnitude)
                    }));
                }
                let width = usize::try_from(width_bits / 8).map_err(|_| {
                    self.failure(
                        ErrorKind::ResourceLimit,
                        "value",
                        "integer width is too large",
                    )
                })?;
                if width > 16 || !width_bits.is_multiple_of(8) {
                    return Err(self.failure(
                        ErrorKind::Malformed,
                        "value",
                        "invalid integer width",
                    ));
                }
                let bytes = self.take(width, "value")?;
                let negative = *signed
                    && match order {
                        StreamByteOrder::Little => bytes.last(),
                        StreamByteOrder::Big => bytes.first(),
                    }
                    .is_some_and(|byte| byte & 0x80 != 0);
                let mut full = [if negative { 0xff } else { 0 }; 16];
                match order {
                    StreamByteOrder::Little => full[..width].copy_from_slice(bytes),
                    StreamByteOrder::Big => full[16 - width..].copy_from_slice(bytes),
                }
                Ok(SerializedValue::Int(match order {
                    StreamByteOrder::Little => i128::from_le_bytes(full),
                    StreamByteOrder::Big => i128::from_be_bytes(full),
                }))
            }
            TypeDefinition::Text { .. } => {
                Ok(SerializedValue::Text(self.text("value", text_limit)?))
            }
            TypeDefinition::Tuple { components, .. } => Ok(SerializedValue::Product(self.values(
                components,
                types,
                order,
                text_limit,
                depth + 1,
                depth_limit,
            )?)),
            TypeDefinition::Record { fields, .. } => Ok(SerializedValue::Product(self.values(
                &fields.iter().map(|(_, id)| *id).collect::<Vec<_>>(),
                types,
                order,
                text_limit,
                depth + 1,
                depth_limit,
            )?)),
            TypeDefinition::Variant { alternatives, .. } => {
                let alternative = self.count("value", alternatives.len().saturating_sub(1))?;
                let value = self.value(
                    &types[alternatives[alternative].1],
                    types,
                    order,
                    text_limit,
                    depth + 1,
                    depth_limit,
                )?;
                Ok(SerializedValue::Variant {
                    alternative,
                    value: Box::new(value),
                })
            }
            TypeDefinition::Sequence { element, .. } => {
                let count = self.count("value", 1_000_000)?;
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    values.push(self.value(
                        &types[*element],
                        types,
                        order,
                        text_limit,
                        depth + 1,
                        depth_limit,
                    )?);
                }
                Ok(SerializedValue::Sequence(values))
            }
            TypeDefinition::ObjectDescription {
                kind,
                schema_payload,
                ..
            } => self.described_value(
                *kind,
                schema_payload,
                types,
                order,
                text_limit,
                depth + 1,
                depth_limit,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)] // Recursive bounds accompany the schema-decoding context.
    fn described_value(
        &mut self,
        kind: u8,
        schema: &[u8],
        types: &[TypeDefinition],
        order: StreamByteOrder,
        text_limit: usize,
        depth: usize,
        depth_limit: usize,
    ) -> Result<SerializedValue, ProtocolError> {
        let references = described_references(kind, schema, types.len(), text_limit, self.offset)?;
        let values = match kind {
            3 | 13..=15 => {
                self.values(&references, types, order, text_limit, depth, depth_limit)?
            }
            5 => {
                let length = self.count("value", self.bytes.len().saturating_sub(self.offset))?;
                vec![SerializedValue::Bytes(self.take(length, "value")?.to_vec())]
            }
            9 => {
                let alternative = self.count("value", references.len().saturating_sub(1))?;
                let value = self.value(
                    &types[references[alternative]],
                    types,
                    order,
                    text_limit,
                    depth,
                    depth_limit,
                )?;
                vec![SerializedValue::Variant {
                    alternative,
                    value: Box::new(value),
                }]
            }
            11 => {
                let count = self.count("value", 1_000_000)?;
                let mut entries = Vec::with_capacity(count);
                for _ in 0..count {
                    entries.push(self.value(
                        &types[references[0]],
                        types,
                        order,
                        text_limit,
                        depth,
                        depth_limit,
                    )?);
                }
                vec![SerializedValue::Sequence(entries)]
            }
            12 => {
                let count = self.count("value", 1_000_000)?;
                let mut entries = Vec::with_capacity(count);
                for _ in 0..count {
                    entries.push(SerializedValue::Product(self.values(
                        &references,
                        types,
                        order,
                        text_limit,
                        depth,
                        depth_limit,
                    )?));
                }
                vec![SerializedValue::Sequence(entries)]
            }
            16 => {
                return Err(self.failure(
                    ErrorKind::Unsupported,
                    "value",
                    "a bare recursive description has no finite value",
                ));
            }
            _ => {
                return Err(self.failure(
                    ErrorKind::Malformed,
                    "value",
                    "unknown described value kind",
                ));
            }
        };
        Ok(SerializedValue::ObjectDescription(values))
    }

    fn values(
        &mut self,
        ids: &[usize],
        types: &[TypeDefinition],
        order: StreamByteOrder,
        text_limit: usize,
        depth: usize,
        depth_limit: usize,
    ) -> Result<Vec<SerializedValue>, ProtocolError> {
        ids.iter()
            .map(|id| self.value(&types[*id], types, order, text_limit, depth, depth_limit))
            .collect()
    }

    fn failure(
        &self,
        kind: ErrorKind,
        stage: &'static str,
        message: &'static str,
    ) -> ProtocolError {
        error(kind, stage, self.offset, message)
    }
}

fn validate_described_schema(
    kind: u8,
    payload: &[u8],
    prior: usize,
    text_limit: usize,
    offset: usize,
) -> Result<(), ProtocolError> {
    described_references(kind, payload, prior, text_limit, offset).map(|_| ())
}

fn described_references(
    kind: u8,
    payload: &[u8],
    prior: usize,
    text_limit: usize,
    offset: usize,
) -> Result<Vec<usize>, ProtocolError> {
    let mut reader = Reader {
        bytes: payload,
        offset: 0,
    };
    let mut references = Vec::new();
    match kind {
        3 => {
            references.push(described_id(&mut reader, prior)?);
            references.push(described_id(&mut reader, prior)?);
        }
        5 => {}
        9 => {
            let count = reader.count("type table", payload.len())?;
            for _ in 0..count {
                references.push(described_id(&mut reader, prior)?);
            }
        }
        11 | 13 => {
            references.push(described_id(&mut reader, prior)?);
            reader.text("type table", text_limit)?;
        }
        12 => {
            references.push(described_id(&mut reader, prior)?);
            references.push(described_id(&mut reader, prior)?);
            reader.text("type table", text_limit)?;
        }
        14 => {
            references.push(described_id(&mut reader, prior)?);
        }
        15 => {
            reader.text("type table", text_limit)?;
            references.push(described_id(&mut reader, prior)?);
        }
        16 => {
            reader.text("type table", text_limit)?;
        }
        _ => {
            return Err(error(
                ErrorKind::Malformed,
                "type table",
                offset,
                "unknown described schema kind",
            ));
        }
    }
    if reader.offset != payload.len() {
        return Err(error(
            ErrorKind::Malformed,
            "type table",
            offset + reader.offset,
            "described schema has trailing bytes",
        ));
    }
    Ok(references)
}

fn described_id(reader: &mut Reader<'_>, prior: usize) -> Result<usize, ProtocolError> {
    reader.count("type table", prior.saturating_sub(1))
}

fn put_ids(ids: &[usize], output: &mut Vec<u8>) {
    put_uvarint(ids.len() as u64, output);
    for id in ids {
        put_uvarint(*id as u64, output);
    }
}

fn put_text(text: &str, output: &mut Vec<u8>) -> Result<(), ProtocolError> {
    if text.contains('\0') || !is_nfc(text) {
        return Err(error(
            ErrorKind::Malformed,
            "text",
            output.len(),
            "text is not valid NFC source text",
        ));
    }
    put_uvarint(text.len() as u64, output);
    output.extend_from_slice(text.as_bytes());
    Ok(())
}

fn put_uvarint(mut value: u64, output: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

const fn error(
    kind: ErrorKind,
    stage: &'static str,
    offset: usize,
    message: &'static str,
) -> ProtocolError {
    ProtocolError {
        kind,
        stage,
        offset,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Stream {
        Stream {
            header: Header {
                language_identity: "topal".into(),
                language_version: LanguageVersion::DESIGN_0,
                byte_order: StreamByteOrder::Little,
                streaming: false,
            },
            types: vec![
                TypeDefinition::Boolean {
                    identity: "Boolean".into(),
                },
                TypeDefinition::Text {
                    identity: "String".into(),
                },
                TypeDefinition::Record {
                    identity: "Answer".into(),
                    fields: vec![("ok".into(), 0), ("text".into(), 1)],
                },
            ],
            events: vec![Event {
                type_id: 2,
                value: SerializedValue::Product(vec![
                    SerializedValue::Boolean(true),
                    SerializedValue::Text("forty-two".into()),
                ]),
            }],
        }
    }

    #[test]
    fn canonical_stream_has_stable_golden_bytes_and_round_trips() {
        let bytes = serialize(&sample()).unwrap();
        assert_eq!(
            bytes,
            [
                0x54, 0x4f, 0x50, 0x41, 0x4c, 0x53, 0x45, 0x52, 1, 0, 5, 0x74, 0x6f, 0x70, 0x61,
                0x6c, 0, 1, 0, 0, 0, 0, 3, 1, 7, 0x42, 0x6f, 0x6f, 0x6c, 0x65, 0x61, 0x6e, 1, 0, 6,
                0x53, 0x74, 0x72, 0x69, 0x6e, 0x67, 4, 1, 0, 6, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72,
                7, 11, 2, 2, 0x6f, 0x6b, 0, 4, 0x74, 0x65, 0x78, 0x74, 1, 12, 2, 1, 9, 0x66, 0x6f,
                0x72, 0x74, 0x79, 0x2d, 0x74, 0x77, 0x6f,
            ]
        );
        assert_eq!(deserialize(&bytes, Limits::default()).unwrap(), sample());
        assert_eq!(serialize(&sample()).unwrap(), bytes);
    }

    #[test]
    fn malformed_and_unsupported_headers_are_distinct() {
        let bytes = serialize(&sample()).unwrap();
        let mut bad_magic = bytes.clone();
        bad_magic[0] = 0;
        assert_eq!(
            deserialize(&bad_magic, Limits::default()).unwrap_err().kind,
            ErrorKind::Malformed
        );
        let mut unsupported = bytes;
        unsupported[8] = 2;
        assert_eq!(
            deserialize(&unsupported, Limits::default())
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
    }

    #[test]
    fn limits_and_trailing_bytes_fail_before_exposure() {
        let bytes = serialize(&sample()).unwrap();
        let limited = Limits {
            types: 2,
            ..Limits::default()
        };
        assert_eq!(
            deserialize(&bytes, limited).unwrap_err().kind,
            ErrorKind::ResourceLimit
        );
        let mut trailing = bytes;
        trailing.push(0);
        let error = deserialize(&trailing, Limits::default()).unwrap_err();
        assert_eq!((error.kind, error.stage), (ErrorKind::Malformed, "stream"));
    }

    #[test]
    fn configured_nesting_limit_precedes_recursive_value_allocation() {
        let stream = Stream {
            header: sample().header,
            types: vec![
                TypeDefinition::Unit {
                    identity: "Unit".into(),
                },
                TypeDefinition::Sequence {
                    identity: "Units".into(),
                    element: 0,
                },
            ],
            events: vec![Event {
                type_id: 1,
                value: SerializedValue::Sequence(vec![SerializedValue::Unit]),
            }],
        };
        let bytes = serialize(&stream).unwrap();
        let error = deserialize(
            &bytes,
            Limits {
                nesting_depth: 0,
                ..Limits::default()
            },
        )
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::ResourceLimit);
        assert_eq!(error.stage, "value");
    }

    #[test]
    fn every_truncated_prefix_and_nonminimal_varint_is_rejected_without_panic() {
        let bytes = serialize(&sample()).unwrap();
        for end in 0..bytes.len() {
            assert!(deserialize(&bytes[..end], Limits::default()).is_err());
        }
        let mut nonminimal = MAGIC.to_vec();
        nonminimal.extend([0x81, 0x00]);
        assert_eq!(
            deserialize(&nonminimal, Limits::default())
                .unwrap_err()
                .kind,
            ErrorKind::Malformed
        );
    }

    #[test]
    fn variant_types_round_trip_and_integers_cannot_be_truncated() {
        let variant = Stream {
            header: sample().header,
            types: vec![
                TypeDefinition::Int {
                    identity: "Byte".into(),
                    signed: false,
                    width_bits: 8,
                },
                TypeDefinition::Variant {
                    identity: "MaybeByte".into(),
                    alternatives: vec![("some".into(), 0)],
                },
            ],
            events: vec![Event {
                type_id: 1,
                value: SerializedValue::Variant {
                    alternative: 0,
                    value: Box::new(SerializedValue::Int(255)),
                },
            }],
        };
        let bytes = serialize(&variant).unwrap();
        assert_eq!(deserialize(&bytes, Limits::default()).unwrap(), variant);

        let mut overflowing = sample();
        overflowing.types = vec![TypeDefinition::Int {
            identity: "Byte".into(),
            signed: false,
            width_bits: 8,
        }];
        overflowing.events = vec![Event {
            type_id: 0,
            value: SerializedValue::Int(256),
        }];
        assert_eq!(
            serialize(&overflowing).unwrap_err().message,
            "integer is outside its declared width"
        );
    }

    #[test]
    fn arbitrary_integers_round_trip_canonically_and_reject_nonminimal_forms() {
        let arbitrary = |value: BigInt| Stream {
            header: sample().header,
            types: vec![TypeDefinition::Int {
                identity: "Int".into(),
                signed: true,
                width_bits: 0,
            }],
            events: vec![Event {
                type_id: 0,
                value: SerializedValue::ArbitraryInt(value),
            }],
        };
        let huge = arbitrary(BigInt::from(1_u8) << 300_usize);
        let bytes = serialize(&huge).unwrap();
        assert_eq!(deserialize(&bytes, Limits::default()).unwrap(), huge);

        let mut nonminimal = serialize(&arbitrary(BigInt::from(256))).unwrap();
        let first_magnitude = nonminimal.len() - 2;
        nonminimal[first_magnitude] = 0;
        assert_eq!(
            deserialize(&nonminimal, Limits::default())
                .unwrap_err()
                .message,
            "arbitrary integer magnitude is not minimal"
        );

        let mut negative_zero = serialize(&arbitrary(BigInt::from(0))).unwrap();
        let sign = negative_zero.len() - 2;
        negative_zero[sign] = 1;
        assert_eq!(
            deserialize(&negative_zero, Limits::default())
                .unwrap_err()
                .message,
            "negative zero is not canonical"
        );
    }

    #[test]
    fn every_protocol_kind_is_preserved_as_a_safe_description() {
        for kind in [3, 5, 9, 11, 12, 13, 14, 15] {
            let schema_payload = match kind {
                3 => vec![0, 0],
                5 => vec![],
                9 => vec![1, 0],
                11 => vec![0, 5, b'o', b'r', b'd', b'e', b'r'],
                12 => vec![0, 0, 5, b'o', b'r', b'd', b'e', b'r'],
                13 => vec![0, 4, b'p', b'r', b'e', b'd'],
                14 => vec![0],
                15 => vec![4, b'k', b'i', b'n', b'd', 0],
                16 => vec![4, b's', b'e', b'l', b'f'],
                _ => unreachable!(),
            };
            let described = match kind {
                3 => vec![SerializedValue::Unit, SerializedValue::Unit],
                5 => vec![SerializedValue::Bytes(vec![1, 2, 3])],
                9 => vec![SerializedValue::Variant {
                    alternative: 0,
                    value: Box::new(SerializedValue::Unit),
                }],
                11 => vec![SerializedValue::Sequence(vec![SerializedValue::Unit])],
                12 => vec![SerializedValue::Sequence(vec![SerializedValue::Product(
                    vec![SerializedValue::Unit, SerializedValue::Unit],
                )])],
                13..=15 => vec![SerializedValue::Unit],
                _ => unreachable!(),
            };
            let stream = Stream {
                header: sample().header,
                types: vec![
                    TypeDefinition::Unit {
                        identity: "Unit".into(),
                    },
                    TypeDefinition::ObjectDescription {
                        identity: format!("kind-{kind}"),
                        kind,
                        schema_payload,
                    },
                ],
                events: vec![Event {
                    type_id: 1,
                    value: SerializedValue::ObjectDescription(described),
                }],
            };
            let bytes = serialize(&stream).unwrap();
            assert_eq!(deserialize(&bytes, Limits::default()).unwrap(), stream);
        }
    }

    #[test]
    fn described_values_are_structurally_checked_instead_of_accepted_as_frame_bytes() {
        let stream = Stream {
            header: sample().header,
            types: vec![
                TypeDefinition::Unit {
                    identity: "Unit".into(),
                },
                TypeDefinition::ObjectDescription {
                    identity: "Rational".into(),
                    kind: 3,
                    schema_payload: vec![0, 0],
                },
            ],
            events: vec![Event {
                type_id: 1,
                value: SerializedValue::ObjectDescription(vec![SerializedValue::Unit]),
            }],
        };
        let error = serialize(&stream).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Malformed);
        assert_eq!(error.stage, "value");
    }

    #[test]
    fn unknown_count_streams_end_only_at_the_zero_frame_terminator() {
        let mut stream = sample();
        stream.header.streaming = true;
        let bytes = serialize(&stream).unwrap();
        assert_eq!(bytes.last(), Some(&0));
        assert_eq!(deserialize(&bytes, Limits::default()).unwrap(), stream);

        let truncated = &bytes[..bytes.len() - 1];
        assert_eq!(
            deserialize(truncated, Limits::default()).unwrap_err().kind,
            ErrorKind::Malformed
        );
    }
}
