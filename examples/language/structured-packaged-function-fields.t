#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Tuple and Record values as fields of one or two-operand
# packages. Explicit fields execute once in source order before closed defaults.
make-pair is fn () -> (Int, Int)
  (20, 22)

make-person is fn () -> Record (name : String, active : Boolean)
  (name is "Ada", active is true)

retain-one is fn ((pair : (Int, Int), person : Record (name : String, active : Boolean))) -> ((Int, Int), String)
  (pair, person name)

retain-mixed is fn ((pair : (Int, Int), person : Record (name : String, active : Boolean) default (name is "default", active is true)), enabled : Boolean) -> ((Int, Int), String, Boolean)
  (pair, person name, enabled)

(
  retain-one (person is make-person (), pair is make-pair ()),
  (person is make-person (), pair is make-pair ()) retain-mixed true,
  (pair is (21, 21)) retain-mixed false,
  ((20, 22), (name is "Grace", active is false)) retain-mixed true
)
