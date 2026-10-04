use super::*;

fn declaration(name: &str, ordinal: usize) -> DeclarationIdentity {
    DeclarationIdentity {
        module: "example".into(),
        name: name.into(),
        ordinal,
    }
}

#[test]
fn predicate_is_the_only_subkind() {
    assert!(ObjectKind::Predicate.satisfies(ObjectKind::Function));
    assert!(!ObjectKind::Type.satisfies(ObjectKind::Value));
    assert!(!ObjectKind::Function.satisfies(ObjectKind::Predicate));
}

#[test]
fn nominal_and_structural_identity_remain_distinct() {
    let shape = TypeIdentity::Structural(StructuralType::Tuple(vec![TypeIdentity::Fundamental(
        "Int",
    )]));
    let nominal = TypeIdentity::Nominal {
        declaration: declaration("Coordinate", 0),
        parameters: vec![shape.clone()],
    };
    assert_ne!(shape, nominal);
}

#[test]
fn effect_union_is_canonical_and_idempotent() {
    let io = QualifiedName(vec!["lang".into(), "io".into()]);
    let clock = QualifiedName(vec!["lang".into(), "clock".into()]);
    let left = EffectSet::from_effects([io.clone(), clock.clone()]);
    let right = EffectSet::from_effects([io]);
    assert_eq!(left.union(&right), left);
    assert_eq!(left.iter().count(), 2);
}

#[test]
fn declarations_retain_source_order() {
    let mut scope = SemanticScope::default();
    scope.declare(
        "choose",
        Declaration {
            identity: declaration("choose", 0),
            value: "first",
        },
    );
    scope.declare(
        "choose",
        Declaration {
            identity: declaration("choose", 1),
            value: "second",
        },
    );
    assert_eq!(scope.candidates("choose")[0].value, "first");
}

#[test]
fn language_versions_expand_and_order_canonically() {
    assert_eq!("v0.1".parse(), Ok(LanguageVersion::DESIGN_0));
    assert_eq!(LanguageVersion::DESIGN_0.to_string(), "v0.1");
    assert!("v0.2".parse::<LanguageVersion>().unwrap() > LanguageVersion::DESIGN_0);
}

#[test]
fn generic_patterns_preserve_exact_substitutions() {
    let pattern = TypePattern::Record(vec![
        ("first".into(), TypePattern::Parameter("T".into())),
        ("second".into(), TypePattern::Parameter("T".into())),
    ]);
    let nominal = TypeIdentity::Nominal {
        declaration: declaration("UserId", 2),
        parameters: Vec::new(),
    };
    let arguments = BTreeMap::from([("T".into(), nominal.clone())]);
    assert_eq!(
        pattern.instantiate(&arguments),
        Some(TypeIdentity::Structural(StructuralType::Record(vec![
            ("first".into(), nominal.clone()),
            ("second".into(), nominal),
        ])))
    );
    assert!(pattern.instantiate(&BTreeMap::new()).is_none());
}

#[test]
fn capability_evidence_is_coherent_per_exact_subject() {
    let subject = TypeIdentity::Fundamental("Int");
    let capability = QualifiedName(vec!["lang".into(), "Equality".into()]);
    let evidence = CapabilityEvidence {
        capability: capability.clone(),
        subject: subject.clone(),
        roles: BTreeMap::from([("equal".into(), declaration("equal-int", 0))]),
    };
    let mut set = CapabilitySet::default();
    assert_eq!(set.insert(evidence.clone()), Ok(()));
    assert_eq!(set.insert(evidence.clone()), Ok(()));
    assert_eq!(set.select(&capability, &subject), Some(&evidence));

    let conflicting = CapabilityEvidence {
        roles: BTreeMap::from([("equal".into(), declaration("other-equal", 1))]),
        ..evidence
    };
    assert!(set.insert(conflicting).is_err());
}

