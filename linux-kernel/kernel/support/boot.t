use language (version is v0.1)
use library support (version is v0.1)
# Keep absence policy shared through a real transitive source-module call.
pub missing? is fn (present : Boolean) -> Boolean
  support logic absent? present
