# Contributing to Nova

## Pre-Commit Checklist

Before every commit, make sure to:

### 1. Add new test files
If you added or changed features, **write tests** and add the new `.nv` test files
to `tests/` (positive tests) and/or `tests/should_fail/` (rejection tests).

### 2. Run the full test suite
```bash
nix-shell --run "bash tests/run_tests.sh"
```
All positive tests must print `PASS:` and all should_fail tests must exit non-zero.

### 3. Run clippy
```bash
nix-shell --run "cargo clippy --release"
```
Fix any warnings before committing.

## Quick Reference

| Task | Command |
|---|---|
| Build | `nix-shell --run "cargo build --release"` |
| Run a file | `nix-shell --run "cargo run --release -- run file.nv"` |
| Run tests | `nix-shell --run "bash tests/run_tests.sh"` |
| Clippy | `nix-shell --run "cargo clippy --release"` |
| Install latest | `nix-env -if default.nix --option tarball-ttl 0` |
