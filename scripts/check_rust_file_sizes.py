#!/usr/bin/env python3
"""Reject Rust source files that exceed the repository reviewability ceiling."""

from pathlib import Path
import os
import sys

MAX_LINES = 2_000


def rust_sources(root: Path):
    # Prune generated dependencies before traversal; retain the limit for every
    # owned source file, including new files that have not been committed yet.
    for directory, directories, files in os.walk(root):
        directories[:] = [
            name for name in directories if name not in {".git", "target", ".cache"}
        ]
        for name in files:
            if name.endswith(".rs"):
                yield Path(directory) / name


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    oversized = []
    for path in rust_sources(root):
        line_count = len(path.read_text(encoding="utf-8").splitlines())
        if line_count > MAX_LINES:
            oversized.append((path.relative_to(root), line_count))
    if not oversized:
        print(f"all Rust source files are at most {MAX_LINES} lines")
        return 0
    for path, line_count in sorted(oversized):
        print(f"{path}: {line_count} lines (maximum {MAX_LINES})", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
