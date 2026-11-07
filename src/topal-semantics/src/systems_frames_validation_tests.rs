//! Complete-state and independent cell-oracle checks for allocator readiness.
use super::*;
use crate::{
    BootMemoryDescription, BootMemoryReservation, BootMemorySourceRange, EnteredBootstrapContext,
    NormalizedBootMemoryRange, normalize_boot_memory,
};

fn provenance() -> BootMemoryProvenance {
    BootMemoryProvenance {
        class: BootMemoryClass::Allocatable,
        source_kind: "ram".into(),
        provider: "test-frames/1".into(),
    }
}

fn description() -> BootMemoryDescription {
    BootMemoryDescription {
        page_size: 4096,
        ranges: vec![NormalizedBootMemoryRange {
            start: 4096,
            end: 17 * 4096,
            class: BootMemoryClass::Allocatable,
            provenance: vec![provenance()],
        }],
    }
}

fn allocator(memory: BootMemoryDescription) -> FrameAllocatorContext {
    let FrameAllocatorResult::Ready(allocator) = MemoryDescribedContext {
        handoff_identity: "same-handoff-label".into(),
        memory,
    }
    .create_frame_allocator("same-allocator-label") else {
        panic!("valid normalized memory")
    };
    allocator
}

fn request(count: u64, alignment: u64) -> PhysicalFrameRequest {
    PhysicalFrameRequest {
        frame_count: count,
        alignment_frames: alignment,
    }
}

#[test]
fn identity_exhaustion_preserves_every_allocator_field() {
    // TOPAL-SYSTEMS-FRAMES-001: refusal cannot consume an unreturned extent.
    let mut owner = allocator(description());
    owner.next_extent_identity = u64::MAX;
    let before = owner.clone();
    assert_eq!(
        owner.allocate(request(3, 4)),
        Err(PhysicalFrameError::Exhausted)
    );
    assert_eq!(owner, before);
}

#[test]
fn equal_labels_and_equal_extent_facts_do_not_confer_allocator_authority() {
    // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-ADDRESS-001.
    let mut first = allocator(description());
    let mut second = allocator(description());
    let a = first.allocate(request(2, 1)).unwrap();
    let b = second.allocate(request(2, 1)).unwrap();
    assert_eq!(a.allocator_identity(), b.allocator_identity());
    assert_eq!(a.model_start_frame(), b.model_start_frame());
    let before = first.clone();
    assert_eq!(
        first.release(b.clone()),
        Err(PhysicalFrameError::WrongAllocator)
    );
    assert_eq!(first, before);
    assert_eq!(
        first.begin_mapping(&b),
        Err(PhysicalFrameError::WrongAllocator)
    );
    assert_eq!(first, before);
    first.begin_mapping(&a).unwrap();
    let mapped = first.clone();
    assert_eq!(
        first.finish_mapping(&b),
        Err(PhysicalFrameError::WrongAllocator)
    );
    assert_eq!(first, mapped);
    first.finish_mapping(&a).unwrap();
    first.release(a).unwrap();
    second.release(b).unwrap();
    first.complete().unwrap();
    second.complete().unwrap();
}

#[test]
fn altered_extent_facts_are_checked_without_overflow_or_state_changes() {
    // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-MAPPING-001.
    let mut owner = allocator(description());
    let extent = owner.allocate(request(2, 1)).unwrap();
    let mut corruptions = vec![extent.clone(); 5];
    corruptions[0].frame_count = u64::MAX;
    corruptions[1].frame_count = 0;
    corruptions[2].alignment_frames = 0;
    corruptions[3].alignment_frames = 3;
    corruptions[4].provenance.clear();
    for forged in &corruptions {
        let before = owner.clone();
        assert_eq!(
            owner.release(forged.clone()),
            Err(PhysicalFrameError::UnknownExtent)
        );
        assert_eq!(owner, before);
        assert_eq!(
            owner.begin_mapping(forged),
            Err(PhysicalFrameError::UnknownExtent)
        );
        assert_eq!(owner, before);
    }
    owner.begin_mapping(&extent).unwrap();
    for forged in &corruptions {
        let before = owner.clone();
        assert_eq!(
            owner.finish_mapping(forged),
            Err(PhysicalFrameError::UnknownExtent)
        );
        assert_eq!(owner, before);
    }
    owner.finish_mapping(&extent).unwrap();
    owner.release(extent).unwrap();
    owner.complete().unwrap();
}

