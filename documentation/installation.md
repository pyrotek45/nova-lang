# Nova — Installation, Building & Distribution Report

## Executive Summary

Nova is a statically typed, bytecode-compiled language implemented in Rust.
It compiles `.nv` source to bytecode and runs on a custom stack-based VM.
The language ships as a single binary (`nova`) with no runtime installer —
all dependencies are resolved at build time through Cargo and (optionally) Nix.

---

## 1. Prerequisites

### Minimum Requirements

| Dependency | Version | Purpose |
|---|---|---|
| Rust (rustc + cargo) | 1.70+ | Compiles the Nova toolchain |
| A C compiler (gcc/clang) | Any recent | Required by the `raylib` crate for FFI binding |
| cmake | 3.x | Builds raylib's C library during `cargo build` |
| pkg-config | Any | Locates system libraries |

### Additional (for graphical programs using raylib)

| Dependency | Purpose |
|---|---|
| libGL (Mesa) | OpenGL rendering |
| X11 libs (libX11, libXrandr, libXinerama, libXcursor, libXi) | Windowing on X11 |
| Wayland + GLFW (optional) | Wayland windowing alternative |

### NixOS / Nix Users

A `shell.nix` is provided at the repository root that pulls every dependency
automatically. Enter the dev shell with:

```bash
nix-shell
```

This supplies gcc, rustup, clang, cmake, pkg-config, Wayland, GLFW, libGL,
all X11 libs, rust-analyzer, and sets `LD_LIBRARY_PATH` + `LIBCLANG_PATH`.

---

## 2. Building from Source

### Standard Build (any Linux distro)

```bash
git clone https://github.com/pyrotek45/nova-lang
cd nova-lang
cargo build --release
```

The final binary is:

```
target/release/nova
```

### NixOS Build

```bash
git clone https://github.com/pyrotek45/nova-lang
cd nova-lang
nix-shell --run "cargo build --release"
```

### Build Times (approximate)

| Build | Time |
|---|---|
| Fresh release build | 60–120 s |
| Incremental (Rust change) | 3–10 s |
| No changes | < 0.1 s |

---

## 3. Project Structure

Nova is a Cargo **workspace** with 12 crates:

```
nova-lang/
├── novacli/       CLI entry point (the `nova` binary)
├── lexer/         Tokeniser
├── parser/        Recursive-descent parser + type checker integration
├── typechecker/   Hindley-Milner-style type inference & checking
├── compiler/      AST → bytecode compiler
├── optimizer/     Bytecode optimisation passes
├── assembler/     Bytecode assembler
├── disassembler/  Bytecode disassembler (nova dis)
├── vm/            Stack-based virtual machine
├── common/        Shared types (opcodes, environment, TType, etc.)
├── native/        Native function bindings (raylib, crossterm, regex, rand)
├── novacore/      Core runtime: builtins, Cast::, Float::, String::, etc.
├── std/           Standard library (.nv files)
├── tests/         Test suite (.nv files + runner script)
├── demo/          Demo programs
├── games/         Game projects (Dungeon, Pong, Breakout, Shooter, Survive)
├── documentation/ Tutorial and reference docs
└── fuzz/          Fuzz testing targets
```

### Key Rust Dependencies

| Crate | Version | Used in |
|---|---|---|
| `raylib` | 5.0.2 | `native/` — graphics, audio, input |
| `crossterm` | 0.27.0 | `native/` — terminal UI |
| `regex` | 1.11.1 | `native/` — string pattern matching |
| `rand` | 0.8.x | `native/` + `novacli/` — random numbers |
| `reedline` | 0.38.0 | `novacli/` — REPL line editing |
| `bincode` | 1.3 | `novacli/` — bytecode serialisation |
| `ureq` | 2.x | `parser/`, `novacli/` — HTTP client for GitHub imports |

---

## 4. Running Nova Programs

### CLI Commands

