# Pre-frame root regression fixture

`pre-frame-kernel-root.t` preserves the exact bytes of the production root at
`3dfc45b1c13534907b540ab654046367bfc8a186`, before its frame-library selection.
Its SHA-256 is
`40495fcdfa7c80fee6af4fb1c76b7b05303a2d0ab6db2039e1ab23bd9a794707`.

Predecessor provider, program-only API, other-library and raw component tests
select this fixture explicitly. They do not erase a frame declaration from a
checked program. Frame tests verify that adding their selected declaration to
these bytes produces the actual production root exactly. The production CLI
tests read the production root and require its complete twenty-object frame
binding.

This fixture retains the preceding profile's regression scope. Current
production selection and application qualification use
[`toolchain-gate.t`](../../../kernel/arch/x86_64/toolchain-gate.t).
