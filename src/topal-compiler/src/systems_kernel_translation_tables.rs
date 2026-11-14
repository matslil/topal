//! Inactive x86 table preparation for the separately selected image refinement.
//!
//! TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001 / OWNERS-001. This module produces ordinary
//! bytes and an exact storage requirement. It supplies no live frame, table,
//! root, entry, publication or protected-context authority. Native consumers
//! must bind those obligations before using any prepared bytes.
use super::{SystemsKernelImageLayout, SystemsKernelImageRole, SystemsKernelImageSpan};
use crate::CompileError;

const PAGE: u64 = 4096;
const LARGE: u64 = 2 * 1024 * 1024;
const LIMIT: u64 = 1024 * 1024 * 1024;
const NX: u64 = 1 << 63;
const ROOT: usize = 0;
const PDPT: usize = 1;
const LOW_PD: usize = 2;
const MMIO_PD: usize = 3;
const USER_HIGH: usize = 4;
const FIXED: [usize; 5] = [ROOT, PDPT, LOW_PD, MMIO_PD, USER_HIGH];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BankKind {
    Bootstrap,
    Process { low_user_table: u64 },
}

/// Provider-specific plan for the whole bank, including its fixed levels and
/// reserved high user table. The separate low user table is not bank storage.
/// Private fields prevent changing a count independently of the actual shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TranslationTables {
    image: Vec<SystemsKernelImageSpan>,
    complete: Vec<SystemsKernelImageSpan>,
    split_blocks: Vec<u64>,
}

fn invalid(message: &str) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-KERNEL-TABLES: {message}"))
}

fn append(
    parts: &mut Vec<SystemsKernelImageSpan>,
    start: u64,
    end: u64,
    role: SystemsKernelImageRole,
) {
    if start == end {
        return;
    }
    if let Some(last) = parts.last_mut() {
        if last.end == start && last.role == role {
            last.end = end;
            return;
        }
    }
    parts.push(SystemsKernelImageSpan { start, end, role });
}

impl TranslationTables {
    pub(super) fn matches_layout(
        &self,
        layout: &SystemsKernelImageLayout,
    ) -> Result<bool, CompileError> {
        Ok(*self == Self::new(layout)?)
    }
    /// The input is descriptive, even when obtained through fresh compilation.
    /// This constructor establishes geometry only; it cannot establish owners.
    pub(super) fn new(layout: &SystemsKernelImageLayout) -> Result<Self, CompileError> {
        use SystemsKernelImageRole::Mutable;
        if layout.minimum_granule != PAGE
            || layout.large_granule != LARGE
            || layout.address_limit != LIMIT
            || layout.spans.is_empty()
        {
            return Err(invalid("unsupported or empty image geometry"));
        }
        let mut complete = Vec::new();
        let mut previous = 0;
        for part in &layout.spans {
            if part.start < previous
                || part.start < 0x200000
                || part.start >= part.end
                || part.end > LIMIT
                || part.start % PAGE != 0
                || part.end % PAGE != 0
                || (part.start < 0x800000 && part.end > 0x400000)
            {
                return Err(invalid("invalid image partition or user-window conflict"));
            }
            // This draft provider template explicitly retains the original
            // supervisor physical coverage, with non-image pages RW/NX. It is
            // not inferred from holes in the image-only description. Live boot
            // stacks, descriptor tables, handlers and memory kinds still need
            // their native owner/mode/transition qualification.
            append(&mut complete, previous, part.start, Mutable);
            append(&mut complete, part.start, part.end, part.role);
            previous = part.end;
        }
        append(&mut complete, previous, LIMIT, Mutable);
        let mut split_blocks = Vec::new();
        for block in (0..LIMIT).step_by(LARGE as usize) {
            let index = complete.partition_point(|p| p.end <= block);
            if complete[index].end < block + LARGE {
                // Adjacent equal roles were merged, so this boundary really
                // changes permissions. A partial mutable-only image does not
                // need a redundant leaf table in the explicit RW/NX coverage.
                split_blocks.push(block);
            }
        }
        Ok(Self {
            image: layout.spans.clone(),
            complete,
            split_blocks,
        })
    }

    pub(super) fn pages(&self) -> u64 {
        (FIXED.len() + self.split_blocks.len()) as u64
    }

    pub(super) fn bytes(&self) -> u64 {
        self.pages() * PAGE
    }