```bash
nova run   <file.nv>    # Compile + run
nova run                # Run main.nv in current directory (project mode)
nova run   --git <path> # Fetch and run from GitHub
nova check <file.nv>    # Type-check only (no execution)
nova check --git <path> # Type-check a file from GitHub
nova time  <file.nv>    # Run with timing information
nova time  --git <path> # Time a GitHub file's execution
nova dis   <file.nv>    # Disassemble to bytecode listing
nova dis   --git <path> # Disassemble a file from GitHub
nova dbg   <file.nv>    # Debug run
nova dbg   --git <path> # Debug a file from GitHub
nova init  <name>       # Create a new project (--with for GitHub deps)
nova install <n> <r/p>  # Install a library into libs/<name>/ from GitHub
nova remove  <name>     # Remove a library from libs/<name>/
nova test               # Run all test_*.nv files in tests/
nova repl               # Interactive REPL
```

See the [Getting Started](getting_started.md) guide for project setup workflow.

### Example

```bash
./target/release/nova run demo/fib.nv
./target/release/nova check tests/test_feature_combos.nv
```

---

## 5. Standard Library — File Resolution

Nova uses **relative-path imports** via the `super` keyword:

```rust
import super.std.core    // resolves to ../std/core.nv relative to source file
import super.std.list    // resolves to ../std/list.nv
```

**How it works:** The parser translates each `super` to `..` and each `.`-separated
segment to a path component, appending `.nv`:

```
import super.std.grid
→ ../std/grid.nv  (relative to the importing file's directory)
```

### Implications for Distribution

- The `std/` directory **must** be in a known relative position to user source files.
- No global install path or environment variable is used.
- Projects are self-contained: copy the repo, and imports work.
- For games in `games/Dungeon/`, the import is `import super.super.std.grid`
  (two levels up to reach the `std/` directory).

### Standard Library Modules (29 files)

| Module | Purpose |
|---|---|
| `core.nv` | Box, Gen, Maybe, Result, range(), Option helpers |
| `list.nv` | map, filter, reduce, sortWith, flatten, zip |
| `iter.nv` | Lazy iterator: fromVec, map, filter, collect |
| `string.nv` | split, padLeft, padRight, capitalize, lines, words |
| `math.nv` | sqrt, pow, abs, floor, trig, isPrime, gcd |
| `io.nv` | prompt(), readLines(), writeLines() |
| `hashmap.nv` | O(1) HashMap |
| `set.nv` | Hash set |
| `deque.nv` | Double-ended queue |
| `maybe.nv` | Maybe(T) — Just/Nothing |
| `result.nv` | Result(A,B) — Ok/Err |
| `option.nv` | Option extensions |
| `tuple.nv` | Tuple utilities |
| `functional.nv` | compose, pipe, identity, flip |
| `grid.nv` | Generic 2D grid — `Grid(T)` |
| `plot.nv` | 2D plotting/charting via raylib |
| `vec2.nv` | 2D vector math |
| `color.nv` | Named RGB color tuples |
| `noise.nv` | Noise generation |
| `tui.nv` | Terminal UI helpers |
| `ansi.nv` | ANSI escape code helpers |
| `widget.nv` | Raylib GUI widgets |
| `input.nv` | Input polling abstraction |
| `camera.nv` | 2D camera |
| `scene.nv` | Scene management |
| `entity.nv` | Entity/component helpers |
| `physics.nv` | Simple physics |
| `timer.nv` | Timer utilities |
| `tween.nv` | Easing/tween functions |

---

## 6. Raylib Integration

Nova bundles raylib via the Rust `raylib` crate (v5.0.2).  This crate
**compiles raylib from C source** during `cargo build`, so:

- No separate raylib installation is needed.
- The C library is statically linked into the `nova` binary.
- Graphics programs (games, demos, plots) work out of the box.
- Only system-level OpenGL + X11/Wayland libs are required.

### What raylib provides to Nova programs