#[test]
fn interface_implementation_requires_exact_roles() {
    let shape = InterfaceShape {
        identity: declaration("Parser", 0),
        operations: BTreeMap::from([(
            "parse".into(),
            InterfaceOperation::Function {
                inputs: vec![TypeIdentity::Fundamental("String")],
                result: TypeIdentity::Fundamental("Boolean"),
            },
        )]),
    };
    let complete = InterfaceImplementation {
        interface: shape.clone(),
        operations: BTreeMap::from([("parse".into(), declaration("parse", 1))]),
    };
    assert_eq!(complete.validate(), Ok(()));
    assert!(
        InterfaceImplementation {
            interface: shape,
            operations: BTreeMap::new(),
        }
        .validate()
        .is_err()
    );
}

#[test]
fn effect_rows_compose_and_check_containment_canonically() {
    let read = QualifiedName(vec!["app".into(), "read".into()]);
    let write = QualifiedName(vec!["app".into(), "write".into()]);
    let left = EffectRow {
        known: EffectSet::from_effects([read.clone()]),
        tail: Some("E".into()),
    };
    let right = EffectRow {
        known: EffectSet::from_effects([write.clone()]),
        tail: Some("E".into()),
    };
    let composed = left.compose(&right).unwrap();
    assert_eq!(composed.known.iter().count(), 2);
    assert!(left.is_contained_by(&composed));
    assert!(right.is_contained_by(&composed));
    assert!(
        left.compose(&EffectRow {
            known: EffectSet::default(),
            tail: Some("Other".into()),
        })
        .is_none()
    );
}

#[test]
fn resource_moves_retain_one_obligation_and_cleanup_once() {
    let first = ResourceIdentity(QualifiedName(vec!["resource".into(), "first".into()]));
    let second = ResourceIdentity(QualifiedName(vec!["resource".into(), "second".into()]));
    let mut tracker = ResourceTracker::default();
    tracker.declare(first.clone(), "input", 1).unwrap();
    tracker.declare(second.clone(), "other", 1).unwrap();
    tracker.move_to(&first, "input", "output").unwrap();
    assert!(tracker.move_to(&first, "input", "again").is_err());
    assert!(matches!(
        tracker.state(&first),
        Some(ResourceState::Owned { binding, .. }) if binding == "output"
    ));
    assert_eq!(tracker.destroy_all(), vec![second.clone(), first.clone()]);
    assert!(tracker.destroy_all().is_empty());
    assert_eq!(tracker.state(&first), Some(&ResourceState::Destroyed));
}

#[test]
fn locations_separate_resources_and_validate_events() {
    let resource = ResourceIdentity(QualifiedName(vec!["memory".into(), "buffer".into()]));
    let location = Location {
        resource: resource.clone(),
        base: 16,
        size: 8,
        layout_size: 8,
        alignment: 8,
        rights: AccessRights {
            read: true,
            write: false,
        },
        lifetime: 1,
        access_widths: BTreeSet::from([1, 2, 4, 8]),
    };
    assert_eq!(location.validate(), Ok(()));
    assert_eq!(
        MemoryEvent::Read {
            location: location.clone(),
            width: 4,
        }
        .validate(),
        Ok(())
    );
    assert!(
        MemoryEvent::Write {
            location: location.clone(),
            width: 4,
            value_identity: "value".into(),
        }
        .validate()
        .is_err()
    );
    let overlapping_other_resource = Location {
        resource: ResourceIdentity(QualifiedName(vec!["memory".into(), "other".into()])),
        ..location.clone()
    };
    assert!(!location.aliases(&overlapping_other_resource));
    assert!(location.aliases(&Location {
        base: 20,
        size: 4,
        layout_size: 4,
        alignment: 4,
        ..location.clone()
    }));
}

