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
                                                              unmapped critical enter (
                                                                domain is local-maskable-interrupts
                                                              )
                                                                Ok critical then {
                                                                  critical console write "TOPAL_KERNEL_INTERRUPTS_MASKED"
                                                                  restored is critical restore
                                                                  restored console write "TOPAL_KERNEL_MEMORY_DESCRIBED"
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
                                                                                  restored fatal "toolchain gate complete"
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

lang systems artifact (
  bootstrap-storage is lang systems bounded-bootstrap-storage (
    capacity-bytes is 65536,
    alignment-bytes is 4096
  ),
  bootstrap is lang systems bootstrap-entry boot,
  debug-break is lang systems synchronous-exception-entry debug-break-handler
)
