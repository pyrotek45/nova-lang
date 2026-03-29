# Nova

Nova is a statically typed language that compiles to bytecode and runs on a stack-based VM.
The type system is strict but the syntax is expressive — list comprehensions, slicing, closures,
pattern matching, and structural dispatch without the ceremony you'd expect.

Runtime errors tell you exactly where things went wrong: file, line, and a clear message.
No silent panics, no opaque crashes.

---

```nova
module main
import super.std.list

let xs      = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let squares = [x in xs[-3:] | x * x]   // last 3, squared
println(squares)   // [64, 81, 100]
```

---

## Language Features

### List slicing

Same syntax as Python. Negative indices, stride, the lot:

```nova
xs[-3:]      // last 3 elements
xs[1:6]      // index 1 up to 5
xs[:$2]      // every 2nd element
xs[-4:-1$2]  // stride with negative indices
```

### List comprehensions

```nova
let evens   = [x in 0.to(20) | x | x % 2 == 0]
let squares = [x in [1,2,3,4,5] | x * x]
let pairs   = [x in [1,2], y in [10,20] | (x, y)]
```

### Bind operator `~>`

Name an intermediate value inline without a throwaway variable:

```nova
let result = someList.len() ~> n { n * n + n }
```

### Trailing closures

```nova
let filtered = [1, 2, 3, 4, 5].filter(): |x: Int| x > 3
let mapped   = [1, 2, 3].map():          |x: Int| x * 10
```

### Pipe operator

```nova
fn double(x: Int) -> Int { return x * 2 }
fn add1(x: Int)   -> Int { return x + 1 }

println(5 |> add1() |> double())   // 12
```

### Dyn types — structural dispatch without inheritance

Accept any struct that has the right fields:

```nova
struct Dog   { name: String, breed: String }
struct Robot { name: String, model: Int }

fn greet(thing: Dyn(T = name: String)) -> String {
    return "I am " + thing.name
}

println(greet(Dog   { name: "Rex",  breed: "Husky" }))
println(greet(Robot { name: "R2D2", model: 2       }))
```

### Option type

`Cast::int` returns `Option(Int)`. No null, no exception:

```nova
if let n = Cast::int("42") {
    println("parsed: " + Cast::string(n))
}

let safe = Cast::int("oops").orDefault(0)
```

### Box — shared mutable state across closures

```nova
let counter = Box(0)
let inc = fn() -> Int { counter.value += 1; return counter.value }

println(inc())   // 1
println(inc())   // 2
```

### Pattern matching on enums

```nova
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

### Universal Function Call Syntax

Any function can be called as a method. Extend any type, anywhere:

```nova
fn extends shout(s: String) -> String {
    return s.toUpper() + "!!!"
}

println("hello".shout())   // HELLO!!!
```

---

## Putting It Together

```nova
module main
import super.std.core
import super.std.list

struct Person { name: String, score: Int }

fn extends rank(p: Person) -> String {
    if p.score >= 90 { return "S" }
    if p.score >= 70 { return "A" }
    return "B"
}

let people = [
    Person { name: "Alice", score: 95 },
    Person { name: "Bob",   score: 73 },
    Person { name: "Carol", score: 61 },
]

let top = [p in people | p | p.score >= 70]
for p in top {
    println(p.name + " -> " + p.rank())
}
// Alice -> S
// Bob   -> A
```

---

## Build and Run

```bash
git clone https://github.com/pyrotek45/nova-lang
cd nova-lang
cargo build --release
./target/release/nova run demo/fib.nv
```

```
nova run   <file.nv>   Run a program
nova check <file.nv>   Type-check without running
nova time  <file.nv>   Run and show execution time
nova dis   <file.nv>   Disassemble bytecode
nova repl              Interactive REPL
```

---

## Demo Programs

| File | What it does |
|---|---|
| `demo/fib.nv` | Fibonacci (recursive) |
| `demo/snake.nv` | Terminal snake game |
| `demo/flappy.nv` | Flappy bird in the terminal |
| `demo/forth.nv` | A Forth interpreter written in Nova |
| `demo/matmul.nv` | Matrix multiplication |
| `demo/option_type.nv` | Option and Maybe patterns |
| `demo/speedtest.nv` | Performance benchmark |

---

## Standard Library

| Module | What it gives you |
|---|---|
| `std/core` | `Box`, `Gen`, `range`, `Maybe`, `Result`, Option helpers |
| `std/list` | `map`, `filter`, `reduce`, `sortWith`, `flatten`, `zip` |
| `std/iter` | Lazy iterators — `map`, `filter`, `collect` |
| `std/string` | `split`, `padLeft`, `padRight`, `capitalize`, `lines`, `words` |
| `std/math` | `sqrt`, `pow`, `abs`, `floor`, trig |
| `std/io` | `prompt`, `readLines`, `writeLines` |
| `std/hashmap` | O(1) `HashMap` |
| `std/maybe` | `Maybe(T)` — `Just`/`Nothing` with pattern matching |
| `std/result` | `Result(A,B)` — `Ok`/`Err` for error propagation |
| `std/tui` | `run`, `clear`, `printAt`, `printColor`, `drawBox`, colour presets, input polling |

---

## License

Licensed under the [GNU Affero General Public License v3.0](LICENSE).

