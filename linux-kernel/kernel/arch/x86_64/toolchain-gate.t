use language (
  version is v0.1,
  features is ( systems )
)

boot is fn (context : BootstrapContext) -> BootstrapDisposition
  context boot describe memory
    Ok described then {
      described memory create frame allocator
        Ok memory then {
          memory frames allocate (
            frame-count is 1,
            alignment-frames is 1
          )
            Ok frames then {
              memory console write "TOPAL_KERNEL_FRAME_ALLOCATED"
              memory kernel map frames (
                rights is read-write,
                execution is denied,
                memory-kind is normal
              )
                Ok mapping then {
                  mapping byte store (offset-bytes is 0, value is 165)
                  mapped-observed : Nat is mapping byte load (offset-bytes is 0)
                  mapped-observed = 165
                    true then {
                      frames is memory kernel unmap mapping
                      memory console write "TOPAL_KERNEL_FRAME_MAPPED"
                      memory frames release frames
                      memory translation begin (
                        template is bootstrap-equivalent,
                        page-policy is provider-selected
                      )
                        Ok update then {
                          memory translation commit update
                            Ok space then {
                              memory translation activate space
                                Ok translated then {
                                  translated console write "TOPAL_KERNEL_TRANSLATION_ACTIVE"
                                  translated frames allocate (
                                    frame-count is 1,
                                    alignment-frames is 1
                                  )
                                    Ok dynamic-frames then {
                                      translated translation edit begin
                                        Ok map-edit then {
                                          map-edit kernel map dynamic-frames (
                                            rights is read-write,
                                            execution is denied,
                                            memory-kind is normal,
                                            placement is provider-selected
                                          )
                                            Ok dynamic-mapping then {
                                              map-edit translation commit
                                                Ok edited then {
                                                  dynamic-mapping byte store (offset-bytes is 0, value is 60)
                                                  dynamic-observed : Nat is dynamic-mapping byte load (offset-bytes is 0)
                                                  dynamic-observed = 60
                                                    true then {
                                                      edited translation edit begin
                                                        Ok unmap-edit then {
                                                          removed-frames is unmap-edit kernel unmap dynamic-mapping
                                                          unmap-edit translation commit
                                                            Ok unmapped then {
                                                              unmapped console write "TOPAL_KERNEL_TRANSLATION_EDITED"
                                                              unmapped frames release removed-frames
                                                              unmapped frames allocate (
                                                                frame-count is 3,
                                                                alignment-frames is 1
                                                              )
                                                                Ok frame-extent then {
                                                                  unmapped console write "TOPAL_KERNEL_FRAME_EXTENT_OK"
                                                                  unmapped frames release frame-extent
                                                                  unmapped frames allocate (
                                                                    frame-count is 503,
                                                                    alignment-frames is 1
                                                                  )
                                                                    Ok static-elf-frames then {
                                                                      unmapped console write "TOPAL_KERNEL_STATIC_ELF_FRAME_EXTENT_OK"
                                                                      unmapped frames allocate (
                                                                        frame-count is 1,
                                                                        alignment-frames is 1
                                                                      )
                                                                        Ok static-elf-leaf-table-frame then {
                                                                          unmapped console write "TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_OK"
                                                                          unmapped static image construction prepare (
                                                                            payload is static-elf-frames,
                                                                            metadata is static-elf-leaf-table-frame,
                                                                            layout is bounded-static-elf-initial-rw-nx
                                                                          )
                                                                            Ok static-elf-construction then {
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_OK"
                                                                              zeroed-static-elf-construction is unmapped static image construction zero payload (
                                                                                owner is static-elf-construction,
                                                                                extent is complete
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_OK"
                                                                              populated-static-elf-construction is unmapped static image construction copy payload (
                                                                                owner is zeroed-static-elf-construction,
                                                                                source is retained-static-user-elf,
                                                                                plan is bounded-static-elf-four-loads
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK"
                                                                              sealed-static-elf-construction is unmapped static image construction seal permissions (
                                                                                owner is populated-static-elf-construction,
                                                                                plan is bounded-static-elf-wx-relro
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK"
                                                                              static-elf-hierarchy-frames is unmapped static image construction reserve hierarchy (
                                                                                owner is sealed-static-elf-construction,
                                                                                frame-count is 4,
                                                                                alignment-frames is 1
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK"
                                                                              attached-static-elf-construction is unmapped static image construction attach address space (
                                                                                owner is sealed-static-elf-construction,
                                                                                hierarchy is static-elf-hierarchy-frames,
                                                                                layout is bounded-static-elf-inactive-address-space
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK"
                                                                              active-static-elf-construction is unmapped static image construction activate address space (
                                                                                owner is attached-static-elf-construction,
                                                                                policy is bounded-static-elf-kernel-continuation
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK"
                                                                              restored-static-elf-construction is unmapped static image construction restore address space (
                                                                                owner is active-static-elf-construction,
                                                                                policy is bounded-bootstrap-kernel-continuation
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK"
                                                                              entry-authorized-static-elf-construction is unmapped static image construction authorize entry (
                                                                                owner is restored-static-elf-construction,
                                                                                policy is validated-static-elf-entry
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK"
                                                                              stack-slot-authorized-static-elf-construction is unmapped static image construction authorize stack slot (
                                                                                owner is entry-authorized-static-elf-construction,
                                                                                policy is bounded-linux-initial-stack-slot
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK"
                                                                              static-elf-stack-backing is unmapped static image construction reserve stack backing (
                                                                                owner is stack-slot-authorized-static-elf-construction,
                                                                                frame-count is 1,
                                                                                alignment-frames is 1
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK"
                                                                              zeroed-stack-backing-construction is unmapped static image construction zero stack backing (
                                                                                owner is stack-slot-authorized-static-elf-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                extent is complete
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK"
                                                                              published-stack-leaf-construction is unmapped static image construction publish stack leaf (
                                                                                owner is zeroed-stack-backing-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                access is writable-non-executable,
                                                                                extent is complete
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK"
                                                                              initialized-stack-entry-construction is unmapped static image construction initialize stack entry (
                                                                                owner is published-stack-leaf-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                profile is bounded-static-elf-busybox-true,
                                                                                extent is complete
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK"
                                                                              pagesz-stack-entry-construction is unmapped static image construction add stack pagesz (
                                                                                owner is initialized-stack-entry-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is provider-base-page,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK"
                                                                              entry-auxv-stack-construction is unmapped static image construction add stack entry fact (
                                                                                owner is pagesz-stack-entry-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is authorized-static-elf-entry,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK"
                                                                              program-header-stack-construction is unmapped static image construction add stack program headers (
                                                                                owner is entry-auxv-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is resident-static-elf-program-headers,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK"
                                                                              executable-name-stack-construction is unmapped static image construction add stack executable name (
                                                                                owner is program-header-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is initial-executable-name,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK"
                                                                              credential-stack-construction is unmapped static image construction add stack credentials (
                                                                                owner is executable-name-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is initial-process-credentials,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK"
                                                                              secure-mode-stack-construction is unmapped static image construction add stack secure mode (
                                                                                owner is credential-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is initial-execution-security,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK"
                                                                              clock-tick-rate-stack-construction is unmapped static image construction add stack clock tick rate (
                                                                                owner is secure-mode-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is linux-user-hz,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK"
                                                                              platform-stack-construction is unmapped static image construction add stack platform (
                                                                                owner is clock-tick-rate-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is native-userspace-platform,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK"
                                                                              hardware-capabilities-stack-construction is unmapped static image construction add stack hardware capabilities (
                                                                                owner is platform-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is native-userspace-hardware-capabilities,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK"
                                                                              load-metadata-stack-construction is unmapped static image construction add stack load metadata (
                                                                                owner is hardware-capabilities-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is static-executable-load-metadata,
                                                                                policy is bounded-linux-auxv-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK"
                                                                              random-slot-authorized-stack-construction is unmapped static image construction authorize stack random slot (
                                                                                owner is load-metadata-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is initial-auxv-random-object,
                                                                                extent-bytes is 16,
                                                                                policy is bounded-linux-auxv-random-before-null
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK"
                                                                              retained-static-process is unmapped static image construction retain prepared process (
                                                                                owner is random-slot-authorized-stack-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                value is bounded-static-elf-initial-process,
                                                                                policy is bounded-static-elf-pre-entropy-retention
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK"
                                                                              retained-static-process-after-disjoint-work is unmapped static image construction qualify retained disjoint work (
                                                                                owner is retained-static-process,
                                                                                frame-count is 1,
                                                                                alignment-frames is 1,
                                                                                policy is bounded-static-elf-retained-disjoint-work
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK"
                                                                              entropy-pending-static-process is unmapped static image construction authorize entropy handoff (
                                                                                owner is retained-static-process-after-disjoint-work,
                                                                                process is bounded-static-elf-initial-process,
                                                                                policy is bounded-static-elf-initial-random-delivery
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK"
                                                                              entropy-acquiring-static-process is unmapped static image construction authorize entropy acquisition (
                                                                                owner is entropy-pending-static-process,
                                                                                output is initial-entropy-output,
                                                                                generation is generation-two,
                                                                                policy is bounded-static-elf-initial-entropy-acquisition
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK"
                                                                              entropy-acquiring-static-process-after-device is unmapped static image construction qualify entropy device (
                                                                                owner is entropy-acquiring-static-process,
                                                                                device is initial-virtio-rng-device,
                                                                                policy is bounded-static-elf-entropy-device-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK"
                                                                              entropy-acquiring-static-process-after-transport is unmapped static image construction qualify entropy transport (
                                                                                owner is entropy-acquiring-static-process-after-device,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                policy is bounded-static-elf-entropy-transport-discovery
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK"
                                                                              entropy-acquiring-static-process-after-negotiation is unmapped static image construction qualify entropy negotiation (
                                                                                owner is entropy-acquiring-static-process-after-transport,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                policy is bounded-static-elf-entropy-transport-negotiation
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK"
                                                                              entropy-acquiring-static-process-after-queue is unmapped static image construction qualify entropy queue (
                                                                                owner is entropy-acquiring-static-process-after-negotiation,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                policy is bounded-static-elf-entropy-queue-discovery
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_OK"
                                                                              entropy-acquiring-static-process-after-bus-master is unmapped static image construction qualify entropy bus master (
                                                                                owner is entropy-acquiring-static-process-after-queue,
                                                                                device is initial-virtio-rng-device,
                                                                                policy is bounded-static-elf-entropy-bus-master
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_OK"
                                                                              entropy-acquiring-static-process-after-dma-buffers is unmapped static image construction qualify entropy dma buffers (
                                                                                owner is entropy-acquiring-static-process-after-bus-master,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                policy is bounded-static-elf-entropy-dma-buffer-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_OK"
                                                                              entropy-acquiring-static-process-after-split-ring is unmapped static image construction qualify entropy split ring (
                                                                                owner is entropy-acquiring-static-process-after-dma-buffers,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                policy is bounded-static-elf-entropy-split-ring-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_OK"
                                                                              entropy-acquiring-static-process-after-dma-preparation is unmapped static image construction qualify entropy dma preparation (
                                                                                owner is entropy-acquiring-static-process-after-split-ring,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                policy is bounded-static-elf-entropy-dma-preparation-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_OK"
                                                                              entropy-acquiring-static-process-after-queue-addresses is unmapped static image construction qualify entropy queue addresses (
                                                                                owner is entropy-acquiring-static-process-after-dma-preparation,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                policy is bounded-static-elf-entropy-queue-address-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_OK"
                                                                              entropy-acquiring-static-process-after-queue-activation is unmapped static image construction qualify entropy queue activation (
                                                                                owner is entropy-acquiring-static-process-after-queue-addresses,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                policy is bounded-static-elf-entropy-empty-queue-activation-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_OK"
                                                                              entropy-acquiring-static-process-after-request-completion is unmapped static image construction qualify entropy request completion (
                                                                                owner is entropy-acquiring-static-process-after-queue-activation,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                policy is bounded-static-elf-entropy-request-completion-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_OK"
                                                                              entropy-acquiring-static-process-after-sample-capture is unmapped static image construction qualify entropy sample capture (
                                                                                owner is entropy-acquiring-static-process-after-request-completion,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                sample is initial-virtio-rng-sample,
                                                                                policy is bounded-static-elf-entropy-sample-capture-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_OK"
                                                                              entropy-acquiring-static-process-after-sample-quality is unmapped static image construction qualify entropy sample quality (
                                                                                owner is entropy-acquiring-static-process-after-sample-capture,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                sample is initial-virtio-rng-sample,
                                                                                source is initial-virtio-rng-source,
                                                                                evidence is initial-q35-virtio-rng-trust-evidence,
                                                                                policy is bounded-static-elf-entropy-sample-quality-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_OK"
                                                                              entropy-acquiring-static-process-after-pool-lifecycle is unmapped static image construction qualify entropy pool lifecycle (
                                                                                owner is entropy-acquiring-static-process-after-sample-quality,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                sample is initial-virtio-rng-sample,
                                                                                source is initial-virtio-rng-source,
                                                                                evidence is initial-q35-virtio-rng-trust-evidence,
                                                                                pool is initial-entropy-pool,
                                                                                conditioner is initial-blake2s-conditioner,
                                                                                contribution is initial-entropy-contribution,
                                                                                policy is bounded-static-elf-entropy-pool-lifecycle-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_OK"
                                                                              entropy-acquiring-static-process-after-generator-lifecycle is unmapped static image construction qualify entropy generator lifecycle (
                                                                                owner is entropy-acquiring-static-process-after-pool-lifecycle,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                sample is initial-virtio-rng-sample,
                                                                                source is initial-virtio-rng-source,
                                                                                evidence is initial-q35-virtio-rng-trust-evidence,
                                                                                pool is initial-entropy-pool,
                                                                                conditioner is initial-blake2s-conditioner,
                                                                                contribution is initial-entropy-contribution,
                                                                                generator is initial-entropy-generator,
                                                                                algorithm is chacha20,
                                                                                output is initial-entropy-output,
                                                                                policy is bounded-static-elf-entropy-generator-lifecycle-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_OK"
                                                                              randomized-static-process-after-initial-random-delivery is unmapped static image construction deliver entropy initial random (
                                                                                owner is entropy-acquiring-static-process-after-generator-lifecycle,
                                                                                device is initial-virtio-rng-device,
                                                                                domain is initial-dma-domain,
                                                                                queue is initial-virtio-rng-queue-zero,
                                                                                transport is initial-virtio-rng-pci-transport,
                                                                                sample is initial-virtio-rng-sample,
                                                                                source is initial-virtio-rng-source,
                                                                                evidence is initial-q35-virtio-rng-trust-evidence,
                                                                                pool is initial-entropy-pool,
                                                                                conditioner is initial-blake2s-conditioner,
                                                                                contribution is initial-entropy-contribution,
                                                                                generator is initial-entropy-generator,
                                                                                algorithm is chacha20,
                                                                                output is initial-entropy-output,
                                                                                interface is initial-linux-random-interface,
                                                                                delivery is initial-auxv-random-delivery,
                                                                                policy is bounded-static-elf-initial-random-delivery-coexistence
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK"
                                                                              completed-static-process-after-entry is unmapped static image process enter (
                                                                                owner is randomized-static-process-after-initial-random-delivery,
                                                                                abi is linux-x86-64-process,
                                                                                terminal is exit-group-zero,
                                                                                policy is bounded-static-elf-process-entry
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_PROCESS_EXITED_OK"
                                                                              randomized-static-process-after-entry-restoration is unmapped static image process restore entry (
                                                                                owner is completed-static-process-after-entry,
                                                                                policy is bounded-static-elf-process-entry-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_PROCESS_ENTRY_RESTORED_OK"
                                                                              entropy-acquiring-static-process-after-initial-random-revocation is unmapped static image construction revoke entropy initial random (
                                                                                owner is randomized-static-process-after-entry-restoration,
                                                                                delivery is initial-auxv-random-delivery,
                                                                                policy is bounded-static-elf-initial-random-delivery-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_REVOKED_OK"
                                                                              entropy-pending-static-process-after-acquisition-rollback is unmapped static image construction revoke entropy acquisition (
                                                                                owner is entropy-acquiring-static-process-after-initial-random-revocation,
                                                                                policy is bounded-static-elf-entropy-acquisition-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK"
                                                                              retained-static-process-after-entropy-rollback is unmapped static image construction revoke entropy handoff (
                                                                                owner is entropy-pending-static-process-after-acquisition-rollback,
                                                                                policy is bounded-static-elf-entropy-handoff-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK"
                                                                              random-slot-authorized-stack-restored-construction is unmapped static image construction restore prepared process (
                                                                                owner is retained-static-process-after-entropy-rollback,
                                                                                policy is bounded-static-elf-retention-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK"
                                                                              load-metadata-stack-restored-construction is unmapped static image construction revoke stack random slot (
                                                                                owner is random-slot-authorized-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-random-slot-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK"
                                                                              hardware-capabilities-stack-restored-construction is unmapped static image construction remove stack load metadata (
                                                                                owner is load-metadata-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-load-metadata-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK"
                                                                              platform-stack-restored-construction is unmapped static image construction remove stack hardware capabilities (
                                                                                owner is hardware-capabilities-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-hardware-capabilities-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK"
                                                                              clock-tick-rate-stack-restored-construction is unmapped static image construction remove stack platform (
                                                                                owner is platform-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-platform-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK"
                                                                              secure-mode-stack-restored-construction is unmapped static image construction remove stack clock tick rate (
                                                                                owner is clock-tick-rate-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-clock-tick-rate-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK"
                                                                              credential-stack-restored-construction is unmapped static image construction remove stack secure mode (
                                                                                owner is secure-mode-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-secure-mode-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK"
                                                                              executable-name-stack-restored-construction is unmapped static image construction remove stack credentials (
                                                                                owner is credential-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-credentials-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK"
                                                                              program-header-stack-restored-construction is unmapped static image construction remove stack executable name (
                                                                                owner is executable-name-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-executable-name-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK"
                                                                              entry-auxv-stack-restored-construction is unmapped static image construction remove stack program headers (
                                                                                owner is program-header-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-program-headers-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK"
                                                                              pagesz-stack-entry-restored-construction is unmapped static image construction remove stack entry fact (
                                                                                owner is entry-auxv-stack-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-entry-fact-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK"
                                                                              initialized-stack-entry-restored-construction is unmapped static image construction remove stack pagesz (
                                                                                owner is pagesz-stack-entry-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-pagesz-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK"
                                                                              published-stack-leaf-restored-construction is unmapped static image construction clear stack entry (
                                                                                owner is initialized-stack-entry-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-entry-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK"
                                                                              zeroed-stack-backing-restored-construction is unmapped static image construction revoke stack leaf (
                                                                                owner is published-stack-leaf-restored-construction,
                                                                                backing is static-elf-stack-backing,
                                                                                policy is bounded-stack-leaf-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_OK"
                                                                              unmapped frames release static-elf-stack-backing
                                                                              stack-slot-restored-static-elf-construction is unmapped static image construction revoke stack slot (
                                                                                owner is zeroed-stack-backing-restored-construction,
                                                                                policy is bounded-stack-slot-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK"
                                                                              entry-restored-static-elf-construction is unmapped static image construction revoke entry (
                                                                                owner is stack-slot-restored-static-elf-construction,
                                                                                policy is bounded-entry-qualification-rollback
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK"
                                                                              detached-static-elf-construction is unmapped static image construction detach address space (
                                                                                owner is entry-restored-static-elf-construction
                                                                              )
                                                                              unmapped console write "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK"
                                                                              returned-static-elf-hierarchy is unmapped static image construction reclaim (
                                                                                owner is detached-static-elf-construction,
                                                                                resource is hierarchy
                                                                              )
                                                                              unmapped frames release returned-static-elf-hierarchy
                                                                              returned-static-elf-metadata is unmapped static image construction revoke (
                                                                                owner is detached-static-elf-construction,
                                                                                resource is metadata
                                                                              )
                                                                              unmapped frames release returned-static-elf-metadata
                                                                              returned-static-elf-payload is unmapped static image construction reclaim (
                                                                                owner is detached-static-elf-construction,
                                                                                resource is payload
                                                                              )
                                                                              unmapped frames release returned-static-elf-payload
                                                                              unmapped critical enter (
                                                                                domain is local-maskable-interrupts
                                                                              )
                                                                                Ok critical then {
                                                                                  critical console write "TOPAL_KERNEL_INTERRUPTS_MASKED"
                                                                                  restored is critical restore
                                                                  restored console write "TOPAL_KERNEL_MEMORY_DESCRIBED"
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 0,
                                                                    expected-value is 272898804
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 52,
                                                                    expected-value is 152
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 152,
                                                                    expected-value is 99345
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 132,
                                                                    expected-value is 85225481
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 136,
                                                                    expected-value is 0
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 140,
                                                                    expected-value is 0
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 144,
                                                                    expected-value is 0
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 112,
                                                                    expected-value is 34889737
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 116,
                                                                    expected-value is 4
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 120,
                                                                    expected-value is 12288
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 124,
                                                                    expected-value is 4096
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 128,
                                                                    expected-value is 4
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 96,
                                                                    expected-value is 68177929
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 100,
                                                                    expected-value is 4
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 104,
                                                                    expected-value is 8192
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 108,
                                                                    expected-value is 4096
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 80,
                                                                    expected-value is 51396617
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 84,
                                                                    expected-value is 4
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 88,
                                                                    expected-value is 4096
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 92,
                                                                    expected-value is 4096
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 64,
                                                                    expected-value is 17825801
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 68,
                                                                    expected-value is 4
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 72,
                                                                    expected-value is 0
                                                                  )
                                                                  restored pci configuration dword read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 76,
                                                                    expected-value is 4096
                                                                  )
                                                                  restored console write "TOPAL_KERNEL_PCI_VIRTIO_RNG_OK"
                                                                  restored console write "TOPAL_KERNEL_VIRTIO_PCI_CAPABILITIES_OK"
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 20,
                                                                    width-bytes is 1,
                                                                    value is 0
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 20,
                                                                    width-bytes is 1,
                                                                    value is 0
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 20,
                                                                    width-bytes is 1,
                                                                    value is 1
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 20,
                                                                    width-bytes is 1,
                                                                    value is 1
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 20,
                                                                    width-bytes is 1,
                                                                    value is 3
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 20,
                                                                    width-bytes is 1,
                                                                    value is 3
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 0,
                                                                    width-bytes is 4,
                                                                    value is 0
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 4,
                                                                    width-bytes is 4,
                                                                    value is 805306368
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 0,
                                                                    width-bytes is 4,
                                                                    value is 1
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 4,
                                                                    width-bytes is 4,
                                                                    value is 257
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 8,
                                                                    width-bytes is 4,
                                                                    value is 0
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 12,
                                                                    width-bytes is 4,
                                                                    value is 0
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 8,
                                                                    width-bytes is 4,
                                                                    value is 1
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 12,
                                                                    width-bytes is 4,
                                                                    value is 1
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 20,
                                                                    width-bytes is 1,
                                                                    value is 11
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 20,
                                                                    width-bytes is 1,
                                                                    value is 11
                                                                  )
                                                                  restored console write "TOPAL_KERNEL_VIRTIO_PCI_NEGOTIATED"
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 18,
                                                                    width-bytes is 2,
                                                                    value is 1
                                                                  )
                                                                  restored pci region register write (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 22,
                                                                    width-bytes is 2,
                                                                    value is 0
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 24,
                                                                    width-bytes is 2,
                                                                    value is 8
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 28,
                                                                    width-bytes is 2,
                                                                    value is 0
                                                                  )
                                                                  restored pci region register read equals (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 30,
                                                                    width-bytes is 2,
                                                                    value is 0
                                                                  )
                                                                  restored console write "TOPAL_KERNEL_VIRTIO_QUEUE_DISCOVERED"
                                                                  restored pci configuration word update (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    register-offset-bytes is 4,
                                                                    mask is 4,
                                                                    expected-masked-value is 0,
                                                                    desired-masked-value is 4
                                                                  )
                                                                  restored console write "TOPAL_KERNEL_PCI_BUS_MASTER_OK"
                                                                  restored dma buffer create (
                                                                    buffer-identity is 1,
                                                                    device-identity is 1,
                                                                    translation-domain-identity is 1,
                                                                    byte-count is 4096,
                                                                    alignment-bytes is 4096,
                                                                    direction is "bidirectional",
                                                                    visibility is "provider-managed",
                                                                    role is "virtqueue-control"
                                                                  )
                                                                  restored dma buffer create (
                                                                    buffer-identity is 2,
                                                                    device-identity is 1,
                                                                    translation-domain-identity is 1,
                                                                    byte-count is 4096,
                                                                    alignment-bytes is 4096,
                                                                    direction is "device-to-memory",
                                                                    visibility is "provider-managed",
                                                                    role is "entropy-payload"
                                                                  )
                                                                  restored console write "TOPAL_KERNEL_DMA_BUFFERS_OK"
                                                                  restored dma buffer bind reference (
                                                                    destination-buffer-identity is 1,
                                                                    destination-offset-bytes is 0,
                                                                    referenced-buffer-identity is 2,
                                                                    referenced-offset-bytes is 0,
                                                                    width-bytes is 8
                                                                  )
                                                                  restored dma buffer integer store (
                                                                    buffer-identity is 1,
                                                                    offset-bytes is 8,
                                                                    width-bytes is 4,
                                                                    value is 32
                                                                  )
                                                                  restored dma buffer integer store (
                                                                    buffer-identity is 1,
                                                                    offset-bytes is 12,
                                                                    width-bytes is 2,
                                                                    value is 2
                                                                  )
                                                                  restored dma buffer integer store (
                                                                    buffer-identity is 1,
                                                                    offset-bytes is 132,
                                                                    width-bytes is 2,
                                                                    value is 0
                                                                  )
                                                                  restored dma buffer integer store (
                                                                    buffer-identity is 1,
                                                                    offset-bytes is 130,
                                                                    width-bytes is 2,
                                                                    value is 1
                                                                  )
                                                                  restored console write "TOPAL_KERNEL_VIRTIO_SPLIT_RING_OK"
                                                                  restored dma buffer prepare device (
                                                                    buffer-identity is 2,
                                                                    device-identity is 1,
                                                                    translation-domain-identity is 1,
                                                                    direction is "device-to-memory",
                                                                    visibility is "provider-managed",
                                                                    role is "entropy-payload"
                                                                  )
                                                                  restored dma buffer prepare device (
                                                                    buffer-identity is 1,
                                                                    device-identity is 1,
                                                                    translation-domain-identity is 1,
                                                                    direction is "bidirectional",
                                                                    visibility is "provider-managed",
                                                                    role is "virtqueue-control"
                                                                  )
                                                                  restored console write "TOPAL_KERNEL_DMA_DEVICE_OWNED"
                                                                  restored pci region write dma reference (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 32,
                                                                    width-bytes is 8,
                                                                    buffer-identity is 1,
                                                                    buffer-offset-bytes is 0,
                                                                    device-identity is 1,
                                                                    translation-domain-identity is 1
                                                                  )
                                                                  restored pci region write dma reference (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 40,
                                                                    width-bytes is 8,
                                                                    buffer-identity is 1,
                                                                    buffer-offset-bytes is 128,
                                                                    device-identity is 1,
                                                                    translation-domain-identity is 1
                                                                  )
                                                                  restored pci region write dma reference (
                                                                    segment is 0,
                                                                    bus is 0,
                                                                    device is 5,
                                                                    function is 0,
                                                                    access-capability-offset-bytes is 132,
                                                                    bar is 4,
                                                                    region-offset-bytes is 0,
                                                                    region-length-bytes is 4096,
                                                                    register-offset-bytes is 48,
                                                                    width-bytes is 8,
                                                                    buffer-identity is 1,
                                                                    buffer-offset-bytes is 148,
                                                                    device-identity is 1,
                                                                    translation-domain-identity is 1
                                                                  )
                                                                  restored console write "TOPAL_KERNEL_VIRTIO_QUEUE_ADDRESSES_OK"
                                                                  restored console write "TOPAL_KERNEL_BOOT"
                                                                  restored debug break
                                                                  restored console write "TOPAL_KERNEL_FAULT_RESUMED"
                                                                  restored bootstrap allocate (
                                                                    byte-count is 64,
                                                                    alignment-bytes is 8,
                                                                    placement is bootstrap-reclaimable
                                                                  )
                                                                    Ok region then {
                                                                      region byte store (offset-bytes is 0, value is 90)
                                                                      observed : Nat is region byte load (offset-bytes is 0)
                                                                      observed = 90
                                                                        true then {
                                                                          atomic is region atomic word create (
                                                                            offset-bytes is 8,
                                                                            initial-value is 41,
                                                                            domain is cpu-shared
                                                                          )
                                                                          atomic compare exchange (
                                                                            expected is 41,
                                                                            desired is 42,
                                                                            success-order is acquire-release,
                                                                            failure-order is acquire
                                                                          )
                                                                            Exchanged previous then {
                                                                              atomic-observed : Nat is atomic load (order is acquire)
                                                                              atomic-observed = 42
                                                                                true then {
                                                                                  region is atomic end
                                                                                  restored console write "TOPAL_KERNEL_ATOMIC_OK"
                                                                                  restored console write "TOPAL_KERNEL_MEMORY_OK"
                                                                                  restored bootstrap release region
                                                                                  pending is restored local notification send
                                                                                  resumed is pending local notification wait
                                                                                  resumed console write "TOPAL_KERNEL_INTERRUPT_OK"
                                                                                  first : Instant InitialMonotonicClock is resumed monotonic clock now
                                                                                  second : Instant InitialMonotonicClock is resumed monotonic clock now
                                                                                  resumed console write "TOPAL_KERNEL_TIME_OK"
                                                                                  deadline : Deadline InitialMonotonicClock is second deadline after 1[ms]
                                                                                  resumed bootstrap allocate (
                                                                                    byte-count is 16384,
                                                                                    alignment-bytes is 16,
                                                                                    placement is bootstrap-reclaimable
                                                                                  )
                                                                                    Ok cooperative-stack then {
                                                                                      one-shot : KernelCompletion InitialProcessor is resumed kernel completion create
                                                                                      cooperative-worker : SuspendedKernelContext InitialProcessor is resumed kernel context create (
                                                                                        stack is cooperative-stack,
                                                                                        entry is kernel-thread
                                                                                      )
                                                                                      resumed bootstrap allocate (
                                                                                        byte-count is 16384,
                                                                                        alignment-bytes is 16,
                                                                                        placement is bootstrap-reclaimable
                                                                                      )
                                                                                        Ok completion-stack then {
                                                                                          completion-worker : SuspendedKernelContext InitialProcessor is resumed kernel context create (
                                                                                            stack is completion-stack,
                                                                                            entry is completion-thread
                                                                                          )
                                                                                          resumed bootstrap allocate (
                                                                                            byte-count is 16384,
                                                                                            alignment-bytes is 16,
                                                                                            placement is bootstrap-reclaimable
                                                                                          )
                                                                                            Ok terminal-stack then {
                                                                                              terminal-worker : SuspendedKernelContext InitialProcessor is resumed kernel context create (
                                                                                                stack is terminal-stack,
                                                                                                entry is terminal-thread
                                                                                              )
                                                                                              runnable : BoundedKernelRunnableQueue InitialProcessor is resumed kernel runnable queue create (capacity is 3)
                                                                                              runnable kernel runnable enqueue cooperative-worker
                                                                                              runnable kernel runnable enqueue completion-worker
                                                                                              runnable kernel runnable enqueue terminal-worker
                                                                                              selected-cooperative : SuspendedKernelContext InitialProcessor is runnable kernel runnable dequeue
                                                                                              preempted : PreemptedKernelContextTransfer InitialProcessor InitialMonotonicClock is resumed kernel context transfer selected-cooperative until deadline
                                                                                              preempted-worker : SuspendedKernelContext InitialProcessor is preempted kernel context take preempted
                                                                                              resumed console write "TOPAL_KERNEL_DEADLINE_PREEMPTED"
                                                                                              runnable kernel runnable enqueue preempted-worker
                                                                                              selected-completion : SuspendedKernelContext InitialProcessor is runnable kernel runnable dequeue
                                                                                              blocked-completion : BlockedKernelContextTransfer InitialProcessor is resumed kernel context transfer selected-completion until one-shot
                                                                                              selected-terminal : SuspendedKernelContext InitialProcessor is runnable kernel runnable dequeue
                                                                                              terminal-completed : CompletedKernelContextTransfer InitialProcessor is resumed kernel context transfer selected-terminal
                                                                                              resumed console write "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED"
                                                                                              after-terminal is terminal-completed kernel context reclaim
                                                                                                awakened-completion : SuspendedKernelContext InitialProcessor is blocked-completion kernel completion signal one-shot
                                                                                                runnable kernel runnable enqueue awakened-completion
                                                                                                selected-cooperative-final : SuspendedKernelContext InitialProcessor is runnable kernel runnable dequeue
                                                                                                cooperative-completed : CompletedKernelContextTransfer InitialProcessor is after-terminal kernel context transfer selected-cooperative-final
                                                                                                after-terminal console write "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED"
                                                                                                context-resumed is cooperative-completed kernel context reclaim
                                                                                                  selected-completion-final : SuspendedKernelContext InitialProcessor is runnable kernel runnable dequeue
                                                                                                  completion-completed : CompletedKernelContextTransfer InitialProcessor is context-resumed kernel context transfer selected-completion-final
                                                                                                  context-resumed console write "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED"
                                                                                                  completion-resumed is completion-completed kernel context reclaim
                                                                                                    completion-resumed kernel completion consume one-shot
                                                                                                    completion-resumed kernel runnable queue consume empty runnable
                                                                                                    completion-resumed frames allocate (
                                                                                                      frame-count is 3,
                                                                                                      alignment-frames is 1
                                                                                                    )
                                                                                                      Ok user-frames then {
                                                                                                        user-space is completion-resumed user address space prepare (
                                                                                                          backing is user-frames,
                                                                                                          layout is initial-probe
                                                                                                        )
                                                                                                          completion-resumed frames allocate (
                                                                                                            frame-count is 2,
                                                                                                            alignment-frames is 1
                                                                                                          )
                                                                                                            Ok scratch-frame then {
                                                                                                              mapped-scratch is completion-resumed user address space map (
                                                                                                                address-space is user-space,
                                                                                                                backing is scratch-frame,
                                                                                                                layout is probe-code
                                                                                                              )
                                                                                                              stored-scratch is completion-resumed user mapping install entry trampoline mapped-scratch
                                                                                                              protected-scratch is completion-resumed user mapping protect read only stored-scratch
                                                                                                              verified-scratch is completion-resumed user mapping verify entry trampoline protected-scratch
                                                                                                              executable-scratch is completion-resumed user mapping permit execute verified-scratch
                                                                                                              completion-resumed frames allocate (
                                                                                                                frame-count is 1,
                                                                                                                alignment-frames is 1
                                                                                                              )
                                                                                                                Ok data-frame then {
                                                                                                                  mapped-data is completion-resumed user address space map (
                                                                                                                    address-space is user-space,
                                                                                                                    backing is data-frame,
                                                                                                                    layout is probe-data
                                                                                                                  )
                                                                                                                  stored-data is completion-resumed user mapping store byte mapped-data
                                                                                                                  protected-data is completion-resumed user mapping protect read only stored-data
                                                                                                                  verified-data is completion-resumed user mapping load byte protected-data
                                                                                                                  writable-data is completion-resumed user mapping permit writable data verified-data
                                                                                                                  completion-resumed frames allocate (
                                                                                                                    frame-count is 1,
                                                                                                                    alignment-frames is 1
                                                                                                                  )
                                                                                                                    Ok anonymous-frame then {
                                                                                                                      anonymous-page is completion-resumed user address space reserve anonymous (
                                                                                                                        address-space is user-space,
                                                                                                                        backing is anonymous-frame,
                                                                                                                        layout is initial-anonymous-page
                                                                                                                      )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_READY"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_BOOT_RESOURCE_OK"
                                                                                                                  user-probe is completion-resumed user probe prepare (
                                                                                                                    process-identity is 1,
                                                                                                                    address-space is user-space
                                                                                                                  )
                                                                                                                  completion-resumed pci region register write (
                                                                                                                    segment is 0,
                                                                                                                    bus is 0,
                                                                                                                    device is 5,
                                                                                                                    function is 0,
                                                                                                                    access-capability-offset-bytes is 132,
                                                                                                                    bar is 4,
                                                                                                                    region-offset-bytes is 0,
                                                                                                                    region-length-bytes is 4096,
                                                                                                                    register-offset-bytes is 28,
                                                                                                                    width-bytes is 2,
                                                                                                                    value is 1
                                                                                                                  )
                                                                                                                  completion-resumed pci region register read equals (
                                                                                                                    segment is 0,
                                                                                                                    bus is 0,
                                                                                                                    device is 5,
                                                                                                                    function is 0,
                                                                                                                    access-capability-offset-bytes is 132,
                                                                                                                    bar is 4,
                                                                                                                    region-offset-bytes is 0,
                                                                                                                    region-length-bytes is 4096,
                                                                                                                    register-offset-bytes is 28,
                                                                                                                    width-bytes is 2,
                                                                                                                    value is 1
                                                                                                                  )
                                                                                                                  completion-resumed pci region register write (
                                                                                                                    segment is 0,
                                                                                                                    bus is 0,
                                                                                                                    device is 5,
                                                                                                                    function is 0,
                                                                                                                    access-capability-offset-bytes is 132,
                                                                                                                    bar is 4,
                                                                                                                    region-offset-bytes is 0,
                                                                                                                    region-length-bytes is 4096,
                                                                                                                    register-offset-bytes is 20,
                                                                                                                    width-bytes is 1,
                                                                                                                    value is 15
                                                                                                                  )
                                                                                                                  completion-resumed pci region register read equals (
                                                                                                                    segment is 0,
                                                                                                                    bus is 0,
                                                                                                                    device is 5,
                                                                                                                    function is 0,
                                                                                                                    access-capability-offset-bytes is 132,
                                                                                                                    bar is 4,
                                                                                                                    region-offset-bytes is 0,
                                                                                                                    region-length-bytes is 4096,
                                                                                                                    register-offset-bytes is 20,
                                                                                                                    width-bytes is 1,
                                                                                                                    value is 15
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE"
                                                                                                                  completion-resumed pci region register write (
                                                                                                                    segment is 0,
                                                                                                                    bus is 0,
                                                                                                                    device is 5,
                                                                                                                    function is 0,
                                                                                                                    access-capability-offset-bytes is 132,
                                                                                                                    bar is 4,
                                                                                                                    region-offset-bytes is 12288,
                                                                                                                    region-length-bytes is 4096,
                                                                                                                    register-offset-bytes is 0,
                                                                                                                    width-bytes is 2,
                                                                                                                    value is 0
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED"
                                                                                                                  completion-resumed pci region register read eventually equals (
                                                                                                                    segment is 0,
                                                                                                                    bus is 0,
                                                                                                                    device is 5,
                                                                                                                    function is 0,
                                                                                                                    access-capability-offset-bytes is 132,
                                                                                                                    bar is 4,
                                                                                                                    region-offset-bytes is 4096,
                                                                                                                    region-length-bytes is 4096,
                                                                                                                    register-offset-bytes is 0,
                                                                                                                    width-bytes is 1,
                                                                                                                    value is 1
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL"
                                                                                                                  completion-resumed dma buffer complete cpu (
                                                                                                                    buffer-identity is 1,
                                                                                                                    device-identity is 1,
                                                                                                                    translation-domain-identity is 1,
                                                                                                                    direction is "bidirectional",
                                                                                                                    visibility is "provider-managed",
                                                                                                                    role is "virtqueue-control"
                                                                                                                  )
                                                                                                                  completion-resumed dma buffer integer read equals (
                                                                                                                    buffer-identity is 1,
                                                                                                                    offset-bytes is 148,
                                                                                                                    width-bytes is 2,
                                                                                                                    expected-value is 0
                                                                                                                  )
                                                                                                                  completion-resumed dma buffer integer read equals (
                                                                                                                    buffer-identity is 1,
                                                                                                                    offset-bytes is 150,
                                                                                                                    width-bytes is 2,
                                                                                                                    expected-value is 1
                                                                                                                  )
                                                                                                                  completion-resumed dma buffer integer read equals (
                                                                                                                    buffer-identity is 1,
                                                                                                                    offset-bytes is 152,
                                                                                                                    width-bytes is 4,
                                                                                                                    expected-value is 0
                                                                                                                  )
                                                                                                                  completion-resumed dma buffer integer read equals (
                                                                                                                    buffer-identity is 1,
                                                                                                                    offset-bytes is 156,
                                                                                                                    width-bytes is 4,
                                                                                                                    expected-value is 32
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_VIRTIO_USED_RING_OK"
                                                                                                                  completion-resumed dma buffer complete cpu (
                                                                                                                    buffer-identity is 2,
                                                                                                                    device-identity is 1,
                                                                                                                    translation-domain-identity is 1,
                                                                                                                    direction is "device-to-memory",
                                                                                                                    visibility is "provider-managed",
                                                                                                                    role is "entropy-payload"
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED"
                                                                                                                  completion-resumed entropy sample capture dma (
                                                                                                                    sample-identity is 1,
                                                                                                                    buffer-identity is 2,
                                                                                                                    offset-bytes is 0,
                                                                                                                    byte-count is 32,
                                                                                                                    device-identity is 1,
                                                                                                                    translation-domain-identity is 1,
                                                                                                                    direction is "device-to-memory",
                                                                                                                    visibility is "provider-managed",
                                                                                                                    role is "entropy-payload",
                                                                                                                    credited-entropy-bits is 0
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED"
                                                                                                                  completion-resumed entropy sample assess quality (
                                                                                                                    sample-identity is 1,
                                                                                                                    source-identity is 1,
                                                                                                                    trust-evidence-identity is 1,
                                                                                                                    byte-count is 32,
                                                                                                                    quality-parts-per-thousand-twenty-four is 1024,
                                                                                                                    credited-entropy-bits is 256
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED"
                                                                                                                  completion-resumed entropy pool create (
                                                                                                                    pool-identity is 1,
                                                                                                                    credit-capacity-bits is 256,
                                                                                                                    initial-credited-entropy-bits is 0
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_ENTROPY_POOL_CREATED"
                                                                                                                  completion-resumed entropy pool mix sample (
                                                                                                                    pool-identity is 1,
                                                                                                                    sample-identity is 1,
                                                                                                                    conditioner-identity is 1,
                                                                                                                    contribution-identity is 1,
                                                                                                                    byte-count is 32,
                                                                                                                    contribution-entropy-bits is 256
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_ENTROPY_POOL_MIXED"
                                                                                                                  completion-resumed entropy pool credit contribution (
                                                                                                                    pool-identity is 1,
                                                                                                                    contribution-identity is 1,
                                                                                                                    conditioner-origin-identity is 1,
                                                                                                                    prior-credited-entropy-bits is 0,
                                                                                                                    contribution-entropy-bits is 256,
                                                                                                                    credit-capacity-bits is 256,
                                                                                                                    resulting-credited-entropy-bits is 256
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_ENTROPY_POOL_CREDITED"
                                                                                                                  completion-resumed entropy generator initialize (
                                                                                                                    pool-identity is 1,
                                                                                                                    generator-identity is 1,
                                                                                                                    algorithm-identity is 1,
                                                                                                                    generation is 1,
                                                                                                                    key-byte-count is 32,
                                                                                                                    required-entropy-bits is 256,
                                                                                                                    observed-pool-entropy-bits is 256
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_ENTROPY_GENERATOR_READY"
                                                                                                                  completion-resumed entropy generator extract (
                                                                                                                    generator-identity is 1,
                                                                                                                    algorithm-identity is 1,
                                                                                                                    prior-generation is 1,
                                                                                                                    resulting-generation is 2,
                                                                                                                    output-identity is 1,
                                                                                                                    block-byte-count is 64,
                                                                                                                    output-offset-bytes is 32,
                                                                                                                    output-byte-count is 32,
                                                                                                                    counter is 0,
                                                                                                                    nonce-identity is 0
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_ENTROPY_OUTPUT_READY"
                                                                                                                  completion-resumed entropy output deliver initial auxv random (
                                                                                                                    output-identity is 1,
                                                                                                                    source-byte-count is 32,
                                                                                                                    selected-offset-bytes is 0,
                                                                                                                    delivered-byte-count is 16,
                                                                                                                    discarded-offset-bytes is 16,
                                                                                                                    discarded-byte-count is 16,
                                                                                                                    address-space-identity is 1,
                                                                                                                    interface-identity is 1,
                                                                                                                    auxiliary-tag is 25,
                                                                                                                    single-use is 1
                                                                                                                  )
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_AUXV_RANDOM_OK"
                                                                                                                  completion-resumed entropy generator extract (
                                                                                                                    generator-identity is 1,
                                                                                                                    algorithm-identity is 1,
                                                                                                                    prior-generation is 2,
                                                                                                                    resulting-generation is 3,
                                                                                                                    output-identity is 2,
                                                                                                                    block-byte-count is 64,
                                                                                                                    output-offset-bytes is 32,
                                                                                                                    output-byte-count is 32,
                                                                                                                    counter is 0,
                                                                                                                    nonce-identity is 0
                                                                                                                  )
                                                                                                                  completion-resumed entropy output reserve initial user random (
                                                                                                                    output-identity is 2,
                                                                                                                    source-byte-count is 32,
                                                                                                                    selected-offset-bytes is 0,
                                                                                                                    maximum-delivery-byte-count is 32,
                                                                                                                    discarded-offset-bytes is 32,
                                                                                                                    discarded-byte-count is 0,
                                                                                                                    process-identity is 1,
                                                                                                                    interface-identity is 1,
                                                                                                                    service-identity is 1,
                                                                                                                    supported-flag-mask is 7,
                                                                                                                    incompatible-flag-mask is 6,
                                                                                                                    single-use is 1
                                                                                                                  )
                                                                                                                  completion-resumed entropy user random bind reusable (
                                                                                                                    generator-identity is 1,
                                                                                                                    generation is 3,
                                                                                                                    initial-output-identity is 2,
                                                                                                                    block-byte-count is 64,
                                                                                                                    output-byte-count is 32,
                                                                                                                    maximum-delivery-byte-count is 32,
                                                                                                                    process-identity is 1,
                                                                                                                    interface-identity is 1,
                                                                                                                    service-identity is 1,
                                                                                                                    supported-flag-mask is 7,
                                                                                                                    incompatible-flag-mask is 6,
                                                                                                                    reserved-first-output is 1,
                                                                                                                    reusable is 1
                                                                                                                  )
                                                                                                                  completion-resumed entropy user random expand small requests (
                                                                                                                    process-identity is 1,
                                                                                                                    interface-identity is 1,
                                                                                                                    service-identity is 1,
                                                                                                                    algorithm-identity is 1,
                                                                                                                    output-byte-count is 32,
                                                                                                                    maximum-output-count is 8,
                                                                                                                    maximum-delivery-byte-count is 256,
                                                                                                                    complete-request is 1
                                                                                                                  )
                                                                                                                  completion-resumed entropy user random expand large requests (
                                                                                                                    process-identity is 1,
                                                                                                                    interface-identity is 1,
                                                                                                                    service-identity is 1,
                                                                                                                    algorithm-identity is 1,
                                                                                                                    output-byte-count is 32,
                                                                                                                    maximum-output-count is 16,
                                                                                                                    maximum-delivery-byte-count is 512,
                                                                                                                    complete-request is 1
                                                                                                                  )
                                                                                                                  user-completed is completion-resumed user context enter user-probe
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_GETRANDOM_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_READ_OK"
completion-resumed console write "TOPAL_KERNEL_USER_CLOSE_OK"
completion-resumed console write "TOPAL_KERNEL_USER_DUP_OK"
completion-resumed console write "TOPAL_KERNEL_USER_FCNTL_OK"
completion-resumed console write "TOPAL_KERNEL_USER_FSTAT_OK"
completion-resumed console write "TOPAL_KERNEL_USER_TCGETS_OK"
completion-resumed console write "TOPAL_KERNEL_USER_WINSIZE_OK"
completion-resumed console write "TOPAL_KERNEL_USER_LSEEK_OK"
completion-resumed console write "TOPAL_KERNEL_USER_COPY_TO_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_COPY_FROM_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_SYSCALL_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_STACK_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_AUXV_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_ELF_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_BSS_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_DATA_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_IMAGE_RETAINED"
                                                                                                                  returned-anonymous is completion-resumed user address space release anonymous (
                                                                                                                    address-space is user-space,
                                                                                                                    reservation is anonymous-page
                                                                                                                  )
                                                                                                                  completion-resumed frames release returned-anonymous
                                                                                                                  returned-data is completion-resumed user address space unmap (
                                                                                                                    address-space is user-space,
                                                                                                                    mapping is writable-data
                                                                                                                  )
                                                                                                                  completion-resumed frames release returned-data
                                                                                                                  returned-code is completion-resumed user address space unmap (
                                                                                                                    address-space is user-space,
                                                                                                                    mapping is executable-scratch
                                                                                                                  )
                                                                                                                  completion-resumed frames release returned-code
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_MAPPING_OK"
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_FRAME_DISJOINT_OK"
                                                                                                                  revoked-space is completion-resumed user execution complete user-completed
                                                                                                                  returned-frames is completion-resumed user address space reclaim revoked-space
                                                                                                                  completion-resumed frames release returned-frames
                                                                                                                  completion-resumed console write "TOPAL_KERNEL_USER_BACKING_OK"
                                                                                                                  completion-resumed fatal "toolchain gate complete"
                                                                                                                    }
                                                                                                                    Error problem then {
                                                                                                                      completion-resumed fatal "anonymous frame allocation failed"
                                                                                                                    }
                                                                                                                }
                                                                                                                Error problem then {
                                                                                                                  completion-resumed fatal "data frame allocation failed"
                                                                                                                }
                                                                                                            }
                                                                                                            Error problem then {
                                                                                                              completion-resumed fatal "disjoint frame allocation failed"
                                                                                                            }
                                                                                                      }
                                                                                                      Error problem then {
                                                                                                        completion-resumed fatal "user backing allocation failed"
                                                                                                      }
                                                                                            }
                                                                                            Error problem then {
                                                                                              resumed fatal "terminal context stack allocation failed"
                                                                                            }
                                                                                        }
                                                                                        Error problem then {
                                                                                          resumed fatal "completion context stack allocation failed"
                                                                                        }
                                                                                    }
                                                                                    Error problem then {
                                                                                      resumed fatal "cooperative context stack allocation failed"
                                                                                    }
                                                                                }
                                                                                false then {
                                                                                  region is atomic end
                                                                                  restored bootstrap release region
                                                                                  restored fatal "atomic load mismatch"
                                                                                }
                                                                            }
                                                                            Observed actual then {
                                                                              region is atomic end
                                                                              restored bootstrap release region
                                                                              restored fatal "atomic compare exchange lost"
                                                                            }
                                                                        }
                                                                        false then {
                                                                          restored bootstrap release region
                                                                          restored fatal "toolchain gate memory mismatch"
                                                                        }
                                                                    }
                                                                    Error problem then {
                                                                      restored fatal "toolchain gate allocation failed"
                                                                    }
                                                                }
                                                                                Error failure then {
                                                                                  failure fatal "critical entry failed"
                                                                                }
                                                                            }
                                                                            Error problem then {
                                                                              unmapped fatal "static ELF initial leaf construction failed"
                                                                            }
                                                                        }
                                                                        Error problem then {
                                                                          unmapped fatal "static ELF metadata frame allocation failed"
                                                                        }
                                                                    }
                                                                    Error problem then {
                                                                      unmapped fatal "static ELF frame allocation failed"
                                                                    }
                                                                }
                                                                Error problem then {
                                                                  unmapped fatal "multi-frame allocation failed"
                                                                }
                                                            }
                                                            Error failure then {
                                                              failure fatal "translation unmap edit commit failed"
                                                            }
                                                        }
                                                        Error failure then {
                                                          failure fatal "translation unmap edit construction failed"
                                                        }
                                                    }
                                                    false then {
                                                      edited translation edit begin
                                                        Ok unmap-edit then {
                                                          removed-frames is unmap-edit kernel unmap dynamic-mapping
                                                          unmap-edit translation commit
                                                            Ok unmapped then {
                                                              unmapped frames release removed-frames
                                                              unmapped fatal "translation edit mapping mismatch"
                                                            }
                                                            Error failure then {
                                                              failure fatal "translation unmap edit commit failed"
                                                            }
                                                        }
                                                        Error failure then {
                                                          failure fatal "translation unmap edit construction failed"
                                                        }
                                                    }
                                                }
                                                Error failure then {
                                                  failure fatal "translation map edit commit failed"
                                                }
                                            }
                                            Error failure then {
                                              failure fatal "translation edit mapping failed"
                                            }
                                        }
                                        Error failure then {
                                          failure fatal "translation map edit construction failed"
                                        }
                                    }
                                    Error problem then {
                                      translated fatal "translation edit frame allocation failed"
                                    }
                                }
                                Error failure then {
                                  failure fatal "translation activation failed"
                                }
                            }
                            Error failure then {
                              failure fatal "translation commit failed"
                            }
                        }
                        Error failure then {
                          failure fatal "translation construction failed"
                        }
                    }
                    false then {
                      frames is memory kernel unmap mapping
                      memory frames release frames
                      memory fatal "kernel mapping mismatch"
                    }
                }
                Error problem then {
                  memory fatal "kernel mapping failed"
                }
            }
            Error problem then {
              memory fatal "physical frame allocation failed"
            }
        }
        Error failure then {
          failure fatal "frame allocator creation failed"
        }
    }
    Error failure then failure fatal "boot memory description failed"

debug-break-handler is fn (context : DebugBreakContext) -> DebugBreakDisposition
  context resume

local-notification-handler is fn (
  context : LocalNotificationInterruptContext
) -> LocalNotificationInterruptDisposition
  completed is context local notification complete
  completed resume

deadline-notification-handler is fn (
  context : DeadlineInterruptContext InitialMonotonicClock
) -> DeadlineInterruptDisposition InitialMonotonicClock
  completed is context deadline notification complete
  completed preempt current kernel context

kernel-thread-handler is fn (
  context : KernelThreadContext InitialProcessor,
  caller : SuspendedKernelContext InitialProcessor
) -> KernelThreadDisposition InitialProcessor
  context console write "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED"
  context kernel context await deadline preemption
  context console write "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED"
  context kernel context retire to caller

completion-thread-handler is fn (
  context : KernelThreadContext InitialProcessor,
  caller : SuspendedKernelContext InitialProcessor
) -> KernelThreadDisposition InitialProcessor
  context console write "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED"
  context kernel completion await
  context console write "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED"
  context kernel context retire to caller

terminal-thread-handler is fn (
  context : KernelThreadContext InitialProcessor,
  caller : SuspendedKernelContext InitialProcessor
) -> KernelThreadDisposition InitialProcessor
  context console write "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED"
  context kernel context retire to caller

user-syscall-handler is fn (
  context : InitialUserSyscallContext
) -> InitialUserSyscallDisposition
  context linux syscall dispatch (
    getpid-result is 1,
    uname-sysname is "Linux",
    uname-nodename is "topal",
    uname-release is "7.2.9-topal",
    uname-version is "#1 Topal",
    uname-machine is "x86_64",
    uname-domainname is "(none)",
    nanosleep-seconds is 0,
    nanosleep-nanoseconds is 0,
    nanosleep-remainder-candidate is 0,
    clock-gettime-result is 0,
    clock-getres-result is 0,
    write-descriptor is 1,
    write-maximum-bytes is 256,
    read-descriptor is 0,
    read-maximum-bytes is 256,
    brk-data-page-bytes is 4096,
    getppid-result is 0,
    gettid-result is 1,
    getuid-result is 0,
    geteuid-result is 0,
    getgid-result is 0,
    getegid-result is 0,
    arch-set-fs-result is 0,
    arch-get-fs-result is 0,
    set-tid-address-result is 1,
    set-robust-list-result is 0,
    getrlimit-result is 0,
    prlimit64-result is 0,
    rt-sigprocmask-result is 0,
    anonymous-page-bytes is 4096,
    madvise-result is 0,
    mprotect-result is 0,
    munmap-result is 0,
    exit-group-status is 0,
    unsupported-syscall-error is 38,
    sched-yield-result is 0,
initial-descriptor-count is 3,
bad-descriptor-error is 9,
too-many-open-files-error is 24,
console-input-description is 1,
console-output-description is 2,
console-error-description is 3,
invalid-argument-error is 22,
fcntl-getfd-command is 1,
fcntl-setfd-command is 2,
fcntl-getfl-command is 3,
close-on-exec-flag is 1,
read-only-status is 0,
write-only-status is 1,
stat-layout-bytes is 144,
console-device is 0,
console-inode is 1,
console-mode is 8576,
console-special-device is 17408,
console-block-size is 4096,
terminal-layout-bytes is 36,
terminal-get-attributes-request is 21505,
terminal-control-flags is 2239,
terminal-minimum-input is 1,
not-a-terminal-error is 25,
window-size-layout-bytes is 8,
window-size-query-request is 21523,
seek-maximum-whence is 4,
illegal-seek-error is 29
)

lang systems artifact (
  bootstrap-storage is lang systems bounded-bootstrap-storage (
    capacity-bytes is 65536,
    alignment-bytes is 4096
  ),
  bootstrap is lang systems bootstrap-entry boot,
  debug-break is lang systems synchronous-exception-entry debug-break-handler,
  local-notification is lang systems external-interrupt-entry local-notification-handler,
  deadline-notification is lang systems external-interrupt-entry deadline-notification-handler,
  kernel-thread is lang systems resumed-thread-entry kernel-thread-handler,
  completion-thread is lang systems resumed-thread-entry completion-thread-handler,
  terminal-thread is lang systems resumed-thread-entry terminal-thread-handler,
  user-syscall is lang systems user-syscall-entry user-syscall-handler
)
