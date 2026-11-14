#[test]
#[allow(clippy::too_many_lines)] // The complete transition artifact trace is one structural audit.
fn publishes_one_closed_freestanding_artifact_directory() {
    // TOPAL-COMP-SYSTEMS-ARTIFACT-001, TOPAL-COMP-SYSTEMS-TEST-001.
    let tools = LlvmTools::discover(None).unwrap();
    let parent = std::env::temp_dir().join(format!(
        "topal-systems-artifact-test-{}-{}",
        std::process::id(),
        NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&parent).unwrap();
    let destination = parent.join("published");
    let published = publish_x86_64_systems_artifact(&program(), &tools, &destination).unwrap();
    assert_eq!(
        fs::read_dir(&destination).unwrap().count(),
        5,
        "only canonical outputs are published"
    );
    assert!(published.kernel.is_file());
    assert!(published.debug.is_file());
    assert!(published.map.is_file());
    assert!(published.initial_user_image.is_file());
    assert!(published.static_user_image.is_none());
    assert!(published.provenance.is_file());
    let encoded = fs::read(&published.provenance).unwrap();
    let mut decoded: SystemsArtifactProvenance = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, published.record);
    assert_eq!(decoded.schema, X86_SYSTEMS_ARTIFACT_REVISION);
    assert_eq!(decoded.target, "x86_64-unknown-none");
    assert_eq!(decoded.outputs.len(), 4);
    assert_eq!(decoded.placements.len(), 65);
    assert_eq!(decoded.bootstrap_storage_capacity, 65_536);
    assert_eq!(decoded.bootstrap_storage_alignment, 4096);
    assert_eq!(decoded.semantic_trace.len(), 483);
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE) })
    );
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE)
    }));
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| entry.starts_with(SYSTEMS_STATIC_IMAGE_PROCESS_ENTER))
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY) })
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY) })
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY) })
    );
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT)
    }));
    assert!(
        decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT)
        })
    );
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE)
    }));
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA) })
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD) })
    );
    assert!(
        decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING)
        })
    );
    assert!(
        decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF)
        })
    );
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY)
    }));
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ) })
    );
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE)
    }));
    assert!(
        decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM)
        })
    );
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME)
    }));
    assert!(decoded.semantic_trace.iter().any(|entry| {
        entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT)
    }));
    assert!(
        decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ)
        })
    );
    assert!(
        decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY)
        })
    );
    assert!(
        decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF)
        })
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD) })
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS) })
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD) })
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| entry.starts_with(SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM))
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| entry.starts_with(SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE))
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| entry.starts_with(SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS))
    );
    assert!(
        decoded
            .semantic_trace
            .iter()
            .any(|entry| entry.starts_with(SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS))
    );
    assert!(decoded.semantic_trace[27].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[28].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(decoded.semantic_trace[29], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[30].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[31].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[32].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[33].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[34],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE
    );
    assert!(decoded.semantic_trace[35].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[36],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD
    );
    assert!(decoded.semantic_trace[37].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[38],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD
    );
    assert!(decoded.semantic_trace[39].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[40],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS
    );
    assert!(decoded.semantic_trace[41].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[42],
        "topal.systems.frames.allocate/1:frames=5:alignment=1"
    );
    assert!(decoded.semantic_trace[43].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[44],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE
    );
    assert!(decoded.semantic_trace[45].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[46],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE
    );
    assert!(decoded.semantic_trace[47].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[48],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE
    );
    assert!(decoded.semantic_trace[49].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[50],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY
    );
    assert!(decoded.semantic_trace[51].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[52],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT
    );
    assert!(decoded.semantic_trace[53].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[54],
        "topal.systems.frames.allocate/1:frames=2:alignment=1"
    );
    assert!(decoded.semantic_trace[55].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[56],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING
    );
    assert!(decoded.semantic_trace[57].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[58],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF
    );
    assert!(decoded.semantic_trace[59].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[60],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY
    );
    assert!(decoded.semantic_trace[61].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[62],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ
    );
    assert!(decoded.semantic_trace[63].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[64],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT
    );
    assert!(decoded.semantic_trace[65].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[66],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS
    );
    assert!(decoded.semantic_trace[67].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[68],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME
    );
    assert!(decoded.semantic_trace[69].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[70],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS
    );
    assert!(decoded.semantic_trace[71].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[72],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE
    );
    assert!(decoded.semantic_trace[73].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[74],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE
    );
    assert!(decoded.semantic_trace[75].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[76],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM
    );
    assert!(decoded.semantic_trace[77].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[78],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES
    );
    assert!(decoded.semantic_trace[79].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[80],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA
    );
    assert!(decoded.semantic_trace[81].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[82],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT
    );
    assert!(decoded.semantic_trace[83].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[84],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS
    );
    assert!(decoded.semantic_trace[85].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[86],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK
    );
    assert!(decoded.semantic_trace[87].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[88],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF
    );
    assert!(decoded.semantic_trace[89].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[90],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION
    );
    assert!(decoded.semantic_trace[91].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[92],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE
    );
    assert!(decoded.semantic_trace[93].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[94],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT
    );
    assert!(decoded.semantic_trace[95].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[96],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION
    );
    assert!(decoded.semantic_trace[97].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[98],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE
    );
    assert!(decoded.semantic_trace[99].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[100],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER
    );
    assert!(decoded.semantic_trace[101].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[102],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS
    );
    assert!(decoded.semantic_trace[103].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[104],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING
    );
    assert!(decoded.semantic_trace[105].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[106],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION
    );
    assert!(decoded.semantic_trace[107].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[108],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES
    );
    assert!(decoded.semantic_trace[109].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[110],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION
    );
    assert!(decoded.semantic_trace[111].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[112],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION
    );
    assert!(decoded.semantic_trace[113].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[114],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE
    );
    assert!(decoded.semantic_trace[115].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[116],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY
    );
    assert!(decoded.semantic_trace[117].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[118],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE
    );
    assert!(decoded.semantic_trace[119].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[120],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE
    );
    assert!(decoded.semantic_trace[121].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[122],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM
    );
    assert!(decoded.semantic_trace[123].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[124],
        SYSTEMS_STATIC_IMAGE_PROCESS_ENTER
    );
    assert!(decoded.semantic_trace[125].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[126].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[127].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[128].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[129].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[130].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[131].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[132].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[133].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[134].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[135].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[136].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[137].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[138].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[139].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[140].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[141].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[142].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[143].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[144],
        SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY
    );
    assert!(decoded.semantic_trace[145].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[146],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM
    );
    assert!(decoded.semantic_trace[147].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[148],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION
    );
    assert!(decoded.semantic_trace[149].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[150],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF
    );
    assert!(decoded.semantic_trace[151].starts_with(SYSTEMS_CONSOLE_WRITE));
    decoded.semantic_trace.drain(88..152);
    assert_eq!(
        decoded.semantic_trace[88],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS
    );
    assert!(decoded.semantic_trace[89].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[90],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT
    );
    assert!(decoded.semantic_trace[91].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[92],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA
    );
    assert!(decoded.semantic_trace[93].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[94],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES
    );
    assert!(decoded.semantic_trace[95].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[96],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM
    );
    assert!(decoded.semantic_trace[97].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[98],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE
    );
    assert!(decoded.semantic_trace[99].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[100],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE
    );
    assert!(decoded.semantic_trace[101].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[102],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS
    );
    assert!(decoded.semantic_trace[103].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[104],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME
    );
    assert!(decoded.semantic_trace[105].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[106],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS
    );
    assert!(decoded.semantic_trace[107].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[108],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT
    );
    assert!(decoded.semantic_trace[109].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[110],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ
    );
    assert!(decoded.semantic_trace[111].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[112],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY
    );
    assert!(decoded.semantic_trace[113].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[114],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF
    );
    assert!(decoded.semantic_trace[115].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(decoded.semantic_trace[116], SYSTEMS_FRAMES_RELEASE);
    assert_eq!(
        decoded.semantic_trace[117],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT
    );
    assert!(decoded.semantic_trace[118].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[119],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY
    );
    assert!(decoded.semantic_trace[120].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[121],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE
    );
    assert!(decoded.semantic_trace[122].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[123],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY
    );
    assert_eq!(decoded.semantic_trace[124], SYSTEMS_FRAMES_RELEASE);
    assert_eq!(
        decoded.semantic_trace[125],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA
    );
    assert_eq!(decoded.semantic_trace[126], SYSTEMS_FRAMES_RELEASE);
    assert_eq!(
        decoded.semantic_trace[127],
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD
    );
    assert_eq!(decoded.semantic_trace[128], SYSTEMS_FRAMES_RELEASE);
    decoded.semantic_trace.drain(27..129);
    for (index, request) in CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
        .iter()
        .enumerate()
    {
        assert_eq!(
            decoded.semantic_trace[31 + index],
            format!(
                "{SYSTEMS_PCI_CONFIGURATION_READ_DWORD}:transaction={}:segment=0:bus=0:device=5:function=0:offset={}:expected={:#010x}",
                index + 1,
                request.register_offset_bytes,
                request.expected_value
            )
        );
    }
    assert_eq!(
        decoded.semantic_trace[55],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_PCI_VIRTIO_RNG_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[56],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_PCI_CAPABILITIES_OK")
        )
    );
    decoded.semantic_trace.drain(31..57);
    for (index, request) in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
        .iter()
        .enumerate()
    {
        let identity = match request.direction {
            CompilerPciRegionRegisterDirection::ReadEquals => SYSTEMS_PCI_REGION_READ,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                SYSTEMS_PCI_REGION_READ_EVENTUALLY
            }
            CompilerPciRegionRegisterDirection::Write => SYSTEMS_PCI_REGION_WRITE,
        };
        assert_eq!(
            decoded.semantic_trace[31 + index],
            format!(
                "{identity}:transaction={}:segment=0:bus=0:device=5:function=0:access-capability-offset=132:bar=4:region-offset=0:region-length=4096:register-offset={}:width={}:value={:#010x}",
                index + 1,
                request.register_offset_bytes,
                request.width_bytes,
                request.value
            )
        );
    }
    assert_eq!(
        decoded.semantic_trace[47],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_PCI_NEGOTIATED")
        )
    );
    decoded.semantic_trace.drain(31..48);
    for (index, request) in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
        .iter()
        .enumerate()
    {
        let identity = match request.direction {
            CompilerPciRegionRegisterDirection::ReadEquals => SYSTEMS_PCI_REGION_READ,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                SYSTEMS_PCI_REGION_READ_EVENTUALLY
            }
            CompilerPciRegionRegisterDirection::Write => SYSTEMS_PCI_REGION_WRITE,
        };
        assert_eq!(
            decoded.semantic_trace[31 + index],
            format!(
                "{identity}:transaction={}:segment=0:bus=0:device=5:function=0:access-capability-offset=132:bar=4:region-offset=0:region-length=4096:register-offset={}:width={}:value={:#010x}",
                index + 17,
                request.register_offset_bytes,
                request.width_bytes,
                request.value
            )
        );
    }
    assert_eq!(
        decoded.semantic_trace[36],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_QUEUE_DISCOVERED")
        )
    );
    decoded.semantic_trace.drain(31..37);
    assert_eq!(
        decoded.semantic_trace[31],
        format!(
            "{SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD}:transaction=1:segment=0:bus=0:device=5:function=0:offset=4:mask=0x0004:expected=0x0000:desired=0x0004"
        )
    );
    assert_eq!(
        decoded.semantic_trace[32],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_PCI_BUS_MASTER_OK")
        )
    );
    decoded.semantic_trace.drain(31..33);
    assert_eq!(
        decoded.semantic_trace[31],
        format!(
            "{SYSTEMS_DMA_BUFFER_CREATE}:buffer=1:device=1:domain=1:bytes=4096:alignment=4096:direction=bidirectional:visibility=provider-managed:role=virtqueue-control"
        )
    );
    assert_eq!(
        decoded.semantic_trace[32],
        format!(
            "{SYSTEMS_DMA_BUFFER_CREATE}:buffer=2:device=1:domain=1:bytes=4096:alignment=4096:direction=device-to-memory:visibility=provider-managed:role=entropy-payload"
        )
    );
    assert_eq!(
        decoded.semantic_trace[33],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_DMA_BUFFERS_OK")
        )
    );
    decoded.semantic_trace.drain(31..34);
    assert_eq!(
        decoded.semantic_trace[31],
        format!(
            "{SYSTEMS_DMA_BUFFER_BIND_REFERENCE}:destination-buffer=1:destination-offset=0:referenced-buffer=2:referenced-offset=0:width=8"
        )
    );
    for (index, request) in CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING
        .iter()
        .enumerate()
    {
        assert_eq!(
            decoded.semantic_trace[32 + index],
            format!(
                "{SYSTEMS_DMA_BUFFER_STORE_INTEGER}:buffer={}:offset={}:width={}:value={}",
                request.buffer_identity, request.offset_bytes, request.width_bytes, request.value,
            )
        );
    }
    assert_eq!(
        decoded.semantic_trace[36],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_SPLIT_RING_OK")
        )
    );
    decoded.semantic_trace.drain(31..37);
    for (index, request) in CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG
        .iter()
        .enumerate()
    {
        let direction = match request.direction {
            CompilerDmaDirection::Bidirectional => "bidirectional",
            CompilerDmaDirection::DeviceToMemory => "device-to-memory",
        };
        let role = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => "virtqueue-control",
            CompilerDmaBufferRole::EntropyPayload => "entropy-payload",
        };
        assert_eq!(
            decoded.semantic_trace[31 + index],
            format!(
                "{SYSTEMS_DMA_BUFFER_PREPARE_DEVICE}:buffer={}:device={}:domain={}:direction={direction}:visibility=provider-managed:role={role}",
                request.buffer_identity,
                request.device_identity,
                request.translation_domain_identity,
            )
        );
    }
    assert_eq!(
        decoded.semantic_trace[33],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_DMA_DEVICE_OWNED")
        )
    );
    decoded.semantic_trace.drain(31..34);
    for (index, request) in
        CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
            .iter()
            .enumerate()
    {
        assert_eq!(
            decoded.semantic_trace[31 + index],
            format!(
                "{SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE}:transaction={}:segment=0:bus=0:device=5:function=0:access-capability-offset=132:bar=4:region-offset=0:region-length=4096:register-offset={}:width=8:buffer=1:buffer-offset={}:device-identity=1:domain=1",
                index + 22,
                request.register_offset_bytes,
                request.buffer_offset_bytes,
            )
        );
    }
    assert_eq!(
        decoded.semantic_trace[34],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_QUEUE_ADDRESSES_OK")
        )
    );
    decoded.semantic_trace.drain(31..35);
    assert!(decoded.semantic_trace[118].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[119].starts_with(SYSTEMS_USER_ADDRESS_SPACE_PREPARE));
    assert!(decoded.semantic_trace[120].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[121].starts_with(SYSTEMS_USER_ADDRESS_SPACE_MAP));
    assert!(decoded.semantic_trace[122].starts_with(SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE));
    assert!(decoded.semantic_trace[123].starts_with(SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY));
    assert!(decoded.semantic_trace[124].starts_with(SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE));
    assert!(decoded.semantic_trace[125].starts_with(SYSTEMS_USER_MAPPING_PERMIT_EXECUTE));
    assert!(decoded.semantic_trace[126].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[127].starts_with(SYSTEMS_USER_ADDRESS_SPACE_MAP));
    assert!(decoded.semantic_trace[128].starts_with(SYSTEMS_USER_MAPPING_STORE_BYTE));
    assert!(decoded.semantic_trace[129].starts_with(SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY));
    assert!(decoded.semantic_trace[130].starts_with(SYSTEMS_USER_MAPPING_LOAD_BYTE));
    assert!(decoded.semantic_trace[131].starts_with(SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA));
    assert!(decoded.semantic_trace[132].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[133].starts_with(SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS));
    assert_eq!(
        decoded.semantic_trace[134],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_READY")
        )
    );
    assert_eq!(
        decoded.semantic_trace[135],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_BOOT_RESOURCE_OK")
        )
    );
    assert!(decoded.semantic_trace[136].starts_with(SYSTEMS_USER_CONTEXT_PREPARE));
    assert_eq!(
        decoded.semantic_trace[201],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[202],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_GETRANDOM_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[203],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[204],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[205],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[206],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[207],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_READ_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[208],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_CLOSE_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[209],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_DUP_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[210],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_FCNTL_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[211],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_FSTAT_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[212],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_TCGETS_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[213],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_WINSIZE_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[214],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_LSEEK_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[218],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_ENTRY_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[219],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_STACK_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[220],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[221],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[222],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[223],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[224],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_ELF_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[225],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[226],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[227],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[228],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[229],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[230],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_BSS_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[231],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_DATA_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[232],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[233],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[234],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[235],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK")
        )
    );
    assert_eq!(
        decoded.semantic_trace[236],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_RETAINED")
        )
    );
    assert!(decoded.semantic_trace[237].starts_with(SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS));
    assert_eq!(decoded.semantic_trace[238], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[239].starts_with(SYSTEMS_USER_ADDRESS_SPACE_UNMAP));
    assert_eq!(decoded.semantic_trace[240], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[241].starts_with(SYSTEMS_USER_ADDRESS_SPACE_UNMAP));
    assert_eq!(decoded.semantic_trace[242], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[243].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[244].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[246].starts_with(SYSTEMS_USER_ADDRESS_SPACE_RECLAIM));
    assert_eq!(decoded.semantic_trace[247], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[248].starts_with(SYSTEMS_CONSOLE_WRITE));
    for (index, request) in
        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
            .iter()
            .enumerate()
    {
        let identity = match request.direction {
            CompilerPciRegionRegisterDirection::ReadEquals => SYSTEMS_PCI_REGION_READ,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                SYSTEMS_PCI_REGION_READ_EVENTUALLY
            }
            CompilerPciRegionRegisterDirection::Write => SYSTEMS_PCI_REGION_WRITE,
        };
        assert_eq!(
            decoded.semantic_trace[137 + index],
            format!(
                "{identity}:transaction={}:segment=0:bus=0:device=5:function=0:access-capability-offset=132:bar=4:region-offset=0:region-length=4096:register-offset={}:width={}:value={:#010x}",
                index + 25,
                request.register_offset_bytes,
                request.width_bytes,
                request.value
            )
        );
    }
    assert_eq!(
        decoded.semantic_trace[141],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE")
        )
    );
    let notification =
        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION[0];
    assert_eq!(
        decoded.semantic_trace[142],
        format!(
            "{SYSTEMS_PCI_REGION_WRITE}:transaction=29:segment=0:bus=0:device=5:function=0:access-capability-offset={}:bar=4:region-offset=12288:region-length=4096:register-offset=0:width=2:value={:#010x}",
            notification.access_capability_offset_bytes, notification.value
        )
    );
    assert_eq!(
        decoded.semantic_trace[143],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED")
        )
    );
    let completion_signal =
        CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL[0];
    assert_eq!(
        decoded.semantic_trace[144],
        format!(
            "{SYSTEMS_PCI_REGION_READ_EVENTUALLY}:transaction=30:segment=0:bus=0:device=5:function=0:access-capability-offset={}:bar=4:region-offset=4096:region-length=4096:register-offset=0:width=1:value={:#010x}",
            completion_signal.access_capability_offset_bytes, completion_signal.value
        )
    );
    assert_eq!(
        decoded.semantic_trace[145],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL")
        )
    );
    assert!(decoded.semantic_trace[152].starts_with(SYSTEMS_DMA_BUFFER_COMPLETE_CPU));
    assert_eq!(
        decoded.semantic_trace[153],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED")
        )
    );
    assert_eq!(
        decoded.semantic_trace[154],
        format!(
            "{SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA}:sample=1:buffer=2:offset=0:bytes=32:device=1:domain=1:direction=device-to-memory:visibility=provider-managed:role=entropy-payload:credited-entropy-bits=0"
        )
    );
    assert_eq!(
        decoded.semantic_trace[155],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED")
        )
    );
    assert_eq!(
        decoded.semantic_trace[156],
        format!(
            "{SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY}:sample=1:source=1:trust-evidence=1:bytes=32:quality-parts-per-1024=1024:credited-entropy-bits=256"
        )
    );
    assert_eq!(
        decoded.semantic_trace[157],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED")
        )
    );
    assert_eq!(
        decoded.semantic_trace[158],
        format!(
            "{SYSTEMS_ENTROPY_POOL_CREATE}:pool=1:credit-capacity-bits=256:initial-credited-entropy-bits=0"
        )
    );
    assert_eq!(
        decoded.semantic_trace[159],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_ENTROPY_POOL_CREATED")
        )
    );
    assert_eq!(
        decoded.semantic_trace[160],
        format!(
            "{SYSTEMS_ENTROPY_POOL_MIX_SAMPLE}:pool=1:sample=1:conditioner=1:contribution=1:bytes=32:contribution-entropy-bits=256"
        )
    );
    assert_eq!(
        decoded.semantic_trace[161],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_ENTROPY_POOL_MIXED")
        )
    );
    assert_eq!(
        decoded.semantic_trace[162],
        format!(
            "{SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION}:pool=1:contribution=1:conditioner-origin=1:prior-credited-entropy-bits=0:contribution-entropy-bits=256:credit-capacity-bits=256:resulting-credited-entropy-bits=256"
        )
    );
    assert_eq!(
        decoded.semantic_trace[163],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_ENTROPY_POOL_CREDITED")
        )
    );
    assert_eq!(
        decoded.semantic_trace[164],
        format!(
            "{SYSTEMS_ENTROPY_GENERATOR_INITIALIZE}:pool=1:generator=1:algorithm=1:generation=1:key-bytes=32:required-entropy-bits=256:observed-pool-entropy-bits=256"
        )
    );
    assert_eq!(
        decoded.semantic_trace[165],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_ENTROPY_GENERATOR_READY")
        )
    );
    assert_eq!(
        decoded.semantic_trace[166],
        format!(
            "{SYSTEMS_ENTROPY_GENERATOR_EXTRACT}:generator=1:algorithm=1:prior-generation=1:resulting-generation=2:output=1:block-bytes=64:output-offset=32:output-bytes=32:counter=0:nonce=0"
        )
    );
    assert_eq!(
        decoded.semantic_trace[167],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_ENTROPY_OUTPUT_READY")
        )
    );
    assert!(
        decoded.semantic_trace[168].starts_with(SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM)
    );
    assert_eq!(
        decoded.semantic_trace[169],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_RANDOM_OK")
        )
    );
    decoded.semantic_trace[137..249].rotate_left(33);
    decoded.semantic_trace.drain(213..216);
    decoded.semantic_trace.drain(171..206);
    decoded.semantic_trace.drain(120..136);
    decoded.semantic_trace.drain(118..120);
    assert!(decoded.semantic_trace[0].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_PROVISION));
    assert_eq!(decoded.semantic_trace[1], "topal.systems.entry.bootstrap/1");
    assert_eq!(decoded.semantic_trace[2], SYSTEMS_BOOT_MEMORY_DESCRIBE);
    assert_eq!(decoded.semantic_trace[3], SYSTEMS_FRAME_ALLOCATOR_CREATE);
    assert!(decoded.semantic_trace[4].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[5].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[6].starts_with(SYSTEMS_KERNEL_MAP));
    assert!(decoded.semantic_trace[7].starts_with(SYSTEMS_KERNEL_MAPPING_STORE_BYTE));
    assert!(decoded.semantic_trace[8].starts_with(SYSTEMS_KERNEL_MAPPING_LOAD_BYTE));
    assert_eq!(decoded.semantic_trace[9], SYSTEMS_KERNEL_UNMAP);
    assert!(decoded.semantic_trace[10].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(decoded.semantic_trace[11], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[12].starts_with(SYSTEMS_TRANSLATION_BEGIN));
    assert_eq!(decoded.semantic_trace[13], SYSTEMS_TRANSLATION_COMMIT);
    assert_eq!(decoded.semantic_trace[14], SYSTEMS_TRANSLATION_ACTIVATE);
    assert!(decoded.semantic_trace[15].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[16].starts_with(SYSTEMS_FRAMES_ALLOCATE));
    assert!(decoded.semantic_trace[17].starts_with(SYSTEMS_TRANSLATION_EDIT_BEGIN));
    assert!(decoded.semantic_trace[18].starts_with(SYSTEMS_TRANSLATION_EDIT_MAP));
    assert!(decoded.semantic_trace[19].starts_with(SYSTEMS_TRANSLATION_EDIT_COMMIT));
    assert!(decoded.semantic_trace[20].starts_with(SYSTEMS_KERNEL_MAPPING_STORE_BYTE));
    assert!(decoded.semantic_trace[21].starts_with(SYSTEMS_KERNEL_MAPPING_LOAD_BYTE));
    assert!(decoded.semantic_trace[22].starts_with(SYSTEMS_TRANSLATION_EDIT_BEGIN));
    assert_eq!(decoded.semantic_trace[23], SYSTEMS_TRANSLATION_EDIT_UNMAP);
    assert!(decoded.semantic_trace[24].starts_with(SYSTEMS_TRANSLATION_EDIT_COMMIT));
    assert!(decoded.semantic_trace[25].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(decoded.semantic_trace[26], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[27].starts_with(SYSTEMS_CRITICAL_ENTER));
    assert!(decoded.semantic_trace[28].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[29].starts_with(SYSTEMS_CRITICAL_RESTORE));
    assert!(decoded.semantic_trace[30].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[31].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(decoded.semantic_trace[32], SYSTEMS_DEBUG_BREAK);
    assert_eq!(
        decoded.semantic_trace[33],
        "topal.systems.entry.synchronous.debug-break/1"
    );
    assert_eq!(decoded.semantic_trace[34], SYSTEMS_RESUME_DEBUG_BREAK);
    assert!(decoded.semantic_trace[35].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[36].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
    assert!(decoded.semantic_trace[37].starts_with(SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE));
    assert!(decoded.semantic_trace[38].starts_with(SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE));
    assert!(decoded.semantic_trace[39].starts_with(SYSTEMS_ATOMIC_WORD_CREATE));
    assert!(decoded.semantic_trace[40].starts_with(SYSTEMS_ATOMIC_COMPARE_EXCHANGE));
    assert!(decoded.semantic_trace[41].starts_with(SYSTEMS_ATOMIC_LOAD));
    assert_eq!(decoded.semantic_trace[42], SYSTEMS_ATOMIC_END);
    assert!(decoded.semantic_trace[43].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[44].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[45],
        SYSTEMS_BOOTSTRAP_STORAGE_RELEASE
    );
    assert!(decoded.semantic_trace[46].starts_with(SYSTEMS_LOCAL_NOTIFICATION_SEND));
    assert!(decoded.semantic_trace[47].starts_with(SYSTEMS_LOCAL_NOTIFICATION_WAIT));
    assert_eq!(
        decoded.semantic_trace[48],
        "topal.systems.observation.local-notification/1"
    );
    assert_eq!(
        decoded.semantic_trace[49],
        "topal.systems.entry.external.local-notification/1"
    );
    assert!(decoded.semantic_trace[50].starts_with(SYSTEMS_LOCAL_NOTIFICATION_COMPLETE));
    assert_eq!(
        decoded.semantic_trace[51],
        SYSTEMS_RESUME_LOCAL_NOTIFICATION
    );
    assert!(decoded.semantic_trace[52].starts_with(SYSTEMS_LOCAL_NOTIFICATION_WAIT));
    assert!(decoded.semantic_trace[53].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(decoded.semantic_trace[54], SYSTEMS_MONOTONIC_CLOCK_NOW);
    assert_eq!(decoded.semantic_trace[55], SYSTEMS_MONOTONIC_CLOCK_NOW);
    assert!(decoded.semantic_trace[56].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[57].starts_with(SYSTEMS_DEADLINE_AFTER));
    assert!(decoded.semantic_trace[58].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
    assert!(decoded.semantic_trace[59].starts_with(SYSTEMS_KERNEL_COMPLETION_CREATE));
    assert!(decoded.semantic_trace[60].starts_with(SYSTEMS_KERNEL_CONTEXT_CREATE));
    assert!(decoded.semantic_trace[61].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
    assert!(decoded.semantic_trace[62].starts_with(SYSTEMS_KERNEL_CONTEXT_CREATE));
    assert!(decoded.semantic_trace[63].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
    assert!(decoded.semantic_trace[64].starts_with(SYSTEMS_KERNEL_CONTEXT_CREATE));
    assert!(decoded.semantic_trace[65].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE));
    assert!(decoded.semantic_trace[66].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
    assert!(decoded.semantic_trace[67].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
    assert!(decoded.semantic_trace[68].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
    assert!(decoded.semantic_trace[69].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
    assert_eq!(decoded.semantic_trace[70], SYSTEMS_DEADLINE_ARM);
    assert!(decoded.semantic_trace[71].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE));
    assert!(decoded.semantic_trace[72].starts_with("topal.systems.entry.resumed.kernel-thread/1"));
    assert!(decoded.semantic_trace[73].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(
        decoded.semantic_trace[74].starts_with(SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION)
    );
    assert_eq!(
        decoded.semantic_trace[75],
        "topal.systems.observation.deadline/1"
    );
    assert_eq!(
        decoded.semantic_trace[76],
        "topal.systems.entry.external.deadline/1"
    );
    assert_eq!(decoded.semantic_trace[77], SYSTEMS_DEADLINE_COMPLETE);
    assert!(decoded.semantic_trace[78].starts_with(SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT));
    assert!(decoded.semantic_trace[79].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE));
    assert!(decoded.semantic_trace[80].starts_with(SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED));
    assert!(decoded.semantic_trace[81].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[82].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
    assert!(decoded.semantic_trace[83].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
    assert!(decoded.semantic_trace[84].starts_with(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE));
    assert!(decoded.semantic_trace[85].starts_with("topal.systems.entry.resumed.kernel-thread/1"));
    assert!(decoded.semantic_trace[86].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[87].starts_with(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE));
    assert!(decoded.semantic_trace[88].starts_with(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE));
    assert!(decoded.semantic_trace[89].starts_with(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE));
    assert!(decoded.semantic_trace[90].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
    assert!(decoded.semantic_trace[91].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
    assert!(decoded.semantic_trace[92].starts_with("topal.systems.entry.resumed.kernel-thread/1"));
    assert!(decoded.semantic_trace[93].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[94].starts_with(SYSTEMS_KERNEL_CONTEXT_RETIRE));
    assert!(decoded.semantic_trace[95].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
    assert!(decoded.semantic_trace[96].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[97].starts_with(SYSTEMS_KERNEL_CONTEXT_RECLAIM));
    assert!(decoded.semantic_trace[98].starts_with(SYSTEMS_KERNEL_COMPLETION_SIGNAL));
    assert!(decoded.semantic_trace[99].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
    assert!(decoded.semantic_trace[100].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
    assert!(decoded.semantic_trace[101].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
    assert!(decoded.semantic_trace[102].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
    assert!(decoded.semantic_trace[103].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[104].starts_with(SYSTEMS_KERNEL_CONTEXT_RETIRE));
    assert!(decoded.semantic_trace[105].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
    assert!(decoded.semantic_trace[106].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[107].starts_with(SYSTEMS_KERNEL_CONTEXT_RECLAIM));
    assert!(decoded.semantic_trace[108].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
    assert!(decoded.semantic_trace[109].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
    assert!(decoded.semantic_trace[110].starts_with(SYSTEMS_KERNEL_COMPLETION_SIGNAL));
    assert!(decoded.semantic_trace[111].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[112].starts_with(SYSTEMS_KERNEL_CONTEXT_RETIRE));
    assert!(decoded.semantic_trace[113].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
    assert!(decoded.semantic_trace[114].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[115].starts_with(SYSTEMS_KERNEL_CONTEXT_RECLAIM));
    assert!(decoded.semantic_trace[116].starts_with(SYSTEMS_KERNEL_COMPLETION_CONSUME));
    assert_eq!(
        decoded.semantic_trace[117],
        SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME
    );
    assert!(decoded.semantic_trace[118].starts_with(SYSTEMS_USER_CONTEXT_PREPARE));
    assert!(decoded.semantic_trace[119].starts_with(SYSTEMS_ENTROPY_GENERATOR_EXTRACT));
    assert!(
        decoded.semantic_trace[120].starts_with(SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM)
    );
    assert!(decoded.semantic_trace[121].starts_with(SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE));
    decoded.semantic_trace.remove(121);
    assert!(
        decoded.semantic_trace[121].starts_with(SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS)
    );
    decoded.semantic_trace.remove(121);
    assert!(
        decoded.semantic_trace[121].starts_with(SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS)
    );
    decoded.semantic_trace.remove(121);
    assert!(decoded.semantic_trace[121].starts_with(SYSTEMS_USER_CONTEXT_ENTER));
    assert!(decoded.semantic_trace[122].starts_with("topal.systems.entry.user-syscall/1"));
    assert!(
        decoded.semantic_trace[123].starts_with("topal.systems.disposition.user-syscall.return/1")
    );
    assert!(decoded.semantic_trace[124].starts_with("topal.systems.entry.user-syscall/1"));
    assert!(decoded.semantic_trace[125].starts_with(SYSTEMS_USER_TRANSFER_BEGIN));
    assert!(decoded.semantic_trace[126].starts_with(SYSTEMS_USER_COPY_TO));
    assert!(decoded.semantic_trace[127].starts_with(SYSTEMS_USER_TRANSFER_END));
    assert!(
        decoded.semantic_trace[128].starts_with("topal.systems.disposition.user-syscall.return/1")
    );
    assert!(decoded.semantic_trace[129].starts_with("topal.systems.entry.user-syscall/1"));
    assert!(decoded.semantic_trace[130].starts_with(SYSTEMS_USER_TRANSFER_BEGIN));
    assert!(decoded.semantic_trace[131].starts_with(SYSTEMS_USER_COPY_TO));
    assert!(decoded.semantic_trace[132].starts_with(SYSTEMS_USER_TRANSFER_END));
    assert!(
        decoded.semantic_trace[133].starts_with("topal.systems.disposition.user-syscall.return/1")
    );
    assert!(decoded.semantic_trace[134].starts_with("topal.systems.entry.user-syscall/1"));
    assert!(decoded.semantic_trace[135].starts_with(SYSTEMS_USER_TRANSFER_BEGIN));
    assert!(decoded.semantic_trace[136].starts_with(SYSTEMS_USER_COPY_FROM));
    assert!(decoded.semantic_trace[137].starts_with(SYSTEMS_USER_TRANSFER_END));
    assert!(
        decoded.semantic_trace[138].starts_with("topal.systems.disposition.user-syscall.return/1")
    );
    assert!(decoded.semantic_trace[139].starts_with("topal.systems.entry.user-syscall/1"));
    assert!(decoded.semantic_trace[140].starts_with(SYSTEMS_USER_TRANSFER_BEGIN));
    assert!(decoded.semantic_trace[141].starts_with(SYSTEMS_USER_COPY_FROM));
    assert!(decoded.semantic_trace[142].starts_with(SYSTEMS_USER_TRANSFER_END));
    assert!(
        decoded.semantic_trace[143].starts_with("topal.systems.disposition.user-syscall.return/1")
    );
    assert!(decoded.semantic_trace[144].starts_with("topal.systems.entry.user-syscall/1"));
    assert!(
        decoded.semantic_trace[145]
            .starts_with("topal.systems.disposition.user-syscall.terminate/1")
    );
    assert!(decoded.semantic_trace[146].starts_with(SYSTEMS_USER_CONTEXT_ENTER));
    assert_eq!(
        decoded.semantic_trace[147],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK")
        )
    );
    decoded.semantic_trace.remove(147);
    assert!(decoded.semantic_trace[147].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert_eq!(
        decoded.semantic_trace[148],
        format!(
            "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
            sha256(b"TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK")
        )
    );
    decoded.semantic_trace.remove(148);
    assert!(decoded.semantic_trace[148].starts_with(SYSTEMS_USER_ADDRESS_SPACE_UNMAP));
    assert_eq!(decoded.semantic_trace[149], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[150].starts_with(SYSTEMS_USER_ADDRESS_SPACE_UNMAP));
    assert_eq!(decoded.semantic_trace[151], SYSTEMS_FRAMES_RELEASE);
    assert!(decoded.semantic_trace[152].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(decoded.semantic_trace[153].starts_with(SYSTEMS_CONSOLE_WRITE));
    assert!(
        decoded.semantic_trace[154].starts_with("topal.systems.context.user.complete/1"),
        "normalized user tail: {:?}",
        &decoded.semantic_trace[148..=158]
    );
    assert!(decoded.semantic_trace[188].starts_with(SYSTEMS_FATAL));
    let repeated_destination = parent.join("repeated");
    let repeated =
        publish_x86_64_systems_artifact(&program(), &tools, &repeated_destination).unwrap();
    for (first, second) in [
        (&published.kernel, &repeated.kernel),
        (&published.debug, &repeated.debug),
        (&published.map, &repeated.map),
        (&published.initial_user_image, &repeated.initial_user_image),
        (&published.provenance, &repeated.provenance),
    ] {
        assert_eq!(fs::read(first).unwrap(), fs::read(second).unwrap());
    }

    let static_destination = parent.join("with-static-user");
    let static_image = busybox_shaped_static_elf();
    let with_static = publish_x86_64_systems_artifact_with_static_user_image(
        &program(),
        &tools,
        &static_image,
        &static_destination,
    )
    .unwrap();
    assert_eq!(fs::read_dir(&static_destination).unwrap().count(), 6);
    let static_path = with_static.static_user_image.unwrap();
    assert_eq!(fs::read(&static_path).unwrap(), static_image);
    assert_eq!(with_static.record.outputs.len(), 5);
    assert_eq!(
        with_static
            .record
            .outputs
            .iter()
            .find(|output| output.identity == SYSTEMS_STATIC_USER_IMAGE_FILE)
            .unwrap()
            .sha256,
        sha256(&static_image)
    );
    fs::remove_dir_all(parent).unwrap();
}
