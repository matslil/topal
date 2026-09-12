#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates empty String construction, adjacent literal composition, exact
# dynamic concatenation, canonical display, and preserved-sequence emptiness.
preserve is fn (value : String) -> String
  value
empty-text is empty String
left is "e"
mark is "́"
joined is left concat mark
quoted-source is preserve text"say "hello""text
quoted is quoted-source concat "!"
collision-source is preserve text__"value "text and "text_ marker"text__
collision is collision-source concat "!"
chained is "a" concat empty-text concat "b"
adjacent is "adjacent " "literals"
(empty-text, empty? empty-text, empty? joined, joined = "é", joined, quoted, collision, chained, adjacent)
