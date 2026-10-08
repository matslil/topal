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
                                  translated console write "TOPAL_KERNEL_MEMORY_DESCRIBED"
                                  translated console write "TOPAL_KERNEL_BOOT"
                                  translated debug break
                                  translated console write "TOPAL_KERNEL_FAULT_RESUMED"
                                  translated bootstrap allocate (
                                    byte-count is 64,
                                    alignment-bytes is 8,
                                    placement is bootstrap-reclaimable
                                  )
                                    Ok region then {
                                      region byte store (offset-bytes is 0, value is 90)
                                      observed : Nat is region byte load (offset-bytes is 0)
                                      observed = 90
                                        true then {
                                          translated console write "TOPAL_KERNEL_MEMORY_OK"
                                          translated bootstrap release region
                                          translated fatal "toolchain gate complete"
                                        }
                                        false then {
                                          translated bootstrap release region
                                          translated fatal "toolchain gate memory mismatch"
                                        }
                                    }
                                    Error problem then {
                                      translated fatal "toolchain gate allocation failed"
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

lang systems artifact (
  bootstrap-storage is lang systems bounded-bootstrap-storage (
    capacity-bytes is 65536,
    alignment-bytes is 4096
  ),
  bootstrap is lang systems bootstrap-entry boot,
  debug-break is lang systems synchronous-exception-entry debug-break-handler
)
