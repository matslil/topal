//! TOPAL-SYSTEMS-TAIL-SESSION-001. Actual owned-byte reference transitions.
//! No opaque source execution, target-size certificate or physical erasure grant.
use super::*;
use num_bigint::BigUint;
use std::sync::Arc;
#[path = "systems_tail_values.rs"]
mod values;
pub use values::{TailScalar, TailScalarBound, TailScalarKind};
use values::{decode, encode};

#[derive(Clone, Debug)]
pub struct TailSessionLimits {
    pub base_bound: BigUint,
    pub decrements: BTreeSet<BigUint>,
    pub frames_per_step: u64,
    pub loans_per_step: u64,
    pub work_bytes: usize,
}

/// Checked reference constraints, not proof of source origin or native binding.
#[derive(Clone, Debug)]
pub struct TailSessionBinding {
    library: RegionLibraryBinding,
    region_functions: BTreeSet<String>,
    roots: Vec<TailScalarBound>,
    measure: usize,
    result: TailScalarKind,
    constants: Vec<TailScalar>,
    limits: TailSessionLimits,
    offsets: Vec<usize>,
    bank_bytes: usize,
}
impl TailSessionBinding {
    /// Canonical reference bank extent, excluding target metadata and stacks.
    pub fn bank_bytes(&self) -> usize {
        self.bank_bytes
    }

    pub fn new(
        library: RegionLibraryBinding,
        region_functions: BTreeSet<String>,
        roots: Vec<TailScalarBound>,
        measure: usize,
        result: TailScalarKind,
        constants: Vec<TailScalar>,
        limits: TailSessionLimits,
    ) -> Result<Self, RegionLibraryError> {
        if roots.is_empty()
            || roots.len() > 16
            || roots.iter().any(|r| !r.valid())
            || !matches!(roots.get(measure), Some(TailScalarBound::Nat { .. }))
            || !region_functions.contains(&library.entry)
            || region_functions
                .iter()
                .any(|f| !library.calls.contains_key(f))
            || limits.frames_per_step == 0
            || limits.loans_per_step == 0
            || limits.decrements.is_empty()
            || limits.base_bound.bits() > 256
            || limits
                .decrements
                .iter()
                .any(|d| d == &BigUint::from(0u8) || d > &(&limits.base_bound + 1u8))
            || constants.len() > 256
            || constants
                .iter()
                .any(|c| c.kind() == TailScalarKind::Unit || encode(c).is_err())
        {
            return Err(RegionLibraryError::Binding);
        }
        let mut offsets = Vec::new();
        let mut bank_bytes = 0;
        for root in &roots {
            offsets.push(bank_bytes);
            bank_bytes += root.bytes();
        }
        bank_bytes = bank_bytes.div_ceil(16) * 16;
        // Tracked reference buffers only. Actual metadata/stack binding is an
        // independent target obligation, never inferred from Rust heap size.
        if limits.work_bytes == 0
            || limits.work_bytes > 65536usize.saturating_sub(2 * bank_bytes + 48)
        {
            return Err(RegionLibraryError::Capacity);
        }
        Ok(Self {
            library,
            region_functions,
            roots,
            measure,
            result,
            constants,
            limits,
            offsets,
            bank_bytes,
        })
    }

