#!/usr/bin/env topal
# Demonstrates preserved strings, Unicode character counting, and products.
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
greeting-character-count is character-count greeting
(
  message,
  greeting,
  literal-composition,
  greeting-character-count,
  flags,
  empty-text,
  person,
  person-name,
  (),
)
