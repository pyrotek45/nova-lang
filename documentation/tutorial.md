# Nova Tutorial

A comprehensive guide to writing Nova — from first program to full games.

---

## Table of Contents

### Part I — The Language

1. [Hello World](#1-hello-world)
2. [Module System](#2-module-system)
3. [Variables and Types](#3-variables-and-types)
4. [Built-in Types](#4-built-in-types)
5. [Operators](#5-operators)
6. [Control Flow](#6-control-flow)
7. [Functions](#7-functions)
8. [Extends and UFCS](#8-extends-and-ufcs)
9. [Structs](#9-structs)
10. [Enums](#10-enums)
11. [Generics](#11-generics)
12. [Option Type](#12-option-type)
13. [Closures and Lambdas](#13-closures-and-lambdas)
14. [Lists](#14-lists)
15. [Tuples](#15-tuples)
16. [Pattern Matching](#16-pattern-matching)
17. [Pipe Operator](#17-pipe-operator)
18. [Dyn Types](#18-dyn-types)
19. [Box and Mutable Shared State](#19-box-and-mutable-shared-state)
20. [Imports and the Standard Library](#20-imports-and-the-standard-library)
21. [Type System Rules](#21-type-system-rules)
22. [Memory Model](#22-memory-model)
23. [Cast and Type Conversion](#23-cast-and-type-conversion)
24. [Iterators](#24-iterators)
25. [String Operations](#25-string-operations)
26. [Design Patterns](#26-design-patterns)
27. [Syntax Sugar Reference](#27-syntax-sugar-reference)
28. [Tips and Tricks](#28-tips-and-tricks)
29. [Common Mistakes](#29-common-mistakes)

### Part II — Game Development

30. [Quick Start — Your First Window](#30-quick-start--your-first-window)
31. [Critical Rules for Game Dev](#31-critical-rules-for-game-dev)
32. [Scene Management](#32-scene-management)
33. [Entity System](#33-entity-system)
34. [Input Handling](#34-input-handling)
35. [Physics and Collision](#35-physics-and-collision)
36. [Camera](#36-camera)
37. [Timers and Tweens](#37-timers-and-tweens)
38. [Vec2 Math](#38-vec2-math)
39. [Tilemaps and Noise](#39-tilemaps-and-noise)
40. [Sprites and Audio](#40-sprites-and-audio)
41. [HUD and UI](#41-hud-and-ui)
42. [Advanced Game Patterns](#42-advanced-game-patterns)
43. [Game Dev Tips and Tricks](#43-game-dev-tips-and-tricks)
44. [Performance Tips](#44-performance-tips)
45. [Complete Example — Breakout](#45-complete-example--breakout)
46. [Complete Example — Top-Down Shooter](#46-complete-example--top-down-shooter)

### Part III — Terminal Applications

47. [Terminal Quick Start](#47-terminal-quick-start)
48. [Raw Mode and Key Input](#48-raw-mode-and-key-input)
49. [Colours and Cursor](#49-colours-and-cursor)
50. [Terminal Menu System](#50-terminal-menu-system)
51. [Terminal Game Loop](#51-terminal-game-loop)
52. [Terminal Patterns](#52-terminal-patterns)

### Part IV — For Python Developers

53. [Nova for Python Developers](#53-nova-for-python-developers)

---

# Part I — The Language

---

## 1. Hello World

```nova
println("Hello, world!")
```

Run with `nova run hello.nv`.

---

## 2. Module System

Every Nova source file starts with a `module` declaration:

```nova
module my_program
```

The module name registers the file so the parser can deduplicate imports.
Files can import each other:

```nova
import super.std.core      // parent-relative import
import sibling             // same-directory import
```

### The `::` Operator — Three Uses

| Context | Meaning | Example |
|---|---|---|
| Module function | Call a function defined in a module namespace | `Cast::string(42)` |
| Enum variant | Construct a variant of an enum | `Color::Red()` |
| Struct function field | Call a function stored as a struct field (no self) | `handler::process("data")` |

The `::` operator is Nova's universal namespace separator. It always means "reach into
this namespace and call/access something."

### Design Pattern: Structs as Namespaces

Because `::` works on structs, you can group related static functions:

```nova
struct Math {}

fn extends(Math) pi() -> Float { return 3.14159 }
fn extends(Math) tau() -> Float { return 6.28318 }

Math::pi()   // 3.14159
```

### Type Hints with `@[]`

When the compiler needs help resolving a generic, annotate with `@[]`:

```nova
let empty = HashMap::default() @[K: String, V: Int]
```

---

## 3. Variables and Types

```nova
let x = 42              // Int
let name = "Nova"       // String
let pi = 3.14           // Float
let yes = true          // Bool
let ch = 'A'            // Char
```

Variables are declared with `let`. The type is determined at declaration. Nova has
**no type inference from usage** — every variable's type is fixed at the point it's created.

### Mutation

Variables can be reassigned, but only to values of the **same type**:

```nova
let x = 10
x = 20      // OK — same type
x = "hello" // ERROR — cannot change type
```

---

## 4. Built-in Types

| Type | Example | Description |
|---|---|---|
| `Int` | `42`, `-7`, `0` | 64-bit signed integer |
| `Float` | `3.14`, `0.0`, `-1.5` | 64-bit floating-point |
| `Bool` | `true`, `false` | Boolean |
| `String` | `"hello"` | UTF-8 text |
| `Char` | `'A'`, `'\n'` | Single Unicode character |
| `Void` | — | No return value |
| `Option(T)` | `Some(5)`, `None(Int)` | Optional value |
| `[T]` | `[1, 2, 3]` | List of T |
| `(A, B)` | `(42, "hi")` | Tuple |

---

## 5. Operators

### Arithmetic

| Operator | Meaning |
|---|---|
| `+` `-` `*` `/` | Add, subtract, multiply, divide |
| `%` | Modulo (Euclidean — always non-negative) |

**Note:** `-10 % 3 == 2` (not -1). Nova uses Euclidean modulo.

### Comparison

| Operator | Meaning |
|---|---|
| `==` `!=` | Equal, not equal |
| `<` `>` `<=` `>=` | Ordering |

### Boolean

| Operator | Meaning |
|---|---|
| `&&` | Logical AND |
| `\|\|` | Logical OR |
| `!` | Logical NOT |

> **Precedence warning:** `||` binds tighter than `&&` in Nova. Use parentheses:
> `(a || b) && c`.

### Compound Assignment

`+=`, `-=`, `/=` are available. There is **no `*=`** — write `x = x * 2`.

### Unary Minus

`-x` negates a value.

---

## 6. Control Flow

### if / elif / else

```nova
if x > 10 {
    println("big")
} elif x > 5 {
    println("medium")
} else {
    println("small")
}
```

> **Important:** Nova uses `elif`, NOT `else if`. `else if` is a syntax error.

### if as Expression

```nova
let msg = if x > 0 { "positive" } else { "non-positive" }
```

Expression `if` only supports a single `if/else` pair — no `elif` chaining.

### for Loop

```nova
// C-style
for let i = 0; i < 10; i += 1 {
    println(i)
}

// Range (exclusive)
for i in 0..10 { println(i) }     // 0, 1, ..., 9

// Range (inclusive)
for i in 0..=10 { println(i) }    // 0, 1, ..., 10

// for-in over a list
for item in myList { println(item) }
```

### while Loop

```nova
while condition {
    // body
}
```

### if let / while let — Safe Option Unwrapping

```nova
if let value = someOption {
    println(value)      // runs only if Some
} else {
    println("was None")
}

while let item = generator() {
    println(item)       // loops until None
}
```

---

## 7. Functions

### Basic Functions

```nova
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn greet(name: String) {
    println("Hello, " + name)
}
```

Every function returning a value **must** have an explicit `return`. Nova has no implicit returns.

### Void Functions

Functions without `->` return `Void`:

```nova
fn log(msg: String) {
    println("[LOG] " + msg)
}
```

### Overloading

Functions can be overloaded by parameter types:

```nova
fn describe(x: Int) -> String { return "int: " + Cast::string(x) }
fn describe(x: String) -> String { return "str: " + x }
```

### Function References with `@`

Use `@` to get a reference to a function. When overloaded, specify the type:

```nova
let f = describe@(Int)     // fn(Int) -> String
```

### Recursion

```nova
fn factorial(n: Int) -> Int {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}
```

### Forward Declarations

Declare a function's signature (no body, no braces) to use it before its definition:

```nova
fn isEven(n: Int) -> Bool     // forward declaration

fn isOdd(n: Int) -> Bool {
    if n == 0 { return false }
    return isEven(n - 1)
}

fn isEven(n: Int) -> Bool {
    if n == 0 { return true }
    return isOdd(n - 1)
}
```

---

## 8. Extends and UFCS

`fn extends` adds methods to any type using Universal Function Call Syntax.
The first parameter becomes the receiver:

```nova
fn extends double(x: Int) -> Int {
    return x * 2
}

5.double()   // 10
```

### Chaining

```nova
fn extends add(x: Int, n: Int) -> Int { return x + n }

5.double().add(3)   // 13
```

### Extends on Structs

```nova
fn extends area(r: Rect) -> Int {
    return r.w * r.h
}

myRect.area()
```

### The `->` Dispatch Operator

When a struct has a function field, `->` calls it passing the struct as the first argument:

```nova
struct Handler { process: fn(Handler, String) -> String }

let h = Handler { process: fn(self: Handler, msg: String) -> String {
    return "handled: " + msg
}}

h->process("hello")   // "handled: hello"
```

---

## 9. Structs

### Basic Struct

```nova
struct Point { x: Int, y: Int }

let p = Point { x: 10, y: 20 }
println(p.x)   // 10
```

### Positional Construction

```nova
struct Pair { a: Int, b: Int }
let p = Pair(1, 2)    // same as Pair { a: 1, b: 2 }
```

### Generic Structs

```nova
struct Box(T) { value: $T }

let b = Box { value: 42 }            // Box(Int)
let s = Box { value: "hello" }       // Box(String)
```

### Function Fields

Structs can store closures:

```nova
struct Button {
    label: String,
    onClick: fn(),
}

let b = Button { label: "Go", onClick: fn() { println("clicked!") } }
b::onClick()    // :: calls without passing self
b->onClick()    // -> calls with b as first argument
```

### The `type` Field

Every struct has a built-in `type` field that returns its type name:

```nova
println(p.type)   // "Point"
```

### Operator Overloading

Define dunder methods via `extends`:

```nova
fn extends __add__(a: Point, b: Point) -> Point {
    return Point { x: a.x + b.x, y: a.y + b.y }
}

fn extends __eq__(a: Point, b: Point) -> Bool {
    return a.x == b.x && a.y == b.y
}
```

Available dunders: `__add__`, `__sub__`, `__mul__`, `__div__`, `__mod__`,
`__eq__`, `__neq__`, `__lt__`, `__gt__`, `__lte__`, `__gte__`, `__neg__`,
`__index__`, `__setindex__`, `__display__`.

---

## 10. Enums

Enums define tagged unions. Variants can carry data:

```nova
enum Shape {
    Circle: Float,
    Rectangle: (Float, Float),
    Point,
}

let c = Shape::Circle(5.0)
let r = Shape::Rectangle((10.0, 20.0))
let p = Shape::Point()               // no-data variants need ()
```

### Pattern Matching on Enums

```nova
match shape {
    Circle(radius) => { println("circle r=" + Cast::string(radius)) }
    Rectangle(dims) => { println("rect " + Cast::string(dims[0])) }
    Point() => { println("point") }
}
```

> Match arms use variant names without the enum prefix. Each variant name must be
> unique across all enums in scope.

### Generic Enums

```nova
enum Maybe(A) { Just: $A, Nothing }

let x = Maybe::Just(42)
let n = Maybe::Nothing() @[A: Int]   // type annotation for no-data generic variant
```

---

## 11. Generics

Generic type parameters use `$T` syntax:

```nova
struct Wrapper(T) { inner: $T }

fn identity(x: $T) -> $T { return x }
```

### Generic Extends

```nova
fn extends(Wrapper) unwrap(self: Wrapper($T)) -> $T {
    return self.inner
}
```

### Constraints

Generics are structurally typed — any type that supports the operations used in the
body will work.

---

## 12. Option Type

`Option(T)` represents a value that may or may not exist:

```nova
let found: Option(Int) = Some(42)
let missing: Option(Int) = None(Int)
```

| Method | Description |
|---|---|
| `.isSome() -> Bool` | True if value present |
| `.unwrap() -> T` | Extract value (runtime error if None) |

### Safe Unwrapping with `if let`

```nova
if let value = found {
    println(value)
} else {
    println("not found")
}
```

### Standard Library Helpers

```nova
import super.std.core

let x: Option(Int) = None(Int)
x.orDefault(0)          // 0
x.orDoFn(|| 99)         // 99 (lazy)
x.orError("Expected")   // exits with message if None
x.isNone()              // true
```

### Option vs Maybe

| | `Option(T)` | `Maybe(T)` |
|---|---|---|
| **Kind** | Built-in VM primitive | User-defined enum (std/core) |
| **Construction** | `Some(42)` / `None(Int)` | `Maybe::Just(42)` / `Maybe::Nothing()` |
| **Pattern matching** | `if let` / `while let` | `match` on `Just`/`Nothing` |
| **Performance** | Zero allocation | Heap-allocated enum |
| **Use case** | Return values, parameters | Pattern matching, generic code |

**Prefer `Option(T)`** for return values. Use `Maybe(T)` when you need `match`.

---

## 13. Closures and Lambdas

### Full Syntax

```nova
let add = fn(x: Int, y: Int) -> Int {
    return x + y
}
```

### Short Lambda

```nova
let square = |x: Int| x * x
```

### Empty Closures

```nova
let greet = || println("hello")
```

### Closures as Arguments

```nova
let doubled = [1, 2, 3].map(|x: Int| x * 2)     // [2, 4, 6]
let evens = [1, 2, 3, 4].filter(|x: Int| x % 2 == 0)  // [2, 4]
```

### Trailing Closures

When the last argument is a closure, write it after `:`:

```nova
let big = [1, 2, 3, 4, 5].filter(): |x: Int| x > 3   // [4, 5]
```

### The Bind Operator (~>)

Name an intermediate result inline:

```nova
let len_sq = [1, 2, 3, 4, 5].len() ~> n { n * n }   // 25
```

### Capturing State

Closures capture scalars **by value** and heap objects **by reference**.
Use `Box(T)` to share mutable scalar state:

```nova
import super.std.core
let counter = Box(0)
let inc = fn() { counter.value += 1 }
inc(); inc(); inc()   // counter.value == 3
```

---

## 14. Lists

### Construction

```nova
let xs = [1, 2, 3]            // [Int]
let empty = []: Int            // empty list — type annotation required
```

### Operations

```nova
xs.push(4)       // append
xs.pop()         // remove last → Option
xs.len()         // length
xs[0]            // index (0-based)
xs[0] = 99       // assignment
```

### Slicing

```nova
let xs = [10, 20, 30, 40, 50]

xs[1:3]      // [20, 30]
xs[2:]       // [30, 40, 50]
xs[:3]       // [10, 20, 30]
xs[:]        // full copy
xs[-2:]      // [40, 50]       — negative indices
xs[:-1]      // [10, 20, 30, 40]
xs[0:8$2]    // every 2nd element (step with $)
```

> **Note:** `xs[-1]` (negative regular index) is a runtime error. Use slicing or
> `xs[xs.len() - 1]`.

### List Comprehensions

```nova
let squares = [x in [1, 2, 3, 4, 5] | x * x]           // [1, 4, 9, 16, 25]
let evens = [x in [1,2,3,4,5,6] | x | x % 2 == 0]      // [2, 4, 6]
let sums = [x in [1,2], y in [10,20] | x + y]            // [11, 21, 12, 22]
```

Guards are separated by `|` and combined with AND:

```nova
import super.std.core
let filtered = [x in 1.to(21) | x | x % 2 == 0 | x > 5 | x < 15]
// [6, 8, 10, 12, 14]
```

### Standard Library Functions

```nova
import super.std.list

[1,2,3].map(|x: Int| x * 2)                               // [2, 4, 6]
[1,2,3,4].filter(|x: Int| x > 2)                          // [3, 4]
[1,2,3].reduce(|acc: Int, x: Int, i: Int| acc + x, 0)     // 6
[1,2,3].foreach(|x: Int| { println(x) })
[[1,2],[3,4]].flatten()                                     // [1, 2, 3, 4]
[3,1,2].bubblesort()                                        // [1, 2, 3]
[0]: Int.fill(0, 5)                                         // [0, 0, 0, 0, 0]
```

---

## 15. Tuples

Fixed-size typed collections:

```nova
let t = (42, "hello", true)
t[0]   // 42
t[1]   // "hello"
```

### Single-Element Tuples

Use a trailing comma:

```nova
let single = (42,)   // tuple
let grouped = (42)    // just the Int 42
```

### Functions Returning Tuples

```nova
fn divmod(a: Int, b: Int) -> (Int, Int) {
    return (a / b, a % b)
}
```

---

## 16. Pattern Matching

`match` works on enum values:

```nova
match color {
    Red()   => { return "red" }
    Green() => { return "green" }
    Blue()  => { return "blue" }
}
```

With data extraction:

```nova
enum Tree(T) { Leaf: $T, Node: (Tree($T), Tree($T)) }

fn depth(t: Tree(Int)) -> Int {
    match t {
        Leaf(x)  => { return 1 }
        Node(ch) => {
            let l = depth(ch[0])
            let r = depth(ch[1])
            if l > r { return l + 1 }
            return r + 1
        }
    }
    return 0
}
```

---

## 17. Pipe Operator

`|>` passes the left value as the first argument:

```nova
fn square(x: Int) -> Int { return x * x }
fn inc(x: Int) -> Int { return x + 1 }

let r = 4 |> inc() |> square()   // 25
```

> Extends functions cannot be used with `|>`. Use UFCS chaining instead.

---

## 18. Dyn Types

Dyn types provide structural, duck-typed dispatch:

```nova
fn get_name(thing: Dyn(T = name: String)) -> String {
    return thing.name
}

struct Dog { name: String, age: Int }
struct Robot { name: String, model: Int }

get_name(Dog { name: "Rex", age: 5 })     // "Rex"
get_name(Robot { name: "R2D2", model: 2 }) // "R2D2"
```

### Type Aliases

```nova
type named = Dyn(T = name: String)
type renderable = Dyn(T = render: fn($T) -> String + label: String)
```

### Multi-Field Dyn

```nova
fn full_info(x: Dyn(T = name: String + age: Int)) -> String {
    return x.name + " (age " + Cast::string(x.age) + ")"
}
```

### Dyn with Function Fields (`->` dispatch)

```nova
type drawable = Dyn(T = draw: fn($T))

fn drawAll(items: [drawable]) {
    for item in items { item->draw() }
}
```

---

## 19. Box and Mutable Shared State

`Box(T)` wraps a value on the heap. Multiple closures can share and mutate it:

```nova
import super.std.core

let shared = Box(0)
let inc = fn() { shared.value += 1 }
let get = fn() -> Int { return shared.value }

inc(); inc(); inc()
get()   // 3
```

---

## 20. Imports and the Standard Library

```nova
import super.std.core       // Box, Gen, Maybe, Result, range(), .orDefault()
import super.std.list       // map, filter, reduce, sort, flatten, ...
import super.std.iter       // Iter type: fromVec, map, filter, collect
import super.std.string     // string manipulation extensions
import super.std.math       // mathematical functions
import super.std.io         // prompt(), readLines(), writeLines()
import super.std.hashmap    // HashMap
import super.std.tuple      // tuple utilities
import super.std.color      // named RGB color tuples
import super.std.tui        // terminal UI helpers
import super.std.widget     // raylib GUI widgets
import super.std.option     // Option extensions
import super.std.maybe      // Maybe(T) enum
import super.std.result     // Result(A,B) enum
```

All imports flatten into the caller's scope — call by name, not with a prefix.

---

## 21. Type System Rules

Nova's type system is strict and static. These are compile-time errors:

- Passing a value of the wrong type
- Returning the wrong type
- Reassigning to a different type
- Using undeclared variables or nonexistent functions
- Wrong number of arguments
- Accessing nonexistent struct fields
- Missing or wrong-typed struct fields
- Pushing wrong type into a list
- Using `Option(T)` where `T` is expected (must unwrap)
- Mixing `Int` and `Float` without cast
- Duplicate enum/struct names
- Identical function signatures
- Missing `return` on any branch
- Redefining a variable in the same scope
- Reusing loop variable names in nested loops

---

## 22. Memory Model

### Value Types vs Reference Types

**Value types** (stack, copied on assignment): `Int`, `Float`, `Bool`, `Char`.

**Reference types** (heap, aliased on assignment): `[T]`, `String`, Struct, Tuple,
Enum (with data), Closure.

### Aliasing

```nova
let a = [1, 2, 3]
let b = a           // alias — same object
b.push(4)
println(a.len())    // 4 — visible through a!
```

### `clone()` — Deep Copy

```nova
let copy = clone(original)
copy.push(4)
println(original.len())   // unchanged
```

Use `clone()` when you need an independent snapshot.

### Reference Counting + Mark-and-Sweep

- Short-lived objects freed immediately (ref-count → 0)
- Cyclic structures collected periodically
- No manual memory management needed

### Loop Variable Capture

Closures capture the variable binding. In loops, rebind to freeze the value:

```nova
for let i = 0; i < 5; i += 1 {
    let captured = i
    fns.push(fn() -> Int { return captured })
}
```

---

## 23. Cast and Type Conversion

```nova
Cast::string(42)        // "42"
Cast::int("42")         // Some(42)
Cast::float(42)         // Some(42.0)
Cast::int("abc")        // None(Int)
```

Always handle the `Option` — use `.unwrap()`, `.orDefault()`, or `if let`.

---

## 24. Iterators

```nova
import super.std.iter

let result = Iter::fromVec([1, 2, 3, 4, 5])
    .map(|x: Int| x * x)
    .filter(|x: Int| x > 5)
    .collect()
// [9, 16, 25]
```

---

## 25. String Operations

```nova
import super.std.string

"hello".len()                  // 5
"hello".chars()                // [Char]
"  hello  ".trim()             // "hello"
"Hello".toLower()              // "hello"
"hello world".split(' ')       // ["hello", "world"]
```

String concatenation uses `+` (String + String only):

```nova
let msg = "Count: " + Cast::string(42)
```

Use `format` for placeholders:

```nova
let msg = format("Hello, {}! Age: {}", [name, Cast::string(age)])
```

---

## 26. Design Patterns

Nova has no classes or inheritance. Instead:

| Pattern | Nova Approach |
|---|---|
| Strategy | Pass different functions / closures |
| Observer | List of callbacks |
| Factory | Centralized constructor function |
| Builder | Chained extends methods returning self |
| Decorator | Function wrapping |
| State Machine | Enum + match |
| Command | Enum variants as operations |
| Composite | Recursive enums |
| Prototype | `clone()` + modify |
| Adapter | Dyn types for structural polymorphism |
| Singleton | Closure-captured Box |
| Template Method | Pluggable function parameters |

**Key insight:** Nova replaces OOP's virtual dispatch with closures, enums + match,
Dyn types, and extends + UFCS.

---

## 27. Syntax Sugar Reference

| Sugar | Example | What It Does |
|---|---|---|
| Exclusive range | `for i in 0..5` | Loop 0, 1, 2, 3, 4 |
| Inclusive range | `for i in 0..=5` | Loop 0, 1, 2, 3, 4, 5 |
| Slice | `xs[1:3]` | Elements at indices 1, 2 |
| Negative slice | `xs[-2:]` | Last 2 elements |
| Step slice | `xs[:$2]` | Every 2nd element |
| Comprehension | `[x in xs \| x*x]` | Transformed list |
| Guard | `[x in xs \| x \| x>0]` | Filtered list |
| Nested comp. | `[x in xs, y in ys \| x+y]` | Cross-product (flat) |
| `if let` | `if let v = opt { }` | Safe Option unwrap |
| `while let` | `while let v = opt { }` | Loop while Some |
| Trailing closure | `f(x): \|y\| y+1` | Closure after `:` |
| Bind operator | `expr ~> x { x+1 }` | Name intermediate value |
| Empty closure | `\|\| expr` | Zero-parameter closure |
| Function ref | `fn@(Int)` | Select overload by type |
| Generic annotation | `Variant() @[A: Int]` | No-data variant type hint |
| `::` (no self) | `s::fn_field()` | Call stored fn without self |
| `->` (with self) | `s->fn_field()` | Call stored fn with s as arg |
| Forward decl | `fn f(x: Int) -> Int` | Signature only, no body |
| Single-elem tuple | `(42,)` | One-element tuple |

---

## 28. Tips and Tricks

- Use `elif` instead of `else if`
- Closures with control flow need full `fn` syntax (not short lambda)
- Every function returning a value needs explicit `return`
- No `*=` operator — write `x = x * 2`
- Empty lists need type annotation: `let xs = []: Int`
- No-data enum variants need `()`: `Color::Red()`
- String concatenation: only `String + String`; use `Cast::string` to convert
- `format` / `printf` use `{}` placeholders

---

## 29. Common Mistakes

| Mistake | Fix |
|---|---|
| `let x = []; x.push(1)` | `let x = []: Int; x.push(1)` |
| `x *= 2` | No `*=` — use `x = x * 2` |
| `fn extends f(x)` called as `f(x)` | Use `x.f()` (UFCS only) |
| `5 \|> myExtendsFn()` | Pipe only works with non-extends |
| `match x { 0 => ... }` | Literals not in match — use `if/elif` |
| `-10 % 3 == -1` | Nova uses Euclidean: `-10 % 3 == 2` |
| `Cast::int(x)` used as `Int` | Returns `Option(Int)` — must unwrap |
| `else if x > 0 { }` | Use `elif` |
| Lambda with control flow | Use `fn(params) -> Type { }` |
| `fn f(x: Int) -> Int { x * x }` | Must have explicit `return` |
| `let b = a` (a is a list) | Creates alias! Use `clone(a)` for copy |
| `Color::Red` (no parens) | Must write `Color::Red()` |
| `\|\| true && false` precedence | `\|\|` binds tighter — use parens |
| Nested loops reusing `i` | Each loop needs a unique variable |
| Missing return on else branch | All paths must return |

---

# Part II — Game Development

---

## 30. Quick Start — Your First Window

```nova
raylib::init("My Game", 800, 600, 60)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    raylib::clear((20, 20, 40))
    raylib::drawText("Hello, Nova!", 280, 270, 36, (255, 255, 255))
    raylib::drawFPS(10, 10)
}
```

The game loop follows **Input → Update → Draw**:

```nova
while raylib::rendering() {
    let dt = raylib::getFrameTime()
    // 1. INPUT
    // 2. UPDATE
    // 3. DRAW
    raylib::clear((0, 0, 0))
}
```

### Recommended Project Structure

```
my_game/
  main.nv           ← entry point
  scenes/
    title.nv        ← makeTitleScene()
    gameplay.nv     ← makeGameplayScene()
  entities/
    player.nv
    enemy.nv
  assets/           ← sprites, sounds
```

---

## 31. Critical Rules for Game Dev

### 31.1 Box-Wrap Mutable Scalars in Closures

> **The #1 Nova game-dev gotcha.**

Closures capture scalars (`Int`, `Float`, `Bool`) **by value**. Mutations inside a
closure are invisible outside it.

**Wrong:**
```nova
let score = 0
let update = fn(dt: Float) { score += 10 }  // writes to a COPY
```

**Right:**
```nova
let score = Box(0)
let update = fn(dt: Float) { score.value += 10 }  // mutates heap cell
```

**Rule:** Wrap in `Box` if the variable is a scalar, declared outside a closure,
and mutated inside. Heap objects (Vec2, EntityWorld, etc.) never need `Box`.

### 31.2 Entity Movement — Manual vs `world.update(dt)`

`world.update(dt)` integrates `pos += vel * dt` AND purges dead entities.
If you move entities manually, call `world.update(0.0)` to only purge dead.

**Never** call `world.update(dt)` with real dt AND also manually move — that doubles movement.

### 31.3 Forward Declarations for Scenes

Scene factory functions often reference each other. Declare signatures first:

```nova
fn makeMenuScene() -> Scene      // forward declaration
fn makePlayScene() -> Scene

fn makeMenuScene() -> Scene {
    // ... can now call makePlayScene() ...
}
```

### 31.4 `elif` Not `else if`

Nova uses `elif` for chained conditionals. In expression context, only `if/else` pairs.

---

## 32. Scene Management

Scenes decouple game states (title, gameplay, pause, game-over):

```nova
import super.std.scene

fn makeGameplayScene(mgr: SceneManager) -> Scene {
    let world = EntityWorld::new()
    let score = Box(0)

    let update = fn(dt: Float) { /* game logic */ }
    let draw = fn() { /* rendering */ }

    return Scene::new(update, draw)
}

raylib::init("My Game", 800, 600, 60)
let mgr = SceneManager::empty()
mgr.switch(makeTitleScene(mgr))
while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

| Method | Effect |
|---|---|
| `mgr.switch(scene)` | Replace current, clear stack |
| `mgr.push(scene)` | Push over current (pause menus) |
| `mgr.pop()` | Return to previous scene |

---

## 33. Entity System

```nova
import super.std.entity
import super.std.vec2

let world = EntityWorld::new()

let player = world.spawn(400.0, 300.0, "player")
player.size = Vec2::new(32.0, 32.0)
```

### Entity Fields

| Field | Type | Purpose |
|---|---|---|
| `id` | `Int` | Auto-assigned ID |
| `pos` | `Vec2` | World position |
| `vel` | `Vec2` | Velocity (units/sec) |
| `size` | `Vec2` | Width × height |
| `tag` | `String` | Category: `"player"`, `"enemy"`, etc. |
| `alive` | `Bool` | Set false to destroy on next update |
| `data` | `Float` | General-purpose slot (health, age, …) |

### Querying and Iterating

```nova
let pList = world.query("player")
world.forEachTagged("enemy", fn(e: Entity) { /* mutate e */ })
world.forEach(fn(e: Entity) { /* all entities */ })
let count = world.countAlive("enemy")
```

### Collision

```nova
if bullet.overlapsAABB(enemy) {
    bullet.alive = false
    enemy.data -= 1.0
}
```

### Draw Helpers

```nova
e.entityDrawRect((60, 200, 100))      // filled rectangle
e.entityDrawCircle((255, 230, 0))     // circle
```

---

## 34. Input Handling

### Raw Raylib Input

```nova
raylib::isKeyPressed("A")        // held down
raylib::isKeyReleased("A")       // released this frame
raylib::mousePosition()          // (Int, Int)
raylib::getMouseWheel()          // Float
```

### InputMap — Action Bindings

```nova
import super.std.input

let keys = InputMap::new()
keys.bindKey("left", "A")
keys.bindKey("right", "D")
keys.bindKey("jump", "Space")
keys.bindMouse("aim", "Left")

let dx = keys.axis("left", "right")   // -1.0, 0.0, or 1.0
if keys.isPressed("jump") { /* jump */ }
```

---

## 35. Physics and Collision

```nova
import super.std.physics

let ball = Body2D::new(400.0, 100.0, 1.0)
ball.restitution = 0.7

let floor = AABB::new(0.0, 560.0, 800.0, 40.0)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    ball.applyGravity(600.0, dt)
    ball.update(dt)
    pushOutAABB(ball, 20.0, 20.0, floor)
}
```

### Raycasting

```nova
let ray = Ray2::new(px, py, dirX, dirY)
let hit = ray.castAABB(wall)
if hit.hit { /* hit.point, hit.normal, hit.t */ }
```

---

## 36. Camera

```nova
import super.std.camera

let cam = Camera2D::new(800, 600)
cam.setZoom(1.5)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    cam.update(dt)
    cam.follow(player.pos, 6.0, dt)

    cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (200, 200, 200))
    cam.shake(12.0, 0.4)   // on explosion
}
```

### Coordinate Conversion

```nova
let worldMouse = cam.screenToWorld(Vec2::new(mx, my))
let screenPos = cam.worldToScreen(entity.pos)
```

---

## 37. Timers and Tweens

### Timers

```nova
import super.std.timer

let fireRate = Timer::cooldown(0.15)
let blink = Timer::repeating(0.5)

fireRate.update(dt)
if keys.isHeld("fire") && fireRate.ready() { spawnBullet() }
```

### Tweens

```nova
import super.std.tween

let fadeIn = Tween::smooth(0.0, 255.0, 1.0)

let alpha = fadeIn.update(dt)
if fadeIn.isDone() { fadeIn.ping() }   // ping-pong
```

---

## 38. Vec2 Math

```nova
import super.std.vec2

let v = Vec2::new(3.0, 4.0)
v.length()                          // 5.0
v.normalized()                      // (0.6, 0.8)
v.add(Vec2::new(1.0, 2.0))         // (4.0, 6.0)

// Aim at target
let dir = target.pos.sub(e.pos).normalized()
e.vel.x = dir.x * SPEED
```

---

## 39. Tilemaps and Noise

### Grid

```nova
import super.std.grid

let map = Grid::new(30, 20, 0)
map.fillRect(0, 0, 30, 20, 1)       // walls
map.fillRect(1, 1, 28, 18, 0)       // hollow
let path = map.bfs(sx, sy, gx, gy, fn(v: Any) -> Bool { v == 0 })
```

### Procedural Noise

```nova
import super.std.noise

let h = fbm(nx, ny, SEED, 5, 2.0, 0.5)
map.set(col, row, if h > 0.6 { 1 } else { 0 })
```

---

## 40. Sprites and Audio

### Sprites

```nova
let hero = raylib::loadSprite("assets/hero.png", 32, 1)
raylib::drawSprite(hero, px, py)
```

Procedural sprites:

```nova
let pixels = []: (Int, Int, Int)
// ... fill pixels ...
let sprite = raylib::buildSprite(size, size, 1, pixels)
```

### Audio

```nova
raylib::initAudio()
let sfx = raylib::loadSound("assets/jump.wav")
raylib::playSound(sfx)

let bgm = raylib::loadMusic("assets/bgm.ogg")
raylib::setMusicLooping(bgm, true)
raylib::playMusic(bgm)

while raylib::rendering() {
    raylib::updateMusic(bgm)    // REQUIRED every frame
}

raylib::closeAudio()
```

---

## 41. HUD and UI

### Health Bar

```nova
fn drawHealthBar(x: Int, y: Int, w: Int, h: Int, current: Int, maxHp: Int) {
    raylib::drawRectangle(x, y, w, h, (60, 60, 60))
    let fillW = (w * current) / maxHp
    let color = if current * 3 < maxHp       { (255, 0, 0)   }
                elif current * 3 < maxHp * 2 { (255, 200, 0) }
                else                          { (0, 255, 0)   }
    raylib::drawRectangle(x, y, fillW, h, color)
    raylib::drawRectangleLines(x, y, w, h, (255, 255, 255))
}
```

### Centered Button

```nova
fn drawButton(x: Int, y: Int, w: Int, h: Int, text: String, hover: Bool) {
    let bg = if hover { (80, 120, 200) } else { (50, 80, 150) }
    raylib::drawRoundedRectangle(x, y, w, h, 0.3, bg)
    let tw = raylib::measureText(text, 20)
    raylib::drawText(text, x + (w - tw) / 2, y + (h - 20) / 2, 20, (255, 255, 255))
}
```

> **Draw order:** background first, then world, then HUD on top.

---

## 42. Advanced Game Patterns

### Enum-Based Entity Kinds

```nova
enum Tag { Player, Enemy: Int, Pickup: Int, Bullet }

match a.tag {
    Bullet() => {
        match b.tag {
            Enemy(hp) => {
                a.active = false
                if hp <= 1 { b.active = false }
                else { b.tag = Tag::Enemy(hp - 1) }
            }
            _ => {}
        }
    }
    _ => {}
}
```

### Vtable Dispatch with `->`

Store `draw` / `update` closures as struct fields. Use `->` for type-dispatched calls:

```nova
struct PlayerEntity {
    x: Int, y: Int, hp: Int,
    draw: fn(PlayerEntity),
}

type drawable = Dyn(T = draw: fn($T))

// Each entity draws itself — no match needed
for item in allDrawables { item->draw() }
```

### Object Pooling

Pre-allocate a fixed pool; reuse slots with an `active` flag:

```nova
let bulletPool = []: Bullet
for let i = 0; i < 256; i += 1 {
    bulletPool.push(Bullet { x: 0, y: 0, vx: 0, vy: -8, active: 0 })
}

fn spawnBullet(px: Int, py: Int) {
    for b in bulletPool {
        if b.active == 0 { b.x = px; b.y = py; b.active = 1; return }
    }
}
```

### Screen Stack Without SceneManager

```nova
enum Screen { MainMenu, Playing, Paused, GameOver: Int }
let stack = []: Screen

fn pushScreen(s: Screen) { stack.push(s) }
fn popScreen() { if stack.len() > 0 { stack.pop() } }
```

---

## 43. Game Dev Tips and Tricks

### Frame Animation

```nova
let frame = Box(0)
let frameTimer = Timer::cooldown(0.1)

frameTimer.update(dt)
if frameTimer.ready() { frame.value = (frame.value + 1) % FRAME_COUNT }
raylib::drawSpriteFrame(sprite, frame.value, px, py)
```

### Screen Shake

Use `Camera2D::shake(intensity, duration)` or manually offset draws.

### Hit Flash

```nova
let flashTimer = Tween::linear(1.0, 0.0, 0.4)
// On hit: flashTimer.reset()
// In draw: tint based on flashTimer.value()
```

### Wave Spawner

```nova
let wave = Box(1)
let waveTimer = Timer::repeating(5.0)

waveTimer.update(dt)
if waveTimer.ready() {
    wave.value += 1
    for let i = 0; i < wave.value * 3; i += 1 { spawnEnemy() }
}
```

### Floating Score Popups

```nova
struct Popup { x: Float, y: Float, text: String, life: Float, active: Bool }

fn spawnPopup(x: Float, y: Float, pts: Int) {
    popups.push(Popup { x: x, y: y, text: "+" + Cast::string(pts),
                        life: 1.0, active: true })
}
```

### Debug Overlay

```nova
let debugOn = Box(false)
if raylib::isKeyReleased("F3") { debugOn.value = !debugOn.value }

if debugOn.value {
    // draw hitboxes, entity counts, FPS
}
```

---

## 44. Performance Tips

| Problem | Solution |
|---|---|
| GC spikes from spawning | Object pool with `active` flag |
| O(n²) collision | Spatial grid — insert then query |
| Temporary lists each frame | Pre-allocate outside loop, `clear()` inside |
| Off-screen entities drawn | Cull with `cam.isVisible(x, y, margin)` |
| `Cast::string` in tight loop | Cache the string; regenerate only on change |
| Music stops | Call `raylib::updateMusic(id)` every frame |

---

## 45. Complete Example — Breakout

```nova
module breakout

import super.std.scene
import super.std.entity
import super.std.input
import super.std.timer
import super.std.tween
import super.std.vec2

let W          = 800
let H          = 600
let PADDLE_W   = 100
let PADDLE_H   = 16
let PADDLE_Y   = H - 48
let BALL_R     = 10.0
let BALL_SPEED = 340.0
let MAX_LIVES  = 3
let BRICK_COLS = 10
let BRICK_ROWS = 5
let BRICK_W    = 64
let BRICK_H    = 22
let BRICK_PAD  = 4
let BRICK_OFF_X = (W - BRICK_COLS * (BRICK_W + BRICK_PAD)) / 2
let BRICK_OFF_Y = 60
let TOTAL_BRICKS = BRICK_COLS * BRICK_ROWS

fn makeMenuScene() -> Scene
fn makePlayScene() -> Scene
fn makeGameOverScene() -> Scene
fn makeWinScene() -> Scene

let mgr        = SceneManager::empty()
let paddleX    = Box(W / 2 - PADDLE_W / 2)
let score      = Box(0)
let lives      = Box(MAX_LIVES)
let hitCount   = Box(0)
let ballOnBoard = Box(true)
let ballVel    = Vec2::new(BALL_SPEED * 0.6, -BALL_SPEED * 0.8)
let world      = EntityWorld::new()
let scoreFlash = Tween::linear(255.0, 0.0, 0.4)
let keys       = InputMap::new()

fn makeMenuScene() -> Scene {
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("fire") { mgr.switch(makePlayScene()) }
        },
        fn() {
            raylib::clear((10, 20, 40))
            let t1 = "BREAKOUT"
            raylib::drawText(t1, (W - raylib::measureText(t1, 60)) / 2, 180, 60, (100, 200, 255))
            raylib::drawText("SPACE to play", (W - raylib::measureText("SPACE to play", 22)) / 2,
                             310, 22, (180, 180, 180))
        }
    )
}

fn makePlayScene() -> Scene {
    score.value     = 0
    lives.value     = MAX_LIVES
    hitCount.value  = 0
    ballOnBoard.value = true
    paddleX.value   = W / 2 - PADDLE_W / 2
    ballVel.x       = BALL_SPEED * 0.6
    ballVel.y       = -BALL_SPEED * 0.8

    world.forEach(fn(e: Entity) { e.alive = false })
    world.update(0.0)

    let ball = world.spawn(Cast::float(W / 2).unwrap(),
                           Cast::float(H / 2).unwrap(), "ball")
    ball.size = Vec2::new(BALL_R * 2.0, BALL_R * 2.0)

    for let row = 0; row < BRICK_ROWS; row += 1 {
        for let col = 0; col < BRICK_COLS; col += 1 {
            let bx = Cast::float(BRICK_OFF_X + col * (BRICK_W + BRICK_PAD)).unwrap()
            let by = Cast::float(BRICK_OFF_Y + row * (BRICK_H + BRICK_PAD)).unwrap()
            let b = world.spawn(bx, by, "brick")
            b.size = Vec2::new(Cast::float(BRICK_W).unwrap(), Cast::float(BRICK_H).unwrap())
            b.data = Cast::float(row + 1).unwrap()
        }
    }

    keys.bindKey("left",  "Left")
    keys.bindKey("right", "Right")
    keys.bindKey("fire",  "Space")

    return Scene::new(
        fn(dt: Float) {
            let dx = keys.axis("left", "right")
            paddleX.value = Cast::int(Cast::float(paddleX.value).unwrap() + dx * 360.0 * dt).unwrap()
            if paddleX.value < 0 { paddleX.value = 0 }
            if paddleX.value > W - PADDLE_W { paddleX.value = W - PADDLE_W }

            let balls = world.query("ball")
            if balls.len() > 0 && ballOnBoard.value {
                let b = balls[0]
                b.pos.x = b.pos.x + ballVel.x * dt
                b.pos.y = b.pos.y + ballVel.y * dt

                if b.pos.x <= 0.0 { b.pos.x = 0.0; ballVel.x = -ballVel.x }
                if b.pos.x + BALL_R * 2.0 >= Cast::float(W).unwrap() {
                    b.pos.x = Cast::float(W).unwrap() - BALL_R * 2.0
                    ballVel.x = -ballVel.x
                }
                if b.pos.y <= 0.0 { b.pos.y = 0.0; ballVel.y = -ballVel.y }

                let px = Cast::float(paddleX.value).unwrap()
                let py = Cast::float(PADDLE_Y).unwrap()
                if b.pos.y + BALL_R * 2.0 >= py && b.pos.y < py + Cast::float(PADDLE_H).unwrap() {
                    if b.pos.x + BALL_R * 2.0 > px && b.pos.x < px + Cast::float(PADDLE_W).unwrap() {
                        ballVel.y = -Float::abs(ballVel.y)
                        let relX = (b.pos.x + BALL_R - px) / Cast::float(PADDLE_W).unwrap() - 0.5
                        ballVel.x = relX * BALL_SPEED * 2.0
                    }
                }

                if b.pos.y > Cast::float(H + 20).unwrap() {
                    lives.value -= 1
                    ballOnBoard.value = false
                    if lives.value <= 0 { mgr.switch(makeGameOverScene()) }
                    else {
                        b.pos.x = Cast::float(W / 2).unwrap()
                        b.pos.y = Cast::float(H / 2).unwrap()
                        ballVel.x = BALL_SPEED * 0.6
                        ballVel.y = -BALL_SPEED * 0.8
                        ballOnBoard.value = true
                    }
                }

                world.forEachTagged("brick", fn(br: Entity) {
                    if b.pos.x + BALL_R * 2.0 > br.pos.x &&
                       b.pos.x < br.pos.x + br.size.x &&
                       b.pos.y + BALL_R * 2.0 > br.pos.y &&
                       b.pos.y < br.pos.y + br.size.y {
                        let overlapL = b.pos.x + BALL_R * 2.0 - br.pos.x
                        let overlapR = br.pos.x + br.size.x - b.pos.x
                        let overlapT = b.pos.y + BALL_R * 2.0 - br.pos.y
                        let overlapB = br.pos.y + br.size.y - b.pos.y
                        let minH = if overlapL < overlapR { overlapL } else { overlapR }
                        let minV = if overlapT < overlapB { overlapT } else { overlapB }
                        if minH < minV { ballVel.x = -ballVel.x }
                        else { ballVel.y = -ballVel.y }
                        br.data -= 1.0
                        if br.data <= 0.0 { br.alive = false }
                        score.value += 10
                        hitCount.value += 1
                        scoreFlash.reset()
                        if hitCount.value >= TOTAL_BRICKS { mgr.switch(makeWinScene()) }
                    }
                })
            }
            world.update(0.0)
        },
        fn() {
            raylib::clear((8, 12, 28))
            world.forEachTagged("brick", fn(br: Entity) {
                let hp = Cast::int(br.data).unwrap()
                let r = 60 + hp * 35; let g = 40 + hp * 20
                raylib::drawRectangle(Cast::int(br.pos.x).unwrap(), Cast::int(br.pos.y).unwrap(),
                                      BRICK_W - 2, BRICK_H - 2, (r, g, 60))
            })
            world.forEachTagged("ball", fn(b: Entity) {
                raylib::drawCircle(Cast::int(b.pos.x + BALL_R).unwrap(),
                                   Cast::int(b.pos.y + BALL_R).unwrap(),
                                   Cast::int(BALL_R).unwrap(), (255, 230, 80))
            })
            raylib::drawRoundedRectangle(paddleX.value, PADDLE_Y, PADDLE_W, PADDLE_H,
                                         0.4, (80, 180, 255))
            raylib::drawText("Score: " + Cast::string(score.value), 10, 10, 20, (255, 255, 255))
            for let i = 0; i < lives.value; i += 1 {
                raylib::drawCircle(W - 20 - i * 24, 18, 8, (255, 80, 80))
            }
            let fa = Cast::int(scoreFlash.value()).unwrap()
            if fa > 5 {
                raylib::drawText("+10", paddleX.value + PADDLE_W / 2 - 15, PADDLE_Y - 28,
                                 20, (255, 230, fa))
            }
            scoreFlash.update(raylib::getFrameTime())
        }
    )
}

fn makeGameOverScene() -> Scene {
    let finalScore = score.value
    return Scene::new(
        fn(dt: Float) { if keys.isPressed("fire") { mgr.switch(makeMenuScene()) } },
        fn() {
            raylib::clear((30, 0, 0))
            raylib::drawText("GAME OVER", (W - raylib::measureText("GAME OVER", 60)) / 2,
                             200, 60, (255, 80, 80))
            raylib::drawText("Score: " + Cast::string(finalScore),
                             (W - raylib::measureText("Score: " + Cast::string(finalScore), 28)) / 2,
                             290, 28, (255, 255, 255))
            raylib::drawText("SPACE to menu",
                             (W - raylib::measureText("SPACE to menu", 20)) / 2,
                             350, 20, (160, 160, 160))
        }
    )
}

fn makeWinScene() -> Scene {
    let finalScore = score.value
    return Scene::new(
        fn(dt: Float) { if keys.isPressed("fire") { mgr.switch(makeMenuScene()) } },
        fn() {
            raylib::clear((0, 30, 0))
            raylib::drawText("YOU WIN!", (W - raylib::measureText("YOU WIN!", 60)) / 2,
                             200, 60, (80, 255, 120))
            raylib::drawText("Score: " + Cast::string(finalScore),
                             (W - raylib::measureText("Score: " + Cast::string(finalScore), 28)) / 2,
                             290, 28, (255, 255, 255))
            raylib::drawText("SPACE to menu",
                             (W - raylib::measureText("SPACE to menu", 20)) / 2,
                             350, 20, (160, 160, 160))
        }
    )
}

raylib::init("Breakout", W, H, 60)
mgr.switch(makeMenuScene())
while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

---

## 46. Complete Example — Top-Down Shooter

```nova
module shooter

import super.std.scene
import super.std.entity
import super.std.input
import super.std.camera
import super.std.timer
import super.std.tween
import super.std.vec2

let SW = 900
let SH = 600
let PLAYER_SPEED   = 200.0
let BULLET_SPEED   = 500.0
let BULLET_LIFETIME = 2.0
let ENEMY_BASE_SPEED = 60.0
let PLAYER_HP_MAX  = 5
let WAVE_INTERVAL  = 8.0

fn makeMenuScene() -> Scene
fn makePlayScene() -> Scene
fn makeGameOverScene() -> Scene

let mgr         = SceneManager::empty()
let world       = EntityWorld::new()
let cam         = Camera2D::new(SW, SH)
let keys        = InputMap::new()
let score       = Box(0)
let wave        = Box(1)
let playerHp    = Box(PLAYER_HP_MAX)
let playerAlive = Box(true)
let fireTimer   = Timer::cooldown(0.12)
let waveTimer   = Timer::repeating(WAVE_INTERVAL)
let hitFlash    = Tween::linear(180.0, 0.0, 0.35)

fn makeMenuScene() -> Scene {
    return Scene::new(
        fn(dt: Float) { if keys.isPressed("fire") { mgr.switch(makePlayScene()) } },
        fn() {
            raylib::clear((10, 10, 24))
            raylib::drawText("TOP-DOWN SHOOTER",
                             (SW - raylib::measureText("TOP-DOWN SHOOTER", 48)) / 2,
                             180, 48, (100, 200, 255))
            raylib::drawText("WASD + Mouse  •  SPACE to start",
                             (SW - raylib::measureText("WASD + Mouse  •  SPACE to start", 18)) / 2,
                             280, 18, (160, 160, 160))
        }
    )
}

fn makePlayScene() -> Scene {
    score.value = 0
    wave.value = 1
    playerHp.value = PLAYER_HP_MAX
    playerAlive.value = true

    world.forEach(fn(e: Entity) { e.alive = false })
    world.update(0.0)

    let player = world.spawn(Cast::float(SW / 2).unwrap(), Cast::float(SH / 2).unwrap(), "player")
    player.size = Vec2::new(20.0, 20.0)

    keys.bindKey("up", "W")
    keys.bindKey("down", "S")
    keys.bindKey("left", "A")
    keys.bindKey("right", "D")
    keys.bindKey("fire", "Space")
    keys.bindMouse("shoot", "Left")

    fn spawnWave() {
        let count = wave.value * 3 + 2
        for let i = 0; i < count; i += 1 {
            let side = i % 4
            let ex = if side == 0 { -40.0 }
                     elif side == 1 { Cast::float(SW + 40).unwrap() }
                     elif side == 2 { Cast::float(i * 80 % SW).unwrap() }
                     else { Cast::float(i * 60 % SW).unwrap() }
            let ey = if side == 2 { -40.0 }
                     elif side == 3 { Cast::float(SH + 40).unwrap() }
                     elif side == 0 { Cast::float(i * 70 % SH).unwrap() }
                     else { Cast::float(i * 50 % SH).unwrap() }
            let e = world.spawn(ex, ey, "enemy")
            e.size = Vec2::new(22.0, 22.0)
            e.data = ENEMY_BASE_SPEED + Cast::float(wave.value).unwrap() * 10.0
        }
    }
    spawnWave()

    return Scene::new(
        fn(dt: Float) {
            if !playerAlive.value { return }
            let pList = world.query("player")
            if pList.len() == 0 { return }
            let p = pList[0]
            let dx = keys.axis("left", "right")
            let dy = keys.axis("up", "down")
            p.pos.x += dx * PLAYER_SPEED * dt
            p.pos.y += dy * PLAYER_SPEED * dt

            fireTimer.update(dt)
            if (keys.isHeld("shoot") || keys.isHeld("fire")) && fireTimer.ready() {
                let (mx, my) = InputMap::mousePos()
                let wm = cam.screenToWorld(Vec2::new(Cast::float(mx).unwrap(),
                                                     Cast::float(my).unwrap()))
                let dir = wm.sub(p.pos)
                if dir.length() > 1.0 {
                    let nd = dir.normalized()
                    let b = world.spawn(p.pos.x, p.pos.y, "bullet")
                    b.vel.x = nd.x * BULLET_SPEED
                    b.vel.y = nd.y * BULLET_SPEED
                    b.size = Vec2::new(6.0, 6.0)
                    b.data = 0.0
                }
            }

            world.forEachTagged("bullet", fn(b: Entity) {
                b.pos.x += b.vel.x * dt
                b.pos.y += b.vel.y * dt
                b.data += dt
                if b.data > BULLET_LIFETIME { b.alive = false }
            })

            let pPos = p.pos
            world.forEachTagged("enemy", fn(e: Entity) {
                let toPlayer = pPos.sub(e.center())
                if toPlayer.length() > 0.5 {
                    let spd = e.data
                    let move = toPlayer.normalized().scale(spd * dt)
                    e.pos.x += move.x
                    e.pos.y += move.y
                }
            })

            world.forEachTagged("bullet", fn(b: Entity) {
                world.forEachTagged("enemy", fn(e: Entity) {
                    if b.overlapsAABB(e) {
                        b.alive = false
                        e.alive = false
                        score.value += 10 * wave.value
                        cam.shake(4.0, 0.1)
                    }
                })
            })

            world.forEachTagged("enemy", fn(e: Entity) {
                let pList2 = world.query("player")
                if pList2.len() > 0 {
                    let p2 = pList2[0]
                    if e.overlapsAABB(p2) {
                        e.alive = false
                        playerHp.value -= 1
                        hitFlash.reset()
                        cam.shake(8.0, 0.2)
                        if playerHp.value <= 0 {
                            playerAlive.value = false
                            mgr.switch(makeGameOverScene())
                        }
                    }
                }
            })

            waveTimer.update(dt)
            if waveTimer.ready() || world.countAlive("enemy") == 0 {
                wave.value += 1
                spawnWave()
            }

            world.update(0.0)
            cam.update(dt)
            cam.follow(p.pos, 5.0, dt)
        },
        fn() {
            raylib::clear((10, 10, 20))
            world.forEachTagged("player", fn(e: Entity) {
                let fa = Cast::int(hitFlash.value()).unwrap()
                let col = if fa > 10 { (255, fa, fa) } else { (60, 220, 100) }
                cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, col)
            })
            hitFlash.update(raylib::getFrameTime())
            world.forEachTagged("enemy", fn(e: Entity) {
                cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (200, 60, 60))
            })
            world.forEachTagged("bullet", fn(b: Entity) {
                cam.drawCircle(b.pos.x + 3.0, b.pos.y + 3.0, 3.0, (255, 240, 80))
            })
            raylib::drawText("Score: " + Cast::string(score.value), 10, 10, 22, (255, 255, 255))
            raylib::drawText("Wave:  " + Cast::string(wave.value), 10, 36, 18, (200, 200, 80))
            for let i = 0; i < playerHp.value; i += 1 {
                raylib::drawRectangle(SW - 20 - i * 22, 12, 16, 16, (255, 80, 80))
            }
        }
    )
}

fn makeGameOverScene() -> Scene {
    let finalScore = score.value
    let finalWave = wave.value
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("fire") || keys.isHeld("shoot") {
                mgr.switch(makeMenuScene())
            }
        },
        fn() {
            raylib::clear((20, 0, 0))
            raylib::drawText("GAME OVER",
                             (SW - raylib::measureText("GAME OVER", 60)) / 2,
                             180, 60, (255, 80, 80))
            raylib::drawText("Score: " + Cast::string(finalScore),
                             (SW - raylib::measureText("Score: " + Cast::string(finalScore), 28)) / 2,
                             270, 28, (255, 255, 255))
            raylib::drawText("Wave:  " + Cast::string(finalWave),
                             (SW - raylib::measureText("Wave:  " + Cast::string(finalWave), 22)) / 2,
                             310, 22, (200, 200, 80))
            raylib::drawText("Click or SPACE to retry",
                             (SW - raylib::measureText("Click or SPACE to retry", 18)) / 2,
                             380, 18, (160, 160, 160))
        }
    )
}

raylib::init("Top-Down Shooter", SW, SH, 60)
mgr.switch(makeMenuScene())
while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

---

# Part III — Terminal Applications

---

## 47. Terminal Quick Start

```nova
terminal::clearScreen()
terminal::moveTo(0, 0)
terminal::print("Hello from the terminal!")
terminal::flush()
sleep(2000)
```

Key built-in functions: `terminal::clearScreen()`, `terminal::moveTo(col, row)`,
`terminal::print(s)`, `terminal::flush()`, `terminal::rawmode(bool)`,
`terminal::getch()`, `terminal::rawread(ms)`, `terminal::getSize()`.

---

## 48. Raw Mode and Key Input

Raw mode disables line buffering — each keypress is available immediately:

```nova
terminal::rawmode(true)
terminal::hideCursor()

let running = true
while running {
    let ch = terminal::rawread(50)    // 50ms timeout
    if let c = ch {
        if c == 'q' { running = false }
        terminal::clearScreen()
        terminal::moveTo(0, 0)
        terminal::print("You pressed: " + Cast::string(c))
        terminal::flush()
    }
}

terminal::rawmode(false)
terminal::showCursor()
```

### Reading Arrow Keys

Arrow keys produce escape sequences. Check for `'\x1b'` then read follow-up bytes:

```nova
if c == '\x1b' {
    let c2 = terminal::rawread(10)
    if let b = c2 {
        if b == '[' {
            let c3 = terminal::rawread(10)
            if let arrow = c3 {
                if arrow == 'A' { /* up */ }
                if arrow == 'B' { /* down */ }
                if arrow == 'C' { /* right */ }
                if arrow == 'D' { /* left */ }
            }
        }
    }
}
```

---

## 49. Colours and Cursor

```nova
terminal::setForeground(255, 100, 50)    // orange text
terminal::setBackground(0, 0, 80)        // dark blue bg
terminal::print("coloured!")
terminal::resetColor()
terminal::flush()
```

Or use `std/ansi` for styled strings:

```nova
import super.std.ansi

println(Ansi::bold(Ansi::red("ERROR")))
println(Ansi::rgb(255, 128, 0, "orange"))
```

---

## 50. Terminal Menu System

Use `std/tui` with `SceneManager` for multi-screen terminal apps:

```nova
import super.std.tui
import super.std.scene

fn makeMenuScene(mgr: SceneManager) -> Scene {
    let selected = Box(0)
    let options = ["Play", "Options", "Quit"]

    return Scene::new(
        fn(dt: Float) {
            let ch = terminal::rawread(50)
            if let c = ch {
                if c == 'w' || c == 'A' { selected.value = (selected.value - 1 + options.len()) % options.len() }
                if c == 's' || c == 'B' { selected.value = (selected.value + 1) % options.len() }
                if c == '\n' {
                    if selected.value == 0 { mgr.switch(makeGameScene(mgr)) }
                    if selected.value == 2 { mgr.switch(makeQuitScene()) }
                }
            }
        },
        fn() {
            terminal::clearScreen()
            for let i = 0; i < options.len(); i += 1 {
                terminal::moveTo(2, i + 2)
                let prefix = if i == selected.value { "> " } else { "  " }
                terminal::print(prefix + options[i])
            }
            terminal::flush()
        }
    )
}
```

---

## 51. Terminal Game Loop

A terminal game loop with frame timing:

```nova
terminal::rawmode(true)
terminal::hideCursor()

let running = Box(true)
let px = Box(5)
let py = Box(5)

while running.value {
    let start = now()

    // Input
    let ch = terminal::rawread(16)
    if let c = ch {
        if c == 'q' { running.value = false }
        if c == 'w' { py.value -= 1 }
        if c == 's' { py.value += 1 }
        if c == 'a' { px.value -= 1 }
        if c == 'd' { px.value += 1 }
    }

    // Draw
    terminal::clearScreen()
    terminal::moveTo(px.value, py.value)
    terminal::setForeground(0, 255, 0)
    terminal::print("@")
    terminal::resetColor()
    terminal::flush()

    // Frame timing (~60fps)
    let elapsed = now() - start
    if elapsed < 16 { sleep(16 - elapsed) }
}

terminal::rawmode(false)
terminal::showCursor()
terminal::clearScreen()
```

---

## 52. Terminal Patterns

### Always Clean Up

```nova
terminal::rawmode(true)
terminal::hideCursor()
// ... app logic ...
terminal::rawmode(false)
terminal::showCursor()
terminal::clearScreen()
```

### Prevent Flicker

Write everything, then flush once:

```nova
// draw all elements
terminal::moveTo(0, 0)
terminal::print(buffer)
terminal::flush()       // single flush at end
```

### Use Structs for State

```nova
struct AppState { x: Int, y: Int, score: Int, running: Bool }
```

### Common Pitfalls

| Pitfall | Fix |
|---|---|
| Terminal stuck in raw mode | Always call `rawmode(false)` before exit |
| Cursor visible during game | `hideCursor()` at start, `showCursor()` at exit |
| Screen flickers | Use one `flush()` per frame, not per draw call |
| Arrow keys not detected | Read the 3-byte escape sequence |
| Text leaves garbage | `clearScreen()` before each frame |

---

# Part IV — For Python Developers

---

## 53. Nova for Python Developers

### Quick Comparison

| Python | Nova |
|---|---|
| Everything is an object | Values; heap via `Box(T)` |
| `class` owns data + methods | `struct` + `fn extends` |
| Duck typing at runtime | Structural typing at compile time |
| `None` anywhere | `Option(T)` must be handled |
| `list` — dynamic, mixed types | `[T]` — typed, single type |
| `[x for x in xs if p]` | `[x in xs \| x \| p]` |
| `xs[1:3]`, `xs[-2:]` | Same slicing syntax |
| `f"hello {x}"` | `format("hello {}", [x])` |
| `lambda x: x+1` | `\|x: Int\| x + 1` |
| `try / except` | `Option(T)` / `Result(T, E)` |

### From Classes to Structs + Extends

```nova
struct Point { x: Float, y: Float }

fn extends distance(self: Point, other: Point) -> Float {
    let dx = self.x - other.x
    let dy = self.y - other.y
    return (dx * dx + dy * dy).sqrt()
}

fn extends __add__(a: Point, b: Point) -> Point {
    return Point { x: a.x + b.x, y: a.y + b.y }
}
```

### From None to Option

```nova
// Combinator style
let name = findUser(db, 42)
    .map(|u: User| u.name.toUpper())
    .orDefault("not found")

// if-let style
if let user = findUser(db, 42) {
    println(user.name.toUpper())
}
```

### From Dict to HashMap

```nova
import super.std.hashmap

let counts = HashMap::default() @[K: String, V: Int]
for word in words { counts.increment(word) }
```

### Syntax Sugars Python Devs Will Love

- List comprehensions: `[x in 0.to(10) | x * x | x % 2 == 0]`
- Slicing with negative indices and step: `xs[-2:]`, `xs[:$2]`
- Pipe operator: `4 |> inc() |> square()`
- Range loops: `for i in 0..10 { }`
- `if let` for safe unwrapping
- Full interactive REPL: `nova repl`