#[test]
fn memory_execution_requires_order_for_conflicts() {
    let location = Location {
        resource: ResourceIdentity(QualifiedName(vec!["memory".into(), "shared".into()])),
        base: 0,
        size: 4,
        layout_size: 4,
        alignment: 4,
        rights: AccessRights {
            read: true,
            write: true,
        },
        lifetime: 1,
        access_widths: BTreeSet::from([4]),
    };
    let mut execution = MemoryExecution::default();
    let write = execution
        .record(MemoryEvent::Write {
            location: location.clone(),
            width: 4,
            value_identity: "one".into(),
        })
        .unwrap();
    let read = execution
        .record(MemoryEvent::Read { location, width: 4 })
        .unwrap();
    assert!(execution.validate_race_free().is_err());
    execution.order_before(write, read).unwrap();
    assert_eq!(execution.validate_race_free(), Ok(()));
    assert!(execution.order_before(read, write).is_err());
}

#[test]
fn task_cancellation_closes_children_before_parent() {
    let mut scheduler = TaskScheduler::default();
    let parent = scheduler.construct(None).unwrap();
    scheduler.start(parent).unwrap();
    let child = scheduler.construct(Some(parent)).unwrap();
    scheduler.start(child).unwrap();
    scheduler.cancel(parent).unwrap();
    assert_eq!(scheduler.state(child), Some(&TaskState::Closing));
    assert!(scheduler.acknowledge_closed(parent).is_err());
    scheduler.acknowledge_closed(child).unwrap();
    scheduler.acknowledge_closed(parent).unwrap();
    assert_eq!(scheduler.state(parent), Some(&TaskState::Completed));
}

#[test]
fn requests_reply_once_and_streams_preserve_order() {
    let endpoint = QualifiedName(vec!["service".into(), "query".into()]);
    let mut ledger = MessageLedger::default();
    let request = ledger
        .send(
            MessageSend {
                sender: TaskIdentity(1),
                receiver: TaskIdentity(2),
                endpoint: endpoint.clone(),
                kind: InteractionKind::Request,
                payload_identity: "question".into(),
            },
            1,
            &AdmissionPolicy::Reject,
        )
        .unwrap();
    assert!(
        ledger
            .send(
                MessageSend {
                    sender: TaskIdentity(1),
                    receiver: TaskIdentity(2),
                    endpoint: endpoint.clone(),
                    kind: InteractionKind::Request,
                    payload_identity: "second".into(),
                },
                1,
                &AdmissionPolicy::Reject,
            )
            .is_err()
    );
    ledger.receive(request).unwrap();
    ledger.reply(request, "answer").unwrap();
    assert!(ledger.reply(request, "duplicate").is_err());

    let stream = ledger
        .send(
            MessageSend {
                sender: TaskIdentity(1),
                receiver: TaskIdentity(2),
                endpoint,
                kind: InteractionKind::Stream,
                payload_identity: "range".into(),
            },
            1,
            &AdmissionPolicy::Wait,
        )
        .unwrap();
    ledger.receive(stream).unwrap();
    ledger.yield_stream(stream, "one").unwrap();
    ledger.yield_stream(stream, "two").unwrap();
    assert!(matches!(
        &ledger.transaction(stream).unwrap().state,
        TransactionState::Streaming { values } if values == &["one", "two"]
    ));
    ledger.close(stream).unwrap();
}

#[test]
fn dependency_schedules_are_canonical_and_reject_internal_cycles() {
    let first = DependencyNode::Task(TaskIdentity(1));
    let second = DependencyNode::Task(TaskIdentity(2));
    let transaction = DependencyNode::Transaction(3);
    let mut graph = DependencyGraph::default();
    for node in [second.clone(), transaction.clone(), first.clone()] {
        graph.add_node(node);
    }
    graph.depends_on(&first, &transaction).unwrap();
    graph.depends_on(&second, &transaction).unwrap();
    assert_eq!(
        graph.schedule().unwrap().order,
        vec![first.clone(), second.clone(), transaction.clone()]
    );
    graph.depends_on(&transaction, &first).unwrap();
    assert!(graph.schedule().is_err());

    let external = DependencyNode::External(QualifiedName(vec!["network".into()]));
    let mut suspended = DependencyGraph::default();
    suspended.add_node(first.clone());
    suspended.add_node(external.clone());
    suspended.depends_on(&first, &external).unwrap();
    suspended.depends_on(&external, &first).unwrap();
    assert!(suspended.schedule().is_ok());
}

