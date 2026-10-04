impl DebugInfo {
    #[allow(clippy::too_many_lines)] // Initialization keeps the complete emitted DWARF type graph visible.
    fn new(source_name: &str, extended_types: bool, generator_close_types: bool) -> Self {
        let path = Path::new(source_name);
        let filename = path.file_name().map_or_else(
            || source_name.into(),
            |name| name.to_string_lossy().into_owned(),
        );
        let directory = path
            .parent()
            .map_or_else(String::new, |parent| parent.to_string_lossy().into_owned());
        let source = topal_source::SourceText::new("").expect("empty source is valid");
        let mut debug = Self {
            nodes: Vec::new(),
            file: 0,
            compile_unit: 0,
            empty: 0,
            unsigned64_type: 0,
            int_type: 0,
            nat_type: 0,
            version_type: 0,
            serialization_stream_type: 0,
            rational_type: 0,
            character_type: 0,
            string_type: 0,
            error_type: 0,
            generator_error_type: 0,
            error_code_type: 0,
            error_domain_type: 0,
            source_location_type: 0,
            int_range_type: 0,
            rational_range_type: 0,
            result_int_type: 0,
            result_nat_type: 0,
            result_rational_type: 0,
            result_string_type: 0,
            result_int_pair_type: 0,
            result_unit_generator_error_type: 0,
            result_header_pointer_type: 0,
            result_types: Vec::new(),
            result_modular_types: Vec::new(),
            optional_int_type: 0,
            optional_rational_type: 0,
            optional_character_type: 0,
            optional_string_type: 0,
            optional_error_type: 0,
            optional_source_location_type: 0,
            optional_header_pointer_type: 0,
            optional_types: Vec::new(),
            traversal_control_types: Vec::new(),
            comparison_type: 0,
            boolean_type: 0,
            unit_type: 0,
            completed_type: 0,
            effect_type: 0,
            enum_types: BTreeMap::new(),
            modular_types: Vec::new(),
            list_types: Vec::new(),
            container_types: Vec::new(),
            refined_types: Vec::new(),
            tuple_types: Vec::new(),
            record_types: Vec::new(),
            sum_types: Vec::new(),
            task_types: Vec::new(),
            location_types: Vec::new(),
            source,
            filename,
        };
        debug.file = debug.node(format!(
            "!DIFile(filename: \"{}\", directory: \"{}\")",
            llvm_string(&debug.filename),
            llvm_string(&directory)
        ));
        debug.empty = debug.node("!{}".into());
        debug.compile_unit = debug.node(format!(
            "distinct !DICompileUnit(language: DW_LANG_C11, file: !{}, producer: \"topalc {}\", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, enums: !{}, retainedTypes: !{})",
            debug.file,
            env!("CARGO_PKG_VERSION"),
            debug.empty,
            debug.empty
        ));
        let unsigned64 = debug.install_base_integer_types();
        debug.version_type = debug.install_version_type();
        debug.serialization_stream_type = debug.install_serialization_stream_type(unsigned64);
        let numerator = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"numerator\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            debug.file, debug.int_type
        ));
        let denominator = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"denominator\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            debug.file, debug.int_type
        ));
        let rational_members = debug.node(format!("!{{!{numerator}, !{denominator}}}"));
        let rational_storage = debug.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalRationalHeader\", file: !{}, size: 128, align: 64, elements: !{rational_members})",
            debug.file
        ));
        let rational_pointer = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{rational_storage}, size: 64, align: 64)"
        ));
        debug.rational_type = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Rational\", file: !{}, baseType: !{rational_pointer})",
            debug.file
        ));
        if extended_types {
            debug.install_string_and_error_types(unsigned64, generator_close_types);
        }
        debug.install_aggregate_types(unsigned64, extended_types, generator_close_types);
        let less = debug.node("!DIEnumerator(name: \"Less\", value: -1)".into());
        let equal = debug.node("!DIEnumerator(name: \"Equal\", value: 0)".into());
        let greater = debug.node("!DIEnumerator(name: \"Greater\", value: 1)".into());
        let comparison_values = debug.node(format!("!{{!{less}, !{equal}, !{greater}}}"));
        debug.comparison_type = debug.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"Comparison\", file: !{}, size: 32, align: 32, elements: !{comparison_values})",
            debug.file
        ));
        debug.install_zero_data_types();
        debug
    }

    fn install_base_integer_types(&mut self) -> usize {
        let unsigned64 =
            self.node("!DIBasicType(name: \"u64\", size: 64, encoding: DW_ATE_unsigned)".into());
        self.unsigned64_type = unsigned64;
        self.install_integer_types(unsigned64);
        unsigned64
    }

    fn install_zero_data_types(&mut self) {
        self.boolean_type =
            self.node("!DIBasicType(name: \"Boolean\", size: 8, encoding: DW_ATE_boolean)".into());
        self.unit_type =
            self.node("!DIBasicType(name: \"Unit\", size: 8, encoding: DW_ATE_unsigned)".into());
        let completed = self.node("!DIEnumerator(name: \"Completed\", value: 0)".into());
        let completed_values = self.node(format!("!{{!{completed}}}"));
        self.completed_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"Completed\", file: !{}, size: 8, align: 8, elements: !{completed_values})",
            self.file
        ));
        let empty_effect = self.node("!DIEnumerator(name: \"empty\", value: 0)".into());
        let effect_values = self.node(format!("!{{!{empty_effect}}}"));
        self.effect_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"Effect\", file: !{}, size: 8, align: 8, elements: !{effect_values})",
            self.file
        ));
    }

    fn install_aggregate_types(
        &mut self,
        unsigned64: usize,
        extended_types: bool,
        generator_close_types: bool,
    ) {
        self.int_range_type = self.range_type("Range Int", self.int_type, unsigned64);
        self.rational_range_type =
            self.range_type("Range Rational", self.rational_type, unsigned64);
        self.install_result_types(unsigned64, generator_close_types);
        if extended_types {
            self.install_optional_types(unsigned64);
        }
    }

    fn install_integer_types(&mut self, unsigned64: usize) {
        let negative = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"negative\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let length = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"length\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{negative}, !{length}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalIntHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.int_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Int\", file: !{}, baseType: !{pointer})",
            self.file
        ));
        self.nat_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Nat\", file: !{}, baseType: !{pointer})",
            self.file
        ));
    }

    fn install_version_type(&mut self) -> usize {
        let members = ["major", "minor", "patch", "build"]
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                self.node(format!(
                    "!DIDerivedType(tag: DW_TAG_member, name: \"{name}\", file: !{}, baseType: !{}, size: 64, align: 64, offset: {})",
                    self.file,
                    self.nat_type,
                    index * 64
                ))
            })
            .collect::<Vec<_>>();
        let members = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalVersionHeader\", file: !{}, size: 256, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Version\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn install_serialization_stream_type(&mut self, unsigned64: usize) -> usize {
        let byte =
            self.node("!DIBasicType(name: \"u8\", size: 8, encoding: DW_ATE_unsigned_char)".into());
        let byte_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{byte}, size: 64, align: 64)"
        ));
        let data = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"data\", file: !{}, baseType: !{byte_pointer}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let byte_count = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"byte_count\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{data}, !{byte_count}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalSerializationStreamHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"SerializationStream\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn install_string_and_error_types(&mut self, unsigned64: usize, generator_close_types: bool) {
        let byte =
            self.node("!DIBasicType(name: \"u8\", size: 8, encoding: DW_ATE_unsigned_char)".into());
        let byte_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{byte}, size: 64, align: 64)"
        ));
        let data = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"data\", file: !{}, baseType: !{byte_pointer}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let length = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"length\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let display = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"display\", file: !{}, baseType: !{byte_pointer}, size: 64, align: 64, offset: 128)",
            self.file
        ));
        let display_length = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"display_length\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 192)",
            self.file
        ));
        let members = self.node(format!(
            "!{{!{data}, !{length}, !{display}, !{display_length}}}"
        ));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalStringHeader\", file: !{}, size: 256, align: 64, elements: !{members})",
            self.file
        ));
        let string_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.string_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"String\", file: !{}, baseType: !{string_pointer})",
            self.file
        ));
        self.character_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Character\", file: !{}, baseType: !{string_pointer})",
            self.file
        ));
        self.error_domain_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"ErrorDomain\", file: !{}, baseType: !{string_pointer})",
            self.file
        ));

        self.install_source_location_type();

        let enumerators = [
            ("out-of-range", 0),
            ("not-representable", 1),
            ("division-by-zero", 2),
            ("indeterminate", 3),
        ]
        .into_iter()
        .map(|(name, value)| self.node(format!("!DIEnumerator(name: \"{name}\", value: {value})")))
        .collect::<Vec<_>>();
        let error_code_values = self.node(format!(
            "!{{{}}}",
            enumerators
                .iter()
                .map(|value| format!("!{value}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        self.error_code_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"lang arithmetic ArithmeticErrorCode\", file: !{}, size: 32, align: 32, elements: !{error_code_values})",
            self.file
        ));
        let domain = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"domain\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.error_domain_type
        ));
        let code = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"code\", file: !{}, baseType: !{}, size: 32, align: 32, offset: 64)",
            self.file, self.error_code_type
        ));
        let error_members = self.node(format!("!{{!{domain}, !{code}}}"));
        let error_storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalErrorHeader\", file: !{}, size: 448, align: 64, elements: !{error_members})",
            self.file
        ));
        let error_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{error_storage}, size: 64, align: 64)"
        ));
        self.error_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Error\", file: !{}, baseType: !{error_pointer})",
            self.file
        ));

        if generator_close_types {
            self.install_generator_error_type();
        }
    }

    fn install_generator_error_type(&mut self) {
        let generator_error_code = self.enum_type(&CompilerEnumType {
            name: "lang generator GeneratorErrorCode".into(),
            alternatives: vec!["generator-closed".into()],
        });
        let generator_domain = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"domain\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.error_domain_type
        ));
        let generator_code = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"code\", file: !{}, baseType: !{generator_error_code}, size: 32, align: 32, offset: 64)",
            self.file
        ));
        let generator_members = self.node(format!("!{{!{generator_domain}, !{generator_code}}}"));
        let generator_storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalGeneratorErrorHeader\", file: !{}, size: 448, align: 64, elements: !{generator_members})",
            self.file
        ));
        let generator_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{generator_storage}, size: 64, align: 64)"
        ));
        self.generator_error_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Error (lang generator GeneratorErrorCode)\", file: !{}, baseType: !{generator_pointer})",
            self.file
        ));
    }

    fn install_source_location_type(&mut self) {
        let line = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"line\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.int_type
        ));
        let column = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"column\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            self.file, self.int_type
        ));
        let members = self.node(format!("!{{!{line}, !{column}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalSourceLocationHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.source_location_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"SourceLocation\", file: !{}, baseType: !{pointer})",
            self.file
        ));
    }

    fn range_type(&mut self, name: &str, endpoint_type: usize, flag_type: usize) -> usize {
        let lower = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"lower\", file: !{}, baseType: !{endpoint_type}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let upper = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"upper\", file: !{}, baseType: !{endpoint_type}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let lower_inclusive = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"lower_inclusive\", file: !{}, baseType: !{flag_type}, size: 64, align: 64, offset: 128)",
            self.file
        ));
        let upper_inclusive = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"upper_inclusive\", file: !{}, baseType: !{flag_type}, size: 64, align: 64, offset: 192)",
            self.file
        ));
        let members = self.node(format!(
            "!{{!{lower}, !{upper}, !{lower_inclusive}, !{upper_inclusive}}}"
        ));
        let storage_name = name.replace(' ', "");
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"Topal{storage_name}Header\", file: !{}, size: 256, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{name}\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn install_result_types(&mut self, unsigned64: usize, generator_close_types: bool) {
        let result_tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"is_error\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let opaque_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{unsigned64}, size: 64, align: 64)"
        ));
        let result_payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{opaque_pointer}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let result_members = self.node(format!("!{{!{result_tag}, !{result_payload}}}"));
        let result_storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalResultHeader\", file: !{}, size: 128, align: 64, elements: !{result_members})",
            self.file
        ));
        let result_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{result_storage}, size: 64, align: 64)"
        ));
        self.result_header_pointer_type = result_pointer;
        self.result_int_type = self.result_type("Int", result_pointer);
        self.result_nat_type = self.result_type("Nat", result_pointer);
        self.result_rational_type = self.result_type("Rational", result_pointer);
        self.result_string_type = self.result_type("String", result_pointer);
        self.result_int_pair_type = self.result_type("(Int, Int)", result_pointer);
        if generator_close_types {
            self.result_unit_generator_error_type = self.node(format!(
                "!DIDerivedType(tag: DW_TAG_typedef, name: \"Result (Unit, lang generator GeneratorErrorCode)\", file: !{}, baseType: !{result_pointer})",
                self.file
            ));
        }
    }

    fn result_type(&mut self, success: &str, pointer: usize) -> usize {
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Result ({success}, lang arithmetic ArithmeticErrorCode)\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn install_optional_types(&mut self, unsigned64: usize) {
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"tag\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let opaque_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{unsigned64}, size: 64, align: 64)"
        ));
        let payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{opaque_pointer}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalOptionalHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.optional_header_pointer_type = pointer;
        self.optional_int_type = self.optional_type("Int", pointer);
        self.optional_rational_type = self.optional_type("Rational", pointer);
        self.optional_character_type = self.optional_type("Character", pointer);
        self.optional_string_type = self.optional_type("String", pointer);
        self.optional_error_type = self.optional_type("Error", pointer);
        self.optional_source_location_type = self.optional_type("SourceLocation", pointer);
    }

    fn optional_type(&mut self, payload: &str, pointer: usize) -> usize {
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Optional {payload}\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn set_source(&mut self, source: topal_source::SourceText) {
        self.source = source;
    }

    fn filename(&self) -> &str {
        &self.filename
    }

    fn node(&mut self, value: String) -> usize {
        let id = self.nodes.len();
        self.nodes.push(value);
        id
    }

    #[allow(clippy::too_many_lines)] // Keep the exhaustive semantic-type to DWARF mapping visible.
    fn type_id(&mut self, value_type: &CompilerType) -> usize {
        match value_type {
            CompilerType::Unit => self.unit_type,
            CompilerType::Completed => self.completed_type,
            CompilerType::Effect => self.effect_type,
            CompilerType::Type => self.enum_type(&fundamental_type_enumeration()),
            CompilerType::Scope => self.enum_type(&scope_enumeration()),
            CompilerType::Function => *self
                .enum_types
                .get("Function")
                .expect("checked Function values install their debug type"),
            CompilerType::Identity
            | CompilerType::TypeView
            | CompilerType::FunctionView
            | CompilerType::LanguageContext
            | CompilerType::Capability
            | CompilerType::NativeSerializer(_)
            | CompilerType::ExternalMetadata => {
                unreachable!("static-only compiler values have no runtime debug type")
            }
            CompilerType::SerializationStream(_) => self.serialization_stream_type,
            CompilerType::Constraint => *self
                .enum_types
                .get("Constraint")
                .expect("checked Constraint values install their debug type"),
            CompilerType::Boolean => self.boolean_type,
            CompilerType::Version => self.version_type,
            CompilerType::Int | CompilerType::InfiniteInt => self.int_type,
            CompilerType::Nat | CompilerType::InfiniteNat => self.nat_type,
            CompilerType::Rational | CompilerType::InfiniteRational => self.rational_type,
            CompilerType::Comparison => self.comparison_type,
            CompilerType::Error => self.error_type,
            CompilerType::ErrorCode => self.error_code_type,
            CompilerType::ErrorDomain => self.error_domain_type,
            CompilerType::SourceLocation => self.source_location_type,
            CompilerType::Modular(modular) => self.modular_type(modular),
            CompilerType::Enum(enumeration) => self.enum_type(enumeration),
            CompilerType::Generator(generator) => self.generator_type(generator),
            CompilerType::Task(task) => self.task_type(task),
            CompilerType::ExternalLocation(location) => self.location_type(location),
            CompilerType::Character => self.character_type,
            CompilerType::String => self.string_type,
            CompilerType::Range(endpoint)
                if matches!(
                    endpoint.as_ref(),
                    CompilerType::Int | CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ) =>
            {
                self.int_range_type
            }
            CompilerType::Range(endpoint)
                if matches!(
                    endpoint.as_ref(),
                    CompilerType::Rational | CompilerType::InfiniteRational
                ) =>
            {
                self.rational_range_type
            }
            CompilerType::Range(_) => unreachable!("unsupported Range endpoint reached codegen"),
            CompilerType::Result(success) if success.as_ref() == &CompilerType::Int => {
                self.result_int_type
            }
            CompilerType::Result(success) if success.as_ref() == &CompilerType::Nat => {
                self.result_nat_type
            }
            CompilerType::Result(success) if success.as_ref() == &CompilerType::Rational => {
                self.result_rational_type
            }
            CompilerType::Result(success) if success.as_ref() == &CompilerType::String => {
                self.result_string_type
            }
            CompilerType::Result(success)
                if success.as_ref()
                    == &CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]) =>
            {
                self.result_int_pair_type
            }
            CompilerType::Result(success)
                if matches!(success.as_ref(), CompilerType::Modular(_)) =>
            {
                let CompilerType::Modular(modular) = success.as_ref() else {
                    unreachable!()
                };
                self.result_modular_type(modular)
            }
            CompilerType::Result(success) => self.dynamic_result_type(success),
            CompilerType::TaskResponse(success) => self.task_response_type(success),
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Int => {
                self.optional_int_type
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Rational => {
                self.optional_rational_type
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Character => {
                self.optional_character_type
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::String => {
                self.optional_string_type
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Error => {
                self.optional_error_type
            }
            CompilerType::Optional(payload)
                if payload.as_ref() == &CompilerType::SourceLocation =>
            {
                self.optional_source_location_type
            }
            CompilerType::Optional(payload) => self.dynamic_optional_type(payload),
            CompilerType::TraversalControl(payload) => self.traversal_control_type(payload),
            CompilerType::List(element) => self.list_type(element),
            CompilerType::Array { .. }
            | CompilerType::Set(_)
            | CompilerType::Bag(_)
            | CompilerType::Map { .. } => self.container_type(value_type),
            CompilerType::Refined { constraint, base } => self.refined_type(constraint, base),
            CompilerType::Tuple(fields) => self.tuple_type(fields),
            CompilerType::Record(fields) => self.record_type(fields),
            CompilerType::Sum(sum) => self.sum_type(sum),
        }
    }

    fn task_type(&mut self, task: &CompilerTaskType) -> usize {
        if let Some((_, type_id)) = self.task_types.iter().find(|(known, _)| known == task) {
            return *type_id;
        }
        let identity = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"identity\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let terminated = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"terminated\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            self.file, self.unsigned64_type
        ));
        let state_type = self.type_id(&task.state_type);
        let state = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"{}\", file: !{}, baseType: !{state_type}, size: 64, align: 64, offset: 128)",
            llvm_string(&task.state_name), self.file
        ));
        let members = self.node(format!("!{{!{identity}, !{terminated}, !{state}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalTask.{}\", file: !{}, size: 192, align: 64, elements: !{members})",
            llvm_string(&task.classifier), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{pointer})",
            llvm_string(&task.classifier),
            self.file
        ));
        self.task_types.push((task.clone(), type_id));
        type_id
    }

    fn location_type(&mut self, location: &CompilerLocationType) -> usize {
        if let Some((_, type_id)) = self
            .location_types
            .iter()
            .find(|(known, _)| known == location)
        {
            return *type_id;
        }
        let range_start = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"range-start\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.nat_type
        ));
        let offset = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"offset\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            self.file, self.nat_type
        ));
        let initialized = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"initialized\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 128)",
            self.file, self.unsigned64_type
        ));
        let layout_value = CompilerType::Refined {
            constraint: location.layout.identity.clone(),
            base: Box::new(CompilerType::Nat),
        };
        let layout_type = self.type_id(&layout_value);
        let value = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"value\", file: !{}, baseType: !{layout_type}, size: 64, align: 64, offset: 192)",
            self.file
        ));
        let members = self.node(format!(
            "!{{!{range_start}, !{offset}, !{initialized}, !{value}}}"
        ));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalLocation.{}\", file: !{}, size: 256, align: 64, elements: !{members})",
            llvm_string(&location.identity), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{pointer})",
            llvm_string(&location.identity),
            self.file
        ));
        self.location_types.push((location.clone(), type_id));
        type_id
    }

    fn modular_type(&mut self, modular: &CompilerModularType) -> usize {
        if let Some((_, type_id)) = self
            .modular_types
            .iter()
            .find(|(known, _)| known == modular)
        {
            return *type_id;
        }
        let negative = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"negative\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let length = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"length\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            self.file, self.unsigned64_type
        ));
        let members = self.node(format!("!{{!{negative}, !{length}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalModular.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&modular.name), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{pointer})",
            llvm_string(&modular.name),
            self.file
        ));
        self.modular_types.push((modular.clone(), type_id));
        type_id
    }

    fn dynamic_optional_type(&mut self, payload: &CompilerType) -> usize {
        let value_type = CompilerType::Optional(Box::new(payload.clone()));
        if let Some((_, type_id)) = self
            .optional_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        if !matches!(payload, CompilerType::Enum(_)) {
            let type_id = self.optional_type(&payload.name(), self.optional_header_pointer_type);
            self.optional_types.push((value_type, type_id));
            return type_id;
        }
        let payload_type = self.type_id(payload);
        let payload_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{payload_type}, size: 64, align: 64)"
        ));
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"tag\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let payload_member = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{payload_pointer}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload_member}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalOptional.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&payload.name()), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.optional_type(&payload.name(), pointer);
        self.optional_types.push((value_type, type_id));
        type_id
    }

    fn dynamic_result_type(&mut self, success: &CompilerType) -> usize {
        let value_type = CompilerType::Result(Box::new(success.clone()));
        if let Some((_, type_id)) = self
            .result_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        if !matches!(success, CompilerType::Enum(_)) {
            let type_id = self.result_type(&success.name(), self.result_header_pointer_type);
            self.result_types.push((value_type, type_id));
            return type_id;
        }
        let success_type = self.type_id(success);
        let success_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{success_type}, size: 64, align: 64)"
        ));
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"is_error\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{success_pointer}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalResult.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&success.name()), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.result_type(&success.name(), pointer);
        self.result_types.push((value_type, type_id));
        type_id
    }

    fn task_response_type(&mut self, success: &CompilerType) -> usize {
        let value_type = CompilerType::TaskResponse(Box::new(success.clone()));
        if let Some((_, type_id)) = self
            .result_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Result ({}, ())\", file: !{}, baseType: !{})",
            llvm_string(&success.name()), self.file, self.result_header_pointer_type
        ));
        self.result_types.push((value_type, type_id));
        type_id
    }

    fn traversal_control_type(&mut self, payload: &CompilerType) -> usize {
        let value_type = CompilerType::TraversalControl(Box::new(payload.clone()));
        if let Some((_, type_id)) = self
            .traversal_control_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let payload_type = self.type_id(payload);
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"is_finish\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let payload_member = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"value\", file: !{}, baseType: !{payload_type}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload_member}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalTraversalControl.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&payload.name()), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"TraversalControl {}\", file: !{}, baseType: !{pointer})",
            llvm_string(&payload.name()), self.file
        ));
        self.traversal_control_types.push((value_type, type_id));
        type_id
    }

    fn list_type(&mut self, element: &CompilerType) -> usize {
        let value_type = CompilerType::List(Box::new(element.clone()));
        if let Some((_, type_id)) = self
            .list_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let element_type = self.type_id(element);
        let element_layout = target_value_layout(element);
        let payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"value\", file: !{}, baseType: !{element_type}, size: {}, align: {}, offset: 0)",
            self.file, element_layout.size, element_layout.alignment
        ));
        let next_offset = align_bits(element_layout.size, 64);
        let opaque_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{}, size: 64, align: 64)",
            self.unsigned64_type
        ));
        let next = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"remaining\", file: !{}, baseType: !{opaque_pointer}, size: 64, align: 64, offset: {next_offset})",
            self.file,
        ));
        let members = self.node(format!("!{{!{payload}, !{next}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalList.{}\", file: !{}, size: {}, align: 64, elements: !{members})",
            llvm_string(&element.name()), self.file, next_offset + 64
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"List {}\", file: !{}, baseType: !{pointer})",
            llvm_string(&element.name()),
            self.file
        ));
        self.list_types.push((value_type, type_id));
        type_id
    }

    fn container_type(&mut self, value_type: &CompilerType) -> usize {
        if let Some((_, type_id)) = self
            .container_types
            .iter()
            .find(|(known, _)| known == value_type)
        {
            return *type_id;
        }
        let entry_count = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"entry_count\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let opaque_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{}, size: 64, align: 64)",
            self.unsigned64_type
        ));
        let mut members = vec![entry_count];
        let (entries_offset, storage_size) = if matches!(value_type, CompilerType::Bag(_)) {
            let distinct_count = self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"distinct_count\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
                self.file, self.unsigned64_type
            ));
            members.push(distinct_count);
            (128, 192)
        } else {
            (64, 128)
        };
        let entries = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"entries\", file: !{}, baseType: !{opaque_pointer}, size: 64, align: 64, offset: {entries_offset})",
            self.file
        ));
        members.push(entries);
        let elements = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalContainer.{}\", file: !{}, size: {storage_size}, align: 64, elements: !{elements})",
            llvm_string(&value_type.name()), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{pointer})",
            llvm_string(&value_type.name()),
            self.file
        ));
        self.container_types.push((value_type.clone(), type_id));
        type_id
    }

    fn result_modular_type(&mut self, modular: &CompilerModularType) -> usize {
        if let Some((_, type_id)) = self
            .result_modular_types
            .iter()
            .find(|(known, _)| known == modular)
        {
            return *type_id;
        }
        let modular_type = self.modular_type(modular);
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"is_error\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{modular_type}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalResult.Modular.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&modular.name), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.result_type(&modular.name, pointer);
        self.result_modular_types.push((modular.clone(), type_id));
        type_id
    }

    fn refined_type(&mut self, constraint: &str, base: &CompilerType) -> usize {
        let value_type = CompilerType::Refined {
            constraint: constraint.to_owned(),
            base: Box::new(base.clone()),
        };
        if let Some((_, type_id)) = self
            .refined_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let base_type = self.type_id(base);
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{base_type})",
            llvm_string(constraint),
            self.file
        ));
        self.refined_types.push((value_type, type_id));
        type_id
    }

    fn tuple_type(&mut self, fields: &[CompilerType]) -> usize {
        let value_type = CompilerType::Tuple(fields.to_vec());
        if let Some((_, type_id)) = self
            .tuple_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }

        let mut offset = 0;
        let mut aggregate_alignment = 8;
        let mut members = Vec::with_capacity(fields.len());
        for (index, field) in fields.iter().enumerate() {
            let layout = target_value_layout(field);
            offset = align_bits(offset, layout.alignment);
            aggregate_alignment = aggregate_alignment.max(layout.alignment);
            let field_type = self.type_id(field);
            members.push(self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"_{index}\", file: !{}, baseType: !{field_type}, size: {}, align: {}, offset: {offset})",
                self.file, layout.size, layout.alignment
            )));
            offset += layout.size;
        }
        let size = align_bits(offset, aggregate_alignment);
        let elements = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let type_id = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"{}\", file: !{}, size: {size}, align: {aggregate_alignment}, elements: !{elements})",
            llvm_string(&value_type.name()),
            self.file
        ));
        self.tuple_types.push((value_type, type_id));
        type_id
    }

    fn record_type(&mut self, fields: &[(String, CompilerType)]) -> usize {
        let value_type = CompilerType::Record(fields.to_vec());
        if let Some((_, type_id)) = self
            .record_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }

        let mut offset = 0;
        let mut aggregate_alignment = 8;
        let mut members = Vec::with_capacity(fields.len());
        for (label, field) in fields {
            let layout = target_value_layout(field);
            offset = align_bits(offset, layout.alignment);
            aggregate_alignment = aggregate_alignment.max(layout.alignment);
            let field_type = self.type_id(field);
            members.push(self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"{}\", file: !{}, baseType: !{field_type}, size: {}, align: {}, offset: {offset})",
                llvm_string(label), self.file, layout.size, layout.alignment
            )));
            offset += layout.size;
        }
        for _ in fields {
            offset = align_bits(offset, 32) + 32;
            aggregate_alignment = aggregate_alignment.max(32);
        }
        let size = align_bits(offset, aggregate_alignment);
        debug_assert_eq!(size, target_value_layout(&value_type).size);
        let elements = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let type_id = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"{}\", file: !{}, size: {size}, align: {aggregate_alignment}, elements: !{elements})",
            llvm_string(&value_type.name()),
            self.file
        ));
        self.record_types.push((value_type, type_id));
        type_id
    }

    fn sum_type(&mut self, sum: &CompilerSumType) -> usize {
        let value_type = CompilerType::Sum(sum.clone());
        if let Some((_, type_id)) = self
            .sum_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let enumerators = sum
            .alternatives
            .iter()
            .enumerate()
            .map(|(value, alternative)| {
                self.node(format!(
                    "!DIEnumerator(name: \"{}\", value: {value})",
                    llvm_string(&alternative.name)
                ))
            })
            .collect::<Vec<_>>();
        let tag_values = self.node(format!(
            "!{{{}}}",
            enumerators
                .iter()
                .map(|value| format!("!{value}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let tag_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"{}.alternative\", file: !{}, size: 32, align: 32, elements: !{tag_values})",
            llvm_string(&sum.name),
            self.file
        ));
        let mut members = vec![self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"tag\", file: !{}, baseType: !{tag_type}, size: 32, align: 32, offset: 0)",
            self.file
        ))];
        let mut offset = 32;
        let mut alignment = 32;
        for (index, alternative) in sum.alternatives.iter().enumerate() {
            let Some(payload) = &alternative.payload else {
                continue;
            };
            let layout = target_value_layout(payload);
            offset = align_bits(offset, layout.alignment);
            alignment = alignment.max(layout.alignment);
            let payload_type = self.type_id(payload);
            members.push(self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"payload_{index}\", file: !{}, baseType: !{payload_type}, size: {}, align: {}, offset: {offset})",
                self.file, layout.size, layout.alignment
            )));
            offset += layout.size;
        }
        let size = align_bits(offset, alignment);
        debug_assert_eq!(size, target_value_layout(&value_type).size);
        let elements = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let kind = if sum.positional { "Variant" } else { "Union" };
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"Topal{kind}.{}\", file: !{}, size: {size}, align: {alignment}, elements: !{elements})",
            llvm_string(&sum.name),
            self.file
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{storage})",
            llvm_string(&sum.name),
            self.file
        ));
        self.sum_types.push((value_type, type_id));
        type_id
    }

    fn enum_type(&mut self, enumeration: &CompilerEnumType) -> usize {
        if let Some(value_type) = self.enum_types.get(&enumeration.name) {
            return *value_type;
        }
        let enumerators = enumeration
            .alternatives
            .iter()
            .enumerate()
            .map(|(value, name)| {
                self.node(format!(
                    "!DIEnumerator(name: \"{}\", value: {value})",
                    llvm_string(name)
                ))
            })
            .collect::<Vec<_>>();
        let elements = self.node(format!(
            "!{{{}}}",
            enumerators
                .iter()
                .map(|value| format!("!{value}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let value_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"{}\", file: !{}, size: 32, align: 32, elements: !{elements})",
            llvm_string(&enumeration.name),
            self.file
        ));
        self.enum_types.insert(enumeration.name.clone(), value_type);
        value_type
    }

    fn generator_type(&mut self, generator: &CompilerGeneratorType) -> usize {
        self.enum_type(&generator_debug_enumeration(generator))
    }

    fn subprogram(
        &mut self,
        name: &str,
        linkage_name: &str,
        span: Span,
        result: &CompilerType,
        parameters: &[CompilerType],
    ) -> usize {
        let result_type = if *result == CompilerType::Unit {
            "null".into()
        } else {
            format!("!{}", self.type_id(result))
        };
        let mut types = vec![result_type];
        for parameter in parameters {
            let parameter_type = self.type_id(parameter);
            types.push(format!("!{parameter_type}"));
        }
        let types = self.node(format!("!{{{}}}", types.join(", ")));
        let signature = self.node(format!("!DISubroutineType(types: !{types})"));
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        self.node(format!(
            "distinct !DISubprogram(name: \"{}\", linkageName: \"{}\", scope: !{}, file: !{}, line: {}, type: !{}, scopeLine: {}, spFlags: DISPFlagDefinition, unit: !{})",
            llvm_string(name),
            llvm_string(linkage_name),
            self.file,
            self.file,
            position.line,
            signature,
            position.line,
            self.compile_unit
        ))
    }

    fn lexical_block(&mut self, span: Span, scope: usize) -> usize {
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        self.node(format!(
            "distinct !DILexicalBlock(scope: !{scope}, file: !{}, line: {}, column: {})",
            self.file, position.line, position.column
        ))
    }

    fn parameter(
        &mut self,
        name: &str,
        argument: usize,
        span: Span,
        value_type: &CompilerType,
        scope: usize,
    ) -> usize {
        self.variable(name, Some(argument), span, value_type, scope)
    }

    fn local(&mut self, name: &str, span: Span, value_type: &CompilerType, scope: usize) -> usize {
        self.variable(name, None, span, value_type, scope)
    }

    fn local_with_type_id(
        &mut self,
        name: &str,
        span: Span,
        value_type: usize,
        scope: usize,
    ) -> usize {
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        self.node(format!(
            "!DILocalVariable(name: \"{}\", scope: !{scope}, file: !{}, line: {}, type: !{value_type})",
            llvm_string(name),
            self.file,
            position.line
        ))
    }

    fn variable(
        &mut self,
        name: &str,
        argument: Option<usize>,
        span: Span,
        value_type: &CompilerType,
        scope: usize,
    ) -> usize {
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        let argument = argument.map_or_else(String::new, |value| format!(", arg: {value}"));
        let value_type = self.type_id(value_type);
        self.node(format!(
            "!DILocalVariable(name: \"{}\"{argument}, scope: !{scope}, file: !{}, line: {}, type: !{})",
            llvm_string(name),
            self.file,
            position.line,
            value_type
        ))
    }

    fn location(&mut self, span: Span, scope: usize) -> usize {
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        self.node(format!(
            "!DILocation(line: {}, column: {}, scope: !{scope})",
            position.line, position.column
        ))
    }

    fn finish(&mut self) -> String {
        let dwarf = self.node("!{i32 2, !\"Dwarf Version\", i32 5}".into());
        let debug_version = self.node("!{i32 2, !\"Debug Info Version\", i32 3}".into());
        let ident = self.node(format!("!{{!\"topalc {}\"}}", env!("CARGO_PKG_VERSION")));
        let mut output = format!(
            "!llvm.dbg.cu = !{{!{}}}\n!llvm.module.flags = !{{!{dwarf}, !{debug_version}}}\n!llvm.ident = !{{!{ident}}}\n",
            self.compile_unit
        );
        for (id, node) in self.nodes.iter().enumerate() {
            let _ = writeln!(output, "!{id} = {node}");
        }
        output
    }
}

fn llvm_type(value_type: &CompilerType) -> String {
    match value_type {
        CompilerType::Unit => "void".into(),
        _ => llvm_value_type(value_type),
    }
}

fn function_parameter_is_context_capture(parameter: &CompilerParameter) -> bool {
    parameter.name.starts_with("root ") || parameter.name.starts_with("@ ")
}

fn compiler_type_contains_function(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Function => true,
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_function),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, field)| compiler_type_contains_function(field)),
        CompilerType::Optional(payload) => compiler_type_contains_function(payload),
        CompilerType::Result(success) => compiler_type_contains_function(success),
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_contains_function)
        }),
        _ => false,
    }
}

