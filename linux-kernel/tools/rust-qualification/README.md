# Rust qualification on a separate runner

The current local host cannot compile every Rust target within its available
bounded memory budget. This runner preserves `scripts/run_bounded.py` unchanged:
the default requested limit is 4 GiB, the normal host reserve is retained, and
the validation cgroup has zero swap. It does not change the user's host or
require privileged guest execution. This is implementation-test infrastructure,
with no language-design, kernel-interface or test-strategy change.

Run on a Linux machine with a functioning systemd user manager, Rust 1.97.0 and
its `llvm-tools` component, C build tools, Python and GDB:

```console
python3 linux-kernel/tools/rust-qualification/run.py --output /tmp/topal-rust-qualification --check-environment
python3 linux-kernel/tools/rust-qualification/run.py --output /tmp/topal-rust-qualification
```

Use a committed checkout and a fresh output directory for each execution.
The driver sequentially runs locked all-target workspace checking, all-target
workspace tests and doc tests. Each command goes through the normal bounded
wrapper. A failed command remains a failure even if other commands pass.
The JSON records the actual commit/tree, tool versions, driver/wrapper hashes,
configuration, command exit statuses and log hashes. An interrupted execution
retains `complete: false` and `passed: false`. It does not qualify QEMU execution
or replace existing machine observations.

The adjacent workflow template prepares a standard Ubuntu GitHub-hosted runner,
uses read-only repository permissions and pinned actions, and uploads the results
even after failed checks. It starts the disposable runner's user service manager
so the same bounded wrapper can operate. No host memory reserve, cgroup limit,
swap policy or security setting is relaxed.

The template is **inactive** here. GitHub requires workflow entry points under
`.github/workflows/`; see its [workflow documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax).
Activating it therefore needs an explicit exception to the
requested placement of Linux-kernel files under `linux-kernel/`; only that entry
point would live outside the requested tree. The script and documentation stay
here. No Actions success or full Rust qualification is claimed by preparing the
template. See the [current report](../../step-4-source-ram-page-allocation.md).
