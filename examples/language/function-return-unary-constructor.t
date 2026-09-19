#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns before strict unary built-in and Union construction.
Wrapped is Union
  Wrap : String

string-exit is fn (value : Int) -> Int
  abandoned : String is String { return value + 1 }
  1000
int-exit is fn (value : Int) -> Int
  abandoned : Int is Int { return value + 2 }
  1000
nat-exit is fn (value : Int) -> Int
  abandoned : Nat is Nat { return value + 3 }
  1000
rational-exit is fn (value : Int) -> Int
  abandoned : Rational is Rational { return value + 4 }
  1000
union-exit is fn (value : Int) -> Int
  abandoned : Wrapped is Wrap { return value + 5 }
  1000
(string-exit 41, int-exit 41, nat-exit 41, rational-exit 41, union-exit 41)
