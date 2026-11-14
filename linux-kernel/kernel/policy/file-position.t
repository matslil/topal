use language (version is v0.1, features is (systems))
use library support (version is v0.1)

# The service supplies its admitted effective base and actual transfer count.
# For append, base is current EOF, not the previous description position.
pub advance is fn ((field : KernelByteRegion, base : Nat, transferred : Nat)) -> Boolean
  end is base + transferred
  end <= 9223372036854775807
    true then support word-bytes store-u64-le (field, end)
    false then false
