use language (version is v0.1)
# ELF wire interpretation is ordinary arithmetic over immutable copied bytes.

byte0 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 0)
    Some value then value
    None then 0

byte1 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 1)
    Some value then value
    None then 0

byte2 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 2)
    Some value then value
    None then 0

byte3 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 3)
    Some value then value
    None then 0

byte4 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 4)
    Some value then value
    None then 0

byte5 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 5)
    Some value then value
    None then 0

byte6 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 6)
    Some value then value
    None then 0

byte16 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 16)
    Some value then value
    None then 0

byte17 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 17)
    Some value then value
    None then 0

byte18 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 18)
    Some value then value
    None then 0

byte19 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 19)
    Some value then value
    None then 0

byte20 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 20)
    Some value then value
    None then 0

byte21 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 21)
    Some value then value
    None then 0

byte22 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 22)
    Some value then value
    None then 0

byte23 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 23)
    Some value then value
    None then 0

byte52 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 52)
    Some value then value
    None then 0

byte53 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 53)
    Some value then value
    None then 0

byte54 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 54)
    Some value then value
    None then 0

byte55 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 55)
    Some value then value
    None then 0

byte56 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 56)
    Some value then value
    None then 0

byte57 is fn (bytes : Array (64, Int)) -> Int
  array-at? (bytes, 57)
    Some value then value
    None then 0

word-two is fn ((low : Int, high : Int)) -> Int
  low + (high * 256)

word-three is fn ((low : Int, middle : Int, high : Int)) -> Int
  low + (middle * 256) + (high * 65536)

word-four is fn ((first : Int, second : Int, third : Int, fourth : Int)) -> Int
  first + (second * 256) + (third * 65536) + (fourth * 16777216)

read-magic is fn (bytes : Array (64, Int)) -> Int
  word-four (byte0 bytes, byte1 bytes, byte2 bytes, byte3 bytes)

read-ident is fn (bytes : Array (64, Int)) -> Int
  word-three (byte4 bytes, byte5 bytes, byte6 bytes)

read-type is fn (bytes : Array (64, Int)) -> Int
  word-two (byte16 bytes, byte17 bytes)

read-machine is fn (bytes : Array (64, Int)) -> Int
  word-two (byte18 bytes, byte19 bytes)

read-version is fn (bytes : Array (64, Int)) -> Int
  word-four (byte20 bytes, byte21 bytes, byte22 bytes, byte23 bytes)

read-header-size is fn (bytes : Array (64, Int)) -> Int
  word-two (byte52 bytes, byte53 bytes)

read-entry-size is fn (bytes : Array (64, Int)) -> Int
  word-two (byte54 bytes, byte55 bytes)

read-count is fn (bytes : Array (64, Int)) -> Int
  word-two (byte56 bytes, byte57 bytes)

check-count is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  count is read-count bytes
  count >= 1
    true then count <= maximum
    false then false

check-entry-size is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  value is read-entry-size bytes
  value = 56
    true then check-count (bytes, machine, maximum)
    false then false

check-header-size is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  value is read-header-size bytes
  value = 64
    true then check-entry-size (bytes, machine, maximum)
    false then false

check-version is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  value is read-version bytes
  value = 1
    true then check-header-size (bytes, machine, maximum)
    false then false

check-machine is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  value is read-machine bytes
  value = machine
    true then check-version (bytes, machine, maximum)
    false then false

check-type is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  value is read-type bytes
  value = 2
    true then check-machine (bytes, machine, maximum)
    false then false

check-ident is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  value is read-ident bytes
  value = 65794
    true then check-type (bytes, machine, maximum)
    false then false

pub admitted? is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  magic is read-magic bytes
  magic = 1179403647
    true then check-ident (bytes, machine, maximum)
    false then false
