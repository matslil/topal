use language (version is v0.1)
use library std (version is v0.1)

Pass is Boolean constraint { value } value = true
captures is std pattern regex captures

simple-expected : List (Boolean, String) is Entry ((true, "ab"), Entry ((true, "a"), Entry ((true, "b"), Empty)))
noncapturing-expected : List (Boolean, String) is Entry ((true, "ab"), Entry ((true, "b"), Empty))
optional-expected : List (Boolean, String) is Entry ((true, "b"), Entry ((false, ""), Empty))
repeated-expected : List (Boolean, String) is Entry ((true, "aa"), Entry ((true, "a"), Empty))
alternative-expected : List (Boolean, String) is Entry ((true, "b"), Entry ((false, ""), Entry ((true, "b"), Empty)))
empty-participation-expected : List (Boolean, String) is Entry ((true, "b"), Entry ((true, ""), Empty))
longest-expected : List (Boolean, String) is Entry ((true, "aaa"), Entry ((true, "aaa"), Empty))
empty-captures : List (Boolean, String) is Empty

simple : Pass is Pass ((captures ("ab", "(a)(b)")) = (true, simple-expected))
noncapturing : Pass is Pass ((captures ("ab", "(?:a)(b)")) = (true, noncapturing-expected))
unmatched-optional : Pass is Pass ((captures ("b", "(a)?b")) = (true, optional-expected))
repeated-last : Pass is Pass ((captures ("aa", "(a)+")) = (true, repeated-expected))
alternative-participation : Pass is Pass ((captures ("b", "(a)|(b)")) = (true, alternative-expected))
participating-empty : Pass is Pass ((captures ("b", "(a*)b")) = (true, empty-participation-expected))
leftmost-longest : Pass is Pass ((captures ("aaa", "(a*)")) = (true, longest-expected))
absent : Pass is Pass ((captures ("Topal", "(Rust)")) = (false, empty-captures))

(simple, noncapturing, unmatched-optional, repeated-last,
 alternative-participation, participating-empty, leftmost-longest, absent)