- Window creation and rendering loop (`raylib::init`, `raylib::rendering`)
- 2D drawing primitives (lines, rectangles, circles, triangles, text)
- Audio playback (sounds, music)
- Input handling (keyboard, mouse, gamepad)
- All exposed as `raylib::*` native functions in Nova

---

## 7. Testing

### Test Suite

```bash
bash tests/run_tests.sh
```

The runner finds all `tests/test_*.nv` files (positive tests) and all
`tests/should_fail/*.nv` files (rejection tests).

**Current count:** 301 tests (124 positive + 177 rejection), all passing.

| Test category | Count | What it tests |
|---|---|---|
| Positive (`test_*.nv`) | 124 | Correct programs that must compile and output "PASS:" |
| Rejection (`should_fail/`) | 177 | Invalid programs that must be rejected by the type checker or parser |

### Key Test Files

| File | Assertions | Focus |
|---|---|---|
| `test_feature_combos.nv` | 32 | Cross-feature interactions (generics+closures, enums+extends, etc.) |
| `test_negative_indexing.nv` | 25 | Negative indices on lists, strings |
| `test_undocumented_features.nv` | ~30 | List concat, block expr, pass, match features |
| `test_std_grid.nv` | ~30 | Generic Grid(T) library |
| `test_closures.nv` | ~20 | Closure capture, higher-order patterns |
| `test_enums.nv` | ~20 | Enum definition, matching, generics |
| `test_generics.nv` | ~20 | Generic structs, enums, extends |

---

## 8. Distribution Options

### Option A: Source Distribution (current)

```
git clone + cargo build --release
```

- **Pros:** Works everywhere Rust compiles, always up-to-date.
- **Cons:** Requires Rust toolchain + C compiler + cmake.

### Option B: Pre-built Binary

```bash
# Build on CI, distribute the single file:
target/release/nova       # ~15-25 MB statically linked
```

- The `nova` binary is fully self-contained (raylib statically linked).
- Users still need the `std/` directory in the expected relative path.
- A tarball distribution would look like:

```
nova-linux-x86_64.tar.gz
├── nova                  # the binary
├── std/                  # standard library
│   ├── core.nv
│   ├── list.nv
│   └── ...
├── demo/                 # example programs
└── README.md
```

### Option C: Nix Flake (NixOS-native)

A `flake.nix` could provide:
```bash
nix run github:pyrotek45/nova-lang -- run demo/fib.nv
```

This would handle all dependencies automatically and is the most
NixOS-idiomatic approach.

### Option D: Cargo Install

```bash
cargo install --path novacli
```

This installs the `nova` binary to `~/.cargo/bin/`, but the `std/`
directory would need manual placement.

---

## 9. Known Limitations

1. **No global std path:** The standard library is found via relative `super`
   imports, not a global search path. This means projects must be structured
   relative to the `std/` directory.

2. **Raylib always compiled:** Even pure-terminal programs link against
   raylib because native functions are registered unconditionally. A future
   feature-flag could make raylib optional.

3. **No package manager:** There is no `nova install <package>` mechanism.
   Third-party libraries can be fetched from GitHub using `import @` in
   source code or `nova init --with` to download an entire folder, but
   there is no versioned dependency resolution or lockfile.

4. **Single-file compilation:** Each `nova run` re-compiles from source.
   There is no separate "compile to `.nvbc`" then "run `.nvbc`" workflow
   (though the infrastructure for bytecode serialisation exists via bincode).

---

## 10. Troubleshooting

### Build Failures

#### `is 'cmake' not installed?`

The `raylib` crate compiles raylib from C source during `cargo build`,
which requires `cmake`. Install it:

```bash
# Debian / Ubuntu
sudo apt install cmake

# Fedora
sudo dnf install cmake

# Arch
sudo pacman -S cmake

# NixOS — already included in shell.nix
nix-shell
```

#### `cc` / `gcc` not found

A C compiler is required by the raylib build script:

