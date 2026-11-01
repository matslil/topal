//! Portable scoped-borrow model; no source/native authority or erasure evidence.
use crate::{BootstrapRegion, BootstrapStorageState, QualifiedName, ResourceIdentity};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionLibraryError {
    Binding,
    Owner,
    View,
    InactiveFrame,
    Bounds,
    ReadOnly,
    ByteValue,
    Capacity,
    IdentityExhausted,
    NotRoot,
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionLibraryAccess {
    ReadOnly,
    ReadWrite,
}

/// Model evidence labels, never a checked executable/native code binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionLibraryBinding {
    identity: String,
    entry: String,
    calls: BTreeMap<String, BTreeSet<String>>,
    frames: usize,
    views: usize,
}

impl RegionLibraryBinding {
    /// Validate a closed acyclic graph and its complete frame bound.
    pub fn new(
        identity: String,
        entry: String,
        calls: BTreeMap<String, BTreeSet<String>>,
        frames: usize,
        views: usize,
    ) -> Result<Self, RegionLibraryError> {
        if identity.is_empty()
            || entry.is_empty()
            || !(1..=64).contains(&frames)
            || !(1..=256).contains(&views)
            || calls.is_empty()
            || calls.len() > 64
            || calls.keys().any(String::is_empty)
        {
            return Err(RegionLibraryError::Binding);
        }
        fn depth(
            name: &str,
            calls: &BTreeMap<String, BTreeSet<String>>,
            active: &mut BTreeSet<String>,
            seen: &mut BTreeMap<String, usize>,
        ) -> Result<usize, RegionLibraryError> {
            if let Some(depth) = seen.get(name) {
                return Ok(*depth);
            }
            if !active.insert(name.into()) {
                return Err(RegionLibraryError::Binding);
            }
            let mut maximum = 1;
            for callee in calls.get(name).ok_or(RegionLibraryError::Binding)? {
                maximum = maximum.max(1 + depth(callee, calls, active, seen)?);
            }
            active.remove(name);
            seen.insert(name.into(), maximum);
            Ok(maximum)
        }
        let mut seen = BTreeMap::new();
        let maximum = depth(&entry, &calls, &mut BTreeSet::new(), &mut seen)?;
        if maximum > frames || seen.len() != calls.len() {
            return Err(RegionLibraryError::Binding);
        }
        Ok(Self {
            identity,
            entry,
            calls,
            frames,
            views,
        })
    }
}

/// Opaque model token. Copies alias one borrow; they do not extend its lifetime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionLibraryView {
    owner: (String, u64),
    generation: u64,
    binding: String,
    frame: u64,
    identity: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Window {
    frame: u64,
    start: u64,
    count: u64,
    access: RegionLibraryAccess,
}

#[derive(Debug)]
struct Frame {
    identity: u64,
    function: String,
}

/// Exclusively borrows the actual existing storage owner until dropped.
#[derive(Debug)]
pub struct RegionLibraryInvocation<'a> {
    storage: &'a mut BootstrapStorageState,
    region: &'a BootstrapRegion,
    binding: &'a RegionLibraryBinding,
    generation: u64,
    frames: Vec<Frame>,
    windows: BTreeMap<u64, Window>,
    next_frame: u64,
    next_view: u64,
    ended: bool,
}

impl<'a> RegionLibraryInvocation<'a> {
    pub fn begin(
        storage: &'a mut BootstrapStorageState,
        region: &'a BootstrapRegion,
        binding: &'a RegionLibraryBinding,
        access: RegionLibraryAccess,
    ) -> Result<(Self, RegionLibraryView), RegionLibraryError> {
        let generation = storage.begin_library_generation(region).map_err(|error| {
            if error.code == "E-SYSTEMS-REGION-LIBRARY-GENERATION" {
                RegionLibraryError::IdentityExhausted
            } else {
                RegionLibraryError::Owner
            }
        })?;
        let invocation = Self {
            storage,
            region,
            binding,
            generation,
            frames: vec![Frame {
                identity: 1,
                function: binding.entry.clone(),
            }],
            windows: BTreeMap::from([(
                1,
                Window {
                    frame: 1,
                    start: 0,
                    count: region.byte_count(),
                    access,
                },
            )]),
            next_frame: 2,
            next_view: 2,
            ended: false,
        };
        let view = invocation.token(1, 1);
        Ok((invocation, view))
    }

    fn token(&self, frame: u64, identity: u64) -> RegionLibraryView {
        RegionLibraryView {
            owner: self.region.library_owner_identity(),
            generation: self.generation,
            binding: self.binding.identity.clone(),
            frame,
            identity,
        }
    }

    fn active(&self) -> Result<&Frame, RegionLibraryError> {
        if self.ended {
            return Err(RegionLibraryError::Ended);
        }
        self.frames.last().ok_or(RegionLibraryError::InactiveFrame)
    }

    fn window(&self, view: &RegionLibraryView) -> Result<Window, RegionLibraryError> {
        let active = self.active()?;
        if view.owner != self.region.library_owner_identity()
            || view.generation != self.generation
            || view.binding != self.binding.identity
        {
            return Err(RegionLibraryError::View);
        }
        if view.frame != active.identity {
            return Err(RegionLibraryError::InactiveFrame);
        }
        let window = *self
            .windows
            .get(&view.identity)
            .filter(|window| window.frame == view.frame)
            .ok_or(RegionLibraryError::View)?;
        Ok(window)
    }