#[test]
fn recursive_layouts_validate_structure_and_boundaries() {
    let octet = Layout::Scalar {
        bits: 8,
        signed: false,
        byte_order: ByteOrder::Little,
    };
    let packet = Layout::Product {
        fields: vec![
            ("tag".into(), octet.clone()),
            (
                "name".into(),
                Layout::Text {
                    code_unit_bits: 8,
                    byte_order: ByteOrder::Little,
                    length_prefix: true,
                    terminator: None,
                },
            ),
        ],
        packing: Packing::Packed,
    };
    assert_eq!(packet.validate(), Ok(()));
    assert!(
        Layout::Product {
            fields: vec![("same".into(), octet.clone()), ("same".into(), octet)],
            packing: Packing::Natural,
        }
        .validate()
        .is_err()
    );
    assert!(
        Layout::Sequence {
            element: Box::new(packet),
            count: None,
        }
        .validate()
        .is_err()
    );
}

#[test]
fn layout_codecs_check_rights_endian_and_malformed_input() {
    let scalar = Layout::Scalar {
        bits: 16,
        signed: true,
        byte_order: ByteOrder::Big,
    };
    let read_write = AccessRights {
        read: true,
        write: true,
    };
    let bytes = scalar.write(&LayoutValue::Integer(-2), read_write).unwrap();
    assert_eq!(bytes, [0xff, 0xfe]);
    assert_eq!(
        scalar.read(&bytes, read_write),
        Ok(LayoutValue::Integer(-2))
    );
    assert!(scalar.read(&bytes[..1], read_write).is_err());
    assert!(
        scalar
            .write(
                &LayoutValue::Integer(1),
                AccessRights {
                    read: true,
                    write: false,
                },
            )
            .is_err()
    );

    let text = Layout::Text {
        code_unit_bits: 8,
        byte_order: ByteOrder::Little,
        length_prefix: true,
        terminator: None,
    };
    let encoded = text
        .write(&LayoutValue::Text("å".into()), read_write)
        .unwrap();
    assert_eq!(
        text.read(&encoded, read_write),
        Ok(LayoutValue::Text("å".into()))
    );
    assert!(text.read(&[2, 0xff, 0xff], read_write).is_err());
}

#[test]
fn compiler_memory_choices_refine_unobservable_semantics() {
    let refinement = SynchronizationRefinement {
        strategy: CompilerSynchronization::MessageQueue,
        source_visible: false,
        preserves_happens_before: true,
        preserves_coherence: true,
    };
    assert_eq!(refinement.validate(), Ok(()));
    assert!(
        SynchronizationRefinement {
            source_visible: true,
            ..refinement
        }
        .validate()
        .is_err()
    );

    let policy = HardwareAccessPolicy {
        volatile: true,
        widths: BTreeSet::from([8, 16]),
        ordering: QualifiedName(vec!["device".into(), "ordered".into()]),
    };
    assert_eq!(policy.validate(), Ok(()));
    let observations = vec![ObservableMemoryOutcome::Effect("write device".into())];
    assert_eq!(validate_optimization(&observations, &observations), Ok(()));
    assert!(validate_optimization(&observations, &[]).is_err());
}

#[test]
fn capability_trust_and_existential_elimination_preserve_evidence() {
    assert_eq!(
        admit_evidence(EvidenceTrust::Verified, SafetyObligation::RaceFreedom),
        Ok(())
    );
    assert!(
        admit_evidence(
            EvidenceTrust::TrustedUnverified,
            SafetyObligation::MemorySafety
        )
        .is_err()
    );
    assert!(admit_evidence(EvidenceTrust::Refuted, SafetyObligation::OrdinaryLaw).is_err());

    let package = ExistentialPackage::pack("private witness", 42);
    assert_eq!(
        package.eliminate(|witness, value| (witness.len(), *value)),
        (15, 42)
    );
}