    fn admission(
        &self,
        initial: &BigUint,
        next: (u64, u64),
    ) -> Result<(u64, u64, u64), RegionLibraryError> {
        let edges = if initial <= &self.limits.base_bound {
            BigUint::from(0u8)
        } else {
            let d = self.limits.decrements.first().unwrap();
            (initial - &self.limits.base_bound + d - 1u8) / d
        };
        let steps = (edges + 1u8)
            .to_string()
            .parse::<u64>()
            .map_err(|_| RegionLibraryError::IdentityExhausted)?;
        let end = |n: u64, p: u64| {
            if n < 2 {
                return Err(RegionLibraryError::IdentityExhausted);
            }
            steps
                .checked_mul(p)
                .and_then(|budget| n.checked_add(budget))
                .ok_or(RegionLibraryError::IdentityExhausted)
        };
        Ok((
            steps,
            end(next.0, self.limits.frames_per_step)?,
            end(next.1, self.limits.loans_per_step)?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TailSessionPhase {
    Ready,
    Running,
    ReturnedSuccessor,
    ReturnedTerminal,
    Terminal,
    Complete,
    Fatal,
}
#[derive(Clone, Debug)]
enum Origin {
    Bank(usize, usize),
    Work(usize),
    Constant(usize),
}
/// Copies alias one issued root. There is no address or public token constructor.
#[derive(Clone, Debug)]
pub struct TailScalarRef {
    session: Arc<()>,
    epoch: u64,
    origin: Origin,
}
#[derive(Debug)]
struct Allocation {
    start: usize,
    bytes: usize,
    kind: TailScalarKind,
}

#[derive(Debug)]
pub struct TailSession<'a> {
    invocation: RegionLibraryInvocation<'a>,
    binding: &'a TailSessionBinding,
    root: RegionLibraryView,
    identity: Arc<()>,
    epoch: u64,
    phase: TailSessionPhase,
    banks: [Vec<u8>; 2],
    bank: usize,
    staged: Vec<bool>,
    work: Vec<u8>,
    used: usize,
    allocations: Vec<Allocation>,
    result: [u8; 48],
    result_present: bool,
    steps: u64,
    step_limit: u64,
    frame_end: u64,
    loan_end: u64,
    step_frame_end: u64,
    step_loan_end: u64,
}
impl<'a> TailSession<'a> {
    pub fn begin(
        storage: &'a mut BootstrapStorageState,
        region: &'a BootstrapRegion,
        binding: &'a TailSessionBinding,
        access: RegionLibraryAccess,
        initial: &[TailScalar],
    ) -> Result<Self, RegionLibraryError> {
        if initial.len() != binding.roots.len()
            || initial
                .iter()
                .zip(&binding.roots)
                .any(|(v, b)| !v.initial_word() || !b.contains(v))
        {
            return Err(RegionLibraryError::Bounds);
        }
        let TailScalar::Nat(measure) = &initial[binding.measure] else {
            return Err(RegionLibraryError::Binding);
        };
        let counters = storage
            .tail_identity_counters(region)
            .map_err(|_| RegionLibraryError::Owner)?;
        let (step_limit, frame_end, loan_end) = binding.admission(measure, counters)?;
        let mut banks = [vec![0; binding.bank_bytes], vec![0; binding.bank_bytes]];
        for (index, value) in initial.iter().enumerate() {
            let bytes = encode(value)?;
            let offset = binding.offsets[index];
            banks[0][offset..offset + bytes.len()].copy_from_slice(&bytes);
        }
        // Every fallible admission and encoding check precedes generation.
        let (mut invocation, root) =
            RegionLibraryInvocation::begin(storage, region, &binding.library, access)?;
        invocation.next_frame = counters.0;
        invocation.next_view = counters.1;
        Ok(Self {
            invocation,
            binding,
            root,
            identity: Arc::new(()),
            epoch: 1,
            phase: TailSessionPhase::Ready,
            banks,
            bank: 0,
            staged: vec![false; initial.len()],
            work: vec![0; binding.limits.work_bytes],
            used: 0,
            allocations: Vec::new(),
            result: [0; 48],
            result_present: false,
            steps: 0,
            step_limit,
            frame_end,
            loan_end,
            step_frame_end: counters.0,
            step_loan_end: counters.1,
        })
    }
    pub fn phase(&self) -> TailSessionPhase {
        self.phase
    }
    pub fn identity_counters(&self) -> (u64, u64) {
        (self.invocation.next_frame, self.invocation.next_view)
    }
    pub fn private_buffers_reclaimed(&self) -> bool {
        matches!(
            self.phase,
            TailSessionPhase::Complete | TailSessionPhase::Fatal
        ) && self
            .banks
            .iter()
            .flatten()
            .chain(self.work.iter())
            .chain(self.result.iter())
            .all(|b| *b == 0)
            && self.allocations.is_empty()
    }
    fn checked<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, RegionLibraryError>,
    ) -> Result<T, RegionLibraryError> {
        if matches!(
            self.phase,
            TailSessionPhase::Complete | TailSessionPhase::Fatal
        ) {
            return Err(RegionLibraryError::Ended);
        }
        let result = f(self);
        if result.is_err() {
            self.fatal();
        }
        result
    }
    fn checked_access<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, RegionLibraryError>,
    ) -> Result<T, RegionLibraryError> {
        if matches!(
            self.phase,
            TailSessionPhase::Complete | TailSessionPhase::Fatal
        ) {
            return Err(RegionLibraryError::Ended);
        }
        let result = f(self);
        if result.as_ref().is_err_and(|error| {
            !matches!(
                error,
                RegionLibraryError::Bounds
                    | RegionLibraryError::ReadOnly
                    | RegionLibraryError::ByteValue
            )
        }) {
            self.fatal();
        }
        result
    }
    fn running(&self) -> Result<(), RegionLibraryError> {
        if self.phase == TailSessionPhase::Running {
            Ok(())
        } else {
            Err(RegionLibraryError::Invariant)
        }
    }
    fn at_root(&self) -> Result<(), RegionLibraryError> {
        if self.invocation.frames.len() == 1 {
            Ok(())
        } else {
            Err(RegionLibraryError::NotRoot)
        }
    }
    fn persist(&mut self) -> Result<(), RegionLibraryError> {
        let Storage::Bootstrap { storage, region } = &mut self.invocation.storage else {
            return Err(RegionLibraryError::Binding);
        };
        storage
            .record_tail_identity_counters(
                region,
                self.invocation.next_frame,
                self.invocation.next_view,
            )
            .map_err(|_| RegionLibraryError::Owner)
    }
    fn erase_buffers(&mut self) {
        self.banks.iter_mut().for_each(|b| b.fill(0));
        self.work.fill(0);
        self.result.fill(0);
        self.allocations.clear();
        self.used = 0;
        self.staged.fill(false);
        self.result_present = false;
    }
    fn fatal(&mut self) {
        self.erase_buffers();
        if let Storage::Bootstrap { storage, region } = &mut self.invocation.storage {
            storage.poison_tail_owner(region);
        }
        self.invocation.revoke();
        self.phase = TailSessionPhase::Fatal;
    }
    pub fn cancel(&mut self) {
        if !matches!(
            self.phase,
            TailSessionPhase::Complete | TailSessionPhase::Fatal
        ) {
            self.fatal();
        }
    }
    pub fn start_step(&mut self) -> Result<(), RegionLibraryError> {
        self.checked(|s| {
            if s.phase != TailSessionPhase::Ready
                || s.steps >= s.step_limit
                || s.result_present
                || s.work.iter().any(|b| *b != 0)
                || s.banks[1 - s.bank].iter().any(|b| *b != 0)
            {
                return Err(RegionLibraryError::Invariant);
            }
            s.at_root()?;
            s.step_frame_end = s
                .invocation
                .next_frame
                .checked_add(s.binding.limits.frames_per_step)
                .filter(|end| *end <= s.frame_end)
                .ok_or(RegionLibraryError::IdentityExhausted)?;
            s.step_loan_end = s
                .invocation
                .next_view
                .checked_add(s.binding.limits.loans_per_step)
                .filter(|end| *end <= s.loan_end)
                .ok_or(RegionLibraryError::IdentityExhausted)?;
            s.steps += 1;
            s.phase = TailSessionPhase::Running;
            Ok(())
        })
    }
    fn reference(&self, origin: Origin) -> TailScalarRef {
        TailScalarRef {
            session: self.identity.clone(),
            epoch: self.epoch,
            origin,
        }
    }
    pub fn input(&mut self, index: usize) -> Result<TailScalarRef, RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            if index >= s.binding.roots.len() {
                return Err(RegionLibraryError::Bounds);
            }
            Ok(s.reference(Origin::Bank(s.bank, index)))
        })
    }
    pub fn constant(&mut self, index: usize) -> Result<TailScalarRef, RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            if index >= s.binding.constants.len() {
                return Err(RegionLibraryError::Bounds);
            }
            Ok(s.reference(Origin::Constant(index)))
        })
    }
    pub fn allocate(&mut self, value: &TailScalar) -> Result<TailScalarRef, RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            if value.kind() == TailScalarKind::Unit {
                return Err(RegionLibraryError::Binding);
            }
            let bytes = encode(value)?;
            if bytes.len() > s.work.len() - s.used {
                return Err(RegionLibraryError::Capacity);
            }
            let index = s.allocations.len();
            s.work[s.used..s.used + bytes.len()].copy_from_slice(&bytes);
            s.allocations.push(Allocation {
                start: s.used,
                bytes: bytes.len(),
                kind: value.kind(),
            });
            s.used += bytes.len();
            Ok(s.reference(Origin::Work(index)))
        })
    }
    fn bank_value(&self, bank: usize, index: usize) -> Result<TailScalar, RegionLibraryError> {
        let bound = self
            .binding
            .roots
            .get(index)
            .ok_or(RegionLibraryError::Bounds)?;
        let offset = self.binding.offsets[index];
        let value = decode(
            bound.kind(),
            &self.banks[bank][offset..offset + bound.bytes()],
        )?;
        if !bound.contains(&value) {
            return Err(RegionLibraryError::Bounds);
        }
        Ok(value)
    }
    fn value(&self, reference: &TailScalarRef) -> Result<TailScalar, RegionLibraryError> {
        self.running()?;
        if !Arc::ptr_eq(&self.identity, &reference.session) || reference.epoch != self.epoch {
            return Err(RegionLibraryError::View);
        }
        match reference.origin {
            Origin::Bank(bank, index) if bank == self.bank => self.bank_value(bank, index),
            Origin::Bank(..) => Err(RegionLibraryError::View),
            Origin::Constant(index) => self
                .binding
                .constants
                .get(index)
                .cloned()
                .ok_or(RegionLibraryError::View),
            Origin::Work(index) => {
                let a = self
                    .allocations
                    .get(index)
                    .ok_or(RegionLibraryError::View)?;
                decode(a.kind, &self.work[a.start..a.start + a.bytes])
            }
        }
    }
    pub fn read(&mut self, reference: &TailScalarRef) -> Result<TailScalar, RegionLibraryError> {
        self.checked(|s| s.value(reference))
    }
    pub fn root_view(&mut self) -> Result<RegionLibraryView, RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            s.at_root()?;
            s.invocation.window(&s.root)?;
            Ok(s.root.clone())
        })
    }
    pub fn enter_helper(
        &mut self,
        function: &str,
        view: Option<&RegionLibraryView>,
    ) -> Result<Option<RegionLibraryView>, RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            if view.is_some() != s.binding.region_functions.contains(function) {
                return Err(RegionLibraryError::Binding);
            }
            if s.invocation.next_frame >= s.step_frame_end
                || (view.is_some() && s.invocation.next_view >= s.step_loan_end)
            {
                return Err(RegionLibraryError::IdentityExhausted);
            }
            let arguments = view.into_iter().cloned().collect::<Vec<_>>();
            let mut views = s.invocation.enter(function, &arguments)?;
            s.persist()?;
            Ok(views.pop())
        })
    }
    pub fn return_helper(&mut self) -> Result<(), RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            s.invocation.return_from_call()
        })
    }
    pub fn load_byte(
        &mut self,
        view: &RegionLibraryView,
        offset: u64,
    ) -> Result<u8, RegionLibraryError> {
        self.checked_access(|s| {
            s.running()?;
            s.invocation.load_byte(view, offset)
        })
    }
    pub fn store_byte(
        &mut self,
        view: &RegionLibraryView,
        offset: u64,
        value: u64,
    ) -> Result<(), RegionLibraryError> {
        self.checked_access(|s| {
            s.running()?;
            s.invocation.store_byte(view, offset, value)
        })
    }
    /// Ordinary source Optional result, with no truncation of a natural index.
    pub fn byte_load(
        &mut self,
        view: &RegionLibraryView,
        index: &BigUint,
    ) -> Result<Option<u8>, RegionLibraryError> {
        self.checked_access(|s| {
            s.running()?;
            s.invocation.window(view)?;
            let Ok(offset) = u64::try_from(index) else {
                return Ok(None);
            };
            match s.invocation.load_byte(view, offset) {
                Ok(value) => Ok(Some(value)),
                Err(RegionLibraryError::Bounds) => Ok(None),
                Err(error) => Err(error),
            }
        })
    }
    /// Ordinary source Boolean result; rights/range refusal reaches no storage.
    pub fn byte_store(
        &mut self,
        view: &RegionLibraryView,
        index: &BigUint,
        value: &BigUint,
    ) -> Result<bool, RegionLibraryError> {
        self.checked_access(|s| {
            s.running()?;
            s.invocation.window(view)?;
            let (Ok(offset), Ok(value)) = (u64::try_from(index), u64::try_from(value)) else {
                return Ok(false);
            };
            match s.invocation.store_byte(view, offset, value) {
                Ok(()) => Ok(true),
                Err(
                    RegionLibraryError::Bounds
                    | RegionLibraryError::ReadOnly
                    | RegionLibraryError::ByteValue,
                ) => Ok(false),
                Err(error) => Err(error),
            }
        })
    }
    pub fn effect_resource(
        &mut self,
        view: &RegionLibraryView,
    ) -> Result<ResourceIdentity, RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            s.invocation.effect_resource(view)
        })
    }
    pub fn stage(
        &mut self,
        index: usize,
        reference: &TailScalarRef,
    ) -> Result<(), RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            s.at_root()?;
            let bound = s
                .binding
                .roots
                .get(index)
                .ok_or(RegionLibraryError::Bounds)?;
            if s.staged[index] || s.result_present {
                return Err(RegionLibraryError::Invariant);
            }
            let value = s.value(reference)?;
            if !bound.contains(&value) {
                return Err(RegionLibraryError::Bounds);
            }
            let bytes = encode(&value)?;
            let offset = s.binding.offsets[index];
            s.banks[1 - s.bank][offset..offset + bytes.len()].copy_from_slice(&bytes);
            s.staged[index] = true;
            Ok(())
        })
    }
    pub fn stage_result(
        &mut self,
        reference: Option<&TailScalarRef>,
    ) -> Result<(), RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            s.at_root()?;
            if s.result_present || s.staged.iter().any(|b| *b) {
                return Err(RegionLibraryError::Invariant);
            }
            let value = match reference {
                Some(r) => s.value(r)?,
                None => TailScalar::Unit,
            };
            if value.kind() != s.binding.result {
                return Err(RegionLibraryError::Binding);
            }
            let bytes = encode(&value)?;
            s.result[..bytes.len()].copy_from_slice(&bytes);
            s.result_present = true;
            Ok(())
        })
    }
    pub fn returned(&mut self, successor: bool) -> Result<(), RegionLibraryError> {
        self.checked(|s| {
            s.running()?;
            s.at_root()?;
            if successor {
                if s.result_present || s.staged.iter().any(|b| !*b) {
                    return Err(RegionLibraryError::Invariant);
                }
                s.phase = TailSessionPhase::ReturnedSuccessor;
            } else {
                if !s.result_present || s.staged.iter().any(|b| *b) {
                    return Err(RegionLibraryError::Invariant);
                }
                s.phase = TailSessionPhase::ReturnedTerminal;
            }
            Ok(())
        })
    }
    pub fn checkpoint(&mut self) -> Result<(), RegionLibraryError> {
        self.checked(|s| {
            let successor = s.phase == TailSessionPhase::ReturnedSuccessor;
            if !successor && s.phase != TailSessionPhase::ReturnedTerminal {
                return Err(RegionLibraryError::Invariant);
            }
            s.at_root()?;
            if successor {
                let TailScalar::Nat(old) = s.bank_value(s.bank, s.binding.measure)? else {
                    return Err(RegionLibraryError::Binding);
                };
                let TailScalar::Nat(next) = s.bank_value(1 - s.bank, s.binding.measure)? else {
                    return Err(RegionLibraryError::Binding);
                };
                // Validate ALL roots before retirement, not just the measure.
                for i in 0..s.binding.roots.len() {
                    s.bank_value(1 - s.bank, i)?;
                }
                if old <= s.binding.limits.base_bound
                    || next >= old
                    || !s.binding.limits.decrements.contains(&(&old - &next))
                {
                    return Err(RegionLibraryError::Invariant);
                }
                let epoch = s
                    .epoch
                    .checked_add(1)
                    .ok_or(RegionLibraryError::IdentityExhausted)?;
                let frame = s.invocation.next_frame;
                let loan = s.invocation.next_view;
                if frame >= s.step_frame_end
                    || loan >= s.step_loan_end
                    || s.invocation.windows.len() != 1
                {
                    return Err(RegionLibraryError::IdentityExhausted);
                }
                let window = s.invocation.window(&s.root)?;
                let next_frame = frame
                    .checked_add(1)
                    .ok_or(RegionLibraryError::IdentityExhausted)?;
                let next_view = loan
                    .checked_add(1)
                    .ok_or(RegionLibraryError::IdentityExhausted)?;
                s.invocation.windows.clear();
                s.invocation
                    .windows
                    .insert(loan, Window { frame, ..window });
                s.invocation.frames[0].identity = frame;
                s.invocation.next_frame = next_frame;
                s.invocation.next_view = next_view;
                s.root = s.invocation.token(frame, loan);
                s.persist()?;
                s.epoch = epoch;
            }
            s.work.fill(0);
            s.used = 0;
            s.allocations.clear();
            s.banks[s.bank].fill(0);
            s.bank = 1 - s.bank;
            s.staged.fill(false);
            s.phase = if successor {
                TailSessionPhase::Ready
            } else {
                TailSessionPhase::Terminal
            };
            Ok(())
        })
    }
    pub fn complete(
        &mut self,
        proof: RegionActionReclamation,
    ) -> Result<TailScalar, RegionLibraryError> {
        self.checked(|s| {
            if s.phase != TailSessionPhase::Terminal
                || !s.result_present
                || !proof.complete()
                || s.banks
                    .iter()
                    .flatten()
                    .chain(s.work.iter())
                    .any(|b| *b != 0)
            {
                return Err(RegionLibraryError::Invariant);
            }
            let result = decode(s.binding.result, &s.result)?;
            s.erase_buffers();
            s.invocation.complete()?;
            s.phase = TailSessionPhase::Complete;
            Ok(result)
        })
    }
}
impl Drop for TailSession<'_> {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
#[path = "systems_tail_session_tests.rs"]
mod tests;
