# Nova

![Nova Logo](nova-logo.png)

Nova is a statically typed, expression-oriented programming language with:

- **No type inference** — types are explicit and checked at compile time
- **Universal Function Call Syntax (UFCS)** — functions feel like methods
- **First-class functions and closures** — functions are values
- **Hybrid garbage collector** — reference counting + mark-and-sweep
- **Structural Dyn types** — duck-typed dispatch without inheritance
- **Generics** — type-parameterized structs and functions

Nova is compiled to bytecode and run by a stack-based virtual machine written in Rust.

---

## Installation

Nova requires Rust (stable). Clone and build:

```bash
git clone https://github.com/pyrotek45/nova-lang
cd nova-lang
cargo build --release
```

The binary will be at `./target/release/nova`.

On NixOS, use:

```bash
nix-shell --run "cargo build --release"
```

---

## Usage

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

// --- Structs ---
struct Person {
    name: String,
    age: Int
}

// --- Extends functions (UFCS) ---
fn extends greet(p: Person) -> String {
    return "Hello, " + p.name + "! You are " + Cast::string(p.age) + " years old."
}

let alice = Person { name: "Alice", age: 30 }
println(alice.greet())

// --- Enums and match ---
enum Shape {
    Circle: Float,
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

// --- First-class functions ---
let nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let evens = nums.filter(|x: Int| x % 2 == 0)
let squared = evens.map(|x: Int| x * x)
println(squared)   // [4, 16, 36, 64, 100]

// --- Box for shared mutable state ---
let counter = Box(0)
let inc = fn() -> Int {
    counter.value = counter.value + 1
    return counter.value
}
println(inc())  // 1
println(inc())  // 2
println(inc())  // 3

// --- Pipe operator ---
fn double(x: Int) -> Int { return x * 2 }
fn add1(x: Int) -> Int { return x + 1 }

let result = 5 |> add1() |> double()
println(result)   // 12

// --- Generics ---
struct Pair(A, B) {
    fst: $A,
    snd: $B
}

fn extends swap(p: Pair($A, $B)) -> Pair($B, $A) {
    return Pair { fst: p.snd, snd: p.fst }
}

let p = Pair { fst: 42, snd: "hello" }
let q = p.swap()
println(q.fst)   // "hello"
println(q.snd)   // 42

// --- Dyn types (structural dispatch) ---
type named = Dyn(T = name: String)

struct Dog { name: String, breed: String }
struct Robot { name: String, model: Int }

fn introduce(thing: Dyn(T = name: String)) -> String {
    return "My name is " + thing.name
}

let dog = Dog { name: "Rex", breed: "Husky" }
let bot = Robot { name: "R2D2", model: 2 }
println(introduce(dog))
println(introduce(bot))
```

---

## Language Guide

See [documentation/how_to_write_nova.md](documentation/how_to_write_nova.md) for the full
language reference, including:

- Module system and imports
- Type system rules (what the compiler rejects)
- Every operator and its precedence quirks
- Structs, enums, generics, closures, Dyn types
- The standard library (`std/core`, `std/list`, `std/iter`, `std/string`, etc.)
- Box and mutable shared state patterns
- The garbage collector and its guarantees
- Common mistakes and how to fix them

---

## Standard Library

| Module | Contents |
|--------|----------|
| `std/core.nv` | `Box`, `Gen`, `Maybe`, `Result`, `range()`, `Option` helpers |
| `std/list.nv` | `map`, `filter`, `reduce`, `foreach`, `sort`, `flatten`, `concat`, ... |
| `std/iter.nv` | Lazy `Iter` type with `map`, `filter`, `collect` |
| `std/string.nv` | String/Char operations: trim, split, toLower, toUpper, ... |
| `std/math.nv` | `sqrt`, `pow`, `abs`, `floor`, `ceil`, `sin`, `cos`, ... |
| `std/io.nv` | `io::prompt`, `io::readFile` |
| `std/hashmap.nv` | `HashMap` |
| `std/tui.nv` | Terminal UI helpers |

---

## Running Tests

The test suite lives in `tests/`. Run it with:

```bash
cargo build --release
bash tests/run_tests.sh
```

The suite has two parts:

1. **Positive tests** (`tests/test_*.nv`) — programs that must compile, run, and print `PASS:`.
   Currently **44 tests** covering: arithmetic, closures, enums, generics, GC stress, UFCS,
   parser stress, lexer stress, Dyn types, iterators, higher-order functions, and more.

2. **Type-rejection tests** (`tests/should_fail/*.nv`) — programs that the compiler **must
   reject**. Currently **20 tests** verifying that ill-typed programs produce compile errors:
   wrong argument types, wrong return types, undefined variables, struct field type mismatches,
   Int/Float confusion, missing struct fields, and more.

Expected output when all tests pass:

```
  Positive tests: 44 passed, 0 failed
  Rejection tests: 20 passed, 0 failed
  Total: 64 passed, 0 failed

All tests passed!
```

---

## Fuzzing

Nova includes a fuzzing infrastructure to find panics in the lexer and parser. It uses
`cargo-fuzz` with libFuzzer:

```bash
# Install cargo-fuzz (requires nightly Rust)
rustup toolchain install nightly
cargo +nightly install cargo-fuzz

# Fuzz the lexer for 60 seconds
./fuzz/run_fuzz.sh lexer 60

# Fuzz the parser for 60 seconds
./fuzz/run_fuzz.sh parser 60

# Fuzz all targets
./fuzz/run_fuzz.sh all 30
```

The fuzzer seeds from real Nova programs in `fuzz/corpus/`. Any panics are saved to
`fuzz/artifacts/` and represent bugs to fix.

---

## Demo Programs

The `demo/` folder contains example Nova programs:

| File | Description |
|------|-------------|
| `demo.nv` | Kitchen-sink feature showcase |
| `fib.nv` | Fibonacci sequence |
| `snake.nv` | Terminal snake game |
| `forth.nv` | Forth-like interpreter |
| `matmul.nv` | Matrix multiplication |
| `structs.nv` | Struct patterns |
| `option_type.nv` | Option / Maybe usage |

Run any demo:

```bash
./target/release/nova run demo/fib.nv
```

---

## Project Structure

```
nova-lang/
  novacli/       CLI entry point (run, check, dis, time, repl)
  lexer/         Tokenizer
  parser/        Parser + type checker
  compiler/      Bytecode compiler
  assembler/     Bytecode assembler
  optimizer/     Optimization passes
  vm/            Stack-based virtual machine + GC
  native/        Built-in functions (IO, string, math, regex, ...)
  common/        Shared types (AST nodes, tokens, errors, types)
  novacore/      Orchestration layer
  std/           Standard library (Nova source)
  demo/          Demo programs
  tests/         Test suite (positive + type-rejection)
  fuzz/          Fuzzing targets
  documentation/ Language docs and guide
```

---

## License

See [LICENSE](LICENSE).
