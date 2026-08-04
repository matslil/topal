#!/usr/bin/env topal
# Demonstrates tagged strings plus positional and fully labeled products.
message is text"Topal strings preserve "quotes",
newlines, and {braces}."text
flags is (true, false)
person is (
  name is "Ada",
  active is true
)
person-name is person name
greeting is "Hello, " concatenate person-name
(
  message,
  greeting,
  flags,
  person,
  person-name,
  (),
)
