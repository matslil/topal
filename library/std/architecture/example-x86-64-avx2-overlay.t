use language (
  version is v0.1
)

# Illustrative specific-processor overlay. The provider must replace the
# placeholder base digest with the canonical digest it actually validated.
addition is fn (value : Record (identity : String, subject : String, kind : String, value : String, depends-on : String)) -> Record (identity : String, subject : String, kind : String, value : String, depends-on : String)
  value
source is fn (value : Record (identity : String, kind : String, revision : String, digest : String)) -> Record (identity : String, kind : String, revision : String, digest : String)
  value

additions : List (Record (identity : String, subject : String, kind : String, value : String, depends-on : String)) is Entry (
  addition (identity is "feature-avx", subject is "cpu", kind is "feature", value is "avx", depends-on is ""),
  Entry (
    addition (identity is "feature-avx2", subject is "cpu", kind is "feature", value is "avx2", depends-on is "feature-avx"),
    Empty
  )
)
replacements : List String is Empty
provenance : List (Record (identity : String, kind : String, revision : String, digest : String)) is Entry (
  source (identity is "example-only", kind is "assumption", revision is "1", digest is "not-qualified"),
  Empty
)

pub architecture-overlay is (
  schema is "topal.architecture-overlay/1",
  name is "example-x86-64-avx2",
  base is "sha256:replace-with-generic-x86-64-linux-digest",
  additions is additions,
  replacements is replacements,
  provenance is provenance
)