```bash
sudo apt install gcc          # or build-essential on Debian/Ubuntu
sudo dnf install gcc          # Fedora
sudo pacman -S gcc            # Arch
```

#### `pkg-config` not found

```bash
sudo apt install pkg-config
```

#### `fatal error: X11/Xlib.h: No such file or directory`

Missing X11 development headers. Install the windowing libraries:

```bash
sudo apt install libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev
```

#### `fatal error: GL/gl.h: No such file or directory`

Missing OpenGL development headers:

```bash
sudo apt install libgl-dev        # or libgl1-mesa-dev on older distros
```

#### `LIBCLANG_PATH` errors (NixOS)

If you see errors about `libclang` not being found, make sure you are
inside the nix-shell:

```bash
nix-shell          # sets LIBCLANG_PATH automatically
cargo build --release
```

The `shell.nix` exports `LIBCLANG_PATH=${pkgs.llvmPackages_18.libclang.lib}/lib`.

### Runtime Errors

#### Raylib programs crash or show "Failed to open display"

Raylib requires a graphical display (X11 or Wayland). This error occurs when:
- Running on a headless server or CI with no display
- The `DISPLAY` or `WAYLAND_DISPLAY` environment variable is not set
- Running via SSH without X forwarding

The error output looks like:
```
WARNING: GLFW: Error: 65550 Description: X11: Failed to open display
WARNING: GLFW: Failed to initialize GLFW
Segmentation fault
```

**Fixes:**
- Run from a desktop session (not SSH or headless)
- For SSH, use `ssh -X` to enable X forwarding
- Non-graphical programs (`fib.nv`, test suite, etc.) work fine without a display

#### `libGL.so.1: cannot open shared object`

The nova binary statically links raylib's C code, but OpenGL/X11 libraries
are loaded dynamically at runtime. If you get missing `.so` errors when
running graphical programs:

```bash
# Debian / Ubuntu
sudo apt install libgl1-mesa-glx libx11-6

# NixOS — the shell.nix sets LD_LIBRARY_PATH automatically
nix-shell
./target/release/nova run demo/bounce.nv
```

On NixOS, **always** run graphical programs from inside `nix-shell` (or
set `LD_LIBRARY_PATH` yourself) because Nix does not use the standard
`/usr/lib` paths.

#### Non-graphical programs work fine outside nix-shell

The `nova` binary only needs system GL/X11 libraries for programs that call
`raylib::init`. Console programs (text I/O, computation, test suite) work
anywhere without nix-shell or graphics libraries.

### NixOS-Specific Notes

1. **Always use `nix-shell` for building.** The `shell.nix` provides cmake,
   clang, pkg-config, and all X11/GL/Wayland libraries. Without it, `cargo
   build` will fail because cmake and headers are not on the default NixOS path.

2. **Always use `nix-shell` for graphical programs.** The shellHook sets
   `LD_LIBRARY_PATH` to include libGL and all X11 libraries. Without it,
   raylib cannot find the OpenGL and X11 shared libraries at runtime.

3. **Console programs work outside nix-shell.** Once built, `nova run demo/fib.nv`
   or `bash tests/run_tests.sh` works without nix-shell.

4. **Updating Nix packages.** If you update `nixpkgs` and the build breaks,
   try `nix-shell --pure` to isolate from stale environment variables.

---

## 11. Quick Start Checklist

```
[ ] Install Rust:          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
[ ] Install system deps:   sudo apt install gcc cmake pkg-config libgl-dev libx11-dev
                           (or equivalent for your distro)
[ ] Clone:                 git clone https://github.com/pyrotek45/nova-lang && cd nova-lang
[ ] Build:                 cargo build --release
[ ] Verify:                ./target/release/nova run demo/fib.nv
[ ] Run tests:             bash tests/run_tests.sh
[ ] Try the REPL:          ./target/release/nova repl
```

For NixOS, replace steps 1-2 and 4 with:
```
[ ] Enter shell:           nix-shell
[ ] Build:                 cargo build --release
```
