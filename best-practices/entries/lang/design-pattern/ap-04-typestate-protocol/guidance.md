# Model protocol states explicitly until typestate is available

Name the protocol alternatives with a `Union` and make each operation validate
the state it receives. This is a reviewable runtime model, not typestate:
current Topal has no affine state-indexed capability that can make an invalid
operation unrepresentable.

The linter warns about a protocol-named Boolean state because it hides useful
alternatives, and warns about a protocol-named union to make the remaining
limitation explicit. Both findings are heuristic; naming alone cannot prove a
protocol or a defect.
