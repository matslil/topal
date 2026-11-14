use language (version is v0.1, features is (systems))
use library support (version is v0.1)

# TK-DEC-281; actual metadata-page effects, never candidate publication.
# TOPAL-SYSTEMS-NAMESPACE-ACTION-001.
# Whole-owner admission precedes this selected-record and input preflight.

word is fn ((region : KernelByteRegion, offset : Nat)) -> Nat
  support namespace-bytes word (region, offset)

write is fn ((region : KernelByteRegion, offset : Nat, value : Nat)) -> Boolean
  region unsigned store (offset-bytes is offset, byte-count is 8, byte-order is little-endian, value is value)

both is fn ((first : Boolean, second : Boolean)) -> Boolean
  first
    true then second
    false then false

four is fn ((first : Boolean, second : Boolean, third : Boolean, fourth : Boolean)) -> Boolean
  both (both (first, second), both (third, fourth))

object-base is fn (slot : Nat) -> Nat
  256 + (slot * 128)

entry-base is fn (slot : Nat) -> Nat
  64 + (slot * 16)

name-base is fn (slot : Nat) -> Nat
  1536 + (slot * 256)

count-association is fn ((region : KernelByteRegion, offset : Nat, identity : Nat)) -> Nat
  (word (region, offset)) = identity
    true then 1
    false then 0

links is fn ((region : KernelByteRegion, identity : Nat)) -> Nat
  seen-0 is count-association (region, 64, identity)
  seen-1 is count-association (region, 80, identity)
  seen-2 is count-association (region, 96, identity)
  seen-3 is count-association (region, 112, identity)
  seen-0 + seen-1 + seen-2 + seen-3

regular is fn ((region : KernelByteRegion, base : Nat, identity : Nat)) -> Boolean
  mode is word (region, base + 24)
  four ((word (region, base)) = 1, (word (region, base + 8)) = identity, mode >= 32768, mode <= 36863)

entry-names is fn ((region : KernelByteRegion, slot : Nat, identity : Nat)) -> Boolean
  base is entry-base slot
  length is word (region, base + 8)
  four ((word (region, base)) = identity, length > 0, length <= 255, identity >= 7)

source-entry-valid is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat)) -> Boolean
  (facts source-entry) < 4
    true then entry-names (region, (facts source-entry), (facts source-identity))
    false then both (operation = 0, (facts source-entry) = 4)

source-valid is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat)) -> Boolean
  base is object-base (facts source-object)
  count is links (region, (facts source-identity))
  intent is word (region, base + 64)
  initial is four (count = 0, operation = 0, intent = 1, (facts source-entry) = 4)
  linked is both (count > 0, intent = 0)
  eligible is not (both (not initial, not linked))
  metadata is four (regular (region, base, (facts source-identity)), count = (facts source-links), (word (region, base + 56)) = count, eligible)
  both (metadata, source-entry-valid (region, facts, operation))

empty-target is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat))) -> Boolean
  base is entry-base (facts target-entry)
  clean is support namespace-bytes zero-name (region, name-base (facts target-entry))
  four ((word (region, base)) = 0, (word (region, base + 8)) = 0, clean, (facts source-links) < 4)

live-target is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat))) -> Boolean
  base is object-base (facts target-object)
  count is links (region, (facts target-identity))
  metadata is four (regular (region, base, (facts target-identity)), count > 0, (word (region, base + 56)) = count, (word (region, base + 64)) = 0)
  entry is entry-names (region, (facts target-entry), (facts target-identity))
  both (metadata, entry)

target-move-or-live is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat)) -> Boolean
  operation = 1
    true then (facts target-entry) = (facts source-entry)
    false then live-target (region, facts)

target-valid is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat)) -> Boolean
  operation = 0
    true then empty-target (region, facts)
    false then target-move-or-live (region, facts, operation)

index-shape is fn ((facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat)) -> Boolean
  absent is both (operation <= 1, both ((facts target-object) = 4, (facts target-identity) = 0))
  different is four ((facts target-object) < 4, (facts target-object) != (facts source-object), (facts target-identity) >= 7, (facts target-identity) != (facts source-identity))
  present is both (operation >= 2, different)
  target is not (both (not absent, not present))
  source is four ((facts source-object) < 4, (facts source-entry) <= 4, (facts source-identity) >= 7, (facts source-links) <= 4)
  four (source, target, (facts target-entry) < 4, operation <= 3)

input-clean is fn ((region : KernelByteRegion, length : Nat)) -> Boolean
  middle is support namespace-bytes zero-quarter (region, 3136)
  end is support namespace-bytes zero-half (region, 3456)
  valid is support namespace-bytes valid-input (region, length)
  both (valid, both (middle, end))