    fn role(&self, address: u64) -> SystemsKernelImageRole {
        self.complete[self.complete.partition_point(|p| p.end <= address)].role
    }

    fn leaf(&self, address: u64, large: bool) -> u64 {
        use SystemsKernelImageRole::{Code, Immutable, Mutable};
        let flags = match self.role(address) {
            Code => 0x61,
            Immutable => NX | 0x61,
            Mutable => NX | 0x63,
        };
        address | flags | if large { 0x80 } else { 0 }
    }

    fn disjoint(&self, start: u64, bytes: u64) -> bool {
        start % PAGE == 0
            && start >= 0x1000000
            && start.checked_add(bytes).is_some_and(|end| {
                end <= LIMIT && self.image.iter().all(|p| start >= p.end || end <= p.start)
            })
    }

    fn validate(&self, base: u64, kind: BankKind) -> Result<(), CompileError> {
        if !self.disjoint(base, self.bytes()) {
            return Err(invalid("invalid or image-aliasing bank extent"));
        }
        if let BankKind::Process { low_user_table } = kind {
            if !self.disjoint(low_user_table, PAGE)
                || (low_user_table < base + self.bytes() && base < low_user_table + PAGE)
            {
                return Err(invalid("invalid or aliasing low user table extent"));
            }
        }
        Ok(())
    }

    fn template(&self) -> (Vec<u64>, Vec<(usize, u64)>) {
        let mut words = vec![0; self.bytes() as usize / 8];
        let mut fixes = vec![
            (ROOT * 512, PDPT as u64 * PAGE | 7),
            (PDPT * 512, LOW_PD as u64 * PAGE | 7),
            (PDPT * 512 + 3, MMIO_PD as u64 * PAGE | 3),
        ];
        let mut put = |table: usize, index: usize, value: u64| {
            words[table * 512 + index] = value;
        };
        let mut next = FIXED.len();
        for block in (0..LIMIT).step_by(LARGE as usize) {
            let index = (block / LARGE) as usize;
            if self.split_blocks.binary_search(&block).is_ok() {
                fixes.push((LOW_PD * 512 + index, next as u64 * PAGE | 3));
                for entry in 0..512 {
                    let physical = block + entry as u64 * PAGE;
                    put(next, entry, self.leaf(physical, false));
                }
                next += 1;
            } else {
                put(LOW_PD, index, self.leaf(block, true));
            }
        }
        assert_eq!(next, self.pages() as usize);
        // Retain the existing target's MMIO cache attributes and add NX.
        // Actual PAT/MTRR interpretation remains part of native qualification.
        put(MMIO_PD, 502, NX | 0xfec0009b);
        put(MMIO_PD, 503, NX | 0xfee0009b);
        (words, fixes)
    }

    /// Encode a complete inactive bank. Numeric addresses here authorize no
    /// physical reads/writes; the result is just a host byte vector.
    pub(super) fn encode(&self, base: u64, kind: BankKind) -> Result<Vec<u8>, CompileError> {
        self.validate(base, kind)?;
        let (mut words, fixes) = self.template();
        for (at, relative) in fixes {
            words[at] = base + relative;
        }
        if let BankKind::Process { low_user_table } = kind {
            words[LOW_PD * 512 + 2] = low_user_table | 7;
            words[LOW_PD * 512 + 3] = (base + USER_HIGH as u64 * PAGE) | 7;
        }
        Ok(words.into_iter().flat_map(u64::to_le_bytes).collect())
    }

    /// Admission geometry for the USER walker. Kernel-only subdivisions are
    /// deliberately excluded even though the same whole bank owns their pages.
    pub(super) fn user_table(&self, base: u64, low: u64, physical: u64, level: u64) -> bool {
        if self
            .validate(
                base,
                BankKind::Process {
                    low_user_table: low,
                },
            )
            .is_err()
        {
            return false;
        }
        match level {
            4 => physical == base,
            3 => physical == base + PAGE,
            2 => [base + 2 * PAGE, base + 3 * PAGE].contains(&physical),
            1 => [low, base + 4 * PAGE].contains(&physical),
            _ => false,
        }
    }
}

#[path = "systems_kernel_translation_prepare.rs"]
mod prepare;

#[path = "systems_kernel_translation_verify.rs"]
mod verify;

#[cfg(test)]
#[path = "systems_kernel_translation_tables_tests.rs"]
mod tests;
