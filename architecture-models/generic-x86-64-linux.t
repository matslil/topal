use language (
  version is v0.1
)

# Declarative input for the architecture provider. It is not an application
# value and does not detect features of the machine evaluating this file.
component is fn (value : Record (identity : String, kind : String, definition : String, count : Nat)) -> Record (identity : String, kind : String, definition : String, count : Nat)
  value
connection is fn (value : Record (identity : String, source : String, destination : String, kind : String)) -> Record (identity : String, source : String, destination : String, kind : String)
  value
source is fn (value : Record (identity : String, kind : String, revision : String, digest : String)) -> Record (identity : String, kind : String, revision : String, digest : String)
  value

features : List String is Entry ("x87", Entry ("sse2", Empty))
components : List (Record (identity : String, kind : String, definition : String, count : Nat)) is Entry (
  component (identity is "cpu", kind is "compute", definition is "generic-x86-64-core", count is 1),
  Entry (
    component (identity is "memory", kind is "memory", definition is "generic-host-memory", count is 1),
    Empty
  )
)
connections : List (Record (identity : String, source : String, destination : String, kind : String)) is Entry (
  connection (identity is "cpu-memory", source is "cpu", destination is "memory", kind is "load-store"),
  Empty
)
provenance : List (Record (identity : String, kind : String, revision : String, digest : String)) is Entry (
  source (identity is "llvm22-qualified-target", kind is "toolchain", revision is "LLVM-22", digest is "pending-stage-5-qualification"),
  Empty
)
qualifications : List String is Entry ("schema-example", Empty)

pub architecture-model is (
  schema is "topal.architecture/1",
  language is "v0.1",
  name is "generic-x86-64-linux",
  target is (
    architecture is "x86-64",
    cpu is "x86-64-baseline",
    platform is "linux-gnu",
    abi is "sysv-amd64",
    object-format is "elf64",
    endian is "little",
    pointer-bits is 64
  ),
  features is features,
  components is components,
  connections is connections,
  provenance is provenance,
  qualifications is qualifications
)
