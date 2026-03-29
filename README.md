# Nova

Nova is a statically typed, expression-oriented language compiled to bytecode and run on a
stack-based VM written in Rust. It is designed for people who want the safety of a strict type
system with the expressiveness of functional-style programming — without the boilerplate.

**What makes Nova interesting:**

- **Explicit static types** — no type inference, every binding is declared
- **Universal Function Call Syntax (UFCS)** — `x.f()` calls `f(x)` for any extends function
- **Python-style list slicing** — `xs[-2:]`, `xs[1:8$2]`, negative indices, stride
- **List comprehensions with guards** — `[x in list | x*x | x > 2]`, nested, multi-guard
- **First-class closures** — trailing closure syntax, bind operator (`~>`), `|| expr`
- **Pattern matching** on enums with associated data
- **Dyn types** for structural duck-typed dispatch without inheritance
- **Generics** with `$T` parameters
- **VM-level Option type** — zero-allocation nullable values with `if let` / `while let`
- **Hybrid GC** — reference counting + mark-and-sweep cycle collection
- **Safe runtime errors** — out-of-bounds, None unwraps, and type mismatches all produce
  clean Nova errors with file and line number — no silent Rust panics

---

## Getting Started

### Build

Nova requires a stable Rust toolchain:

```bash
git clone https://github.com/pyrotek45/nova-lang
cd nova-lang
cargo build --release
```

On NixOS:

```bash
nix-shell --run "cargo build --release"
```

The binary is at `./target/release/nova`.

### Run

```
nova run   <file.nv>   Run a Nova program
nova check <file.nv>   Type-check without running
nova dis   <file.nv>   Disassemble bytecode
nova time  <file.nv>   Run and show execution time
nova dbg   <file.nv>   Run with debug output
nova repl              Interactive REPL
```

---

## Hello, World

```nova
module main

println("Hello, World!")
```

---

## A Taste of Nova

```nova
module main

import super.std.core
import super.std.list

// ── Structs and UFCS ─────────────────────────────────────────────────────────

struct Person { name: String, age: Int }

fn extends greet(p: Person) -> String {
    return "Hello, " + p.name + "! Age: " + Cast::string(p.age)
}

let alice = Person { name: "Alice", age: 30 }
println(alice.greet())

// ── Enums and pattern matching ────────────────────────────────────────────────

enum Shape {
    Circle:    Float,
    Rectangle: (Float, Float)
}

fn area(s: Shape) -> Float {
    match s {
        Circle(r)    => { return 3.14159 * r * r }
        Rectangle(d) => { return d[0] * d[1] }
    }
    return 0.0
}

println(area(Shape::Circle(5.0)))
println(area(Shape::Rectangle((4.0, 3.0))))

// ── Python-style list slicing ─────────────────────────────────────────────────

let xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
println(xs[2:5])    // [2, 3, 4]
println(xs[-3:])    // [7, 8, 9]  — last 3
println(xs[:$2])    // [0, 2, 4, 6, 8]  — every 2nd

// ── List comprehensions with guards ──────────────────────────────────────────

let evens   = [x in 0.to(10) | x | x % 2 == 0]
let squares = [x in [1,2,3,4,5] | x * x]
let pairs   = [x in [1,2], y in [10,20] | x + y]   // nested, flat result
println(squares)    // [1, 4, 9, 16, 25]

// ── Option type ───────────────────────────────────────────────────────────────

if let n = Cast::int("42") {
    println("parsed: " + Cast::string(n))
}
let safe = Cast::int("abc").orDefault(0)   // 0

// ── Closures ─────────────────────────────────────────────────────────────────

// trailing closure syntax
let big = [1,2,3,4,5].filter(): |x: Int| x > 3   // [4, 5]

// bind operator: name intermediate values inline
let len_sq = [1,2,3].len() ~> n { n * n }         // 9

// Box for mutable shared state
let counter = Box(0)
let inc = fn() -> Int { counter.value += 1; return counter.value }
println(inc())   // 1
println(inc())   // 2

// ── Pipe operator ─────────────────────────────────────────────────────────────

fn double(x: Int) -> Int { return x * 2 }
fn add1(x: Int)   -> Int { return x + 1 }
println(5 |> add1() |> double())   // 12

// ── Dyn types — structural dispatch ──────────────────────────────────────────

struct Dog   { name: String, breed: String }
struct Robot { name: String, model: Int }

fn introduce(thing: Dyn(T = name: String)) -> String {
    return "I am " + thing.name
}

println(introduce(Dog   { name: "Rex",  breed: "Husky" }))
println(introduce(Robot { name: "R2D2", model: 2 }))
```

---

## Syntax Sugar Highlights

### Python-style slicing

```nova
xs[1:4]      // sublist [1..4)
xs[-2:]      // last 2 elements
xs[:-1]      // all but last
xs[:$3]      // every 3rd element (stride)
xs[-4:-1$2]  // stride with negative indices
```