    fn reserve_views(&self, count: usize) -> Result<u64, RegionLibraryError> {
        if count > self.binding.views.saturating_sub(self.windows.len()) {
            return Err(RegionLibraryError::Capacity);
        }
        self.next_view
            .checked_add(count as u64)
            .ok_or(RegionLibraryError::IdentityExhausted)
    }

    /// Dependency evidence only; observing this label grants no storage access.
    /// Windows, rights, frames and invocations do not split the owner's effects.
    pub fn effect_resource(
        &self,
        view: &RegionLibraryView,
    ) -> Result<ResourceIdentity, RegionLibraryError> {
        self.window(view)?;
        let (execution, allocation) = self.region.library_owner_identity();
        Ok(ResourceIdentity(QualifiedName(vec![
            "systems".into(),
            "borrowed-region".into(),
            execution,
            allocation.to_string(),
        ])))
    }

    pub fn derive(
        &mut self,
        view: &RegionLibraryView,
        offset: u64,
        count: u64,
        access: RegionLibraryAccess,
    ) -> Result<RegionLibraryView, RegionLibraryError> {
        let window = self.window(view)?;
        if window.access == RegionLibraryAccess::ReadOnly
            && access == RegionLibraryAccess::ReadWrite
        {
            return Err(RegionLibraryError::ReadOnly);
        }
        if offset
            .checked_add(count)
            .is_none_or(|end| end > window.count)
        {
            return Err(RegionLibraryError::Bounds);
        }
        let start = window
            .start
            .checked_add(offset)
            .ok_or(RegionLibraryError::Bounds)?;
        let next = self.reserve_views(1)?;
        let identity = self.next_view;
        self.windows.insert(
            identity,
            Window {
                frame: window.frame,
                start,
                count,
                access,
            },
        );
        self.next_view = next;
        Ok(self.token(window.frame, identity))
    }

    /// Validate every argument before changing the active frame or counters.
    pub fn enter(
        &mut self,
        function: &str,
        arguments: &[RegionLibraryView],
    ) -> Result<Vec<RegionLibraryView>, RegionLibraryError> {
        let current = self.active()?;
        if !self.binding.calls[&current.function].contains(function) {
            return Err(RegionLibraryError::Binding);
        }
        let next_view = self.reserve_views(arguments.len())?;
        let windows = arguments
            .iter()
            .map(|view| self.window(view))
            .collect::<Result<Vec<_>, _>>()?;
        if self.frames.len() >= self.binding.frames {
            return Err(RegionLibraryError::Capacity);
        }
        let next_frame = self
            .next_frame
            .checked_add(1)
            .ok_or(RegionLibraryError::IdentityExhausted)?;
        let frame = self.next_frame;
        let views = windows
            .iter()
            .enumerate()
            .map(|(index, _)| self.token(frame, self.next_view + index as u64))
            .collect();
        for (index, window) in windows.into_iter().enumerate() {
            self.windows
                .insert(self.next_view + index as u64, Window { frame, ..window });
        }
        self.frames.push(Frame {
            identity: frame,
            function: function.into(),
        });
        self.next_frame = next_frame;
        self.next_view = next_view;
        Ok(views)
    }

    pub fn return_from_call(&mut self) -> Result<(), RegionLibraryError> {
        let frame = self.active()?.identity;
        if self.frames.len() == 1 {
            return Err(RegionLibraryError::NotRoot);
        }
        self.windows.retain(|_, window| window.frame != frame);
        self.frames.pop();
        Ok(())
    }

    fn byte_offset(&self, window: Window, offset: u64) -> Result<u64, RegionLibraryError> {
        if offset >= window.count {
            return Err(RegionLibraryError::Bounds);
        }
        window
            .start
            .checked_add(offset)
            .filter(|offset| *offset < self.region.byte_count())
            .ok_or(RegionLibraryError::Bounds)
    }

    pub fn load_byte(
        &mut self,
        view: &RegionLibraryView,
        offset: u64,
    ) -> Result<u8, RegionLibraryError> {
        let offset = self.byte_offset(self.window(view)?, offset)?;
        self.storage
            .load_byte(self.region, offset)
            .map_err(|_| RegionLibraryError::Owner)
    }

    pub fn store_byte(
        &mut self,
        view: &RegionLibraryView,
        offset: u64,
        value: u64,
    ) -> Result<(), RegionLibraryError> {
        let window = self.window(view)?;
        if window.access != RegionLibraryAccess::ReadWrite {
            return Err(RegionLibraryError::ReadOnly);
        }
        let offset = self.byte_offset(window, offset)?;
        let value = u8::try_from(value).map_err(|_| RegionLibraryError::ByteValue)?;
        self.storage
            .store_byte(self.region, offset, value)
            .map_err(|_| RegionLibraryError::Owner)
    }

    pub fn complete(&mut self) -> Result<(), RegionLibraryError> {
        self.active()?;
        if self.frames.len() != 1 {
            return Err(RegionLibraryError::NotRoot);
        }
        self.revoke();
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), RegionLibraryError> {
        self.active()?;
        self.revoke();
        Ok(())
    }

    fn revoke(&mut self) {
        self.windows.clear();
        self.frames.clear();
        self.ended = true;
    }
}

impl Drop for RegionLibraryInvocation<'_> {
    fn drop(&mut self) {
        self.revoke();
    }
}

#[cfg(test)]
#[path = "systems_region_library_tests.rs"]
mod tests;
