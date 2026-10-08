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
              memory frames release frames
              memory console write "TOPAL_KERNEL_MEMORY_DESCRIBED"
              memory console write "TOPAL_KERNEL_BOOT"
              memory debug break
              memory console write "TOPAL_KERNEL_FAULT_RESUMED"
              memory bootstrap allocate (
                byte-count is 64,
                alignment-bytes is 8,
                placement is bootstrap-reclaimable
              )
                Ok region then {
                  region byte store (offset-bytes is 0, value is 90)
                  observed : Nat is region byte load (offset-bytes is 0)
                  observed = 90
                    true then {
                      memory console write "TOPAL_KERNEL_MEMORY_OK"
                      memory bootstrap release region
                      memory fatal "toolchain gate complete"
                    }
                    false then {
                      memory bootstrap release region
                      memory fatal "toolchain gate memory mismatch"
                    }
                }
                Error problem then {
                  memory fatal "toolchain gate allocation failed"
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
