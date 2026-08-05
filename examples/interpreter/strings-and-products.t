#!/usr/bin/env topal
# Demonstrates tagged strings plus positional and fully labeled products.
message is text"Topal strings preserve "quotes",
newlines, and {braces}."text
flags is (true, false)
empty-text is empty String
person is (
  name is "Ada",
  active is true
)
person-name is person name
literal-composition is "adjacent " "literals"
greeting is "Hello, " concat person-name
(
  message,
  greeting,
  literal-composition,
  flags,
  empty-text,
  person,
  person-name,
  (),
)
