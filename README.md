# Nova

A statically typed language with expressive syntax, a built-in package manager,
and game development out of the box. Compiles to bytecode, runs on a stack-based VM.

---

## Why Nova?

**One binary does everything.** `nova run`, `nova check`, `nova test`, `nova install` —
no separate toolchain, no config files, no build system. Write code, run it.

**Import directly from GitHub.** No registry, no publish step. Point at a repo and go:

```rust
import @ "pyrotek45/nova-utils/strings.nv"
import @ "pyrotek45/nova-utils/strings.nv" ! "a1b2c3d"   // pin to a commit
```

**Ship games from day one.** Raylib is built in. Open a window, draw sprites, play audio —
all with zero setup. The standard library includes scene management, entity systems,
physics, tweens, tilemaps, and a charting library.

**Structural typing without inheritance.** `Dyn(T)` accepts any struct with the right shape.
No interfaces to declare, no traits to implement:

```rust
fn greet(thing: Dyn(T = name: String)) -> String {
    return "Hello, " + thing.name
}
```

Works on `Dog`, `Robot`, or anything else with a `name: String` field.

**Extend any type, anywhere.** UFCS lets you add methods to types you don't own:

```rust
fn extends shout(s: String) -> String {
    return s.toUpper() + "!!!"
}
println("hello".shout())   // HELLO!!!
```

---

## Quick Look

```rust
module main
import super.std.list

let xs      = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let squares = [x in xs[-3:] | x * x]   // last 3, squared
println(squares)   // [64, 81, 100]
```

### Slicing and comprehensions

```rust
xs[-3:]                                   // last 3 elements
xs[1:6]                                   // index 1..5
[x in 0.to(20) | x | x % 2 == 0]        // even numbers
[x in [1,2], y in [10,20] | (x, y)]      // cartesian product
```

### Pattern matching

```rust
enum Shape {
    Circle:    Float,
    Rectangle: (Float, Float),
}

fn area(s: Shape) -> Float {
    match s {
        Circle(r)    => { return 3.14159 * r * r }
        Rectangle(d) => { return d[0] * d[1]     }
    }
    return 0.0
}
```

### Pipe and bind

```rust
5 |> add1() |> double()                     // pipe chain
someList.len() ~> n { n * n + n }           // bind operator
[1, 2, 3, 4, 5].filter(): |x: Int| x > 3   // trailing closure
```

### Safe casts, no null

```rust
if let n = Cast::int("42") {
    println("parsed: " + Cast::string(n))
}
let safe = Cast::int("oops").orDefault(0)
```

---

## Get Started

```bash
git clone https://github.com/pyrotek45/nova-lang
cd nova-lang
cargo build --release
./target/release/nova run demo/fib.nv
```

Or on NixOS:

```bash
nix-env -if default.nix
nova run demo/fib.nv
```

### Project workflow

```bash
nova init myapp               # scaffold a new project
nova init myapp --with user/repo/libs   # ...with a GitHub dependency
nova run                      # run main.nv in project mode
nova test                     # run all test_*.nv files
nova install utils pyrotek45/nova-utils/src   # add a library
nova remove utils             # remove it
```

### All commands

```
nova run   <file.nv>          Run a program
nova run                      Run main.nv (project mode)
nova check <file.nv>          Type-check without running
nova time  <file.nv>          Run and show execution time
nova dis   <file.nv>          Disassemble bytecode
nova dbg   <file.nv>          Debug run
nova init  <name>             New project
nova install <name> <repo>    Install library from GitHub
nova remove  <name>           Remove a library
nova test                     Run tests
nova repl                     Interactive REPL
```

Every file command also accepts `--git owner/repo/path.nv` to work directly from GitHub.

---

## Standard Library

| Module | Purpose |
|---|---|
| `std/core` | `Box`, `Gen`, `range`, `Maybe`, `Result`, Option helpers |
| `std/list` | `map`, `filter`, `reduce`, `sortWith`, `flatten`, `zip` |
| `std/iter` | Lazy iterators |
| `std/string` | `split`, `padLeft`, `capitalize`, `lines`, `words` |
| `std/math` | `sqrt`, `pow`, `abs`, trig |
| `std/io` | `prompt`, `readLines`, `writeLines` |
| `std/hashmap` | O(1) hash map |
| `std/grid` | Generic 2D grid for tilemaps and pathfinding |
| `std/plot` | Line, bar, scatter, pie, and fill charts (raylib) |
| `std/tui` | Terminal UI — cursor, colour, input, draw primitives |

Game development: `std/camera`, `std/entity`, `std/physics`, `std/scene`,
`std/tween`, `std/vec2`, `std/noise`, `std/timer`, `std/widget`

---

## Documentation

- [Tutorial](documentation/tutorial.md) — 59 sections from hello world to shipping games
- [Reference](documentation/reference.md) — complete language reference
- [Getting Started](documentation/getting_started.md) — project setup and workflow
- [Installation](documentation/installation.md) — build from source or install via Nix

---

## License

Licensed under the [GNU Affero General Public License v3.0](LICENSE).

