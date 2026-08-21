# Data-transfer platform qualification

This record prevents cross-compilation or API presence from being reported as
behavioral conformance. The common deterministic suite is portable; native
claims additionally require execution on the named target with its OS build,
SDK, filesystem, sandbox policy, and feature probes recorded.

| Target | Implemented boundary | Qualification disposition |
| --- | --- | --- |
| Linux | injected files and sockets; positioned I/O; Linux `i2c-dev` `I2C_RDWR` | executed on Linux by the workspace suite; physical-bus tests remain opt-in |
| Windows | injected `File`/Winsock resources through documented Rust/Win32 bindings | platform-specific: run completion, cancellation, reparse-point, and IOCP qualification on Windows |
| macOS | injected descriptor/socket resources through documented Rust/Darwin bindings | platform-specific: run Dispatch/Network.framework, sandbox, and `kqueue` qualification on macOS |
| Android | injected descriptors/sockets; framework broker remains the authority source | platform-specific: run NDK/framework and scoped-storage qualification on Android; raw I2C unavailable to ordinary apps |
| iOS | injected app-approved files/sockets; framework broker remains the authority source | platform-specific: run Dispatch/Network.framework/URLSession and sandbox qualification on iOS; raw I2C unavailable |

The initial release therefore supports deterministic semantics and the
executed Linux backend. The other target modules are build scaffolds, not
behavioral support claims, until their recorded native qualification passes.
