impl RootEncoder {
    fn image_backing_call(&mut self, symbol: &'static str) -> Result<(), CompileError> {
        self.bytes.push(0xe8);
        self.frame_relocations.push((self.bytes.len() as u64, symbol, 0, true));
        self.bytes.extend_from_slice(&[0; 4]);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)
    }

    fn reserve_kernel_image_backing(&mut self, tables: bool) -> Result<(), CompileError> {
        let plan = self.image_backing_plan.as_ref().ok_or_else(|| CompileError::Tool(
            "E-SYSTEMS-KERNEL-IMAGE: native preparation requires complete current-image sizing and source binding".into()))?;
        let pages = plan.pages();
        if plan.is_tables() != tables { return Err(CompileError::Tool("image preparation resource and native plan differ".into())); }
        if !self.frame_allocator_created || self.frame_policy.is_none() || self.image_backing_live
            || self.physical_frame_count.is_some() || self.kernel_mapping_live
            || self.translation != RootTranslationState::BootstrapActive
            || self.frame_policy.as_ref().unwrap().image_preparation.is_some()
        {
            return Err(CompileError::Tool("image preparation requires the unborrowed initial allocator".into()));
        }
        use crate::systems_frame_ownership::Purpose as P;
        self.frame_internal_allocate(pages, "prepared-image-bootstrap", P::PreparedImageBootstrap)?;
        self.frame_internal_allocate(pages, "prepared-image-process", P::PreparedImageProcess)?;
        if tables { self.frame_internal_allocate(1, "prepared-image-low-leaf", P::PreparedImageLowUserLeaf)?; }
        let frame = self.frame_policy.as_mut().unwrap();
        let banks = ["prepared-image-bootstrap", "prepared-image-process"].map(|name|
            frame.capture_owner(name).ok_or_else(|| CompileError::Tool("missing whole image-preparation owner".into())));
        let [bootstrap, process] = banks;
        frame.image_preparation = Some(crate::systems_image_backing::Bindings {
            root_object_sha256: String::new(), token_bytes: 0,
            banks: [bootstrap?, process?],
            low_user_leaf: if tables { Some(frame.capture_owner("prepared-image-low-leaf").ok_or_else(|| CompileError::Tool("missing low leaf owner".into()))?) } else { None },
        });
        self.image_backing_call(if tables { crate::systems_image_backing::TABLE_PREPARE } else { crate::systems_image_backing::PREPARE })?;
        self.image_backing_live = true;
        Ok(())
    }

    fn discard_kernel_image_backing(&mut self, tables: bool) -> Result<(), CompileError> {
        if !self.image_backing_live || self.image_backing_plan.as_ref().is_none_or(|plan| plan.is_tables() != tables) {
            return Err(CompileError::Tool("image-backing discard requires its live preparation".into()));
        }
        self.image_backing_call(if tables { crate::systems_image_backing::TABLE_DISCARD } else { crate::systems_image_backing::DISCARD })?;
        let mut owners = vec!["prepared-image-bootstrap", "prepared-image-process"];
        if tables { owners.push("prepared-image-low-leaf"); }
        for owner in owners {
            self.frame_return(owner)?;
            self.frame_release_source()?;
        }
        self.image_backing_call(if tables { crate::systems_image_backing::TABLE_RELEASED } else { crate::systems_image_backing::RELEASED })?;
        self.image_backing_live = false;
        Ok(())
    }
}