parent-times is fn ((region : KernelByteRegion, seconds : Nat)) -> Boolean
  modified is write (region, 48, seconds)
  changed is write (region, 56, seconds)
  both (modified, changed)

link-effect is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), seconds : Nat)) -> Boolean
  base is object-base (facts source-object)
  entry is entry-base (facts target-entry)
  effect-0 is support namespace-bytes copy-name (region, name-base (facts target-entry), 3200)
  effect-1 is write (region, entry, (facts source-identity))
  effect-2 is write (region, entry + 8, (facts length))
  effect-3 is write (region, base + 56, (facts source-links) + 1)
  effect-4 is write (region, base + 64, 0)
  effect-5 is write (region, base + 48, seconds)
  objects-changed is both (effect-0, both (effect-1, both (effect-2, both (effect-3, both (effect-4, effect-5)))))
  parent-changed is parent-times (region, seconds)
  both (objects-changed, parent-changed)

move-effect is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), seconds : Nat)) -> Boolean
  entry is entry-base (facts source-entry)
  effect-0 is support namespace-bytes copy-name (region, name-base (facts source-entry), 3200)
  effect-1 is write (region, entry + 8, (facts length))
  effect-2 is write (region, (object-base (facts source-object)) + 48, seconds)
  objects-changed is both (effect-0, both (effect-1, effect-2))
  parent-changed is parent-times (region, seconds)
  both (objects-changed, parent-changed)

replace-effect is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), seconds : Nat)) -> Boolean
  source is entry-base (facts source-entry)
  victim is object-base (facts target-object)
  remaining : Nat is absolute ((word (region, victim + 56)) + -1)
  effect-0 is write (region, entry-base (facts target-entry), (facts source-identity))
  effect-1 is write (region, source, 0)
  effect-2 is write (region, source + 8, 0)
  effect-3 is support namespace-bytes clear-name (region, name-base (facts source-entry))
  effect-4 is write (region, victim + 56, remaining)
  effect-5 is write (region, victim + 48, seconds)
  effect-6 is write (region, (object-base (facts source-object)) + 48, seconds)
  objects-changed is both (effect-0, both (effect-1, both (effect-2, both (effect-3, both (effect-4, both (effect-5, effect-6))))))
  parent-changed is parent-times (region, seconds)
  both (objects-changed, parent-changed)

exchange-effect is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), seconds : Nat)) -> Boolean
  source is entry-base (facts source-entry)
  effect-0 is write (region, source, (facts target-identity))
  effect-1 is write (region, entry-base (facts target-entry), (facts source-identity))
  effect-2 is write (region, (object-base (facts source-object)) + 48, seconds)
  effect-3 is write (region, (object-base (facts target-object)) + 48, seconds)
  objects-changed is both (effect-0, both (effect-1, both (effect-2, effect-3)))
  parent-changed is parent-times (region, seconds)
  both (objects-changed, parent-changed)

effect-replace-or-exchange is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat, seconds : Nat)) -> Boolean
  operation = 2
    true then replace-effect (region, facts, seconds)
    false then exchange-effect (region, facts, seconds)

effect-rename is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat, seconds : Nat)) -> Boolean
  operation = 1
    true then move-effect (region, facts, seconds)
    false then effect-replace-or-exchange (region, facts, operation, seconds)

effect is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat, seconds : Nat)) -> Boolean
  operation = 0
    true then link-effect (region, facts, seconds)
    false then effect-rename (region, facts, operation, seconds)


checked is fn ((region : KernelByteRegion, facts : Record (source-object : Nat, source-entry : Nat, target-object : Nat, target-entry : Nat, length : Nat, source-identity : Nat, target-identity : Nat, source-links : Nat), operation : Nat, seconds : Nat)) -> Boolean
  distinct is not (both (operation != 1, (facts target-entry) = (facts source-entry)))
  valid is four (source-valid (region, facts, operation), target-valid (region, facts, operation), input-clean (region, (facts length)), distinct)
  valid
    true then effect (region, facts, operation, seconds)
    false then false


read-input is fn ((region : KernelByteRegion, operation : Nat, seconds : Nat)) -> Boolean
  facts is (source-object is word (region, 3072), source-entry is word (region, 3080), target-object is word (region, 3088), target-entry is word (region, 3096), length is word (region, 3104), source-identity is word (region, 3112), target-identity is word (region, 3120), source-links is word (region, 3128))
  index-shape (facts, operation)
    true then checked (region, facts, operation, seconds)
    false then false

pub apply is fn ((region : KernelByteRegion, operation : Nat, seconds : Nat)) -> Boolean
  extent is region byte count
  geometry is four (extent >= 4096, operation <= 3, seconds <= 18446744073709551615, extent <= 18446744073709551615)
  geometry
    true then read-input (region, operation, seconds)
    false then false