#[test]
fn invalid_normalized_descriptions_never_create_allocatable_authority() {
    // TOPAL-SYSTEMS-BOOT-MEMORY-001, TOPAL-SYSTEMS-FRAMES-001.
    let original = description();
    let mut invalid = vec![original.clone(); 9];
    invalid[0].ranges.push(original.ranges[0].clone());
    invalid[1].ranges.push(NormalizedBootMemoryRange {
        start: 2 * 4096,
        end: 3 * 4096,
        class: BootMemoryClass::Reserved,
        provenance: vec![provenance()],
    });
    invalid[2].ranges[0].start += 1;
    invalid[3].ranges[0].end -= 1;
    invalid[4].ranges[0].start = original.ranges[0].end;
    invalid[5].ranges[0].end = 0;
    invalid[6].ranges[0].provenance.clear();
    invalid[7].ranges[0].provenance[0].class = BootMemoryClass::Unknown;
    invalid[8].page_size = 3;
    for memory in invalid {
        let result = MemoryDescribedContext {
            handoff_identity: "handoff".into(),
            memory,
        }
        .create_frame_allocator("owner");
        assert!(
            matches!(
                result,
                FrameAllocatorResult::Failure(FrameAllocatorFailureContext {
                    error: PhysicalFrameError::InvalidDescription,
                    ..
                })
            ),
            "{result:?}"
        );
    }
}

#[test]
fn valid_unordered_ranges_and_reserved_edge_fragments_keep_each_frame_geometry() {
    // TOPAL-SYSTEMS-BOOT-MEMORY-001, TOPAL-SYSTEMS-FRAMES-001.
    // Range order and sub-page reserved edges do not grant or remove complete RAM.
    for page_size in [4096, 16384, 65536] {
        let source = [BootMemorySourceRange {
            start: page_size + 1,
            length: 6 * page_size - 2,
            provenance: provenance(),
        }];
        let mut memory = normalize_boot_memory(&source, &[], page_size).unwrap();
        memory.ranges.reverse();
        let mut owner = allocator(memory);
        assert_eq!(owner.page_size(), page_size);
        let extent = owner.allocate(request(4, 2)).unwrap();
        assert_eq!(extent.model_start_frame(), 2);
        assert_eq!(extent.frame_count, 4);
        let before = owner.clone();
        assert_eq!(
            owner.allocate(request(1, 1)),
            Err(PhysicalFrameError::Exhausted)
        );
        assert_eq!(owner, before);
        owner.release(extent).unwrap();
        owner.complete().unwrap();
    }
}

#[test]
fn mathematical_model_boundary_refusals_preserve_live_extents_and_history() {
    // TOPAL-SYSTEMS-FRAMES-001: these frame indexes are model facts, not addresses.
    let mut memory = description();
    memory.page_size = 1;
    memory.ranges[0].start = u64::MAX - 2;
    memory.ranges[0].end = u64::MAX;
    let mut owner = allocator(memory);
    for invalid in [request(0, 1), request(1, 0), request(1, 3)] {
        let before = owner.clone();
        assert_eq!(
            owner.allocate(invalid),
            Err(PhysicalFrameError::InvalidRequest)
        );
        assert_eq!(owner, before);
    }
    for exhausted in [request(u64::MAX, 1), request(1, 1 << 63)] {
        let before = owner.clone();
        assert_eq!(
            owner.allocate(exhausted),
            Err(PhysicalFrameError::Exhausted)
        );
        assert_eq!(owner, before);
    }
    let extent = owner.allocate(request(2, 1)).unwrap();
    let stale = extent.clone();
    owner.begin_mapping(&extent).unwrap();
    let mapped = owner.clone();
    assert_eq!(
        owner.release(extent.clone()),
        Err(PhysicalFrameError::MappedExtent)
    );
    assert_eq!(owner, mapped);
    owner.finish_mapping(&extent).unwrap();
    owner.release(extent).unwrap();
    let replacement = owner.allocate(request(2, 1)).unwrap();
    assert_eq!(replacement.model_start_frame(), stale.model_start_frame());
    let before = owner.clone();
    assert_eq!(owner.release(stale), Err(PhysicalFrameError::UnknownExtent));
    assert_eq!(owner, before);
    owner.release(replacement).unwrap();
    owner.complete().unwrap();
}

