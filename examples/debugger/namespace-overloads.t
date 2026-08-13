#!/usr/bin/env topal
# Demonstrates reversible overload selection within an aliased namespace.
identity is fn (value : Int) -> Int
  value
identity is fn (value : String) -> String
  value
api is root
(api identity 42, api identity "Topal")
