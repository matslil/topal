use language (version is v0.1)
# TK-DEC-252; TOPAL-SYSTEMS-KERNEL-POLICY-BINDING-001.
# Semantic facts and choices; native flags and errors stay at the ABI boundary.
pub BootOpenDecision is Enum (ReadDescription, NotDirectory, DirectoryWrite, ReadOnlyView)

deny-write is fn (directory : Boolean) -> BootOpenDecision
  directory
    true then DirectoryWrite
    false then ReadOnlyView

after-kind is fn (write-intent : Boolean, directory : Boolean) -> BootOpenDecision
  write-intent
    false then ReadDescription
    true then deny-write directory

require-directory is fn (write-intent : Boolean, directory : Boolean) -> BootOpenDecision
  directory
    false then NotDirectory
    true then after-kind (write-intent, directory)

pub boot-open-decision is fn ((write-intent : Boolean, directory : Boolean, directory-only : Boolean)) -> BootOpenDecision
  directory-only
    true then require-directory (write-intent, directory)
    false then after-kind (write-intent, directory)
