/// Private owned-store adaptation. Ordinary source borrows a published Record;
/// only a checked copy into the inactive bank survives invocation reclamation.
fn emit_retained_identity_prelude(unit:&crate::systems_numeric_library::CheckedNumericLibrary)->String {
    let mut bridge=format!(r#"define {{ i64, i64 }} @{}(i64 %cursor, ptr %store, i64 %owner, i64 %code) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {{
entry:
  %cursor.zero = icmp eq i64 %cursor, 0
  %cursor.minimum = icmp uge i64 %cursor, 2
  %cursor.maximum = icmp ule i64 %cursor, 2147483647
  %cursor.range = and i1 %cursor.minimum, %cursor.maximum
  %cursor.valid = or i1 %cursor.zero, %cursor.range
  %store.address = ptrtoint ptr %store to i64
  %store.end = add i64 %store.address, 4096
  %store.nonnull = icmp ne ptr %store, null
  %store.nonwrapping = icmp ugt i64 %store.end, %store.address
  %store.alignment = and i64 %store.address, 4095
  %store.aligned = icmp eq i64 %store.alignment, 0
  %owner.nonzero = icmp ne i64 %owner, 0
  %code.nonzero = icmp ne i64 %code, 0
  %entry.a = and i1 %cursor.valid, %store.nonnull
  %entry.b = and i1 %store.nonwrapping, %store.aligned
  %entry.c = and i1 %owner.nonzero, %code.nonzero
  %entry.d = and i1 %entry.a, %entry.b
  %entry.valid = and i1 %entry.d, %entry.c
  br i1 %entry.valid, label %header, label %invalid
header:
"#,unit.policy.native_export());
    for (name,offset) in [("owner",0),("code",8),("initialized",16),("bank",24),("epoch",32),
        ("prepared",40),("prepared_bank",48),("prepared_next",56)] {
        let _=writeln!(bridge,"  %stored.{name}.address = getelementptr i8, ptr %store, i64 {offset}\n  %stored.{name} = load i64, ptr %stored.{name}.address, align 8");
    }
    bridge.push_str(r#"  %owner.matches = icmp eq i64 %stored.owner, %owner
  %code.matches = icmp eq i64 %stored.code, %code
  %initialized.valid = icmp ule i64 %stored.initialized, 1
  %bank.valid = icmp ule i64 %stored.bank, 1
  %epoch.available = icmp ult i64 %stored.epoch, 18446744073709551615
  %prepared.clear = icmp eq i64 %stored.prepared, 0
  %prepared.bank.clear = icmp eq i64 %stored.prepared_bank, 0
  %prepared.next.clear = icmp eq i64 %stored.prepared_next, 0
"#);
    let mut previous="owner.matches".to_owned();
    for (i,check) in ["code.matches","initialized.valid","bank.valid","epoch.available",
        "prepared.clear","prepared.bank.clear","prepared.next.clear"].iter().enumerate() {
        let current=format!("header.valid.{i}");
        let _=writeln!(bridge,"  %{current} = and i1 %{previous}, %{check}");
        previous=current;
    }
    let _=writeln!(bridge,"  br i1 %{previous}, label %banks, label %invalid");
    bridge.push_str(r#"banks:
  %bank.zero = icmp eq i64 %stored.bank, 0
  %bank.first = getelementptr i8, ptr %store, i64 64
  %bank.second = getelementptr i8, ptr %store, i64 2080
  %published.bank = select i1 %bank.zero, ptr %bank.first, ptr %bank.second
  %prepared.bank = select i1 %bank.zero, ptr %bank.second, ptr %bank.first
  %prepared.index = xor i64 %stored.bank, 1
"#);
    emit_retained_zero_scan(&mut bridge,"inactive","prepared.bank",0,252,"banks","inactive.clean");
    bridge
}

fn emit_retained_identity_bridge(unit:&crate::systems_numeric_library::CheckedNumericLibrary)->String {
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
    bridge.push_str(r#"  %initial.number = call ptr @topal.runtime.int.from.u64(i64 0)
  %initial.value = insertvalue { ptr, i32 } undef, ptr %initial.number, 0
  %initial.state = insertvalue { ptr, i32 } %initial.value, i32 0, 1
  br label %invoke
published.guard:
  %epoch.nonzero = icmp ne i64 %stored.epoch, 0
  %published.valid = and i1 %epoch.nonzero, %cursor.range
  br i1 %published.valid, label %published.graph, label %invalid
published.graph:
  %published.pointer = load ptr, ptr %published.bank, align 8
  %published.node = getelementptr i8, ptr %published.bank, i64 16
  %pointer.valid = icmp eq ptr %published.pointer, %published.node
  %published.order.address = getelementptr i8, ptr %published.bank, i64 8
  %published.order = load i64, ptr %published.order.address, align 8
  %published.order.valid = icmp eq i64 %published.order, 0
  %published.sign = load i64, ptr %published.node, align 8
  %sign.valid = icmp eq i64 %published.sign, 0
  %published.length.address = getelementptr i8, ptr %published.node, i64 8
  %published.length = load i64, ptr %published.length.address, align 8
  %length.valid = icmp eq i64 %published.length, 1
  %published.data.address = getelementptr i8, ptr %published.node, i64 16
  %published.data = load i64, ptr %published.data.address, align 8
  %value.matches = icmp eq i64 %published.data, %cursor
  %graph.a = and i1 %pointer.valid, %published.order.valid
  %graph.b = and i1 %sign.valid, %length.valid
  %graph.c = and i1 %graph.a, %graph.b
  %graph.valid = and i1 %graph.c, %value.matches
  br i1 %graph.valid, label %published.tail, label %invalid
published.tail:
"#);
    emit_retained_zero_scan(&mut bridge,"tail","published.bank",40,247,"published.tail","published.view");
    bridge.push_str(r#"  %published.state = load { ptr, i32 }, ptr %published.bank, align 8
  br label %invoke
invoke:
  %state = phi { ptr, i32 } [ %initial.state, %initial.view ], [ %published.state, %published.view ]
"#);
    bridge.push_str(&emit_identity_value_adaptation(unit));
    bridge.push_str(r#"  %next.node = getelementptr i8, ptr %prepared.bank, i64 16
  store ptr %next.node, ptr %prepared.bank, align 8
  %next.order.address = getelementptr i8, ptr %prepared.bank, i64 8
  store i64 0, ptr %next.order.address, align 8
  store i64 0, ptr %next.node, align 8
  %next.length.address = getelementptr i8, ptr %next.node, i64 8
  store i64 1, ptr %next.length.address, align 8
  %next.data.address = getelementptr i8, ptr %next.node, i64 16
  store i64 %next.value, ptr %next.data.address, align 8
  store i64 %prepared.index, ptr %stored.prepared_bank.address, align 8
  store i64 %next.value, ptr %stored.prepared_next.address, align 8
  store i64 1, ptr %stored.prepared.address, align 8
"#);
    bridge.push_str(identity_scalar_return());
    bridge
}

/// Each loop scans a fixed bounded owned extent before any graph reference can
/// be used. The provider separately proves the store's complete live ownership.
fn emit_retained_zero_scan(bridge:&mut String,name:&str,base:&str,offset:u64,words:u64,from:&str,to:&str) {
    let _=writeln!(bridge,r#"  br label %{name}.scan
{name}.scan:
  %{name}.index = phi i64 [ 0, %{from} ], [ %{name}.next, %{name}.advance ]
  %{name}.scaled = mul i64 %{name}.index, 8
  %{name}.offset = add i64 %{name}.scaled, {offset}
  %{name}.address = getelementptr i8, ptr %{base}, i64 %{name}.offset
  %{name}.word = load i64, ptr %{name}.address, align 8
  %{name}.zero = icmp eq i64 %{name}.word, 0
  br i1 %{name}.zero, label %{name}.advance, label %invalid
{name}.advance:
  %{name}.next = add i64 %{name}.index, 1
  %{name}.more = icmp ult i64 %{name}.next, {words}
  br i1 %{name}.more, label %{name}.scan, label %{to}
{to}:"#);
}