#[test]
fn fragmented_dynamic_requests_match_an_independent_frame_cell_oracle() {
    // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-BOOT-MEMORY-001.
    // The oracle uses individual physical cells, not allocator free-run state.
    let mut second_provenance = provenance();
    second_provenance.provider = "second-provider".into();
    let source = [
        BootMemorySourceRange {
            start: 0,
            length: 32 * 4096,
            provenance: provenance(),
        },
        BootMemorySourceRange {
            start: 32 * 4096,
            length: 32 * 4096,
            provenance: second_provenance.clone(),
        },
    ];
    let reservations = [BootMemoryReservation {
        start: 17 * 4096,
        length: 6 * 4096,
        owner: "live-reservation".into(),
        reclaimable: false,
    }];
    let memory = normalize_boot_memory(&source, &reservations, 4096).unwrap();
    // Also exercise the actual described-context refinement, not a host address.
    let described =
        EnteredBootstrapContext::new("handoff").describe_memory(&source, &reservations, 4096, true);
    assert!(matches!(described, crate::BootMemoryResult::Described(_)));
    let mut owner = allocator(memory);
    let mut free = [true; 64];
    free[17..23].fill(false);
    let original = free;
    let mut live = Vec::<PhysicalFrameExtent>::new();
    let mut random = 0x5eed_u64;
    for step in 0..768 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        if step % 3 == 0 && !live.is_empty() {
            let extent = live.swap_remove(random as usize % live.len());
            let duplicate = extent.clone();
            let start = extent.model_start_frame() as usize;
            let end = start + extent.frame_count as usize;
            owner.release(extent).unwrap();
            free[start..end].fill(true);
            let before = owner.clone();
            assert_eq!(
                owner.release(duplicate),
                Err(PhysicalFrameError::UnknownExtent)
            );
            assert_eq!(owner, before);
        } else {
            let count = ((random >> 8) % 9 + 1) as usize;
            let alignment = 1_usize << ((random >> 24) % 4);
            let expected = (0..=64 - count).find(|&start| {
                start % alignment == 0
                    && (start < 32) == (start + count - 1 < 32)
                    && free[start..start + count].iter().all(|cell| *cell)
            });
            let before = owner.clone();
            match (
                expected,
                owner.allocate(request(count as u64, alignment as u64)),
            ) {
                (Some(start), Ok(extent)) => {
                    assert_eq!(extent.model_start_frame(), start as u64);
                    assert_eq!(extent.frame_count, count as u64);
                    assert_eq!(extent.alignment_frames, alignment as u64);
                    assert_eq!(
                        extent.provenance,
                        vec![if start < 32 {
                            provenance()
                        } else {
                            second_provenance.clone()
                        }]
                    );
                    free[start..start + count].fill(false);
                    live.push(extent);
                }
                (None, Err(PhysicalFrameError::Exhausted)) => assert_eq!(owner, before),
                (expected, actual) => panic!("step {step}: expected {expected:?}, got {actual:?}"),
            }
        }
    }
    for extent in live {
        let start = extent.model_start_frame() as usize;
        let end = start + extent.frame_count as usize;
        owner.release(extent).unwrap();
        free[start..end].fill(true);
    }
    assert_eq!(free, original);
    assert_eq!(
        owner
            .free
            .iter()
            .map(|run| (run.start_frame, run.end_frame))
            .collect::<Vec<_>>(),
        vec![(0, 17), (23, 32), (32, 64)]
    );
    owner.complete().unwrap();
}
