# Nova

![Nova Logo](nova-logo.png)

Nova is a statically typed, expression-oriented programming language compiled to bytecode and
executed by a stack-based virtual machine written in Rust.

**Key features:**

- Explicit, static type system -- no type inference
- Universal Function Call Syntax (UFCS)
- First-class functions and closures
- Generics with `$T` type parameters
- Enums with associated data and pattern matching
- Structural `Dyn` types for duck-typed dispatch
- Pipe operator (`|>`) for function chaining
- Hybrid garbage collector (reference counting + mark-and-sweep)

---

## Getting Started

### Build

Nova requires a stable Rust toolchain. Clone and build:

```bash
git clone https://github.com/pyrotek45/nova-lang
cd nova-lang
cargo build --release
```

The binary is placed at `./target/release/nova`.

On NixOS:

```bash
nix-shell --run "cargo build --release"
```

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

// Structs
struct Person {
    name: String,
    age: Int
}

// UFCS extends
fn extends greet(p: Person) -> String {
    return "Hello, " + p.name + "! You are " + Cast::string(p.age) + " years old."
}

let alice = Person { name: "Alice", age: 30 }
println(alice.greet())

// Enums and match
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

// First-class functions
let nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let evens = nums.filter(|x: Int| x % 2 == 0)
let squared = evens.map(|x: Int| x * x)
println(squared)   // [4, 16, 36, 64, 100]

// Box for shared mutable state
let counter = Box(0)
let inc = fn() -> Int {
    counter.value = counter.value + 1
    return counter.value
}
println(inc())  // 1
println(inc())  // 2
println(inc())  // 3

// Pipe operator
fn double(x: Int) -> Int { return x * 2 }
fn add1(x: Int) -> Int { return x + 1 }

let result = 5 |> add1() |> double()
println(result)   // 12

// Generics
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

// Dyn types (structural dispatch)
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
language reference covering:

- Module system and imports
- Variables, types, and operators
- Control flow (`if`/`elif`/`else`, `for`, `while`, `match`)
- Functions, overloading, and recursion
- UFCS extends and method chaining
- Structs, enums, generics, closures
- Dyn types and structural dispatch
- Box and mutable shared state
- The standard library
- Cast and type conversions
- Iterators and string operations
- Common mistakes and how to fix them

---

## Standard Library

| Module | Description |
|---|---|
| `std/core.nv` | `Box`, `Gen`, `Maybe`, `Result`, `range()`, `Option` helpers |
| `std/list.nv` | `map`, `filter`, `reduce`, `foreach`, `sort`, `flatten`, `concat` |
| `std/iter.nv` | Lazy `Iter` type with `map`, `filter`, `collect` |
| `std/string.nv` | `trim`, `split`, `toLower`, `toUpper`, char operations |
| `std/math.nv` | `sqrt`, `pow`, `abs`, `floor`, `ceil`, `sin`, `cos` |
| `std/io.nv` | `io::prompt`, `io::readFile` |
| `std/hashmap.nv` | `HashMap` |
| `std/tuple.nv` | Tuple utilities |
| `std/tui.nv` | Terminal UI helpers |

---

## Tests

The test suite is in `tests/`. Run it with:

```bash
cargo build --release
bash tests/run_tests.sh
```

Two categories:

- **Positive tests** (`tests/test_*.nv`) -- 63 programs that must compile, run, and print
  `PASS:`. Covers arithmetic, closures, enums, generics, GC, UFCS, Dyn types, iterators,
  higher-order functions, parser/lexer stress, native functions, std library, and more.

- **Rejection tests** (`tests/should_fail/*.nv`) -- 157 programs that the compiler must reject
  with a type or parse error. Covers wrong argument types, wrong return types, undefined
  variables, struct mismatches, missing fields, and other ill-typed programs.

```
  Positive tests: 63 passed, 0 failed
  Rejection tests: 157 passed, 0 failed
  Total: 220 passed, 0 failed

All tests passed!
```

---

## Fuzzing

Nova includes fuzzing targets for the lexer and parser using
[cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz):

```bash
rustup toolchain install nightly
cargo +nightly install cargo-fuzz

./fuzz/run_fuzz.sh lexer 60    # fuzz lexer for 60 seconds
./fuzz/run_fuzz.sh parser 60   # fuzz parser for 60 seconds
./fuzz/run_fuzz.sh all 30      # fuzz all targets
```

Crash inputs are saved to `fuzz/artifacts/`. The fuzzer seeds from real Nova programs in
`fuzz/corpus/`.

---

## Demo Programs

| File | Description |
|---|---|
| `demo/demo.nv` | Feature showcase |
| `demo/fib.nv` | Fibonacci sequence |
| `demo/snake.nv` | Terminal snake game |
| `demo/flappy.nv` | Flappy bird clone |
| `demo/breakout.nv` | Breakout game |
| `demo/forth.nv` | Forth-like interpreter |
| `demo/matmul.nv` | Matrix multiplication |
| `demo/structs.nv` | Struct patterns |
| `demo/vtable.nv` | Dyn vtable dispatch |
| `demo/option_type.nv` | Option / Maybe usage |
| `demo/speedtest.nv` | Performance benchmark |

```bash
./target/release/nova run demo/fib.nv
```

---

## Project Structure

```
nova-lang/
  novacli/       CLI entry point (run, check, dis, time, repl)
  lexer/         Tokenizer
  parser/        Parser and type checker
  typechecker/   Type checking logic
  compiler/      Bytecode compiler
  assembler/     Bytecode assembler
  optimizer/     Optimization passes
  disassembler/  Bytecode disassembler
  vm/            Stack-based virtual machine and GC
  native/        Built-in functions (IO, string, math, regex, raylib)
  common/        Shared types (AST, tokens, errors, type definitions)
  novacore/      Orchestration layer
  std/           Standard library (Nova source)
  demo/          Example programs
  tests/         Test suite (positive + rejection)
  fuzz/          Fuzzing targets and corpus
  documentation/ Language guide and reference
```

---

## License

Licensed under the [GNU Affero General Public License v3.0](LICENSE).
