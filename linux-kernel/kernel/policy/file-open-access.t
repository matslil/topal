use language (version is v0.1)
# TK-DEC-266; TOPAL-SYSTEMS-FILE-OPEN-PLAN-001.
# Facts and plans have no native flags, pointers, errno or storage authority.
pub FileOpenDecision is Enum (OpenDescription, NotDirectory, DirectoryWrite, ReadOnlyView, Missing, AlreadyExists, CreateDescription, TruncateDescription)

any? is fn (a : Boolean, b : Boolean) -> Boolean
  a
    true then true
    false then b

truncate-plan is fn (truncate : Boolean) -> FileOpenDecision
  truncate
    false then OpenDescription
    true then TruncateDescription

mutable-plan is fn (mutable : Boolean, truncate : Boolean) -> FileOpenDecision
  mutable
    false then ReadOnlyView
    true then truncate-plan truncate

regular-open is fn ((write : Boolean, mutable : Boolean, truncate : Boolean)) -> FileOpenDecision
  any? (write, truncate)
    false then OpenDescription
    true then mutable-plan (mutable, truncate)

directory-open is fn ((write : Boolean, create : Boolean, truncate : Boolean)) -> FileOpenDecision
  any? (write, any? (create, truncate))
    true then DirectoryWrite
    false then OpenDescription

by-kind is fn ((write : Boolean, directory : Boolean, mutable : Boolean, create : Boolean, truncate : Boolean)) -> FileOpenDecision
  directory
    false then regular-open (write, mutable, truncate)
    true then directory-open (write, create, truncate)

required-kind is fn ((write : Boolean, directory : Boolean, mutable : Boolean, create : Boolean, truncate : Boolean)) -> FileOpenDecision
  directory
    false then NotDirectory
    true then by-kind (write, directory, mutable, create, truncate)

existing-kind is fn ((write : Boolean, directory : Boolean, required : Boolean, mutable : Boolean, create : Boolean, truncate : Boolean)) -> FileOpenDecision
  required
    true then required-kind (write, directory, mutable, create, truncate)
    false then by-kind (write, directory, mutable, create, truncate)

exclusive-open is fn ((write : Boolean, directory : Boolean, required : Boolean, mutable : Boolean, create : Boolean, exclusive : Boolean, truncate : Boolean)) -> FileOpenDecision
  exclusive
    true then AlreadyExists
    false then existing-kind (write, directory, required, mutable, create, truncate)

existing-open is fn ((write : Boolean, directory : Boolean, required : Boolean, mutable : Boolean, create : Boolean, exclusive : Boolean, truncate : Boolean)) -> FileOpenDecision
  create
    true then exclusive-open (write, directory, required, mutable, create, exclusive, truncate)
    false then existing-kind (write, directory, required, mutable, create, truncate)

missing-kind is fn (required : Boolean) -> FileOpenDecision
  required
    true then NotDirectory
    false then CreateDescription

missing-view is fn (mutable : Boolean, required : Boolean) -> FileOpenDecision
  mutable
    false then ReadOnlyView
    true then missing-kind required

missing-open is fn ((mutable : Boolean, required : Boolean, create : Boolean)) -> FileOpenDecision
  create
    false then Missing
    true then missing-view (mutable, required)

pub file-open-decision is fn ((write : Boolean, directory : Boolean, required : Boolean, mutable : Boolean, exists : Boolean, create : Boolean, exclusive : Boolean, truncate : Boolean)) -> FileOpenDecision
  exists
    true then existing-open (write, directory, required, mutable, create, exclusive, truncate)
    false then missing-open (mutable, required, create)
