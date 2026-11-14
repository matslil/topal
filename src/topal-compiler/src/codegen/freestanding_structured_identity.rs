/// Whole ordinary state transport. All stored offsets come from the same
/// checked description as ordinary Record layout and native graph validation.
fn emit_structured_identity_bridge(
    unit:&crate::systems_numeric_library::CheckedNumericLibrary,
) -> Result<String,crate::CompileError> {
    let layout=crate::systems_retained_layout::RetainedValueLayout::identity(true)?;
    let state_type=&layout.llvm_type;
    let result_type=llvm_value_type(&crate::systems_retained_layout::structured_result_type());
    let mut bridge=emit_retained_identity_prelude(unit);
    bridge.push_str(r#"  %uninitialized = icmp eq i64 %stored.initialized, 0
  br i1 %uninitialized, label %initial.guard, label %published.guard
initial.guard:
  %epoch.zero = icmp eq i64 %stored.epoch, 0
  %initial.a = and i1 %epoch.zero, %bank.zero
  %initial.valid = and i1 %initial.a, %cursor.zero
  br i1 %initial.valid, label %initial.bank, label %invalid
initial.bank:
"#);
    emit_retained_zero_scan(&mut bridge,"initial","published.bank",0,252,"initial.bank","initial.view");
    emit_structured_initial_value(&mut bridge,&layout);
    bridge.push_str(r#"  br label %invoke
published.guard:
  %epoch.nonzero = icmp ne i64 %stored.epoch, 0
  %published.valid = and i1 %epoch.nonzero, %cursor.range
  br i1 %published.valid, label %published.graph, label %invalid
published.graph:
"#);
    emit_structured_input_validation(&mut bridge,&layout);
    emit_retained_zero_scan(&mut bridge,"tail","published.bank",layout.graph_bytes,
        (2016-layout.graph_bytes)/8,"published.tail","published.view");
    let _=writeln!(bridge,"  %published.state = load {state_type}, ptr %published.bank, align 8\n  br label %invoke\ninvoke:\n  %state = phi {state_type} [ %initial.state, %initial.view ], [ %published.state, %published.view ]");
    let _=writeln!(bridge,"  %result = call fastcc {result_type} @{}{}({state_type} %state)",
        unit.policy.function_prefix(),unit.entry_symbol);
    let _=writeln!(bridge,"  %source.state = extractvalue {result_type} %result, 1");
    let mut checks=Vec::new();
    let mut outer=Vec::new();
    for index in 0..2 {
        let value=format!("%source.result.order.{index}");
        let _=writeln!(bridge,"  {value} = extractvalue {result_type} %result, {}",index+2);
        outer.push(value);
    }
    emit_structured_order_checks(&mut bridge,"source.outer",&outer,&mut checks);
    for (record_index,record) in layout.orders.iter().enumerate() {
        let mut values=Vec::new();
        for index in 0..record.fields.len() {
            let mut indices=record.indices.clone();indices.push((record.fields.len()+index) as u32);
            let value=format!("%source.state.order.{record_index}.{index}");
            let _=writeln!(bridge,"  {value} = extractvalue {state_type} %source.state, {}",structured_indices(&indices));
            values.push(value);
        }
        emit_structured_order_checks(&mut bridge,&format!("source.state.order.{record_index}"),&values,&mut checks);
    }
    emit_structured_guard(&mut bridge,"source.order",&checks,"source.numbers");
    bridge.push_str("source.numbers:\n");
    let _=writeln!(bridge,"  %allocated.object = extractvalue {result_type} %result, 0");
    emit_structured_number_result(&mut bridge,"allocated",2,i32::MAX as u64);
    for number in &layout.numbers {
        let name=if number.path==["next"] {"next"} else {"creations"};
        let _=writeln!(bridge,"  %{name}.object = extractvalue {state_type} %source.state, {}",structured_indices(&number.indices));
        emit_structured_number_result(&mut bridge,name,number.minimum,number.maximum);
    }
    for (index,boolean) in layout.booleans.iter().enumerate() {
        let _=writeln!(bridge,"  %source.boolean.{index} = extractvalue {state_type} %source.state, {}\n  %source.boolean.byte.{index} = zext i1 %source.boolean.{index} to i8",structured_indices(&boolean.indices));
    }
    // No prepared store is touched before every actual source result is valid.
    for (index,number) in layout.numbers.iter().enumerate() {
        let name=if number.path==["next"] {"next"} else {"creations"};
        let _=writeln!(bridge,r#"  %prepared.number.{index}.node = getelementptr i8, ptr %prepared.bank, i64 {}
  %prepared.number.{index}.field = getelementptr i8, ptr %prepared.bank, i64 {}
  store ptr %prepared.number.{index}.node, ptr %prepared.number.{index}.field, align 8
  store i64 0, ptr %prepared.number.{index}.node, align 8
  %prepared.number.{index}.length = getelementptr i8, ptr %prepared.number.{index}.node, i64 8
  store i64 %{name}.length, ptr %prepared.number.{index}.length, align 8
  %prepared.number.{index}.data = getelementptr i8, ptr %prepared.number.{index}.node, i64 16
  store i64 %{name}.value, ptr %prepared.number.{index}.data, align 8"#,number.node_offset,number.pointer_offset);
    }
    for (index,boolean) in layout.booleans.iter().enumerate() {
        let _=writeln!(bridge,"  %prepared.boolean.{index} = getelementptr i8, ptr %prepared.bank, i64 {}\n  store i8 %source.boolean.byte.{index}, ptr %prepared.boolean.{index}, align 1",boolean.offset);
    }
    for (record_index,record) in layout.orders.iter().enumerate() {
        for (index,offset) in record.offsets.iter().enumerate() {
            let _=writeln!(bridge,"  %prepared.order.{record_index}.{index} = getelementptr i8, ptr %prepared.bank, i64 {offset}\n  store i32 %source.state.order.{record_index}.{index}, ptr %prepared.order.{record_index}.{index}, align 4");
        }
    }
    bridge.push_str("  store i64 %prepared.index, ptr %stored.prepared_bank.address, align 8\n  store i64 %next.value, ptr %stored.prepared_next.address, align 8\n  store i64 1, ptr %stored.prepared.address, align 8\n");
    bridge.push_str(identity_scalar_return());
    Ok(bridge)
}

fn structured_indices(indices:&[u32])->String {
    indices.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")
}
fn emit_structured_guard(bridge:&mut String,name:&str,checks:&[String],target:&str) {
    assert!(!checks.is_empty());
    let mut previous=checks[0].clone();
    for (index,check) in checks.iter().enumerate().skip(1) {
        let current=format!("%{name}.combined.{index}");
        let _=writeln!(bridge,"  {current} = and i1 {previous}, {check}");
        previous=current;
    }
    let _=writeln!(bridge,"  br i1 {previous}, label %{target}, label %invalid");
}
fn emit_structured_order_checks(bridge:&mut String,name:&str,values:&[String],checks:&mut Vec<String>) {
    for (index,value) in values.iter().enumerate() {
        let check=format!("%{name}.range.{index}");
        let _=writeln!(bridge,"  {check} = icmp ult i32 {value}, {}",values.len());
        checks.push(check);
        for (other,previous) in values.iter().enumerate().take(index) {
            let check=format!("%{name}.distinct.{index}.{other}");
            let _=writeln!(bridge,"  {check} = icmp ne i32 {value}, {previous}");
            checks.push(check);
        }
    }
}
fn emit_structured_initial_value(bridge:&mut String,layout:&crate::systems_retained_layout::RetainedValueLayout) {
    let mut insertions=Vec::new();
    for (index,number) in layout.numbers.iter().enumerate() {
        let value=format!("%initial.number.{index}");
        let _=writeln!(bridge,"  {value} = call ptr @topal.runtime.int.from.u64(i64 0)");
        insertions.push(("ptr",value,number.indices.clone()));
    }
    for boolean in &layout.booleans {insertions.push(("i1","false".into(),boolean.indices.clone()));}
    for record in &layout.orders {
        for index in 0..record.fields.len() {
            let mut indices=record.indices.clone();indices.push((record.fields.len()+index) as u32);
            insertions.push(("i32",index.to_string(),indices));
        }
    }
    let mut previous="undef".to_owned();
    for (index,(ty,value,indices)) in insertions.iter().enumerate() {
        let current=if index+1==insertions.len() {"%initial.state".to_owned()} else {format!("%initial.field.{index}")};
        let _=writeln!(bridge,"  {current} = insertvalue {} {previous}, {ty} {value}, {}",layout.llvm_type,structured_indices(indices));
        previous=current;
    }
}
fn emit_structured_input_validation(bridge:&mut String,layout:&crate::systems_retained_layout::RetainedValueLayout) {
    for index in 0..layout.graph_bytes/8 {
        let _=writeln!(bridge,"  %published.word.{index}.address = getelementptr i8, ptr %published.bank, i64 {}\n  %published.word.{index} = load i64, ptr %published.word.{index}.address, align 8",index*8);
    }
    let mut checks=Vec::new();
    for (index,number) in layout.numbers.iter().enumerate() {
        let p=format!("input.number.{index}");
        let pointer=number.pointer_offset/8;
        let sign=number.node_offset/8;
        let length=sign+1;
        let data=sign+2;
        let _=writeln!(bridge,r#"  %{p}.node = getelementptr i8, ptr %published.bank, i64 {}
  %{p}.address = ptrtoint ptr %{p}.node to i64
  %{p}.pointer = icmp eq i64 %published.word.{pointer}, %{p}.address
  %{p}.sign = icmp eq i64 %published.word.{sign}, 0
  %{p}.one = icmp eq i64 %published.word.{length}, 1
  %{p}.two = icmp eq i64 %published.word.{length}, 2
  %{p}.low = icmp ule i64 %published.word.{data}, 4294967295
  %{p}.high = icmp ugt i64 %published.word.{data}, 4294967295
  %{p}.single = and i1 %{p}.one, %{p}.low
  %{p}.double = and i1 %{p}.two, %{p}.high
  %{p}.normal = or i1 %{p}.single, %{p}.double
  %{p}.minimum = icmp uge i64 %published.word.{data}, {}
  %{p}.maximum = icmp ule i64 %published.word.{data}, {}"#,number.node_offset,number.minimum,number.input_maximum);
        for field in ["pointer","sign","normal","minimum","maximum"] {checks.push(format!("%{p}.{field}"));}
        if number.path==["next"] {
            let _=writeln!(bridge,"  %input.cursor.matches = icmp eq i64 %published.word.{data}, %cursor");
            checks.push("%input.cursor.matches".into());
        }
    }
    for (index,mask) in layout.zero_masks.iter().enumerate() {
        let _=writeln!(bridge,"  %input.padding.{index}.bits = and i64 %published.word.{}, {}\n  %input.padding.{index}.clean = icmp eq i64 %input.padding.{index}.bits, 0",mask.offset/8,mask.mask);
        checks.push(format!("%input.padding.{index}.clean"));
    }
    for (record_index,record) in layout.orders.iter().enumerate() {
        let mut values=Vec::new();
        for (index,offset) in record.offsets.iter().enumerate() {
            let value=format!("%input.order.{record_index}.{index}");
            let word=format!("%published.word.{}",offset/8);
            if offset%8==4 {
                let _=writeln!(bridge,"  {value}.shifted = lshr i64 {word}, 32\n  {value} = trunc i64 {value}.shifted to i32");
            } else {let _=writeln!(bridge,"  {value} = trunc i64 {word} to i32");}
            values.push(value);
        }
        emit_structured_order_checks(bridge,&format!("input.order.{record_index}"),&values,&mut checks);
    }
    emit_structured_guard(bridge,"input.graph",&checks,"published.tail");
    bridge.push_str("published.tail:\n");
}
fn emit_structured_number_result(bridge:&mut String,name:&str,minimum:u64,maximum:u64) {
    let limbs=if maximum<=u32::MAX as u64 {1}else{2};
    let _=writeln!(bridge,r#"  %{name}.sign.address = getelementptr i8, ptr %{name}.object, i64 0
  %{name}.length.address = getelementptr i8, ptr %{name}.object, i64 8
  %{name}.sign = load i64, ptr %{name}.sign.address, align 8
  %{name}.length = load i64, ptr %{name}.length.address, align 8
  %{name}.positive = icmp eq i64 %{name}.sign, 0
  %{name}.nonempty = icmp uge i64 %{name}.length, 1
  %{name}.bounded = icmp ule i64 %{name}.length, {limbs}
  %{name}.shape.a = and i1 %{name}.positive, %{name}.nonempty
  %{name}.shape = and i1 %{name}.shape.a, %{name}.bounded
  br i1 %{name}.shape, label %{name}.low, label %invalid
{name}.low:
  %{name}.low.address = getelementptr i8, ptr %{name}.object, i64 16
  %{name}.low.word = load i32, ptr %{name}.low.address, align 4
  %{name}.low.value = zext i32 %{name}.low.word to i64
  %{name}.single = icmp eq i64 %{name}.length, 1
  br i1 %{name}.single, label %{name}.check, label %{name}.high
{name}.high:
  %{name}.high.address = getelementptr i8, ptr %{name}.object, i64 20
  %{name}.high.word = load i32, ptr %{name}.high.address, align 4
  %{name}.high.nonzero = icmp ne i32 %{name}.high.word, 0
  br i1 %{name}.high.nonzero, label %{name}.high.valid, label %invalid
{name}.high.valid:
  %{name}.high.value = zext i32 %{name}.high.word to i64
  %{name}.shifted = shl i64 %{name}.high.value, 32
  %{name}.combined = or i64 %{name}.low.value, %{name}.shifted
  br label %{name}.check
{name}.check:
  %{name}.value = phi i64 [ %{name}.low.value, %{name}.low ], [ %{name}.combined, %{name}.high.valid ]
  %{name}.minimum = icmp uge i64 %{name}.value, {minimum}
  %{name}.maximum = icmp ule i64 %{name}.value, {maximum}
  %{name}.range = and i1 %{name}.minimum, %{name}.maximum
  br i1 %{name}.range, label %{name}.ready, label %invalid
{name}.ready:"#);
}
