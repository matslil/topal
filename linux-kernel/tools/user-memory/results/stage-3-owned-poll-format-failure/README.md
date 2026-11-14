# Included-file formatting failure

The retained post-check stops at explicit rustfmt with exit one. Workspace
formatting does not format this included Rust file. The required correction
wraps one private function signature across three lines; it changes no token.
The complete pre-format file is retained. All three preceding VM runs passed.
After correction, a fresh focused VM must verify the formatted producer and
compare its complete objects/kernel/boot image with those prior runs. The
earlier regressions can then support only the exact matching native bytes;
they are not represented as executions from the formatted source input.
This check remains failed. It is unrelated to AI safeguards or resource limits.