### List comprehensions

```nova
[x in list | expr]                    // transform
[x in list | expr | guard]            // filter
[x in list | expr | g1 | g2]         // multiple guards (AND)
[x in l1, y in l2 | x + y]           // nested (flat result)
```

### Safe Option handling

```nova
if let val = opt { ... }              // bind if Some
while let val = gen() { ... }         // loop while Some
opt.orDefault(fallback)               // unwrap or default
opt.orError("message")                // exit with message if None
```

### Closure ergonomics

```nova
fn(x): |v| v + 1                     // trailing closure
expr ~> name { use name }            // bind operator
|| expr                               // zero-parameter closure
fn_name@(Int)                         // select overload by type
```

See [documentation/how_to_write_nova.md §27](documentation/how_to_write_nova.md) for the
complete syntax sugar reference.

---

## Standard Library

| Module | Import | Provides |
|---|---|---|
| `std/core.nv` | `import super.std.core` | `Box`, `Gen`, `range()`, `Maybe`, `Result`, Option helpers |
| `std/option.nv` | `import super.std.option` | Standalone Option extensions |
| `std/maybe.nv` | `import super.std.maybe` | `Maybe(T)` enum (`Just`/`Nothing`) |
| `std/result.nv` | `import super.std.result` | `Result(A,B)` enum (`Ok`/`Err`) |
| `std/list.nv` | `import super.std.list` | `map`, `filter`, `reduce`, `sort`, `flatten`, … |
| `std/iter.nv` | `import super.std.iter` | Lazy `Iter` with `map`, `filter`, `collect` |
| `std/string.nv` | `import super.std.string` | `trim`, `split`, `toLower`, `toUpper` |
| `std/math.nv` | `import super.std.math` | `sqrt`, `pow`, `abs`, `floor`, trig |
| `std/io.nv` | `import super.std.io` | `prompt`, `readLines`, `writeLines` |
| `std/hashmap.nv` | `import super.std.hashmap` | O(1) `HashMap` |
| `std/tuple.nv` | `import super.std.tuple` | `swap`, `mapFirst`, `mapSecond` |
| `std/tui.nv` | `import super.std.tui` | Terminal UI toolkit |

---

## Option vs Maybe

Nova has two nullable concepts with different trade-offs:

| | `Option(T)` | `Maybe(T)` |
|---|---|---|
| Kind | **VM primitive** | User-defined enum |
| Allocation | Zero | Heap object |
| Construction | `Some(42)` / `None(Int)` | `Maybe::Just(42)` / `Maybe::Nothing()` |
| Unwrapping | `if let` / `.isSome()` | `match` |
| Best for | Return values, function params | Lists of nullables, full enum match |

```nova
import super.std.core

// Option — fast, zero allocation
if let n = Cast::int("42") { println(n) }

// Maybe — full enum, pattern matchable
let m = Maybe::Just(42)
match m {
    Just(x)   => { println(x) }
    Nothing() => { println("empty") }
}
```

---

## Tests

```bash
cargo build --release
bash tests/run_tests.sh
```

- **Positive tests** — 75 programs that must compile, run, and print `PASS:`.
- **Rejection tests** — 157 programs that must be rejected at compile time.

```
  Positive tests:   75 passed, 0 failed
  Rejection tests: 157 passed, 0 failed
  Total:           232 passed, 0 failed

All tests passed!
```

---

## Fuzzing

```bash
rustup toolchain install nightly
cargo +nightly install cargo-fuzz
./fuzz/run_fuzz.sh lexer  60
./fuzz/run_fuzz.sh parser 60
```

---

## Demo Programs

| File | Description |
|---|---|
| `demo/fib.nv` | Fibonacci sequence |
| `demo/snake.nv` | Terminal snake game |
| `demo/flappy.nv` | Flappy bird clone |
| `demo/forth.nv` | Forth-like interpreter |
| `demo/matmul.nv` | Matrix multiplication |
| `demo/option_type.nv` | Option / Maybe usage |
| `demo/speedtest.nv` | Performance benchmark |

```bash
./target/release/nova run demo/fib.nv
```

---

## Project Structure

```
nova-lang/
  novacli/       CLI (run, check, dis, time, repl)
  lexer/         Tokenizer
  parser/        Parser and type checker
  compiler/      Bytecode compiler
  assembler/     Bytecode assembler
  vm/            Stack-based VM + hybrid GC
  native/        Built-in functions (IO, string, math, regex, terminal, raylib)
  common/        Shared types (AST, tokens, errors)
  novacore/      Orchestration layer
  std/           Standard library (Nova source)
  demo/          Example programs
  tests/         Test suite
  fuzz/          Fuzzing targets and corpus
  documentation/ Language guide and reference
```

---

## License

Licensed under the [GNU Affero General Public License v3.0](LICENSE).

