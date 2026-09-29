#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the declarative architecture-model package vocabulary.
pub schema-revision is 1

### Construct an exact cost dimension with explicit unit and provenance.
pub exact-cost is fn (
  dimension : String,
  amount : Nat,
  unit : String,
  provenance : String
) -> Record (dimension : String, kind : String, lower : Nat, upper : Nat, unit : String, provenance : String)
  (
    dimension is dimension,
    kind is "exact",
    lower is amount,
    upper is amount,
    unit is unit,
    provenance is provenance
  )

### Construct a bounded cost dimension.
pub interval-cost is fn (
  dimension : String,
  lower : Nat,
  upper : Nat,
  unit : String,
  provenance : String
) -> Record (dimension : String, kind : String, lower : Nat, upper : Nat, unit : String, provenance : String)
  (
    dimension is dimension,
    kind is "interval",
    lower is lower,
    upper is upper,
    unit is unit,
    provenance is provenance
  )

### An interval is locally well formed; complete provider validation is external.
pub cost-bounds-valid? is fn (
  cost : Record (dimension : String, kind : String, lower : Nat, upper : Nat, unit : String, provenance : String)
) -> Boolean
  (cost lower) <= (cost upper)
