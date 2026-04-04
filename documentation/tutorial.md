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
30. [Built-in Functions](#30-built-in-functions)
31. [CLI and REPL](#31-cli-and-repl)
32. [Novel Feature Combinations](#32-novel-feature-combinations)

### Part II — Game Development

33. [Quick Start — Your First Window](#33-quick-start--your-first-window)
34. [Critical Rules for Game Dev](#34-critical-rules-for-game-dev)
35. [Scene Management](#35-scene-management)
36. [Entity System](#36-entity-system)
37. [Input Handling](#37-input-handling)
38. [Physics and Collision](#38-physics-and-collision)
39. [Camera](#39-camera)
40. [Timers and Tweens](#40-timers-and-tweens)
41. [Vec2 Math](#41-vec2-math)
42. [Tilemaps and Noise](#42-tilemaps-and-noise)
43. [Sprites and Audio](#43-sprites-and-audio)
44. [HUD and UI](#44-hud-and-ui)
45. [Advanced Game Patterns](#45-advanced-game-patterns)
46. [Game Dev Tips and Tricks](#46-game-dev-tips-and-tricks)
47. [Performance Tips](#47-performance-tips)
48. [Complete Example — Breakout](#48-complete-example--breakout)
49. [Complete Example — Top-Down Shooter](#49-complete-example--top-down-shooter)

### Part III — Terminal Applications

50. [Terminal Quick Start](#50-terminal-quick-start)
51. [Raw Mode and Key Input](#51-raw-mode-and-key-input)
52. [Colours and Cursor](#52-colours-and-cursor)
53. [Terminal Menu System](#53-terminal-menu-system)
54. [Terminal Game Loop](#54-terminal-game-loop)
55. [Terminal Patterns](#55-terminal-patterns)

### Part IV — For Python Developers

56. [Nova for Python Developers](#56-nova-for-python-developers)

---

# Part I — The Language

---

## 1. Hello World

Every Nova file begins with a `module` declaration, then your code:

```nova
module hello

println("Hello, world!")
```

Save as `hello.nv` and run with `nova run hello.nv`. See [§31 CLI and REPL](#31-cli-and-repl) for all commands, or the [Getting Started](getting_started.md) guide for project setup.

---

## 2. Module System

Every Nova source file starts with a `module` declaration:

```nova
module my_program
```

The module name registers the file so the parser can deduplicate imports.
If two files import the same module, it is only parsed once.

### Import Syntax

Nova uses dot-separated names to navigate the folder structure. Each dot
becomes a `/`, and `.nv` is appended automatically. The keyword `super`
means "go up one directory" (like `..` on the filesystem).

```nova
import helper              // → ./helper.nv            (same directory)
import utils.math          // → ./utils/math.nv        (subfolder)
import super.std.core      // → ../std/core.nv         (up one, then into std/)
import super.super.std.io  // → ../../std/io.nv        (up two directories)
```

All paths are relative to the file that contains the `import` statement.

You can also use a **string literal** if you prefer a raw path:

```nova
import "libs/helper.nv"        // same as: import libs.helper
import "../std/core.nv"        // same as: import super.std.core
```

All imports flatten into the caller's scope — you call imported functions
by name, not with a prefix.

### How `super` Works

`super` translates to `..` (parent directory). You can chain it:

| Import statement | Filesystem path (relative to current file) |
|---|---|
| `import helper` | `./helper.nv` |
| `import libs.helper` | `./libs/helper.nv` |
| `import super.std.core` | `../std/core.nv` |
| `import super.super.std.grid` | `../../std/grid.nv` |

### GitHub Imports (`import @`)

You can import a file directly from a public GitHub repository using the
`@` symbol. The module name comes from the `module` declaration inside the
fetched file — you don't need to name it yourself:

```nova
import @ "pyrotek45/nova-lang/std/core.nv"
```

The string after `@` has the format `"owner/repo/path/to/file.nv"`.
Nova fetches the file from GitHub's `main` branch by default.

To lock to a specific commit (so your code doesn't break if the repo changes),
add `!` followed by a commit hash:

```nova
import @ "pyrotek45/nova-lang/std/core.nv" ! "a1b2c3d"
```

When a commit hash is given, Nova fetches that exact revision instead of `main`.

**How it works under the hood:**

1. Nova sees `import @` and reads the string literal
2. It builds a URL: `https://raw.githubusercontent.com/owner/repo/branch/path`
3. It fetches the file contents over HTTP
4. The fetched source is parsed as if it were a local file
5. The `module` declaration inside the file registers the module name
6. If that module was already imported, it is skipped (no duplicates)
7. All exported functions, structs, and enums become available in the caller's scope

**Import form summary:**

| Form | Example | Resolves to |
|---|---|---|
| Dot-path (local) | `import libs.helper` | `./libs/helper.nv` |
| Super (local) | `import super.std.core` | `../std/core.nv` |
| String literal (local) | `import "libs/helper.nv"` | `./libs/helper.nv` |
| GitHub | `import @ "owner/repo/path.nv"` | fetched from GitHub |
| GitHub + lock | `import @ "owner/repo/path.nv" ! "hash"` | fetched at commit |

### The `::` Operator — Four Uses

| Context | Meaning | Example |
|---|---|---|
| Module / type namespace | Call a function in a namespace | `Cast::string(42)`, `terminal::args()` |
| Enum variant | Construct a variant of an enum | `Color::Red()` |
| Struct function field | Call a function stored as a struct field (no self) | `handler::process("data")` |
| UFCS method lookup | When you call `value.method()`, Nova looks for `Type::method` where `Type` matches the value's type | `myOption.isSome()` → finds `Option::isSome` |

The `::` operator is Nova's universal namespace separator. It always means "reach into
this namespace and call/access something."

The UFCS lookup is how built-in methods like `.isSome()` and `.unwrap()` work on
Option values — they are registered as `Option::isSome` and `Option::unwrap`, and
Nova's method resolution finds them when you call `.isSome()` on any `Option(T)` value.

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
| `String` | `"hello"` | UTF-8 text; indexable (`"hi"[0]` → `'h'`) |
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

### Block Expressions

A `{ }` block used in expression position evaluates to the value of its
**last expression**:

```nova
let x = {
    let a = 10
    let b = 20
    a + b          // ← this is the block's value
}
// x == 30
```

Block expressions can be nested, used as function arguments, or combined
with other expressions:

```nova
let doubled = double({
    let n = 7
    n + 3
})
// doubled == 20
```

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

### The `pass` Statement

Use `pass` as a placeholder function body when you want to define a
function's signature but leave the implementation empty (like Python's
`pass`):

```nova
fn todo(x: Int) {
    pass
}
```

This is useful during iterative development when you want the code to
compile before the implementation is ready.

### Variadic Arguments (Varargs)

Any function whose **last parameter is a typed list** can be called in
"varargs" style — pass the trailing elements directly instead of wrapping
them in a list literal.  The type checker automatically collects
same-typed trailing arguments into a list when no exact signature match
exists.

```nova
fn sum(xs: [Int]) -> Int {
    let total = 0
    for x in xs {
        total = total + x
    }
    return total
}

sum([1, 2, 3])   // normal call — passes a list
sum(1, 2, 3)     // varargs call — compiler wraps into [1, 2, 3]
sum(1)           // single vararg — wraps into [1]
```

#### Leading parameters + varargs

The list must be the **last** parameter, but you can have as many
leading parameters of any type as you like:

```nova
fn tag_sum(label: String, xs: [Int]) -> String {
    let total = 0
    for x in xs { total = total + x }
    return label + Cast::string(total)
}

tag_sum("sum=", 1, 2, 3)   // "sum=6"
```

This works with any number of leading parameters and any types — closures
included:

```nova
fn apply_all(f: fn(Int) -> Int, xs: [Int]) -> [Int] {
    let result = []: Int
    for x in xs { result.push(f(x)) }
    return result
}

apply_all(|x: Int| x * 2, 10, 20, 30)   // [20, 40, 60]
```

#### Varargs and the type system

- **All** trailing arguments collected into the list must have the
  **same type**, which must match the list's element type.
- If an **exact** signature already matches the call, it takes priority.
  Varargs resolution only kicks in when there is no direct match.

```nova
fn describe(x: Int) -> String    { return "single" }
fn describe(xs: [Int]) -> String { return "list" }

describe(42)        // "single" — exact Int overload wins
describe(1, 2, 3)   // "list"   — no fn(Int,Int,Int) → varargs
```

- Works with **every type**: `Int`, `Float`, `String`, `Bool`, `Char`,
  structs, and even nested lists (`[[Int]]`).

#### Varargs with `fn extends` (UFCS)

Extends methods follow the same rule — the last parameter can be a list:

```nova
fn extends sum_with(self: Int, xs: [Int]) -> Int {
    let total = self
    for x in xs { total = total + x }
    return total
}

10.sum_with(1, 2, 3)   // 16
10.sum_with([1, 2, 3]) // 16 — explicit list still works
```

#### Quick rules

| Call style | What happens |
|---|---|
| `f([1,2,3])` | Normal list argument — no magic |
| `f(1, 2, 3)` where `f(xs: [Int])` exists | Trailing args wrapped into `[1,2,3]` |
| `f("hi", 1, 2)` where `f(s: String, xs: [Int])` exists | `"hi"` kept, `1,2` → `[1,2]` |
| `f(5)` where both `f(Int)` and `f([Int])` exist | Exact `f(Int)` wins |

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

### Auto-Infer from First Parameter

You can omit the `(Type)` after `extends` and the compiler will infer
the target type from the first parameter:

```nova
// explicit target
fn extends(Point) translate(p: Point, dx: Int, dy: Int) -> Point {
    return Point { x: p.x + dx, y: p.y + dy }
}

// auto-infer — same result
fn extends translate(p: Point, dx: Int, dy: Int) -> Point {
    return Point { x: p.x + dx, y: p.y + dy }
}
```

Auto-infer also works with built-in types:

```nova
fn extends isPositive(n: Int) -> Bool { return n > 0 }
5.isPositive()    // true
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

### `::` — Call without Self

The `::` operator calls a function stored in a struct field **without**
passing the struct as the first argument. Use it when the closure doesn't
need to know about its owner:

```nova
struct Config { transform: fn(Int) -> Int }

let cfg = Config { transform: fn(x: Int) -> Int { return x * 2 } }
cfg::transform(5)   // 10  —  transform receives only x
```

### `->` — Call with Self

The `->` operator calls a function stored in a struct field **and
prepends the struct itself** as the first argument. Use it when the
closure needs access to the struct's data:

```nova
struct Entity { name: String, greet: fn(Entity) -> String }

let e = Entity {
    name: "Nova",
    greet: fn(self: Entity) -> String {
        return "Hi, I'm " + self.name
    }
}

e->greet()   // "Hi, I'm Nova"  —  greet(e) is called
```

Both operators also work through `Dyn` types (see section 18).

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

### Generic Constructors

Use `@[T: ConcreteType]` to specify the type parameter at construction:

```nova
struct Grid(T) { cells: [$T], cols: Int, rows: Int }

fn extends(Grid) new(cols: Int, rows: Int, default: $T) -> Grid($T) {
    let cells = []: $T
    let total = cols * rows
    for let i = 0; i < total; i += 1 {
        cells.push(default)
    }
    return Grid { cells: cells, cols: cols, rows: rows }
}

let intGrid  = Grid::new(10, 10, 0)     @[T: Int]     // Grid(Int)
let boolGrid = Grid::new(5, 5, false)   @[T: Bool]    // Grid(Bool)
let strGrid  = Grid::new(3, 3, ".")     @[T: String]  // Grid(String)
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

### Concatenation

Use `+` to concatenate two lists of the same type:

```nova
let a = [1, 2, 3]
let b = [4, 5, 6]
let c = a + b      // [1, 2, 3, 4, 5, 6]
```

Both lists must have the same element type. The originals are not modified.
Works reliably with value types (`Int`, `Float`, `Bool`, `Char`).

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

### Negative Indexing

Negative indices count from the end — just like Python:

```nova
let xs = [10, 20, 30, 40, 50]
xs[-1]     // 50  — last element
xs[-2]     // 40  — second-to-last
xs[-5]     // 10  — first element

xs[-1] = 99   // write to last element (xs is now [10, 20, 30, 40, 99])
```

This works for lists, strings, and tuples. Negative indices also work
in assignment position (lists only).

```nova
let s = "hello"
s[-1]     // 'o'
s[-5]     // 'h'
```

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

### Trailing Commas

You can separate match arms with commas. Trailing commas after the last
arm are also allowed:

```nova
match color {
    Red()   => { return "red" },
    Green() => { return "green" },
    Blue()  => { return "blue" },
}
```

### Expression Arms (No Braces)

For short arms you can write a single expression instead of a block:

```nova
match direction {
    Up()   => println("up")
    Down() => println("down")
}
```

Both styles can be mixed:

```nova
match shape {
    Circle(r)  => println(Cast::string(r)),
    Point()    => { println("point") },
}
```

### With Data Extraction

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
import super.std.grid       // Grid(T) — generic 2D grid
import super.std.plot       // PlotArea — charts & graphs (raylib)
```

All imports flatten into the caller's scope — call by name, not with a prefix.

### Adding Functions to a Module (`fn mod`)

After importing a module you can inject new functions into its namespace
with `fn mod(ModuleName)`:

```nova
import super.std.math

fn mod(math) clamp(x: Int, lo: Int, hi: Int) -> Int {
    if x < lo { return lo }
    if x > hi { return hi }
    return x
}

math::clamp(15, 0, 10)   // 10
```

The module must already exist (imported or declared) before you can add
functions to it.

### Importing from GitHub

You can pull in any `.nv` file from a public GitHub repository using `import @`:

```nova
import @ "pyrotek45/nova-lang/std/core.nv"
```

The string is `"owner/repo/path/to/file.nv"`. Nova fetches from the `main`
branch by default. To lock to a specific commit, add `! "hash"`:

```nova
import @ "pyrotek45/nova-lang/std/core.nv" ! "a1b2c3d4e5f6"
```

The fetched file must contain a `module` declaration — that's how Nova knows
the module name and prevents duplicate imports. Everything the file exports
(functions, structs, enums) becomes available in the importing file's scope.

> **Note:** GitHub imports require network access. If you're working offline,
> use `nova init myproject --with owner/repo/folder` to pre-download files
> into your project's `libs/` folder, then import them locally with
> `import libs.filename`.

### Import Resolution Rules

1. **Dot-path imports** are relative to the current file's directory.
   Each dot becomes a `/`, and `.nv` is appended.
2. **`super`** translates to `..` (parent directory). Chain it for deeper paths.
3. **String literal imports** use the path as-is (relative to the current file).
4. **`@` imports** fetch from GitHub. The path before `@` is not needed —
   the module name comes from the `module` declaration inside the fetched file.
5. **Duplicate prevention:** if a module with the same name was already imported,
   the import is silently skipped regardless of import method.

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

### The `Any` Type (Advanced)

Nova has an internal `Any` type that matches any non-`None` type during
type checking. It is used by a handful of built-in functions:

| Built-in          | Signature uses `Any`                      |
| ----------------- | ----------------------------------------- |
| `print`           | `fn print(value: Any)`                    |
| `println`         | `fn println(value: Any)`                  |
| `typeof`          | `fn typeof(value: Any) -> String`         |
| `Option::isSome`  | `fn isSome(self: Option(Any)) -> Bool`    |

> **⚠ Warning:** Do **not** use `Any` in your own structs or functions.
> It bypasses type safety, loses compile-time type information, and makes
> code harder to reason about. If you need a container that works with
> multiple types, **use generics** (`$T`, `struct Grid(T) { ... }`).

**Wrong — using `Any`:**

```nova
// ❌ Don't do this — loses type info
struct BadGrid { cells: [Any], cols: Int }
fn extends set(self: BadGrid, c: Int, r: Int, val: Any) { ... }
```

**Right — using generics:**

```nova
// ✅ Correct — fully type-safe
struct Grid(T) { cells: [$T], cols: Int, rows: Int }
fn extends(Grid) set(self: Grid($T), col: Int, row: Int, value: $T) { ... }

let g = Grid::new(10, 10, 0) @[T: Int]   // Grid(Int), type-safe
g.set(0, 0, 42)                           // OK: 42 is Int
// g.set(0, 0, "hello")                   // ERROR: String ≠ Int
```

Generics give you:

- **Compile-time type checking** on every operation
- **No code bloat** — Nova uses *type erasure*, not monomorphisation. The compiler emits the **same opcodes** regardless of the concrete type, so a `Grid(Int)` and a `Grid(String)` share identical bytecode. The type parameter exists only at compile time for safety; at runtime the VM handles all values uniformly.
- **Better error messages** — the compiler knows the exact types
- **Full IDE/editor support** — auto-complete, hover info, etc.

Reserve `Any` for truly polymorphic built-in operations (printing,
debugging) where the concrete type genuinely doesn't matter.

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

### String Indexing

Strings can be indexed just like lists. Indexing returns a `Char`:

```nova
let msg = "hello"
let first = msg[0]     // 'h'
let last  = msg[-1]    // 'o'
let third = msg[2]     // 'l'
```

Negative indices count from the end: `-1` is the last character, `-2` is second-to-last, etc.

```nova
let word = "Nova"
word[0]   // 'N'
word[-1]  // 'a'
word[-2]  // 'v'
```

> **Note:** Indexing returns a `Char`, not a `String`. Use the `Char` type in annotations.

### String Concatenation

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
| Negative index | `xs[-1]` | Last element (read or write) |
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
| Generic construct  | `Grid::new(w,h,0) @[T: Int]` | Type parameter for generic struct |
| `::` (no self) | `s::fn_field()` | Call stored fn without self |
| `->` (with self) | `s->fn_field()` | Call stored fn with s as arg |
| List concat | `[1,2] + [3,4]` | Produces `[1,2,3,4]` |
| Block expression | `let x = { ...; expr }` | Last expression is the value |
| `pass` body | `fn f(x: Int) -> Int { pass }` | Placeholder (returns default) |
| `fn extends` infer | `fn extends m(s: T, ...)` | Infers extends type from first param |
| `fn mod` | `fn mod(M) f(x: Int) -> Int { }` | Add function to module M |
| Match commas | `A() => { ... }, B() => ...` | Optional commas between arms |
| Match expr arm | `A() => expr` | Single-expression arm, no braces |
| Varargs | `f(1, 2, 3)` where `f(xs: [Int])` | Trailing args auto-wrapped into list |
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
- Strings are indexable: `"hello"[0]` returns `'h'` (Char). Negative indices work: `"hello"[-1]` → `'o'`
- Use `typeof(x)` for runtime type inspection — returns the full type as a string (e.g. `"[Int]"`, `"(Int,String)"`)
- Use `clone(x)` to break aliasing — especially for lists and structs
- Use `todo() @[T: ReturnType]` as a placeholder in unfinished code
- Use `nova check file.nv` to typecheck without executing

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
| Varargs with mixed types `f(1, "hi")` | All trailing varargs must be the same type |
| Varargs on non-last param | The list param must be last in the signature |

---

## 30. Built-in Functions

Nova provides several built-in functions available in every program without imports.

### I/O Functions

| Function | Signature | Description |
|---|---|---|
| `print(x)` | `fn(Any) -> Void` | Print a value without newline |
| `println(x)` | `fn(Any) -> Void` | Print a value with newline |

`print` and `println` accept **any** type — Int, Float, String, Struct, List, etc.

### Inspection

| Function | Signature | Description |
|---|---|---|
| `typeof(x)` | `fn(Any) -> String` | Returns the runtime type name as a string |
| `clone(x)` | `fn(T) -> T` | Deep-copy a value (breaks aliasing) |

```nova
typeof(42)          // "Int"
typeof(3.14)        // "Float"
typeof("hello")     // "String"
typeof([1, 2, 3])   // "[Int]"
typeof((1, "a"))    // "(Int,String)"

struct Point { x: Float, y: Float }
let p = Point { x: 1.0, y: 2.0 }
typeof(p)           // "Point"
```

`clone` creates an independent copy. Without it, lists and structs are shared by reference:

```nova
let a = [1, 2, 3]
let b = clone(a)   // independent copy
b.push(4)           // a is still [1, 2, 3]
```

### Option Functions

| Function | Signature | Description |
|---|---|---|
| `Some(x)` | `fn(T) -> Option(T)` | Wrap a value in an Option |
| `Option::isSome(x)` | `fn(Any) -> Bool` | Check if an Option has a value |
| `Option::unwrap(x)` | `fn(Option(T)) -> T` | Extract the value (panics if None) |

`Option::isSome` and `Option::unwrap` are looked up via UFCS, so you call them
as methods: `myOption.isSome()` and `myOption.unwrap()`.

### Command-Line Arguments

| Function | Signature | Description |
|---|---|---|
| `terminal::args()` | `fn() -> Option([String])` | Get command-line arguments passed after the script name |

Returns `None` if no arguments were provided, or `Some([String])` with the list of
arguments. Arguments are everything after `nova run file.nv`:

```nova
// Run with: nova run myapp.nv hello world 42
let args = terminal::args()

if let arglist = args {
    for a in arglist {
        println("arg: " + a)
    }
} else {
    println("No arguments provided")
}

// You can also use .isSome() and .unwrap():
if args.isSome() {
    let arglist = args.unwrap()
    println("Got " + Cast::string(arglist.len()) + " args")
}
```

### Control Flow Functions

| Function | Signature | Description |
|---|---|---|
| `exit()` | `fn() -> Void` | Terminate the program immediately |
| `error()` | `fn() -> Void` | Trigger a runtime error and halt |

### Placeholder Functions

| Function | Signature | Description |
|---|---|---|
| `todo()` | `fn() -> T` | Marks unimplemented code; compiles as any type |
| `unreachable()` | `fn() -> T` | Marks code that should never run; compiles as any type |

`todo()` and `unreachable()` are generic — they satisfy **any** return type. This lets
you stub out functions during development:

```nova
fn processData(data: [Int]) -> String {
    return todo() @[T: String]
}
```

Use `unreachable()` in match branches or conditionals that logically cannot occur:

```nova
fn safeDivide(a: Int, b: Int) -> Int {
    if b == 0 {
        error()
    }
    return a / b
}

fn getSign(x: Int) -> String {
    if x > 0  { return "positive" }
    if x < 0  { return "negative" }
    return "zero"
}
```

### The `@[T: Type]` Annotation

When calling a generic function whose return type can't be inferred, provide an explicit
type annotation with `@[T: Type]`:

```nova
return todo() @[T: String]
return unreachable() @[T: Int]
```

The syntax is `@[GenericName: ConcreteType]`. Multiple type parameters are comma-separated:

```nova
let map = HashMap::default() @[K: String, V: Int]
```

---

## 31. CLI and REPL

### Command-Line Interface

Nova programs are compiled and run through the `nova` CLI tool.

| Command | Usage | Description |
|---|---|---|
| `run` | `nova run file.nv` | Compile and execute a Nova source file |
| `run` | `nova run` | Run `main.nv` in the current directory (project mode) |
| `run --git` | `nova run --git owner/repo/path.nv` | Fetch and run a file from GitHub |
| `check` | `nova check file.nv` | Typecheck without executing; reports compile time |
| `check --git` | `nova check --git owner/repo/path.nv` | Typecheck a file from GitHub |
| `time` | `nova time file.nv` | Run and print execution time in milliseconds |
| `time --git` | `nova time --git owner/repo/path.nv` | Time a file from GitHub |
| `dbg` | `nova dbg file.nv` | Run in debug mode with extra runtime information |
| `dbg --git` | `nova dbg --git owner/repo/path.nv` | Debug a file from GitHub |
| `dis` | `nova dis file.nv` | Disassemble: show the compiled bytecode |
| `dis --git` | `nova dis --git owner/repo/path.nv` | Disassemble a file from GitHub |
| `init` | `nova init myproject` | Create a new project folder with `main.nv`, `libs/`, and `tests/` |
| `init --with` | `nova init myproject --with owner/repo/folder` | Create project and fetch an entire folder from GitHub |
| `test` | `nova test` | Run all `test_*.nv` files in `tests/` and report results |
| `repl` | `nova repl` | Start the interactive REPL |
| `help` | `nova help` | Show usage information |

#### `nova run`

The primary command. Compiles the file and all its imports, then runs the resulting bytecode:

```bash
nova run my_program.nv
```

If the file has errors, they are printed with line numbers and hints, and the process exits with code 1.

**Project mode:** If you omit the file argument, Nova looks for `main.nv` in the
current directory. This lets you `cd` into any project folder and just type `nova run`:

```bash
cd myproject
nova run
# (detected Nova project: running main.nv)
# Hello from myproject!
```

This project detection also works with `check`, `time`, `dbg`, and `dis`.

#### `nova run --git`

Run a file directly from a public GitHub repository without downloading it first:

```bash
nova run --git pyrotek45/nova-lang/demo/fib.nv
```

The path format is `owner/repo/path/to/file.nv`. Nova fetches the file from
the `main` branch by default. To run from a specific commit:

```bash
nova run --git pyrotek45/nova-lang/demo/fib.nv a1b2c3d
```

The `--git` flag works with all file-based commands — `check`, `time`, `dis`,
and `dbg` — not just `run`:

```bash
nova check --git pyrotek45/nova-lang/demo/fib.nv   # type-check only
nova time  --git pyrotek45/nova-lang/demo/fib.nv   # run with timing
nova dis   --git pyrotek45/nova-lang/demo/fib.nv   # disassemble
nova dbg   --git pyrotek45/nova-lang/demo/fib.nv   # debug run
```

#### `nova check`

Parses and typechecks the file without executing it. Useful for catching errors quickly:

```bash
nova check my_program.nv
# OK | Compile time: 12ms
```

#### `nova time`

Runs the program and prints how long execution took:

```bash
nova time my_program.nv
# Execution time: 45ms
```

#### `nova dis`

Shows the disassembled bytecode — useful for understanding what the compiler generates:

```bash
nova dis my_program.nv
```

#### `nova dbg`

Runs in debug mode, which shows additional runtime information for diagnosing issues.

#### `nova init`

Creates a new project folder with a ready-to-run structure:

```bash
nova init myproject
# Created myproject/main.nv
# Created myproject/libs/
# Created myproject/tests/test_example.nv
#
# Project 'myproject' is ready!
#   cd myproject
#   nova run
#   nova test
```

This creates:
- `myproject/main.nv` — entry point with a hello-world template
- `myproject/libs/` — directory for shared modules and dependencies
- `myproject/tests/test_example.nv` — a starter test file

Use `--with` to fetch an **entire folder** from a GitHub repository into `libs/`:

```bash
nova init mygame --with pyrotek45/nova-lang/std
```

This uses the GitHub Contents API to list all `.nv` files in the specified
folder and downloads them into `libs/`. You can use multiple `--with` flags:

```bash
nova init mygame --with pyrotek45/nova-lang/std --with someuser/utils/helpers
```

After init, import the fetched files locally:

```nova
import libs.math
import libs.core
```

> **Template pattern:** Use `--with` to pull your own template libraries from
> GitHub. Keep a `template/` folder in a repo and start every project with:
> `nova init myproject --with myuser/myrepo/template`

#### `nova test`

Runs all `test_*.nv` files in the `tests/` directory and reports pass/fail results:

```bash
cd myproject
nova test
# ========================================
#   Nova Test Runner
# ========================================
# Running 3 test files from tests/
#
#   ✓ test_example
#   ✓ test_math
#   ✗ test_broken (runtime error)
#
# ========================================
#   2 passed, 1 failed
# ========================================
```

You can also specify a different test directory:

```bash
nova test path/to/tests
```

**Test file convention:**
- Files must be named `test_*.nv` (e.g., `test_math.nv`, `test_utils.nv`)
- Use `assert(condition, "message")` for test assertions
- End with `println("PASS: test_name")` for the test runner

Example test file (`tests/test_math.nv`):

```nova
module test_math

assert(1 + 1 == 2, "basic addition")
assert(10 / 2 == 5, "division")
assert(2 * 3 == 6, "multiplication")

println("PASS: test_math")
```

### Project Structure

A Nova project is any folder with a `main.nv` file. There is no config file —
Nova uses a simple convention:

```
myproject/
    main.nv          ← entry point (module main)
    libs/            ← shared modules
        math.nv      ← import with: import libs.math
        helper.nv    ← import with: import libs.helper
    tests/           ← test files (run with: nova test)
        test_math.nv ← must be named test_*.nv
    src/             ← optional: organize your own code
        game.nv      ← import from main: import src.game
```

When you run `nova run` (no file argument) from inside the project folder,
Nova automatically finds and runs `main.nv`.

Import paths are always relative to the importing file:
- From `main.nv`: `import libs.math` → `./libs/math.nv`
- From `src/game.nv`: `import super.libs.math` → `../libs/math.nv` (use `super` to go up)

### The REPL

Start with `nova repl`. The REPL lets you type Nova expressions and see results
immediately. It supports multi-line input, history, and tab completion.

| Command | Description |
|---|---|
| `help` | Show all REPL commands |
| `show` | Print the current session's accumulated code |
| `exit` | Quit the REPL |
| `clear` | Clear the terminal screen |
| `new` | Start a fresh session (discard all state) |
| `back` | Undo — revert to the previous session state |
| `session N` | Jump to session snapshot N |
| `save file.nv` | Save the current session to a `.nv` file |
| `keep CODE` | Evaluate code and always keep it in the session |
| `ast CODE` | Evaluate code, keep it, and print the AST |
| `banner` | Print a random ASCII banner |

#### Session Model

The REPL uses a **session snapshot** model. Each successful input creates a new
snapshot. You can navigate back through snapshots:

```
Session: 1  $ let x = 42
Session: 2  $ let y = x + 1
Session: 3  $ back           // reverts to session 2
Session: 2  $ session 1      // jumps to session 1
Session: 1  $
```

`new` clears everything and starts at session 1.

#### `keep` vs Regular Input

Regular input is only kept in the session if it does **not** contain `print` or `println`.
Use `keep` to force code into the session regardless:

```
Session: 1  $ keep println("debug")
debug
Session: 2  $ show
println("debug")
```

#### Saving Your Work

`save myfile.nv` writes the accumulated session code to a file, automatically
prepending `module repl`. If the file already exists, you'll be asked to confirm.

---

## 32. Novel Feature Combinations

Nova's features are designed to compose. This section shows creative ways to combine
them into powerful, concise idioms.

### Builder Pattern with Extends + Closures

Combine `extends` with closures to make fluent APIs:

```nova
struct Config { width: Int, height: Int, title: String }

fn extends withWidth(c: Config, w: Int) -> Config {
    return Config { width: w, height: c.height, title: c.title }
}
fn extends withHeight(c: Config, h: Int) -> Config {
    return Config { width: c.width, height: h, title: c.title }
}
fn extends withTitle(c: Config, t: String) -> Config {
    return Config { width: c.width, height: c.height, title: t }
}

let cfg = Config { width: 0, height: 0, title: "" }
    .withWidth(800)
    .withHeight(600)
    .withTitle("My App")
```

### Pipeline Processing with Pipe + Extends

Chain transformations using the pipe operator and extends:

```nova
fn extends double(x: Int) -> Int { return x * 2 }
fn extends addOne(x: Int) -> Int { return x + 1 }
fn square(x: Int) -> Int { return x * x }
fn negate(x: Int) -> Int { return 0 - x }

let result = 3 |> square() |> negate()  // -9
let chained = 5.double().addOne()        // 11
```

### Enum + Dyn for Polymorphic Dispatch

Use enums with pattern matching for type-safe dispatch, or `Dyn` for structural contracts:

```nova
// Approach 1: Enum-based dispatch
enum Shape { Circle: Float, Rect: (Float, Float) }

fn extends area(s: Shape) -> Float {
    return match s {
        Circle(r) => 3.14159 * r * r,
        Rect(wh) => wh.0 * wh.1,
    }
}

// Approach 2: Dyn-based dispatch (structural)
struct Drawable { draw: fn(Int, Int) -> Void }

fn renderAll(items: [Dyn(Drawable)], x: Int, y: Int) {
    for item in items {
        item.draw(x, y)
    }
}
```

### Varargs + Extends for Natural APIs

Varargs let extends methods accept flexible argument lists:

```nova
fn extends containsAll(haystack: [String], needles: [String]) -> Bool {
    for n in needles {
        let found = false
        for h in haystack {
            if h == n { found = true }
        }
        if !found { return false }
    }
    return true
}

let tags = ["nova", "lang", "vm"]
tags.containsAll("nova", "vm")    // true — varargs pack into [String]
```

### Block Expressions + Let Bindings

Use block expressions to compute complex initial values:

```nova
let direction = {
    if angle < 90   { "north" }
    elif angle < 180 { "east"  }
    elif angle < 270 { "south" }
    else             { "west"  }
}
```

### Match Expression Arms for Inline Decisions

Match expressions return values, perfect for assigning computed results:

```nova
enum Priority { High, Medium, Low }

fn extends color(p: Priority) -> String {
    return match p {
        High()   => "red",
        Medium() => "yellow",
        Low()    => "green",
    }
}
```

### Currying + Higher-Order Functions

Create specialized functions from general ones:

```nova
fn adder(n: Int) -> fn(Int) -> Int {
    return fn(x: Int) -> Int { return x + n }
}

let add5 = adder(5)
let add10 = adder(10)
add5(3)    // 8
add10(3)   // 13

// Use with list operations
let nums = [1, 2, 3, 4, 5]
let incremented = nums.map(adder(1))  // [2, 3, 4, 5, 6]
```

### Module Namespaces + Extends for Library Design

Use `fn mod(Module)` to organize library functions alongside extends:

```nova
fn mod(MathUtils) clamp(val: Int, lo: Int, hi: Int) -> Int {
    if val < lo { return lo }
    if val > hi { return hi }
    return val
}

fn extends clampTo(x: Int, lo: Int, hi: Int) -> Int {
    return MathUtils::clamp(x, lo, hi)
}

// Both calling styles work:
MathUtils::clamp(150, 0, 100)  // 100
150.clampTo(0, 100)             // 100
```

### Generics + Dunder Operators for Reusable Types

Define custom container types with operator overloading:

```nova
struct Stack(T) { data: [$T] }

fn extends push(s: Stack(T), val: $T) {
    s.data.push(val)
}

fn extends pop(s: Stack(T)) -> Option($T) {
    if s.data.len() == 0 { return None(T) }
    return Some(s.data.remove(s.data.len() - 1))
}

fn extends __eq__(a: Stack(T), b: Stack(T)) -> Bool {
    return a.data == b.data
}

fn extends len(s: Stack(T)) -> Int {
    return s.data.len()
}
```

### If-Let Chains for Safe Nested Unwrapping

Chain `if let` statements to safely unwrap nested Option values:

```nova
fn findUserName(db: Database, id: Int) -> String {
    if let user = db.find(id) {
        if let name = user.displayName {
            return name
        }
    }
    return "unknown"
}
```

### Closure Capture + Extends for Stateful Methods

Closures capture their environment, creating lightweight stateful objects:

```nova
fn counter(start: Int) -> fn() -> Int {
    let n = start
    return fn() -> Int {
        n = n + 1
        return n
    }
}

let c = counter(0)
c()  // 1
c()  // 2
c()  // 3
```

### Arrow Syntax for Compact Struct Field Access

The `->` operator combines field access with a function call:

```nova
struct EventHandler { onClick: fn(Int) -> Void }

fn fireClick(handler: EventHandler, x: Int) {
    handler->onClick(x)    // same as handler.onClick(x) but calls the fn
}
```

### for-in + Enumerate Pattern

Use range and indexing together for index-value iteration:

```nova
let names = ["Alice", "Bob", "Carol"]
for i in 0..names.len() {
    println(format("{}: {}", [Cast::string(i), names[i]]))
}
```

### Forward Declarations for Mutual Recursion

Declare a function signature before defining it:

```nova
fn isEven(n: Int) -> Bool           // forward declaration

fn isOdd(n: Int) -> Bool {
    if n == 0 { return false }
    return isEven(n - 1)
}

fn isEven(n: Int) -> Bool {         // definition
    if n == 0 { return true }
    return isOdd(n - 1)
}
```

---

# Part II — Game Development

---

## 33. Quick Start — Your First Window

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

## 34. Critical Rules for Game Dev

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

## 35. Scene Management

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

## 36. Entity System

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

## 37. Input Handling

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

## 38. Physics and Collision

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

## 39. Camera

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

## 40. Timers and Tweens

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

## 41. Vec2 Math

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

## 42. Tilemaps and Noise

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

## 43. Sprites and Audio

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

## 44. HUD and UI

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

## 45. Advanced Game Patterns

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

## 46. Game Dev Tips and Tricks

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

## 47. Performance Tips

| Problem | Solution |
|---|---|
| GC spikes from spawning | Object pool with `active` flag |
| O(n²) collision | Spatial grid — insert then query |
| Temporary lists each frame | Pre-allocate outside loop, `clear()` inside |
| Off-screen entities drawn | Cull with `cam.isVisible(x, y, margin)` |
| `Cast::string` in tight loop | Cache the string; regenerate only on change |
| Music stops | Call `raylib::updateMusic(id)` every frame |

---

## 48. Complete Example — Breakout

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

## 49. Complete Example — Top-Down Shooter

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

## 50. Terminal Quick Start

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

## 51. Raw Mode and Key Input

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

## 52. Colours and Cursor

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

## 53. Terminal Menu System

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

## 54. Terminal Game Loop

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

## 55. Terminal Patterns

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

## 56. Nova for Python Developers

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

---

## 57. Charts and Plotting

Nova ships with `std/plot` — a charting library built on raylib.

### Setup

```nova
module main
import super.std.plot

raylib::init("My Chart", 800, 600, 30)
```

### PlotArea

Every chart lives inside a `PlotArea` that maps data coordinates to screen pixels:

```nova
// Manual bounds
let area = PlotArea::new(50, 50, 700, 400, 0.0, 10.0, 0.0, 100.0)

// Auto-detect bounds from data
let data = [3.0, 7.0, 2.0, 9.0, 5.0]
let area = PlotArea::auto(50, 50, 700, 400, data)
```

### Drawing Charts

All chart functions are `extends` methods on `PlotArea`, called inside
a `while raylib::rendering() { ... }` loop:

```nova
while raylib::rendering() {
    raylib::clear((25, 25, 30))

    // Decorations
    area.drawGrid(5, 4, (50, 50, 55))
    area.drawAxes((150, 150, 150))
    area.drawBorder((100, 100, 100))
    area.drawTitle("Sales", 20, (230, 230, 230))

    // Charts (overlay multiple on the same area)
    area.barChart(data, (60, 120, 220))
    area.lineChart(data, (220, 60, 60))
    area.fillArea(data, (60, 200, 80))

    // Scatter plot (takes list of (Float, Float) tuples)
    let pts = [(1.0, 2.0), (3.0, 7.0), (5.0, 4.0)]
    area.scatter(pts, 4, (200, 80, 200))

    // Reference lines
    area.hLine(5.0, (200, 60, 60))   // horizontal at y=5
    area.vLine(3.0, (60, 200, 80))   // vertical at x=3
}
```

### Pie Charts

Pie charts are standalone functions (not PlotArea methods):

```nova
let pieData   = [35.0, 25.0, 20.0, 12.0, 8.0]
let labels    = ["Rust", "Nova", "C", "Go", "Lua"]
let colors    = [(220,60,60), (60,120,220), (60,200,80), (230,160,40), (160,80,200)]

// Basic pie
drawPieChart(400, 300, 100, pieData, colors)

// Pie with labels
drawPieChartLabeled(400, 300, 100, pieData, labels, colors, (230,230,230), 14)
```

### Demo

See `demo/plotdemo.nv` for a complete 7-chart showcase:
line chart, bar chart, scatter, filled area, multi-line overlay,
thick line with reference lines, and a labeled pie chart — all in one window.
