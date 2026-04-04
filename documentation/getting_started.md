# Getting Started with Nova

## Install

Build from source (requires Rust, a C compiler, and cmake):

```bash
git clone https://github.com/pyrotek45/nova-lang
cd nova-lang
cargo build --release
sudo cp target/release/nova /usr/local/bin/   # or ~/.local/bin/
```

<details>
<summary>System dependencies by distro</summary>

**Debian / Ubuntu:**
```bash
sudo apt install gcc cmake pkg-config libgl-dev libx11-dev \
     libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev
```

**Fedora:**
```bash
sudo dnf install gcc cmake pkg-config mesa-libGL-devel libX11-devel \
     libXrandr-devel libXinerama-devel libXcursor-devel libXi-devel
```

**Arch:**
```bash
sudo pacman -S gcc cmake pkg-config mesa libx11 libxrandr libxinerama libxcursor libxi
```

**NixOS:**
```bash
nix-env -if default.nix    # installs the nova binary directly
```

</details>

For the full guide (troubleshooting, NixOS specifics, runtime deps for
graphics), see [Installation](installation.md).

---

## Quick Start

The fastest way to start a project with the standard library:

```bash
nova init myapp --with pyrotek45/nova-lang/std
cd myapp
nova run
```

This creates a project folder, fetches the entire standard library into
`libs/`, and gives you a runnable `main.nv`:

```rust
module main
import libs.core

fn main() {
    println("Hello from myapp!")
}

main()
```

You now have `Box`, `Maybe`, `Result`, `range()`, and all Option helpers.
Add more imports as needed:

```rust
import libs.list       // map, filter, reduce, sortWith, zip
import libs.string     // split, padLeft, capitalize, lines
import libs.math       // sqrt, pow, abs, trig
import libs.hashmap    // HashMap
import libs.grid       // Grid(T) for 2D tilemaps
import libs.io         // prompt, readLines, writeLines
```

---

## Project Structure

```
myapp/
    main.nv           <- entry point (module main)
    libs/             <- your modules + downloaded libraries
    tests/            <- test files (test_*.nv)
```

No config files. If a folder has a `main.nv`, it's a project.

---

## Imports

### Local imports (dot-path)

Each `.` becomes a `/`, and `.nv` is appended. Paths are relative to the
importing file:

```rust
import libs.core              // -> ./libs/core.nv
import libs.utils.strings     // -> ./libs/utils/strings.nv
import super.std.core         // -> ../std/core.nv  (super = go up one dir)
```

### GitHub imports (`import @`)

Pull a file directly from a public GitHub repo -- no download step:

```rust
import @ "owner/repo/path/to/file.nv"
```

The format is **three parts** separated by `/`:

```
         owner / repo / path to file
         -----   ----   ------------
import @ "pyrotek45/nova-lang/std/core.nv"
```

- **owner** -- the GitHub username or org
- **repo** -- the repository name
- **path** -- the file path inside the repo (from the root)

Nova fetches from the `main` branch automatically. **Do not include the
branch name in the path** -- this is wrong:

```rust
// WRONG -- "main" is doubled in the URL:
import @ "pyrotek45/nova-lang/main/std/core.nv"

// RIGHT:
import @ "pyrotek45/nova-lang/std/core.nv"
```

To lock to a specific commit (recommended for reproducibility):

```rust
import @ "pyrotek45/nova-lang/std/core.nv" ! "a1b2c3d"
```

If the fetched file has its own imports (e.g. `core.nv` imports `option.nv`),
Nova resolves those from GitHub automatically -- no extra work needed.

### When to use which

| I want to... | Do this |
|---|---|
| Start a new project | `nova init myapp --with pyrotek45/nova-lang/std` |
| Quick script, no project | `import @ "pyrotek45/nova-lang/std/core.nv"` |
| Add a lib to existing project | `nova install utils myuser/myrepo/src` |
| Pin a dependency | `import @ "..." ! "commithash"` |

---

## Running Code

```bash
nova run                 # run main.nv in current dir (project mode)
nova run file.nv         # run a specific file
nova check file.nv       # type-check only, no execution
nova test                # run all test_*.nv in tests/
nova repl                # interactive REPL
```

Every file command also works with `--git`:

```bash
nova run --git pyrotek45/nova-lang/demo/fib.nv
```

---

## Adding Modules

Create files in `libs/` and import them:

**libs/greetings.nv:**
```rust
module greetings

fn hello(name: String) -> String {
    return "Hello, " + name + "!"
}
```

**main.nv:**
```rust
module main
import libs.greetings

println(hello("Nova"))   // Hello, Nova!
```

All exported functions, structs, and enums are available by name -- no prefixing.

---

## Writing Tests

Test files go in `tests/` and must be named `test_*.nv`:

```rust
module test_math
import super.libs.math_utils    // super because we're in tests/

assert(square(3) == 9, "square of 3")
assert(clamp(15, 0, 10) == 10, "clamp above")

println("PASS: test_math")
```

Run them:

```bash
nova test
```

---

## Installing and Removing Libraries

Add a library from GitHub to an existing project:

```bash
nova install std pyrotek45/nova-lang/std
```

This creates `libs/std/` with all `.nv` files. Import with:

```rust
import libs.std.core
import libs.std.list
```

Remove it:

```bash
nova remove std
```

---

## Command-Line Arguments

```rust
module main

if let args = terminal::args() {
    for a in args {
        println("arg: " + a)
    }
}
```

```bash
nova run main.nv hello world 42
# arg: hello
# arg: world
# arg: 42
```

`terminal::args()` returns `Option([String])`. Use `Cast::int()` or
`Cast::float()` to parse numbers.

---

## What Next?

- [Tutorial](tutorial.md) -- 59 sections from basics to game development
- [Reference](reference.md) -- complete language and standard library reference
- [Installation](installation.md) -- detailed build and troubleshooting guide
- Try `nova run demo/fib.nv` or explore `demo/` and `games/`
