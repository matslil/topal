use language (
  version is v0.1
)

# Standard-library expectations are executable Topal constraints. The host
# runner only discovers files and reports source diagnostics.
Pass is Boolean constraint { value } value = true
harness-accepts-true : Pass is Pass true
harness-accepts-true
