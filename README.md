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

> **Tip:** Copy the binary somewhere on your `PATH` so you can run `nova`
> from any directory:
>
> ```bash
> sudo cp target/release/nova /usr/local/bin/
> # or, without sudo:
> cp target/release/nova ~/.local/bin/
> ```

Or on NixOS:

```bash
nix-env -if default.nix
nova run demo/fib.nv
```

### Project workflow

```bash
nova init myapp --with pyrotek45/nova-lang/std   # new project with stdlib
cd myapp
nova run                      # run main.nv in project mode
nova test                     # run all test_*.nv files
```

The `--with` flag fetches `.nv` files from a GitHub path into your
project's `libs/` folder — that's all you need to get started.

### Managing extra libraries

For adding or removing libraries in an existing project, use `install` and
`remove`.  The difference from `--with` is that you give the library a name
so you can remove it later:

```bash
nova install utils maniospas/nova-helpers/src   # → libs/utils/
nova remove  utils                               # deletes libs/utils/
```

Then import as usual:

```rust
import libs.utils.strings     // → libs/utils/strings.nv
```

### Try a game — no install needed

You can run files directly from GitHub without downloading anything:

```bash
nova run --git pyrotek45/nova-lang/games/Breakout/breakout.nv
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
nova init  <name> --with <repo/path>   New project with GitHub deps
nova install <name> <repo>    Install library from GitHub
nova remove  <name>           Remove a library
nova test                     Run tests
nova repl                     Interactive REPL
```

Every file command also accepts `--git owner/repo/path.nv` to work directly from GitHub.

---

## Documentation

- [Tutorial](documentation/tutorial.md) — 59 sections from hello world to shipping games
- [Reference](documentation/reference.md) — complete language reference
- [Getting Started](documentation/getting_started.md) — project setup and workflow
- [Installation](documentation/installation.md) — build from source or install via Nix

---

## License

Licensed under the [MIT License](LICENSE).

