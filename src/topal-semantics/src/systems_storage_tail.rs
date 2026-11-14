//! Allocation-owned tail history; no caller-provided counter can reset it.
use super::*;

impl BootstrapStorageState {
    pub(crate) fn tail_identity_counters(
        &self,
        region: &BootstrapRegion,
    ) -> Result<(u64, u64), SystemsModelError> {
        let identity = self.validate_byte_access(region, 0)?;
        let &(frame, loan, _) = self.tail_histories.get(&identity).unwrap_or(&(2, 2, false));
        Ok((frame, loan))
    }

    pub(crate) fn record_tail_identity_counters(
        &mut self,
        region: &BootstrapRegion,
        frame: u64,
        loan: u64,
    ) -> Result<(), SystemsModelError> {
        let (old_frame, old_loan) = self.tail_identity_counters(region)?;
        if frame < old_frame || loan < old_loan {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-TAIL-HISTORY",
                "traversal identity history cannot move backwards",
            ));
        }
        self.tail_histories
            .insert(region.allocation_id, (frame, loan, false));
        Ok(())
    }

    // Only an admitted wrapper holding the exclusive actual owner calls this.
    pub(crate) fn poison_tail_owner(&mut self, region: &BootstrapRegion) {
        self.tail_histories
            .entry(region.allocation_id)
            .or_insert((2, 2, false))
            .2 = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RegionActionReclamation, RegionLibraryAccess, RegionLibraryBinding, TailScalar,
        TailScalarBound, TailScalarKind, TailSession, TailSessionBinding, TailSessionLimits,
    };
    use std::collections::BTreeSet;
    fn proof() -> RegionActionReclamation {
        RegionActionReclamation {
            arena_erased: true,
            control_erased: true,
            retired_stack_erased: true,
            bindings_revoked: true,
            caller_restored: true,
        }
    }
    fn owner() -> (BootstrapStorageState, BootstrapRegion) {
        let mut s = BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 16,
                alignment_bytes: 8,
            },
            "persistent-history",
        )
        .unwrap();
        let r = s
            .allocate(BootstrapStorageRequest {
                byte_count: 8,
                alignment_bytes: 8,
                placement: BootstrapStoragePlacement::BootstrapReclaimable,
            })
            .unwrap();
        (s, r)
    }
    fn binding() -> TailSessionBinding {
        TailSessionBinding::new(
            RegionLibraryBinding::new(
                "labels-only".into(),
                "entry".into(),
                BTreeMap::from([("entry".into(), BTreeSet::new())]),
                1,
                1,
            )
            .unwrap(),
            BTreeSet::from(["entry".into()]),
            vec![TailScalarBound::Nat {
                maximum: u64::MAX.into(),
            }],
            0,
            TailScalarKind::Boolean,
            vec![TailScalar::Boolean(true)],
            TailSessionLimits {
                base_bound: 0u8.into(),
                decrements: BTreeSet::from([1u8.into()]),
                frames_per_step: 3,
                loans_per_step: 3,
                work_bytes: 64,
            },
        )
        .unwrap()
    }
    #[test]
    fn admission_matches_an_independent_finite_remaining_identity_oracle() {
        // TOPAL-SYSTEMS-TAIL-SESSION-001. Exactly one representable next counter
        // must remain; an issued label never wraps at u64::MAX.
        let b = binding();
        for frame_remaining in 0..17 {
            for loan_remaining in 0..17 {
                for measure in 0..9 {
                    let (mut s, r) = owner();
                    s.tail_histories.insert(
                        r.allocation_id,
                        (u64::MAX - frame_remaining, u64::MAX - loan_remaining, false),
                    );
                    let before = format!("{s:?}");
                    let expected = (measure + 1) * 3 <= frame_remaining.min(loan_remaining);
                    match TailSession::begin(
                        &mut s,
                        &r,
                        &b,
                        RegionLibraryAccess::ReadWrite,
                        &[TailScalar::Nat(measure.into())],
                    ) {
                        Ok(mut session) => {
                            assert!(expected);
                            session.start_step().unwrap();
                            let yes = session.constant(0).unwrap();
                            session.stage_result(Some(&yes)).unwrap();
                            session.returned(false).unwrap();
                            session.checkpoint().unwrap();
                            assert_eq!(
                                session.complete(proof()).unwrap(),
                                TailScalar::Boolean(true)
                            );
                        }
                        Err(error) => {
                            assert!(!expected);
                            assert_eq!(error, crate::RegionLibraryError::IdentityExhausted);
                        }
                    }
                    if !expected {
                        assert_eq!(format!("{s:?}"), before);
                    }
                    assert_eq!(
                        s.tail_identity_counters(&r).unwrap(),
                        (u64::MAX - frame_remaining, u64::MAX - loan_remaining)
                    );
                }
            }
        }
    }
    #[test]
    fn fatal_exclusion_retains_actual_stores_and_allows_only_terminal_release() {
        let b = binding();
        for disposition in 0..3 {
            let (mut s, r) = owner();
            {
                let mut session = TailSession::begin(
                    &mut s,
                    &r,
                    &b,
                    RegionLibraryAccess::ReadWrite,
                    &[TailScalar::Nat(1u8.into())],
                )
                .unwrap();
                session.start_step().unwrap();
                let view = session.root_view().unwrap();
                session.store_byte(&view, 3, 0x6a).unwrap();
                if disposition == 0 {
                    session.cancel();
                }
                if disposition == 1 {
                    let yes = session.constant(0).unwrap();
                    session.stage_result(Some(&yes)).unwrap();
                    session.returned(false).unwrap();
                    session.checkpoint().unwrap();
                    let mut p = proof();
                    p.caller_restored = false;
                    assert!(session.complete(p).is_err());
                }
                // disposition 2 abandons the active borrow.
            }
            assert_eq!(s.contents.get(&(r.allocation_id, 3)), Some(&0x6a));
            assert_eq!(s.tail_histories.get(&r.allocation_id), Some(&(2, 2, true)));
            let before = format!("{s:?}");
            assert_eq!(s.load_byte(&r, 3).unwrap_err().code, "E-SYSTEMS-TAIL-FATAL");
            assert_eq!(
                s.store_byte(&r, 3, 0).unwrap_err().code,
                "E-SYSTEMS-TAIL-FATAL"
            );
            assert_eq!(
                s.atomic_load_word(&r, 0).unwrap_err().code,
                "E-SYSTEMS-TAIL-FATAL"
            );
            assert!(
                TailSession::begin(
                    &mut s,
                    &r,
                    &b,
                    RegionLibraryAccess::ReadWrite,
                    &[TailScalar::Nat(0u8.into())]
                )
                .is_err()
            );
            assert_eq!(format!("{s:?}"), before);
            s.release(r).unwrap();
            assert!(s.contents.is_empty());
            assert!(s.tail_histories.is_empty());
            let next = s
                .allocate(BootstrapStorageRequest {
                    byte_count: 8,
                    alignment_bytes: 8,
                    placement: BootstrapStoragePlacement::BootstrapReclaimable,
                })
                .unwrap();
            assert_eq!(next.allocation_id, 1);
            assert_eq!(s.tail_identity_counters(&next).unwrap(), (2, 2));
            s.store_byte(&next, 3, 7).unwrap();
            assert_eq!(s.load_byte(&next, 3).unwrap(), 7);
        }
    }
    #[test]
    fn owned_history_rejects_counter_reset_without_observable_mutation() {
        let (mut s, r) = owner();
        s.record_tail_identity_counters(&r, 9, 13).unwrap();
        for (frame, loan) in [(8, 13), (9, 12), (0, 0)] {
            let before = format!("{s:?}");
            assert!(s.record_tail_identity_counters(&r, frame, loan).is_err());
            assert_eq!(format!("{s:?}"), before);
        }
    }
}
