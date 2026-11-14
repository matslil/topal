// Private owning-frame lowering. The public source-bearing publisher remains
// closed until the complete root/provider/native/load binding is reconstructed.
const FRAME_ROOT_TOKENS: &str = "topal_x86_frame_root_tokens_v1";
#[derive(Default)]
struct RootFramePolicy {
    image_preparation: Option<crate::systems_image_backing::Bindings>,
    next_slot: u64,
    current: Option<u64>,
    payload: Option<u64>,
    owners: std::collections::BTreeMap<&'static str, u64>,
    allocations: std::collections::BTreeMap<u64, crate::systems_frame_ownership::Allocation>,
    user_owners: Vec<Option<crate::systems_frame_ownership::EntryOwners>>,
}
impl RootFramePolicy {
    fn capture_owner(&self, name: &'static str) -> Option<crate::systems_user_extent::Owner> {
        let slot = *self.owners.get(name)?;
        let allocation = *self.allocations.get(&slot)?;
        Some(crate::systems_user_extent::Owner {
            slot, count: allocation.count, alignment: allocation.alignment,
            purpose: allocation.purpose,
            pins: self.owners.values().filter(|&&owner| owner == slot).count() as u64,
        })
    }
}
impl RootEncoder {
    fn frame_address(&mut self, opcode: &[u8], symbol: &'static str, offset: u64) {
        self.bytes.extend_from_slice(opcode);
        self.frame_relocations
            .push((self.bytes.len() as u64, symbol, offset, false));
        self.bytes.extend_from_slice(&[0; 4]);
    }
    fn frame_imm(&mut self, opcode: &[u8], value: u64) {
        self.bytes.extend_from_slice(opcode);
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn frame_call(
        &mut self,
        op: u64,
        a: Option<u64>,
        b: u64,
        slot: Option<u64>,
        facts: bool,
    ) -> Result<(), CompileError> {
        self.frame_imm(&[0x48, 0xbf], op);
        if let Some(a) = a {
            self.frame_imm(&[0x48, 0xbe], a);
        }
        self.frame_imm(&[0x48, 0xba], b);
        if let Some(slot) = slot {
            self.frame_address(
                &[0x48, 0x8d, 0x0d],
                FRAME_ROOT_TOKENS,
                slot.checked_mul(24)
                    .ok_or_else(|| CompileError::Tool("frame token offset overflow".into()))?,
            );
        } else {
            self.bytes.extend_from_slice(&[0x31, 0xc9]);
        }
        if facts {
            self.frame_address(&[0x4c, 0x8d, 0x05], "topal_x86_frame_details_v1", 0);
        } else {
            self.bytes.extend_from_slice(&[0x45, 0x31, 0xc0]);
        }
        self.bytes.push(0xe8);
        self.frame_relocations.push((
            self.bytes.len() as u64,
            "topal_x86_frame_gateway_v1",
            0,
            true,
        ));
        self.bytes.extend_from_slice(&[0; 4]);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)
    }
    fn frame_new_allocation(
        &mut self,
        count: u64,
        alignment: u64,
        purpose: crate::systems_frame_ownership::Purpose,
    ) -> Result<u64, CompileError> {
        let policy = self.frame_policy.as_mut().unwrap();
        let slot = policy.next_slot;
        policy.next_slot = slot
            .checked_add(1)
            .ok_or_else(|| CompileError::Tool("frame token count overflow".into()))?;
        policy.allocations.insert(slot, crate::systems_frame_ownership::Allocation {
            count, alignment, purpose,
        });
        self.frame_call(1, Some(count), alignment, Some(slot), false)?;
        self.frame_call(4, Some(0), 0, Some(slot), true)?;
        self.frame_address(&[0x4c, 0x8d, 0x1d], "topal_x86_frame_details_v1", 0);
        self.bytes
            .extend_from_slice(&[0x49, 0x8b, 0x03, 0x48, 0xc1, 0xe0, 0x0c]);
        // Consume the private provider result after materializing its base.
        self.bytes
            .extend_from_slice(&[0x49, 0xc7, 0x03, 0, 0, 0, 0]);
        for offset in [8, 16, 24] {
            self.bytes
                .extend_from_slice(&[0x49, 0xc7, 0x43, offset, 0, 0, 0, 0]);
        }
        Ok(slot)
    }
    fn frame_allocate_source(
        &mut self,
        count: u64,
        alignment: u64,
        purpose: crate::systems_frame_ownership::Purpose,
    ) -> Result<(), CompileError> {
        let old = self.frame_policy.as_mut().unwrap().current.take();
        if let Some(old) = old {
            if self.physical_frame_count != Some(503)
                || self.frame_policy.as_ref().unwrap().payload.is_some()
            {
                return Err(CompileError::Tool(format!(
                    "unexpected nested source frame owner: previous {:?}, request {count}, slot {old}, owners {:?}",
                    self.physical_frame_count,
                    self.frame_policy.as_ref().unwrap().owners
                )));
            }
            self.frame_policy.as_mut().unwrap().payload = Some(old);
        }
        let slot = self.frame_new_allocation(count, alignment, purpose)?;
        self.frame_policy.as_mut().unwrap().current = Some(slot);
        Ok(())
    }
    fn frame_pin(&mut self, slot: u64, acquire: bool) -> Result<(), CompileError> {
        self.frame_call(3, Some(u64::from(acquire)), 0, Some(slot), false)
    }
    fn frame_take(&mut self, owner: &'static str) -> Result<u64, CompileError> {
        self.frame_policy
            .as_mut()
            .unwrap()
            .owners
            .remove(owner)
            .ok_or_else(|| CompileError::Tool(format!("missing frame owner {owner}")))
    }
    fn frame_hold(&mut self, owner: &'static str, slot: u64) -> Result<(), CompileError> {
        if self
            .frame_policy
            .as_mut()
            .unwrap()
            .owners
            .insert(owner, slot)
            .is_some()
        {
            return Err(CompileError::Tool(format!("duplicate frame owner {owner}")));
        }
        self.frame_pin(slot, true)
    }
    fn frame_move(&mut self, owner: &'static str) -> Result<(), CompileError> {
        let slot = self
            .frame_policy
            .as_mut()
            .unwrap()
            .current
            .take()
            .ok_or_else(|| CompileError::Tool("missing current frame handle".into()))?;
        self.frame_hold(owner, slot)
    }
    fn frame_return(&mut self, owner: &'static str) -> Result<(), CompileError> {
        if self.frame_policy.as_ref().unwrap().current.is_some() {
            return Err(CompileError::Tool(
                "frame return overlaps current owner".into(),
            ));
        }
        let slot = self.frame_take(owner)?;
        self.frame_pin(slot, false)?;
        self.frame_policy.as_mut().unwrap().current = Some(slot);
        Ok(())
    }
    fn frame_release_source(&mut self) -> Result<(), CompileError> {
        let slot = self
            .frame_policy
            .as_mut()
            .unwrap()
            .current
            .take()
            .ok_or_else(|| CompileError::Tool("release has no affine frame handle".into()))?;
        self.frame_call(2, Some(0), 0, Some(slot), false)
    }
    fn frame_internal_allocate(
        &mut self,
        count: u64,
        owner: &'static str,
        purpose: crate::systems_frame_ownership::Purpose,
    ) -> Result<(), CompileError> {
        let slot = self.frame_new_allocation(count, 1, purpose)?;
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xc4]); // retain physical base in r12
        self.frame_hold(owner, slot)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe0]);
        Ok(())
    }
    fn frame_after_operation(&mut self, op: &CompilerSystemsOperation) -> Result<(), CompileError> {
        use CompilerSystemsOperation as O;
        match op {
            O::ReleasePhysicalFrames if self.physical_frame_count == Some(503) => {
                let slot = self
                    .frame_policy
                    .as_mut()
                    .unwrap()
                    .payload
                    .take()
                    .ok_or_else(|| {
                        CompileError::Tool("missing retained payload frame handle".into())
                    })?;
                self.frame_policy.as_mut().unwrap().current = Some(slot);
            }
            O::MapKernelFrames { .. } | O::MapTranslationFrames { .. } => {
                self.frame_move("kernel-map")?
            }
            O::UnmapKernelFrames => self.frame_return("kernel-map")?,
            O::CommitTranslationEdit {
                kind: CompilerTranslationEditKind::Unmap,
                ..
            } => {
                self.frame_return("kernel-map")?;
                let metadata = self.frame_take("translation-edit")?;
                self.frame_pin(metadata, false)?;
                self.frame_call(2, Some(0), 0, Some(metadata), false)?;
            }
            O::PrepareStaticImageConstruction { .. } => {
                let frame = self.frame_policy.as_mut().unwrap();
                let metadata = frame.current.ok_or_else(|| CompileError::Tool("missing construction metadata".into()))?;
                frame.allocations.get_mut(&metadata)
                    .ok_or_else(|| CompileError::Tool("missing metadata allocation binding".into()))?
                    .refine_construction(crate::systems_frame_ownership::Purpose::StaticImageLeaf)?;
                self.frame_move("construction-metadata")?;
                let payload = self
                    .frame_policy
                    .as_mut()
                    .unwrap()
                    .payload
                    .take()
                    .ok_or_else(|| CompileError::Tool("missing construction payload".into()))?;
                self.frame_policy.as_mut().unwrap().allocations.get_mut(&payload)
                    .ok_or_else(|| CompileError::Tool("missing payload allocation binding".into()))?
                    .refine_construction(crate::systems_frame_ownership::Purpose::StaticImagePayload)?;
                self.frame_hold("construction-payload", payload)?;
            }
            O::RevokeStaticImageConstructionMetadata => {
                self.frame_return("construction-metadata")?;
                let payload = self.frame_take("construction-payload")?;
                self.frame_pin(payload, false)?;
                self.frame_policy.as_mut().unwrap().payload = Some(payload);
            }
            O::AttachStaticImageAddressSpace => self.frame_move("construction-hierarchy")?,
            O::ReclaimStaticImageConstructionHierarchy => {
                self.frame_return("construction-hierarchy")?
            }
            O::PublishStaticImageStackLeaf => {
                let slot = self
                    .frame_policy
                    .as_ref()
                    .unwrap()
                    .current
                    .ok_or_else(|| CompileError::Tool("missing stack backing".into()))?;
                self.frame_hold("stack-loan", slot)?;
            }
            O::RevokeStaticImageStackLeaf => {
                let slot = self.frame_take("stack-loan")?;
                self.frame_pin(slot, false)?;
            }
            O::RetainStaticImagePreparedProcess => self.frame_move("prepared-stack")?,
            O::RestoreStaticImagePreparedProcess => self.frame_return("prepared-stack")?,
            O::EnterStaticImageProcess => {
                // Capture compile-time affine owners at the actual process
                // call, before their later restoration/release. This records
                // no runtime address and changes no emitted root instruction.
                let frame = self.frame_policy.as_mut().unwrap();
                let capture = |name| {
                    let slot = *frame.owners.get(name)?;
                    let allocation = *frame.allocations.get(&slot)?;
                    Some(crate::systems_user_extent::Owner {
                        slot,
                        count: allocation.count,
                        alignment: allocation.alignment,
                        purpose: allocation.purpose,
                        pins: frame.owners.values().filter(|&&owner| owner == slot).count() as u64,
                    })
                };
                let roles = [
                    "construction-hierarchy",
                    "construction-metadata",
                    "construction-payload",
                    "prepared-stack",
                ]
                .into_iter()
                .map(capture)
                .collect::<Option<Vec<_>>>()
                .and_then(|roles| roles.try_into().ok());
                let captured = capture("bootstrap-translation").zip(roles).map(|(bootstrap, process)|
                    crate::systems_frame_ownership::EntryOwners { bootstrap, process });
                frame.user_owners.push(captured);
            }
            O::PrepareUserAddressSpace => self.frame_move("user-address-space")?,
            O::ReclaimUserAddressSpace => self.frame_return("user-address-space")?,
            O::MapUserAddressSpace => {
                let owner = if self
                    .frame_policy
                    .as_ref()
                    .unwrap()
                    .owners
                    .contains_key("user-code")
                {
                    "user-data"
                } else {
                    "user-code"
                };
                self.frame_move(owner)?;
            }
            O::UnmapUserAddressSpace => {
                self.frame_return(if self.physical_frame_count == Some(2) {
                    "user-code"
                } else {
                    "user-data"
                })?
            }
            O::ReserveAnonymousUserMapping => self.frame_move("user-anonymous")?,
            O::ReleaseAnonymousUserMapping => self.frame_return("user-anonymous")?,
            _ => {}
        }
        Ok(())
    }
}
