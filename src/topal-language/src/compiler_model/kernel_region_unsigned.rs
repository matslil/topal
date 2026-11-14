// Static field encodings in the existing qualified systems member namespace.
impl Analyzer {
    fn analyze_kernel_region_unsigned_member(
        &mut self,
        receiver: &Expression,
        member: Span,
        rest: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        use topal_semantics::{
            RegionLibraryByteOrder as Order, RegionLibraryUnsignedFormat as Format,
        };
        let store = match self.source.slice(member) {
            "load" => false,
            "store" => true,
            _ => {
                return Err(unsupported(
                    &self.source,
                    member,
                    "unknown kernel-region unsigned member",
                ));
            }
        };
        let [Expression::Product { fields, .. }] = rest else {
            return Err(unsupported(
                &self.source,
                span,
                "unsigned field requires named facts",
            ));
        };
        let names = if store {
            &["offset-bytes", "byte-count", "byte-order", "value"][..]
        } else {
            &["offset-bytes", "byte-count", "byte-order"][..]
        };
        if fields.len() != names.len()
            || fields.iter().any(|field| field.label.is_none())
            || names.iter().any(|name| {
                fields
                    .iter()
                    .filter(|field| self.source.slice(field.label.unwrap()) == *name)
                    .count()
                    != 1
            })
        {
            return Err(unsupported(&self.source, span, "unsigned field fact set"));
        }
        let selected = |label: &str| {
            fields
                .iter()
                .find(|field| self.source.slice(field.label.unwrap()) == label)
                .unwrap()
        };
        let count = selected("byte-count");
        let Expression::Integer(literal) = &count.value else {
            return Err(unsupported(
                &self.source,
                count.value.span(),
                "static unsigned byte-count literal",
            ));
        };
        let count = parse_integer(self.source.slice(*literal))
            .and_then(|value| (1..=8_u8).find(|width| value == BigInt::from(*width)))
            .ok_or_else(|| {
                unsupported(
                    &self.source,
                    *literal,
                    "unsigned byte-count from one through eight",
                )
            })?;
        let order = selected("byte-order");
        let Expression::Identifier(literal) = &order.value else {
            return Err(unsupported(
                &self.source,
                order.value.span(),
                "static unsigned byte-order",
            ));
        };
        let order = match self.source.slice(*literal) {
            "little-endian" => Order::LittleEndian,
            "big-endian" => Order::BigEndian,
            _ => return Err(unsupported(&self.source, *literal, "unsigned byte-order")),
        };
        let format = Format::new(count, order).expect("checked static field format");
        let region = Box::new(self.analyze_expression(receiver, environment)?);
        let mut arguments = Vec::new();
        let mut argument_roles = Vec::new();
        for field in fields {
            let label = self.source.slice(field.label.unwrap());
            let role = match label {
                "offset-bytes" => 0,
                "value" => 1,
                _ => continue,
            };
            let value = self.analyze_expression(&field.value, environment)?;
            let value = adapt_function_call_argument(&CompilerType::Nat, &value)
                .or_else(|| self.adapt_proven_nonnegative_nat_argument(&CompilerType::Nat, &value))
                .ok_or_else(|| {
                    unsupported(
                        &self.source,
                        field.value.span(),
                        "unsigned field fact must be Nat",
                    )
                })?;
            arguments.push(value);
            argument_roles.push(role);
        }
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::KernelRegion {
                operation: if store {
                    CompilerKernelRegionOperation::UnsignedStore(format)
                } else {
                    CompilerKernelRegionOperation::UnsignedLoad(format)
                },
                region,
                arguments,
                argument_roles,
            },
            value_type: if store {
                CompilerType::Boolean
            } else {
                CompilerType::Optional(Box::new(CompilerType::Nat))
            },
            int_range: None,
            rational_value: None,
            span,
        }))
    }
}