fn function_llvm_return_type(function: &CompilerFunction) -> String {
    if function.result_captures.is_empty() {
        llvm_type(&function.result_type)
    } else {
        format!(
            "{{ {} }}",
            std::iter::once(llvm_type(&function.result_type))
                .chain(
                    function
                        .result_captures
                        .iter()
                        .map(|capture| llvm_value_type(&capture.value_type))
                )
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Clone, Copy)]
struct TargetValueLayout {
    size: u64,
    alignment: u64,
}

#[allow(clippy::too_many_lines)] // Every semantic representation has an explicit target layout.
fn target_value_layout(value_type: &CompilerType) -> TargetValueLayout {
    match value_type {
        CompilerType::Unit
        | CompilerType::Completed
        | CompilerType::Effect
        | CompilerType::Boolean => TargetValueLayout {
            size: 8,
            alignment: 8,
        },
        CompilerType::Type
        | CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Constraint
        | CompilerType::Comparison
        | CompilerType::ErrorCode
        | CompilerType::Enum(_)
        | CompilerType::Generator(_) => TargetValueLayout {
            size: 32,
            alignment: 32,
        },
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata => {
            unreachable!("static-only compiler values have no target value layout")
        }
        CompilerType::Version
        | CompilerType::SerializationStream(_)
        | CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat
        | CompilerType::InfiniteRational
        | CompilerType::Modular(_)
        | CompilerType::Rational
        | CompilerType::Error
        | CompilerType::ErrorDomain
        | CompilerType::SourceLocation
        | CompilerType::Range(_)
        | CompilerType::Result(_)
        | CompilerType::TaskResponse(_)
        | CompilerType::Optional(_)
        | CompilerType::TraversalControl(_)
        | CompilerType::List(_)
        | CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }
        | CompilerType::Character
        | CompilerType::String
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => TargetValueLayout {
            size: 64,
            alignment: 64,
        },
        CompilerType::Refined { base, .. } => target_value_layout(base),
        CompilerType::Tuple(fields) => {
            let mut size = 0;
            let mut alignment = 8;
            for field in fields {
                let field = target_value_layout(field);
                size = align_bits(size, field.alignment) + field.size;
                alignment = alignment.max(field.alignment);
            }
            TargetValueLayout {
                size: align_bits(size, alignment),
                alignment,
            }
        }
        CompilerType::Record(fields) => {
            let mut size = 0;
            let mut alignment = 8;
            for (_, field) in fields {
                let field = target_value_layout(field);
                size = align_bits(size, field.alignment) + field.size;
                alignment = alignment.max(field.alignment);
            }
            for _ in fields {
                size = align_bits(size, 32) + 32;
                alignment = alignment.max(32);
            }
            TargetValueLayout {
                size: align_bits(size, alignment),
                alignment,
            }
        }
        CompilerType::Sum(sum) => {
            let mut size = 32;
            let mut alignment = 32;
            for payload in sum
                .alternatives
                .iter()
                .filter_map(|alternative| alternative.payload.as_ref())
            {
                let payload = target_value_layout(payload);
                size = align_bits(size, payload.alignment) + payload.size;
                alignment = alignment.max(payload.alignment);
            }
            TargetValueLayout {
                size: align_bits(size, alignment),
                alignment,
            }
        }
    }
}

fn compiler_int_string_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Character | CompilerType::String
            ])
    )
}

fn compiler_int_list_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::List(element),
            ] if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat)
                || compiler_integer_pair(element.as_ref()))
    )
}

fn compiler_string_int_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::String, CompilerType::Int]
    )
}

fn compiler_string_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::String, CompilerType::String]
    )
}

fn compiler_integer_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Int | CompilerType::Nat
            ])
    )
}

fn compiler_rational_nat_pair(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Tuple(fields)
        if fields.as_slice() == [CompilerType::Rational, CompilerType::Nat])
}

fn compiler_rational_pair(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Tuple(fields)
        if fields.as_slice() == [CompilerType::Rational, CompilerType::Rational])
}

fn compiler_list_integer_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::List(element), CompilerType::Int | CompilerType::Nat]
                if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat)
                    || compiler_nested_int_list_element(element.as_ref()))
    )
}
