use language (
  version is v0.1,
  features is ( systems )
)

boot is fn (context : BootstrapContext) -> BootstrapDisposition
  context console write "TOPAL_KERNEL_BOOT"
  context debug break
  context console write "TOPAL_KERNEL_FAULT_RESUMED"
  context bootstrap allocate (
    byte-count is 64,
    alignment-bytes is 8,
    placement is bootstrap-reclaimable
  )
    Ok region then {
      region byte store (offset-bytes is 0, value is 90)
      observed : Nat is region byte load (offset-bytes is 0)
      observed = 90
        true then {
          context console write "TOPAL_KERNEL_MEMORY_OK"
          context bootstrap release region
          context fatal "toolchain gate complete"
        }
        false then {
          context bootstrap release region
          context fatal "toolchain gate memory mismatch"
        }
    }
    Error problem then context fatal "toolchain gate allocation failed"

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
