


# How to Write Nova

Nova is a statically typed, expression-oriented programming language with garbage collection,
universal function call syntax (UFCS), first-class functions, and a structural dynamic-dispatch
system called Dyn types. This guide covers every major feature with working examples.

---

## Table of Contents

1. [Module System](#1-module-system)
2. [Variables and Types](#2-variables-and-types)
3. [Built-in Types](#3-built-in-types)
4. [Operators](#4-operators)
5. [Control Flow](#5-control-flow)
6. [Functions](#6-functions)
7. [Extends / UFCS](#7-extends--ufcs)
8. [Structs](#8-structs)
9. [Enums](#9-enums)
10. [Generics](#10-generics)
11. [Option Type](#11-option-type)
12. [Closures and Lambdas](#12-closures-and-lambdas)
13. [Lists](#13-lists)
14. [Tuples](#14-tuples)
15. [Pattern Matching](#15-pattern-matching)
16. [Pipe Operator](#16-pipe-operator)
17. [Dyn Types](#17-dyn-types)
18. [Box and Mutable Shared State](#18-box-and-mutable-shared-state)
19. [Imports and the Standard Library](#19-imports-and-the-standard-library)
20. [Type System Rules](#20-type-system-rules)
21. [Memory Model Deep Dive](#21-memory-model-deep-dive)
22. [Cast and Type Conversion](#22-cast-and-type-conversion)
23. [Iterators](#23-iterators)
24. [String Operations](#24-string-operations)
25. [Design Patterns Without OOP](#25-design-patterns-without-oop)
26. [The Fuzzer](#26-the-fuzzer)
27. [Syntax Sugar Reference](#27-syntax-sugar-reference)
28. [Tips and Tricks](#28-tips-and-tricks)
29. [Quick Reference: Common Mistakes](#29-quick-reference-common-mistakes)

---

## 1. Module System

Every Nova source file must begin with a `module` declaration. The module name is an
identifier that registers the file with the compiler.

```nova
module my_module

// all code goes here
```

There is no way to have code at the top level without a `module` declaration — the parser
will reject the file.



### Importing modules

To import another module, use the `import` statement. The path is resolved relative to the
current file's directory. The special identifier `super` means "go up one directory."

```nova
module main

import super.std.core     // imports ../std/core.nv (relative to this file)
import super.std.list
import super.std.math
```

Standard library modules live in the `std/` folder of the project root. When your file is
inside a subfolder (e.g., `demo/`, `tests/`), you need `super` to reach `std/`.

### What importing does

**Import flattens all definitions into the caller's scope.** When you write
`import super.std.math`, every function, struct, and enum defined in `std/math.nv` becomes
directly available — you call them by name, not with a module prefix:

```nova
module main
import super.std.math

// These are all directly available after importing math:
let b = bin(10)                  // "1010"
let h = hex(255)                 // "ff"
let r = lerp(0.0, 10.0, 0.5)    // 5.0
let f = fib(10)                  // 55

// Extension methods are also available via dot notation:
let v = (-7).abs()               // 7
let m = 5.min(3)                 // 3
```

> **Important:** There is no `module::function()` namespace syntax for calling regular
> functions. `math::bin(10)` is a **compile error**. Just call `bin(10)` directly.

### Re-importing and deduplication

If a module has already been imported (directly or transitively), importing it again is a
no-op — the parser skips it. This means you can safely import the same module in multiple
files without duplicate-definition errors.

### Module names reserve identifiers

When a module is imported, its module name becomes a reserved identifier in the current
scope. **You cannot use a module name as a variable name.** For example, if you import a
file whose first line is `module result`, then `let result = ...` will be a compile error
in any file that (transitively) imports it.

This is why the standard library module is named `core` rather than having `core.nv`
import separate `result.nv` and `maybe.nv` files — a module named `result` would prevent
the extremely common `let result = ...` pattern everywhere.

> Use `fn mod(ModuleName) funcName()` only when you want to force a function to be called as `ModuleName::funcName()` instead of being flattened into the importer's scope. This is useful for:
> - Avoiding name collisions for utility functions that are too generic (like `init`, `run`, `main`, or `config`)
> - Creating APIs where you want to make it clear a function is part of a specific module
> - Exposing only a few namespaced entry points from a large module, while keeping helpers private
>
> **Example:**
>
> ```nova
> module logger
>
> fn mod(logger) init() { ... }
> fn mod(logger) log(msg: String) { ... }
>
> // Usage (after import logger):
> logger::init()
> logger::log("hello")
> ```
>
> For most standard library and application code, you do **not** need `mod`—just use regular functions and `extends` patterns. Use `mod` only when you want to enforce explicit namespacing for clarity or to prevent accidental shadowing.
# How to Write Nova

Nova is a statically typed, expression-oriented programming language with garbage collection,
universal function call syntax (UFCS), first-class functions, and a structural dynamic-dispatch
system called Dyn types. This guide covers every major feature with working examples.

---

**Tip:** Choose module names that won't collide with common variable names. Prefer
descriptive names like `string_utils`, `game_state`, or `my_module` over generic words
like `data`, `value`, or `result`.

### The `::` operator

The `::` operator has three distinct uses in Nova. None of them are general namespace access:

**1. Enum variant construction** — `EnumName::Variant(value)`:

```nova
import super.std.core

let r = Result::Ok(42) @[B: String]
let e = Result::Err("bad") @[A: Int]
let m = Maybe::Just(10)
let n = Maybe::Nothing() @[A: Int]
```

**2. Type-level static functions** — `TypeName::function()`:

These are created with `fn extends(TypeName)` syntax and are called on the type name, not
on an instance. They are typically constructors or factory functions:

```nova
// Definition (in std/hashmap.nv):
fn extends(HashMap) default() -> HashMap($K,$V) { ... }

// Usage:
let map = HashMap::default() @[K: String, V: Int]

// Definition (in std/iter.nv):
fn extends(Iter) fromVec(input: [$A]) -> Iter($A) { ... }

// Usage:
let it = Iter::fromVec([1, 2, 3])
```

This also works for built-in types: `Cast::string()`, `Cast::int()`, `Cast::float()`,
`Float::pi()`, `Float::e()`.

**3. Struct function-field call (no self)** — `instance::field()`:

When a struct has a function stored in a field, `::` calls it without passing the struct
as an argument:

```nova
struct Formatter { format: fn(String) -> String }
let f = Formatter { format: |s: String| s.toUpper() }
f::format("hello")   // calls format("hello") — no self passed
```

Compare with `->` which passes the struct as the first argument:

```nova
struct Handler { handle: fn(Handler, String) -> String }
let h = Handler { handle: fn(self: Handler, msg: String) -> String {
    return "handled: " + msg
}}
h->handle("test")    // calls handle(h, "test") — h passed as self
```

### Instance methods vs. static functions vs. module functions

Nova has three `extends` forms. Understanding the difference is key:

**Instance methods** — `fn extends funcName(self: Type)`:

The first parameter is the receiver. Called with dot notation (UFCS):

```nova
fn extends double(x: Int) -> Int { return x * 2 }
5.double()   // 10
```

**Type-level static functions** — `fn extends(TypeName) funcName(...)`:

No self parameter. Called with `TypeName::funcName()`:

```nova
fn extends(Point) origin() -> Point {
    return Point { x: 0, y: 0 }
}
Point::origin()   // creates a new Point
```

**Module-level functions** — `fn mod(ModuleName) funcName(...)`:

Rare. Creates a function callable as `ModuleName::funcName()`. The module must already be
declared (via `module ModuleName` at the top of a file that has been imported):

```nova
// In some_module.nv:
module some_module

fn mod(some_module) greet() -> String {
    return "hello from module"
}

// In another file:
import some_module
some_module::greet()   // "hello from module"
```

> **When to use which:** Most code uses instance methods (`fn extends func(self: Type)`)
> and type-level static functions (`fn extends(Type) func()`). Module-level functions
> (`fn mod(...)`) are rarely needed since import already flattens everything into
> scope.

### Design pattern: Structs as namespaces

Since `fn extends(TypeName) funcName()` creates a function callable as
`TypeName::funcName()`, you can use a struct as a **namespace** to group related
functions under a common prefix. This is Nova's idiomatic way to build "toolbox" modules
with clean, collision-free APIs.

**Basic pattern — empty struct as a namespace:**

```nova
module string_utils

// The struct acts as a namespace. It has no fields.
struct StringUtils {}

fn extends(StringUtils) repeat(s: String, n: Int) -> String {
    let result = ""
    for let i = 0; i < n; i += 1 { result = result + s }
    return result
}

fn extends(StringUtils) isPalindrome(s: String) -> Bool {
    return s == s.reverse()
}

// Usage:
StringUtils::repeat("ha", 3)       // "hahaha"
StringUtils::isPalindrome("racecar") // true
```

**Constructor pattern — `@[]` for generic type hints:**

When a struct has generic type parameters, the caller may need to supply types that
can't be inferred from the arguments. The `@[Param: ConcreteType]` syntax provides
these type hints at the call site:

```nova
fn extends(HashMap) default() -> HashMap($K,$V) {
    let nb = 16
    return HashMap {
        buckets: _makeBuckets(nb) @[K: $K, V: $V],
        numBuckets: nb,
        count: 0,
    }
}

// At the call site, K and V can't be inferred from zero arguments,
// so you supply them with @[]:
let map = HashMap::default() @[K: String, V: Int]
```

The `@[]` annotation is only needed when the compiler can't infer the generic
parameters from context. If the function's arguments already determine the types,
no annotation is needed:

```nova
fn extends(Iter) fromVec(input: [$A]) -> Iter($A) { ... }

// $A is inferred from the argument type — no @[] needed:
let it = Iter::fromVec([1, 2, 3])   // Iter(Int)
```

**Real-world example — the standard library HashMap:**

The `HashMap` struct in `std/hashmap.nv` demonstrates this pattern perfectly.
The struct is both a data container **and** a namespace for its own constructor:

```nova
import super.std.hashmap

// Construct via static function (TypeName::func):
let scores = HashMap::default() @[K: String, V: Int]

// Instance methods via dot notation (UFCS):
scores.insert("Alice", 100)
scores.insert("Bob", 85)
let v = scores.get("Alice").orDefault(0)   // 100
```

**When to use this pattern:**

- Group related utility functions under a descriptive type name
- Create multiple named constructors for a type (`Point::origin()`, `Point::fromAngle()`)
- Build "toolbox" libraries where collision-free naming matters
- Provide factory functions that configure complex structs with sensible defaults

---

## 2. Variables and Types

Variables are declared with `let`. Nova is statically typed -- every variable has a fixed type
that is set at declaration time and never changes. Nova has no type inference beyond the initial
binding: the type of `x` is exactly the type of the expression on the right.

```nova
let x = 42          // x : Int
let y = 3.14        // y : Float
let s = "hello"     // s : String
let b = true        // b : Bool
let c = 'A'         // c : Char
```

You can add an explicit type annotation:

```nova
let x: Int = 42
let name: String = "Alice"
```

Reassignment uses bare `=` (no `let`):

```nova
let count = 0
count = count + 1   // OK -- same type Int
count = "hello"     // COMPILE ERROR -- type mismatch
```

Compound assignment operators (`+=`, `-=`, `*=`, `/=`) return Void, so they cannot be used as
expressions. Use them only as statements:

```nova
let x = 10
x += 5      // OK as a statement
x -= 2      // OK
// let y = (x += 1)  // ERROR -- Void cannot be assigned
```

---

## 3. Built-in Types

| Type | Description | Example |
|---|---|---|
| `Int` | 64-bit signed integer | `42`, `-7`, `0` |
| `Float` | 64-bit floating point | `3.14`, `-0.5` |
| `Bool` | Boolean | `true`, `false` |
| `String` | UTF-8 text | `"hello"`, `""` |
| `Char` | Single Unicode character | `'a'`, `'\n'`, `'\t'` |
| `Void` | No value (function return) | -- |
| `Option(T)` | Maybe a value of type T | `Some(42)`, `None(Int)` |
| `[T]` | List of T | `[1,2,3]`, `[]: Int` |
| `(A, B, ...)` | Tuple | `(1, "hello", true)` |
| `fn(A,B) -> R` | Function type | `fn(Int) -> Int` |

---

## 4. Operators

### Arithmetic

```nova
let a = 10 + 3    // 13
let b = 10 - 3    // 7
let c = 10 * 3    // 30
let d = 10 / 3    // 3  (integer division)
let e = 10 % 3    // 1  (Euclidean modulo -- always non-negative)
let f = -10 % 3   // 2  (NOT -1; Nova uses Euclidean modulo)
```

### Comparison

```nova
1 == 1      // true
1 != 2      // true
3 < 5       // true
5 > 3       // true
3 <= 3      // true
4 >= 5      // false
```

### Boolean -- Precedence Warning

Nova's `||` binds more tightly than `&&` (opposite of most languages). Always use parentheses
when mixing:

```nova
// Surprising -- may not do what you expect:
let r = true || false && false   // = (true || false) && false = false

// Safe -- use parens to be explicit:
let r = (true || false) && (true || false)   // = true
```

### Unary

```nova
let neg = -5
let not = !true   // false
```

---

## 5. Control Flow

### if / elif / else

```nova
let x = 42

if x > 100 {
    println("big")
} elif x > 10 {
    println("medium")
} else {
    println("small")
}
```

The condition must be `Bool` -- using an `Int` or any other type is a compile error.

### for (C-style)

```nova
for let i = 0; i < 10; i += 1 {
    println(i)
}
```

### for-in (range)

```nova
for i in 0..10 {
    println(i)   // prints 0 through 9
}
```

### for-in (collection)

```nova
let items = ["a", "b", "c"]
for item in items {
    println(item)
}
```

### while

```nova
let n = 10
while n > 0 {
    n = n - 1
}
```

### if let (option unwrap)

```nova
let opt: Option(Int) = Some(42)
if let value = opt {
    println(value)   // value is Int here
}
```

`if let` can have an `else` branch for the `None` case:

```nova
let opt: Option(Int) = None(Int)
if let value = opt {
    println(value)
} else {
    println("no value")
}
```

### while let (loop while Some)

`while let` repeatedly unwraps an option and runs the body while it's `Some`:

```nova
let items = [1, 2, 3]
while let val = items.pop() {
    println(val)   // prints 3, 2, 1
}
```

---

## 6. Functions

### Basic functions

```nova
fn add(a: Int, b: Int) -> Int {
    return a + b
}

let result = add(3, 4)   // 7
```

### Void functions

A function with no return type annotation returns `Void`:

```nova
fn greet(name: String) {
    println("Hello, " + name + "!")
}
greet("Alice")
```

### Overloading

Multiple functions may share a name if their parameter types differ:

```nova
fn describe(x: Int) -> String { return "int" }
fn describe(x: Float) -> String { return "float" }
fn describe(x: String) -> String { return "string" }

describe(1)       // "int"
describe(1.0)     // "float"
describe("hi")    // "string"
```

### Function references with `@`

To pass a named function as a value, use `name@(ArgTypes...)`:

```nova
fn square(x: Int) -> Int { return x * x }

fn apply(f: fn(Int) -> Int, x: Int) -> Int {
    return f(x)
}

apply(square@(Int), 5)   // 25
```

For overloaded functions, the type signature disambiguates:

```nova
let int_add = add@(Int, Int)
let float_add = add@(Float, Float)
```

### Recursive functions

```nova
fn factorial(n: Int) -> Int {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}
```

> **Note:** Nova supports forward declarations for mutual recursion. Declare the function
> signature (with no body) before its first use, then provide the full definition later:
>
> ```nova
> fn isEven(n: Int) -> Bool        // forward declaration
> fn isOdd(n: Int) -> Bool         // forward declaration
>
> fn isEven(n: Int) -> Bool {
>     if n == 0 { return true }
>     return isOdd(n - 1)
> }
>
> fn isOdd(n: Int) -> Bool {
>     if n == 0 { return false }
>     return isEven(n - 1)
> }
> ```

---

## 7. Extends / UFCS

The `extends` keyword makes a function callable with Universal Function Call Syntax. The first
parameter becomes the receiver.

```nova
fn extends double(x: Int) -> Int {
    return x * 2
}

let b = 5.double()   // 10 -- UFCS syntax
```

Extends functions can only be called with UFCS dot syntax. Calling them as regular functions
is a compile error.

### Chaining

UFCS calls chain naturally left-to-right:

```nova
fn extends square(x: Int) -> Int { return x * x }
fn extends negate(x: Int) -> Int { return -x }

let r = 5.double().square().negate()   // ((5*2)^2) * -1 = -100
```

### Overloaded extends

Different receiver types can share the same extends function name:

```nova
fn extends describe(x: Int) -> String { return "Int: " + Cast::string(x) }
fn extends describe(x: String) -> String { return "String: " + x }

42.describe()       // "Int: 42"
"hi".describe()     // "String: hi"
```

### Extends on structs

```nova
struct Point { x: Int, y: Int }

fn extends magnitude(p: Point) -> Float {
    let sum = Cast::float(p.x * p.x + p.y * p.y)
    return sum.unwrap()
}
```

### The `->` dispatch operator

When a struct field holds a function, `->` calls that function passing the struct as its first
argument:

```nova
struct Button {
    label: String,
    on_click: fn(Button)
}

let btn = Button {
    label: "OK",
    on_click: fn(self: Button) { println("Clicked: " + self.label) }
}

btn->on_click()   // calls on_click(btn)
```

The `::` operator calls a struct's function field without passing self:

```nova
btn::on_click()   // calls on_click() with no arguments
```

---

## 8. Structs

### Basic struct

```nova
struct Person {
    name: String,
    age: Int
}

let p = Person { name: "Alice", age: 30 }
println(p.name)   // "Alice"
p.age = 31        // field mutation
```

### Positional initialization

```nova
let p2 = Person("Bob", 25)   // positional, must match field order
```

### Generic struct

```nova
struct Box(T) {
    value: $T
}

let b = Box { value: 42 }       // Box(Int)
let s = Box { value: "hello" }  // Box(String)
```

### Structs with function fields

```nova
struct Timer {
    ticks: Int,
    tick: fn(Timer)
}

let t = Timer {
    ticks: 0,
    tick: fn(self: Timer) { self.ticks = self.ticks + 1 }
}

t->tick()
t->tick()
// t.ticks is now 2
```

### The `type` field

Every struct instance automatically has a `type` field that stores the struct's name as a
string:

```nova
struct Dog { name: String }
let d = Dog { name: "Rex" }
println(d.type)   // "Dog"
```

---

## 9. Enums

```nova
enum Shape {
    Circle: Float,              // carries a Float (radius)
    Rectangle: (Float, Float),  // carries a tuple
    Triangle                    // no payload
}
```

Construct variants with `EnumName::VariantName(value)`:

```nova
let c = Shape::Circle(3.14)
let r = Shape::Rectangle((4.0, 5.0))
let t = Shape::Triangle()   // no-payload variant still needs ()
```

Match on enum values:

```nova
fn area(s: Shape) -> Float {
    match s {
        Circle(r) => { return 3.14159 * r * r }
        Rectangle(dims) => { return dims[0] * dims[1] }
        Triangle() => { return 0.0 }
    }
    return 0.0
}
```

---

## 10. Generics

Generic functions use `$T`-prefixed type variables:

```nova
fn extends first(list: [$T]) -> Option($T) {
    if list.len() == 0 {
        return None($T)
    }
    return Some(list[0])
}
```

Generic structs declare type parameters in the struct header:

```nova
struct Pair(A, B) {
    fst: $A,
    snd: $B
}

fn extends swap(p: Pair($A, $B)) -> Pair($B, $A) {
    return Pair { fst: p.snd, snd: p.fst }
}
```

> Nova has no type inference. Every type variable must be inferrable from the argument types at
> the call site.

> Raw generic extends (`fn extends id(x: $A)`) is not supported. Generic extends functions must
> have a concrete type constructor as the receiver, e.g. `Wrapper($T)`.

---

## 11. Option Type

### The built-in Option type

`Option(T)` is a **VM-level construct** — not a struct or enum. It is baked into the virtual
machine itself. A value of type `Option(T)` is either:

- `Some(value)` — the value is present (stored directly, no wrapper object)
- `None(T)` — no value (a zero-cost null sentinel)

```nova
let found: Option(Int) = Some(42)
let empty: Option(Int) = None(Int)

if found.isSome() {
    println(found.unwrap())   // 42
}
```

**Built-in methods on `Option(T)`:**

| Method | Description |
|---|---|
| `.isSome() -> Bool` | True if a value is present |
| `.unwrap() -> T` | Extract the value (runtime error if None) |

Use `if let` for safe unwrapping — it only executes the body if the option holds a value:

```nova
if let value = found {
    println(value)
}
```

`while let` loops until the option becomes None:

```nova
let gen = Gen(0)        // from std/core
while let n = gen() {   // stops when generator returns None
    println(n)
}
```

Standard library helpers (from `std/core.nv` or `std/option.nv`):

```nova
import super.std.core

let x: Option(Int) = None(Int)
let v  = x.orDefault(0)          // 0 — return default if None
let v2 = x.orDoFn(|| 99)         // 99 — compute lazily if None
let v3 = x.orError("Expected")   // exits with message if None
let ok = x.isNone()              // true if None
```

### Option vs Maybe — what's the difference?

Nova has two "nullable" concepts that look similar but are fundamentally different at the VM
level:

| | `Option(T)` | `Maybe(T)` |
|---|---|---|
| **Kind** | Built-in VM primitive | User-defined enum (from `std/core`) |
| **Representation** | A direct value or a null sentinel | A heap-allocated enum object |
| **Construction** | `Some(42)` / `None(Int)` | `Maybe::Just(42)` / `Maybe::Nothing()` |
| **Pattern matching** | `if let` / `while let` / `.isSome()` | `match` on `Just`/`Nothing` |
| **Performance** | Zero allocation — no heap object | Allocates an enum object on the heap |
| **Use case** | Return values, optional parameters | When you need to pattern match, store in a list, or pass to generic code that uses `match` |

```nova
import super.std.core

// Option(T) — the lightweight VM primitive
let a: Option(Int) = Some(10)
if let x = a { println(x) }          // pattern: if let

// Maybe(T) — the full enum (from std/core)
let b = Maybe::Just(10)
match b {
    Just(x)   => { println(x) }
    Nothing() => { println("nothing") }
}

// Converting between them:
let opt: Option(Int) = Some(42)
let maybe = opt.toMaybe()             // Option -> Maybe::Just(42)
```

**Prefer `Option(T)`** for return values and function parameters — it's faster and integrates
with `if let` / `while let`. **Use `Maybe(T)`** when you need to store nullable values in a
list, match exhaustively across enum variants, or use it as an enum in generic code.

### Functions that return Option

The standard library's `Cast::int`, `Cast::float`, `xs.pop()`, and many others return
`Option(T)`:

```nova
let n = Cast::int("42")           // Option(Int)
let f = Cast::float("3.14")       // Option(Float)

// Handle the option
if let parsed = Cast::int("abc") {
    println(parsed)
} else {
    println("not a number")
}

// Or use orDefault
let safe = Cast::int("abc").orDefault(0)   // 0
```

---

## 12. Closures and Lambdas

### Full closure syntax

```nova
let add = fn(x: Int, y: Int) -> Int {
    return x + y
}
add(3, 4)   // 7
```

### Short lambda syntax

```nova
let square = |x: Int| x * x
square(5)   // 25
```

Closures capture variables from the surrounding scope. By default, captured values are copied.
To share mutable state across closures, use `Box`:

```nova
import super.std.core

let counter = Box(0)

let inc = fn() -> Int {
    counter.value = counter.value + 1
    return counter.value
}

inc()   // 1
inc()   // 2
inc()   // 3
```

### Closures as function arguments

```nova
let nums = [1, 2, 3, 4, 5]
let doubled = nums.map(|x: Int| x * 2)        // [2, 4, 6, 8, 10]
let evens = nums.filter(|x: Int| x % 2 == 0)  // [2, 4]
```

### Trailing closures

When the last argument to a function is a closure, you can pass it **after** the closing
parenthesis using `:` syntax. This makes function calls read more naturally:

```nova
fn apply(x: Int, f: fn(Int) -> Int) -> Int {
    return f(x)
}

// Normal call:
let a = apply(5, |x: Int| x * 2)

// Trailing closure — same result, cleaner look:
let b = apply(5): |x: Int| x * 2
```

Trailing closures work with UFCS too:

```nova
fn extends transform(x: Int, f: fn(Int) -> Int) -> Int {
    return f(x)
}

let result = 10.transform(): |x: Int| x + 5   // 15
```

This is especially useful with higher-order functions:

```nova
import super.std.list
let nums = [1, 2, 3, 4, 5]
let big = nums.filter(): |x: Int| x > 3       // [4, 5]
```

### The bind operator (~>)

The `~>` operator lets you name the result of an expression and use it in a block. The block's
value becomes the overall expression value:

```nova
// expr ~> name { block }
let result = 42 ~> x { x + 8 }
// result == 50

// Name a complex sub-expression
let len_sq = [1, 2, 3, 4, 5].len() ~> n { n * n }
// len_sq == 25

// Useful for intermediate calculations
let area = (3 + 4) ~> side { side * side }
// area == 49
```

Think of `~>` as a lightweight `let` that works inline in expressions.

---

## 13. Lists

### Construction

```nova
let xs = [1, 2, 3]          // [Int]
let empty = []: Int          // empty list -- type annotation required
let strs = ["a", "b", "c"]  // [String]
```

### Operations

```nova
xs.push(4)       // append
xs.pop()         // remove last, returns Option(Int)
xs.len()         // length
xs[0]            // index (0-based)
xs[0] = 99       // element assignment
```

### Standard library list functions

```nova
import super.std.list

[1,2,3].map(|x: Int| x * 2)                                   // [2,4,6]
[1,2,3,4].filter(|x: Int| x > 2)                              // [3,4]
[1,2,3].reduce(|acc: Int, x: Int, i: Int| acc + x, 0)         // 6
[1,2,3].foreach(|x: Int| { println(x) })
[[1,2],[3,4]].flatten()                                        // [1,2,3,4]
[3,1,2].bubblesort()                                           // [1,2,3]
[5,3,1].sortWith(|a: Int, b: Int| a > b)                      // descending
[1,2,3].concat([4,5,6])                                        // [1,2,3,4,5,6]
[0]: Int.fill(0, 5)                                            // [0,0,0,0,0]
```

### List Slicing

Nova supports Python-style list slicing with `[start:end]` syntax. Slicing returns a **new
list** containing the selected elements.

```nova
let xs = [10, 20, 30, 40, 50]

xs[0:3]     // [10, 20, 30]      — elements 0, 1, 2
xs[2:4]     // [30, 40]          — elements 2, 3
xs[2:]      // [30, 40, 50]      — from index 2 to end
xs[:3]      // [10, 20, 30]      — from start to index 3
xs[:]       // [10, 20, 30, 40, 50]  — full copy
```

### Negative Slice Indices

Negative indices count from the end of the list, just like Python:

```nova
let xs = [10, 20, 30, 40, 50]

xs[-2:]     // [40, 50]          — last 2 elements
xs[:-2]     // [10, 20, 30]      — all but last 2
xs[-3:-1]   // [30, 40]          — from 3rd-last to 2nd-last
xs[-1:]     // [50]              — last element as a list
xs[1:-1]    // [20, 30, 40]      — drop first and last
```

### Slice with Step (Stride)

Use `$` after the end index to specify a step size:

```nova
let xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

xs[0:8$2]   // [0, 2, 4, 6]     — every 2nd element from 0 to 8
xs[:$3]     // [0, 3, 6, 9]     — every 3rd element
xs[1:$2]    // [1, 3, 5, 7, 9]  — every 2nd starting at 1
xs[2:8$3]   // [2, 5]           — every 3rd from 2 to 8
xs[-4:-1$1] // [6, 7, 8]        — negative indices work with step too
```

The syntax is `list[start:end$step]`. Both `start` and `end` can be omitted. The step
defaults to 1 when not specified.

> **Note:** Negative indices work with slicing but NOT with regular indexing.
> `xs[-1]` is a runtime error. Use `xs[-1:]` and take the first element, or
> `xs[xs.len() - 1]` instead.

### List Comprehensions

List comprehensions create new lists by transforming and filtering existing ones:

```nova
// Basic: [variable in source | expression]
let squares = [x in [1, 2, 3, 4, 5] | x * x]
// squares == [1, 4, 9, 16, 25]
```

Add a **guard clause** after a second `|` to filter elements:

```nova
// With guard: [variable in source | expression | condition]
let even_squares = [x in [1, 2, 3, 4, 5, 6] | x * x | x % 2 == 0]
// even_squares == [4, 16, 36]
```

Multiple guards are separated by `|` and are combined with AND logic:

```nova
// Multiple guards: [x in list | expr | guard1 | guard2 | guard3]
import super.std.core
let filtered = [x in 1.to(21) | x | x % 2 == 0 | x > 5 | x < 15]
// filtered == [6, 8, 10, 12, 14]
```

### Nested List Comprehensions

Nested comprehensions iterate over multiple lists (like nested for loops) and produce a flat
result:

```nova
// 2-level: [x in l1, y in l2 | expr]
let pairs = [x in [1, 2], y in [10, 20] | x + y]
// pairs == [11, 21, 12, 22]

// 3-level: [x in l1, y in l2, z in l3 | expr]
let triples = [x in [1, 2], y in [10, 20], z in [100, 200] | x + y + z]
// triples == [111, 211, 121, 221, 112, 212, 122, 222]
```

Nested comprehensions can have guards too:

```nova
// Skip the diagonal (where x == y)
let off_diag = [x in 1.to(4), y in 1.to(4) | x * 10 + y | x != y]
// off_diag == [12, 13, 21, 23, 31, 32]
```

Use function calls in the expression body:

```nova
fn cube(n: Int) -> Int { return n * n * n }
let cubes = [x in [1, 2, 3, 4] | cube(x)]
// cubes == [1, 8, 27, 64]
```

> **Note:** The source in a comprehension must be a list. The range syntax `0..5` only works
> in `for` loops. Use `.to()` from `std/core` to create a list for comprehension sources:
> `[x in 0.to(10) | x * x]`

---

## 14. Tuples

Tuples are fixed-size, typed collections. Access elements by index:

```nova
let t = (42, "hello", true)
let n = t[0]    // Int: 42
let s = t[1]    // String: "hello"
let b = t[2]    // Bool: true
```

### Single-element tuples

Use a trailing comma to create a tuple with one element. Without the comma, parentheses are
just grouping:

```nova
let single = (42,)     // a tuple containing one Int
let grouped = (42)     // just the Int 42 — NOT a tuple

single[0]              // 42
```

Functions returning multiple values use tuples:

```nova
fn divmod(a: Int, b: Int) -> (Int, Int) {
    return (a / b, a % b)
}
let result = divmod(17, 5)
// result[0] == 3, result[1] == 2
```

---

## 15. Pattern Matching

`match` works on enum values:

```nova
enum Tree(T) {
    Leaf: $T,
    Node: (Tree($T), Tree($T))
}

fn depth(t: Tree(Int)) -> Int {
    match t {
        Leaf(x) => { return 1 }
        Node(children) => {
            let l = depth(children[0])
            let r = depth(children[1])
            if l > r { return l + 1 }
            return r + 1
        }
    }
    return 0
}
```

Match arms use the variant name (without the enum prefix) as the pattern. An identifier in the
arm captures the associated data:

```nova
match color {
    Red()   => { return "red" }
    Green() => { return "green" }
    Blue()  => { return "blue" }
}
```

---

## 16. Pipe Operator

The `|>` operator passes the left-hand value as the first argument to the right-hand function
call. The right-hand side must use call syntax (with `()`) and must be a non-extends function.

```nova
fn square(x: Int) -> Int { return x * x }
fn inc(x: Int) -> Int { return x + 1 }

let r = 4 |> inc() |> square()   // square(inc(4)) = 25
```

> Extends functions cannot be used with `|>`. Use UFCS chaining (`.method()`) instead.

---

## 17. Dyn Types

Dyn types provide structural, duck-typed dispatch without inheritance. A Dyn type describes the
fields a value must have -- any struct with those fields satisfies the constraint.

```nova
// Any struct with a 'name: String' field satisfies this
struct Dog { name: String, age: Int }
struct Robot { name: String, model: Int }

fn get_name(thing: Dyn(T = name: String)) -> String {
    return thing.name
}

let d = Dog { name: "Rex", age: 5 }
let r = Robot { name: "R2D2", model: 2 }

get_name(d)   // "Rex"
get_name(r)   // "R2D2"
```

### Type aliases

```nova
type named = Dyn(T = name: String)
```

### Multi-field Dyn

```nova
fn full_info(x: Dyn(T = name: String + age: Int)) -> String {
    return x.name + " (age " + Cast::string(x.age) + ")"
}
```

### Dyn with function fields

Use `->` to call a function field through a Dyn type:

```nova
type renderable = Dyn(T = render: fn($T) -> String + label: String)

fn render(item: renderable) -> String {
    return item->render()
}
```

### Dyn lists

```nova
type named = Dyn(T = name: String)

let items = []: named
items.push(Dog { name: "Rex", age: 5 })
items.push(Robot { name: "R2D2", model: 2 })
```

---

## 18. Box and Mutable Shared State

`Box(T)` (from `std/core.nv`) wraps a value in a heap-allocated object. Multiple closures can
share a `Box` and mutate it through `.value`:

```nova
import super.std.core

let shared = Box(0)

let inc = fn() { shared.value = shared.value + 1 }
let get = fn() -> Int { return shared.value }

inc(); inc(); inc()
get()   // 3
```

`Box` is the standard way to implement:

- Mutable counters shared between closures
- Generator functions
- Mutable fields inside closures captured by `foreach`

---

## 19. Imports and the Standard Library

### Import syntax

```nova
import super.std.core     // Box, Gen, Maybe, Result, Option helpers, range()
import super.std.list     // map, filter, reduce, foreach, sort, flatten, ...
import super.std.iter     // Iter type: fromVec, map, filter, collect, ...
import super.std.string   // string manipulation extensions
import super.std.math     // mathematical functions and Int/Float extensions
import super.std.io       // prompt(), readLines(), writeLines()
import super.std.hashmap  // HashMap type with insert, get, delete, ...
import super.std.tuple    // tuple utilities (swap, mapFirst, mapSecond)
import super.std.color    // named colors as (Int,Int,Int) tuples for raylib
import super.std.tui      // terminal UI helpers (rawmode, drawBox, etc.)
import super.std.widget   // raylib GUI widgets (Button, Label, Panel, etc.)
import super.std.option   // Option extensions (orDefault, orError, isNone)
import super.std.maybe    // Maybe(T) enum (Just/Nothing) -- standalone
import super.std.result   // Result(A,B) enum (Ok/Err) -- standalone
```

All imports flatten their definitions into the caller's scope. You call functions by
name — not with a module prefix. For example, after `import super.std.math`, you write
`bin(10)` not `math::bin(10)`.

### How modules are organized

Each `.nv` file starts with `module name` and defines types, functions, and extensions.
The `module` declaration registers the name so the parser can deduplicate imports.
Files can import each other using relative paths (`import sibling`) or
parent-relative paths (`import super.folder.file`).

### Key std/core.nv exports

| Name | Kind | Description |
|---|---|---|
| `Box(T)` | Struct | Heap wrapper for mutable shared state |
| `Gen(start)` | Function | Creates an integer generator starting at `start` |
| `range(n)` | Function | Returns `[0, 1, ..., n-1]` |
| `range(start, end)` | Function | Returns `[start, ..., end-1]` |
| `n.to(end)` | Extension | Returns `[n, n+1, ..., end-1]` (UFCS) |
| `Maybe(T)` | Enum | `Maybe::Just(value)` or `Maybe::Nothing()` |
| `Result(A, B)` | Enum | `Result::Ok(value)` or `Result::Err(error)` |
| `.orDefault(v)` | Extension | Unwrap Option/Maybe/Result or return default |
| `.orError(msg)` | Extension | Unwrap Option or print error and exit |
| `.isNone()` | Extension | True if Option is None |
| `.toMaybe()` | Extension | Convert Option to Maybe |
| `.toResult(err)` | Extension | Convert Option to Result |

---

## 20. Type System Rules

Nova's type system is strict and static. The following are all compile-time errors:

- Passing a value of the wrong type to a function
- Returning the wrong type from a function
- Assigning a value of a different type to an existing variable
- Using a variable that hasn't been declared
- Calling a function that doesn't exist
- Calling a function with the wrong number of arguments
- Accessing a struct field that doesn't exist
- Constructing a struct with missing or wrong-typed fields
- Pushing a wrong-typed element into a list
- Using an `Option(T)` where `T` is expected (must unwrap first)
- Mixing `Int` and `Float` without explicit cast
- Calling an extends function on the wrong receiver type

Nova has no type inference. The type of every binding is determined at declaration, and the
compiler does not attempt to infer types from usage.

---

## 21. Memory Model Deep Dive

Nova uses a hybrid memory model: **reference counting** for deterministic cleanup plus
**mark-and-sweep** garbage collection for cycle breaking. Understanding this model will help
you write efficient, correct code.

### Value Types vs. Reference Types

**Value types** live on the stack and are copied on assignment and when passed to functions:

| Type | Storage | Assignment behaviour |
|---|---|---|
| `Int` | Stack (64-bit) | Copied |
| `Float` | Stack (64-bit) | Copied |
| `Bool` | Stack | Copied |
| `Char` | Stack | Copied |

**Reference types** live on the heap and are **aliased** on assignment — both variables point
to the same underlying object:

| Type | Storage | Assignment behaviour |
|---|---|---|
| `[T]` (List) | Heap | Aliased (shared reference) |
| `String` | Heap | Aliased (shared reference) |
| Struct | Heap | Aliased (shared reference) |
| Tuple | Heap | Aliased (shared reference) |
| Enum (with data) | Heap | Aliased (shared reference) |
| Closure | Heap | Aliased (shared reference) |

### Aliasing in Action

When you assign a list, struct, or string to another variable, you create a second name for
the **same** object. Mutations through either variable are visible through the other:

```nova
let a = [1, 2, 3]
let b = a           // b is an ALIAS of a — same object
b.push(4)
println(a.len())    // 4 — the push is visible through a!

let p = Point { x: 1, y: 2 }
let q = p           // q is an alias of p
q.x = 99
println(p.x)        // 99 — both p and q see the change
```

This also applies when passing to functions. Heap objects are passed by reference:

```nova
fn add_item(list: [Int]) {
    list.push(99)
}

let my_list = [1, 2]
add_item(my_list)
println(my_list.len())   // 3 — the function modified the original
```

### The `clone()` Built-in

`clone()` creates a **deep copy** of any value. The clone is fully independent — mutations to
the clone never affect the original, and vice versa.

```nova
let original = [1, 2, 3]
let copy = clone(original)
copy.push(4)
copy[0] = 99
println(original.len())   // 3 — original is untouched
println(original[0])      // 1 — still the original value
```

Deep cloning applies recursively to nested structures:

```nova
struct Container { items: [Int], label: String }

let c1 = Container { items: [10, 20], label: "box" }
let c2 = clone(c1)
c2.items.push(30)
c2.label = "copy"
println(c1.items.len())   // 2 — inner list was deep-cloned
println(c1.label)         // "box" — string was deep-cloned
```

**When to use `clone()`:**

- When you need an independent snapshot of a list or struct
- In loops where you accumulate snapshots of changing state
- When passing data to a function that should not modify the original
- When building prototype/template patterns (clone a prototype to create variants)

**When NOT to use `clone()`:**

- For primitives (Int, Float, Bool, Char) — they're already copied automatically
- When you intentionally want shared mutation (e.g., a shared counter)
- In tight performance loops where sharing is fine

### Clone and Box

`Box(T)` is a struct, so it follows reference semantics. Assigning a `Box` creates an alias:

```nova
import super.std.core

let b1 = Box(42)
let b2 = b1        // alias — SAME Box
b2.value = 0
println(b1.value)   // 0 — both see the change
```

Cloning a `Box` creates an independent copy:

```nova
let b3 = Box(42)
let b4 = clone(b3)
b4.value = 0
println(b3.value)   // 42 — b3 is independent
```

### Reference Counting

Every heap object has a reference count that tracks how many variables point to it. When the
count drops to zero, the object is freed immediately. This gives Nova deterministic cleanup
for most objects — you don't need to wait for a GC pause.

### Mark-and-Sweep Cycle Collector

Reference counting alone cannot handle cycles (A points to B, B points to A). Nova
periodically runs a mark-and-sweep collector to find and free cyclic garbage. The collector:

1. Marks all objects reachable from the stack (roots)
2. Sweeps all unmarked objects
3. Adjusts the GC threshold adaptively (targeting ~16ms frame time)

In practice, this means:
- Short-lived allocations are freed immediately (ref-count drops to zero)
- Cyclic structures are collected periodically
- No manual memory management is ever needed
- There are no latency spikes for typical workloads

### Loop Variable Capture

Closures capture the **variable binding**, not a snapshot of its value. In a loop, all
closures would capture the same loop variable (which ends at its final value). To capture
the current iteration's value, **rebind** with `let`:

```nova
let fns = []: fn() -> Int

for let i = 0; i < 5; i += 1 {
    let captured = i   // ← rebind to freeze the value
    fns.push(fn() -> Int { return captured })
}

fns[0]()   // 0
fns[4]()   // 4
```

Without the rebind, all closures would return 5 (the final value of `i`).

---

## 22. Cast and Type Conversion

### `Cast::string(x)` -- any value to String

```nova
Cast::string(42)       // "42"
Cast::string(3.14)     // "3.14"
Cast::string(true)     // "true"
Cast::string('A')      // "A"
```

### `Cast::int(x)` -- any value to Option(Int)

Returns `None` if conversion fails:

```nova
let n = Cast::int("42")    // Some(42)
let bad = Cast::int("abc") // None(Int)
```

### `Cast::float(x)` -- any value to Option(Float)

```nova
let f = Cast::float(42)    // Some(42.0)
```

### `toString(x)` -- alias for Cast::string

```nova
toString(42)   // "42"
```

---

## 23. Iterators

The `Iter` type (from `std/iter.nv`) provides lazy iteration:

```nova
import super.std.iter

let result = Iter::fromVec([1,2,3,4,5])
    .map(|x: Int| x * x)
    .filter(|x: Int| x > 5)
    .collect()
// result == [9, 16, 25]
```

Key iterator methods:

| Method | Description |
|---|---|
| `Iter::fromVec(list)` | Create iterator from a list |
| `.map(f)` | Transform each element |
| `.filter(f)` | Keep elements where `f` returns true |
| `.collect()` | Materialize into a list |

---

## 24. String Operations

Strings support concatenation with `+`:

```nova
let greeting = "Hello, " + "World!"
```

Key string functions (from `std/string.nv`):

```nova
import super.std.string

"hello".len()                  // 5
"hello".chars()                // [Char]
"hello".chars().string()       // back to String
"  hello  ".trim()             // "hello"
"Hello".toLower()              // "hello"
"hello".toUpper()              // "HELLO"
"hello world".split(' ')       // ["hello", "world"]
```

Char operations:

```nova
'a'.isAlpha()    // true
'5'.isDigit()    // true
'A'.toLower()    // 'a'
```

---

## 25. Design Patterns Without OOP

Nova has no classes, no inheritance, and no interfaces. Instead, it achieves the same goals
through composition of its core features. Here's how common design patterns map to Nova.

### Strategy Pattern

Replace an algorithm at runtime by passing different functions:

```nova
fn process(data: [Int], strategy: fn(Int) -> Int) -> [Int] {
    return data.map(strategy)
}

process([1, 2, 3], |x: Int| x * 2)    // [2, 4, 6]
process([1, 2, 3], |x: Int| x * x)    // [1, 4, 9]
```

Or use a struct with a function field:

```nova
struct Formatter { format: fn(String) -> String }

let upper = Formatter { format: |s: String| s.toUpper() }
upper::format("hello")   // "HELLO"
```

### Observer Pattern

Use a list of callbacks that are invoked when state changes:

```nova
import super.std.core

struct EventBus { listeners: [fn(String)] }

fn extends subscribe(bus: EventBus, cb: fn(String)) {
    bus.listeners.push(cb)
}

fn extends emit(bus: EventBus, event: String) {
    for cb in bus.listeners { cb(event) }
}
```

### Factory Pattern

Centralize creation behind a function:

```nova
fn createEnemy(kind: String) -> Enemy {
    if kind == "goblin" { return Enemy { name: "Goblin", hp: 30 } }
    if kind == "dragon" { return Enemy { name: "Dragon", hp: 200 } }
    return Enemy { name: "Slime", hp: 10 }
}
```

### Builder Pattern

Chain extends methods that modify and return the struct:

```nova
fn extends where_(q: Query, cond: String) -> Query {
    q.conditions.push(cond)
    return q
}

fn extends limit(q: Query, n: Int) -> Query {
    q.limit_val = n
    return q
}

let sql = QueryNew("users").where_("age > 18").limit(10).toSQL()
```

### Decorator Pattern

Wrap a function with additional behaviour:

```nova
fn with_log(name: String, f: fn(Int) -> Int) -> fn(Int) -> Int {
    return |x: Int| {
        println(name + "(" + Cast::string(x) + ")")
        f(x)
    }
}

let logged = with_log("double", |x: Int| x * 2)
logged(5)   // prints "double(5)", returns 10
```

### State Machine Pattern

Use enums to model states, extends for transitions:

```nova
enum Light { Red, Yellow, Green }

fn extends next(l: Light) -> Light {
    match l {
        Red()    => { return Light::Green() }
        Green()  => { return Light::Yellow() }
        Yellow() => { return Light::Red() }
    }
    return Light::Red()
}
```

### Command Pattern

Encode operations as enum variants:

```nova
enum Cmd { Insert: String, Delete: Int }

fn apply(text: String, cmd: Cmd) -> String {
    match cmd {
        Insert(s) => { return text + s }
        Delete(n) => { return text.substring(0, text.len() - n) }
    }
    return text
}
```

### Composite Pattern

Model tree structures with recursive enums:

```nova
enum Expr { Num: Int, Add: (Expr, Expr), Mul: (Expr, Expr) }

fn extends eval(e: Expr) -> Int {
    match e {
        Num(n)    => { return n }
        Add(pair) => { return pair[0].eval() + pair[1].eval() }
        Mul(pair) => { return pair[0].eval() * pair[1].eval() }
    }
    return 0
}
```

### Prototype Pattern

Clone an existing object and modify the copy:

```nova
let proto = Config { host: "localhost", port: 8080 }
let dev = clone(proto)
dev.port = 3000   // proto.port is still 8080
```

### Adapter Pattern (Dyn Types)

Use Dyn types to accept any struct with the required fields:

```nova
fn get_name(thing: Dyn(T = name: String)) -> String {
    return thing.name
}

// Works with any struct that has a 'name' field
get_name(Dog { name: "Rex" })
get_name(Robot { name: "R2" })
```

### Singleton Pattern

Use a closure-captured Box that initializes once:

```nova
import super.std.core

fn make_config() -> fn(String) -> String {
    let data = Box("default")
    let init = Box(false)
    return fn(action: String) -> String {
        if action == "init" && !init.value {
            data.value = "ready"
            init.value = true
        }
        return data.value
    }
}
```

### Template Method Pattern

Define a skeleton algorithm with pluggable steps:

```nova
fn process(data: [Int], filter: fn(Int) -> Bool,
           transform: fn(Int) -> Int,
           combine: fn(Int, Int) -> Int, init: Int) -> Int {
    let acc = init
    for item in data {
        if filter(item) { acc = combine(acc, transform(item)) }
    }
    return acc
}
```

> **Key insight:** Nova replaces OOP's virtual dispatch with:
> - **Closures** for strategy/command/observer
> - **Enums + match** for state machines and variant types
> - **Dyn types** for structural polymorphism
> - **Extends + UFCS** for method-like syntax
> - **clone()** for the prototype pattern
>
> See `tests/test_design_patterns.nv` and `tests/test_design_patterns2.nv` for 27+ fully
> tested implementations.

---

## 26. The Fuzzer

Nova includes fuzzing targets for the lexer and parser using
[cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

### Setup

```bash
rustup toolchain install nightly
cargo +nightly install cargo-fuzz
```

### Running

```bash
./fuzz/run_fuzz.sh lexer 30    # fuzz lexer for 30 seconds
./fuzz/run_fuzz.sh parser 60   # fuzz parser for 60 seconds
./fuzz/run_fuzz.sh all 30      # fuzz all targets
```

Crash inputs are saved in `fuzz/artifacts/`. The fuzzer starts from a seed corpus of real Nova
programs in `fuzz/corpus/`.

What the fuzzer checks:

- The lexer must never panic on arbitrary UTF-8 input.
- The parser must never panic on any sequence of tokens.
- Errors (parse/type errors) are expected and acceptable -- panics are bugs.

---

## 27. Syntax Sugar Reference

Nova has a number of syntactic conveniences that make code cleaner. This section collects them
all in one place with practical examples.

### Range loops: `start..end` and `start..=end`

`for` loops support exclusive and inclusive integer ranges directly:

```nova
// Exclusive (end not included)
for i in 0..5 {
    print(Cast::string(i) + " ")    // 0 1 2 3 4
}

// Inclusive (end included)
for i in 1..=5 {
    print(Cast::string(i) + " ")    // 1 2 3 4 5
}
```

Ranges only work directly in `for` loops. For comprehensions, convert to a list first with
`.to()` from `std/core`:

```nova
import super.std.core
let squares = [x in 0.to(5) | x * x]   // [0, 1, 4, 9, 16]
```

### List slicing: `xs[start:end]`

Python-style slice syntax. Returns a new list.

```nova
let xs = [10, 20, 30, 40, 50]

xs[1:3]     // [20, 30]          indices 1 and 2
xs[2:]      // [30, 40, 50]      from 2 to end
xs[:3]      // [10, 20, 30]      from start to 3
xs[:]       // full copy

// Negative indices count from the end
xs[-2:]     // [40, 50]          last 2
xs[:-1]     // [10, 20, 30, 40]  all but last
xs[-3:-1]   // [30, 40]          3rd-last to 2nd-last

// Step with $
xs[::$2]    // same as xs[:$2] — every 2nd element
xs[:$2]     // [10, 30, 50]      every 2nd
xs[1:$2]    // [20, 40]          every 2nd starting at 1
xs[0:4$2]   // [10, 30]          0,2 — every 2nd up to index 4
```

> **Caution:** `xs[-1]` (negative regular index) is a runtime error. Use slicing or
> `xs[xs.len() - 1]` for the last element.

### List comprehensions: `[x in list | expr]`

```nova
// Basic
let evens = [x in [1,2,3,4,5,6] | x | x % 2 == 0]   // [2,4,6]

// Transform
let squares = [x in [1,2,3,4,5] | x * x]             // [1,4,9,16,25]

// Guard clause (filter):  [x in src | expr | condition]
let big_sq = [x in [1,2,3,4,5] | x * x | x > 2]     // [9,16,25]

// Multiple guards (AND logic):  [x in src | expr | g1 | g2]
import super.std.core
let mid = [x in 1.to(20) | x | x % 2 == 0 | x > 4 | x < 14]  // [6,8,10,12]

// Nested (flat result — like nested for loops):
let sums = [x in [1,2], y in [10,20] | x + y]        // [11,21,12,22]

// Nested with guard:
let pairs = [x in 1.to(4), y in 1.to(4) | (x, y) | x != y]

// Side-effect before output (comma separates expressions):
let logged = [x in [1,2,3] | print("x="+Cast::string(x)), x * 2]
```

### `if let` / `while let` — safe Option unwrapping

```nova
// if let: run body only if option holds a value, binding it to name
let x: Option(Int) = Cast::int("42")
if let n = x {
    println(n)    // 42
} else {
    println("not a number")
}

// while let: loop while option is Some
import super.std.core
let counter = Gen(0)
while let i = counter() {
    println(i)    // prints forever (Gen never returns None unless you stop it)
    if i >= 4 { break }
}
```

### Trailing closures: `fn(args): |params| expr`

When the last argument to a function is a closure, you can write it after `:` instead of
inside the parentheses:

```nova
fn apply(x: Int, f: fn(Int) -> Int) -> Int { return f(x) }

// These are identical:
let a = apply(5, |x: Int| x * 2)
let b = apply(5): |x: Int| x * 2   // trailing closure

// Works with UFCS and std library:
import super.std.list
let evens = [1,2,3,4,5].filter(): |x: Int| x % 2 == 0   // [2,4]
let big   = [1,2,3,4,5].map():    |x: Int| x * x        // [1,4,9,16,25]
```

### The bind operator: `expr ~> name { block }`

Names an intermediate result inline without introducing a `let` statement:

```nova
// expr ~> name { use name inside }
let r = [1,2,3,4,5].len() ~> n { n * n }   // 25

// Useful for avoiding repetition:
let area = (base * height) ~> a { a / 2 }

// And for readable pipelines:
let msg = "hello world" ~> s {
    s.chars().filter(|c: Char| c != ' ').string()
}
```

### Empty closures: `|| expr`

Closures with no parameters use `||`:

```nova
let greet = || println("hello")
greet()

let val = || 42
val()   // 42

// Common with Box-based counters:
import super.std.core
let n = Box(0)
let next = || { n.value += 1; n.value }
```

### Function references with `@(Type)`: overload selection

When a function is overloaded for multiple types, use `@(Type)` to select which version:

```nova
fn process(x: Int) -> String { return "int: " + Cast::string(x) }
fn process(x: String) -> String { return "str: " + x }

let int_processor = process@(Int)       // fn(Int) -> String
let str_processor = process@(String)    // fn(String) -> String
```

### Generic annotation `@[T: Type]`: no-data generic variants

When constructing a generic enum's no-data variant, annotate the type parameter:

```nova
import super.std.core

let nothing: Maybe(Int) = Maybe::Nothing() @[A: Int]
let none: Option(Int) = None(Int)   // built-in Option doesn't need this
```

### `::` vs `->` on structs: stored functions

Two operators for calling functions stored as struct fields:

```nova
struct Handler {
    name: String,
    handle: fn(Handler, String) -> String
}

let h = Handler { name: "test", handle: fn(self: Handler, msg: String) -> String {
    return self.name + ": " + msg
}}

// :: calls without passing self
h::handle("hello")              // fn is called with just the explicit args

// -> calls WITH h as first argument (like method dispatch)
h->handle("hello")              // equivalent to h.handle(h, "hello")
```

### Forward declarations: mutual recursion

Declare a function's signature without a body to use it before its definition:

```nova
fn isEven(n: Int) -> Bool   // forward declaration — no body, no {}

fn isOdd(n: Int) -> Bool {
    if n == 0 { return false }
    return isEven(n - 1)
}

fn isEven(n: Int) -> Bool {
    if n == 0 { return true }
    return isOdd(n - 1)
}

println(isEven(10))   // true
println(isOdd(7))     // true
```

### Single-element tuples

A trailing comma inside parentheses creates a one-element tuple. Without the comma, parens are
just grouping:

```nova
let pair   = (1, 2)    // (Int, Int)
let single = (42,)     // (Int,)  — one-element tuple
let value  = (42)      // just the Int 42

single[0]              // 42
```

### All syntax sugar at a glance

| Sugar | Example | What it does |
|---|---|---|
| Exclusive range | `for i in 0..5` | Loop 0, 1, 2, 3, 4 |
| Inclusive range | `for i in 0..=5` | Loop 0, 1, 2, 3, 4, 5 |
| Slice | `xs[1:3]` | Elements 1 and 2 |
| Negative slice | `xs[-2:]` | Last 2 elements |
| Slice with step | `xs[:$2]` | Every 2nd element |
| Comprehension | `[x in xs \| x*x]` | Transformed list |
| Comprehension guard | `[x in xs \| x \| x>0]` | Filtered list |
| Nested comprehension | `[x in xs, y in ys \| x+y]` | Cross-product (flat) |
| `if let` | `if let v = opt { }` | Safe Option unwrap |
| `while let` | `while let v = opt { }` | Loop while Some |
| Trailing closure | `f(x): \|y\| y+1` | Closure as last arg after `:` |
| Bind operator | `expr ~> x { x+1 }` | Name an intermediate value |
| Empty closure | `\|\| expr` | Zero-parameter closure |
| Function reference | `fn@(Int)` | Select overload by type |
| Generic annotation | `Variant() @[A: Int]` | No-data variant type hint |
| Field call no-self | `s::fn_field()` | Call stored fn without self |
| Field call with-self | `s->fn_field()` | Call stored fn with s as first arg |
| Forward declaration | `fn f(x: Int) -> Int` | Signature only, enables mutual recursion |
| Single-element tuple | `(42,)` | One-element tuple |

---

## 28. Tips and Tricks

### Use `elif` instead of `else if`

Nova uses `elif` for chained conditionals. `else if` is a syntax error:

```nova
// ✓ Correct
if x > 10 {
    println("big")
} elif x > 5 {
    println("medium")
} else {
    println("small")
}

// ✗ Wrong — this will NOT compile
// } else if x > 5 {
```

### Closures that return values need `fn` syntax

Short lambdas (`|x: Int| expr`) work for single expressions. But if a closure has control flow
(if/return), use the full `fn` syntax with an explicit return type:

```nova
// ✗ May fail type inference:
// let f = |x: Int| { if x > 0 { return x } else { return 0 } }

// ✓ Use full fn syntax:
let f = fn(x: Int) -> Int {
    if x > 0 { return x }
    return 0
}
```

### Every function returning a value needs an explicit `return`

Nova does not have implicit returns. The last expression in a function body is NOT
automatically returned — you must write `return`:

```nova
// ✗ Compile error: "Function must return a value"
// fn square(x: Int) -> Int { x * x }

// ✓ Correct:
fn square(x: Int) -> Int { return x * x }
```

### There is no `*=` operator

Only `+=`, `-=`, and `/=` are available. For multiplication assignment, write it out:

```nova
let x = 5
x = x * 3   // not x *= 3
```

### Empty lists need a type annotation

```nova
// ✗ Error:
// let xs = []

// ✓ Correct:
let xs = []: Int
let strs = []: String
let nested = []: [Int]    // list of int lists
```

### No-data enum variants still need `()`

```nova
enum Color { Red, Green, Blue }

let c = Color::Red()    // ← the () is required, even though Red has no data
```

### Use `@[A: Type]` for generic enum no-data variants

When constructing a generic enum's no-data variant, the compiler can't infer the type
parameter. Use the `@[T: ConcreteType]` annotation:

```nova
enum Maybe(A) { Just: $A, Nothing }

let x = Maybe::Nothing() @[A: Int]   // tells the compiler this is Maybe(Int)
```

### String concatenation with `+`

You can only concatenate `String + String`. To include other types, convert first:

```nova
let msg = "Count: " + Cast::string(42)    // "Count: 42"
let pi  = "Pi is " + Cast::string(3.14)   // "Pi is 3.14"
```

### The `->` operator vs `.` vs `::`

These three have distinct meanings on structs:

| Syntax | Meaning |
|---|---|
| `s.field` | Access a data field |
| `s.method()` | Call an extends method (UFCS) |
| `s::fn_field()` | Call a function stored in a field (no self passed) |
| `s->fn_field()` | Call a function stored in a field, passing `s` as first arg |

### `format` and `printf` for string interpolation

Nova doesn't have string interpolation syntax, but `format` and `printf` work with `{}`
placeholders:

```nova
let name = "Alice"
let age = 30
let msg = format("Hello, {}! You are {} years old.", [name, Cast::string(age)])
printf("{} is {} years old\n", [name, Cast::string(age)])
```

### Sorting with custom comparators

```nova
import super.std.list

let nums = [5, 3, 1, 4, 2]
let sorted = nums.sortWith(|a: Int, b: Int| a > b)   // ascending
```

### Use `if let` for safe Option unwrapping

```nova
let maybe_val = Cast::int("42")
if let val = maybe_val {
    println("Got: " + Cast::string(val))
}
```

---

## 29. Quick Reference: Common Mistakes

| Mistake | Fix |
|---|---|
| `let x = []; x.push(1)` | `let x = []: Int; x.push(1)` |
| `x += 1` used as expression | Use `x = x + 1` in expressions; `+=` is statement-only |
| `x *= 2` | No `*=` operator — use `x = x * 2` |
| `fn extends f(x)` called as `f(x)` | Use `x.f()` (UFCS only) |
| `5 \|> myExtendsFn()` | Pipe only works with non-extends functions |
| `fn extends id(x: $A)` | Not supported — use concrete type wrapper |
| `match x { 0 => ... }` | Literals not allowed in match — use `if`/`elif` |
| `-10 % 3 == -1` | Wrong — Nova uses Euclidean modulo: `-10 % 3 == 2` |
| `Cast::int(x)` used as `Int` | Returns `Option(Int)` — must `.unwrap()` |
| `Cast::float(x)` used as `Float` | Returns `Option(Float)` — must `.unwrap()` |
| Mutual recursion without forward decl | Use forward declarations: `fn name(params) -> Type` |
| `else if x > 0 { }` | Use `elif x > 0 { }` — `else if` is a syntax error |
| Lambda with control flow | Use `fn(params) -> Type { }` not `\|params\| { }` |
| `fn f(x: Int) -> Int { x * x }` | Must have explicit `return x * x` |
| `let b = a` where `a` is a list | Creates an alias! Use `clone(a)` for a copy |
| `Color::Red` (no parens) | Must write `Color::Red()` even for no-data variants |
| `Maybe::Nothing()` without type | Use `Maybe::Nothing() @[A: Int]` for generic no-data |
| `\|\| true && false` precedence | `\|\|` binds tighter than `&&` in Nova — use parens |

---

## 30. Standard Library Reference

All standard library modules live in `nova-lang/std/`. Import them with:

```nova
import super.std.<module_name>
```

---

### `std/core` — Foundation types

```nova
import super.std.core
```

| Type / Function | Description |
|---|---|
| `Box(T)` | Mutable shared heap cell. `box.value` to read/write. Essential for closures that mutate state. |
| `Gen(start)` | Stateful integer counter. `gen.next()` → next Int. |
| `Maybe(A)` | Generic tagged union: `Maybe::Just(v)` / `Maybe::Nothing() @[A: T]`. Methods: `.isJust()`, `.isNothing()`, `.unwrapJust()`, `.orElse(v)`, `.map(f)`, `.flatMap(f)`. |
| `Result(A,B)` | Success/failure union: `Result::Ok(v)` / `Result::Err(e)`. Methods: `.isOk()`, `.isErr()`, `.unwrapOk()`, `.unwrapErr()`, `.map(f)`, `.mapErr(f)`, `.orElse(v)`, `.andThen(f)`. |
| `range(start, end, step)` | Produce a `[Int]` from start to end by step. |
| `toMaybe(opt)` | Convert `Option(T)` → `Maybe(T)`. |
| `toResult(opt, err)` | Convert `Option(T)` → `Result(T,E)`. |

---

### `std/math` — Extended mathematics

```nova
import super.std.math
```

**Int extensions (UFCS: `n.func()`):**

| Method | Description |
|---|---|
| `n.min(other)` | Smaller of n and other |
| `n.max(other)` | Larger of n and other |
| `n.abs()` | Absolute value |
| `n.pow(exp)` | Integer exponentiation |
| `n.sqrt()` → Float | Square root |
| `n.clamp(lo, hi)` | Clamp into range |
| `n.factorial()` | n! |
| `n.gcd(other)` | Greatest common divisor |
| `n.lcm(other)` | Least common multiple |
| `n.isEven()` | True when n % 2 == 0 |
| `n.isOdd()` | True when n % 2 != 0 |
| `n.sign()` | -1, 0, or 1 |
| `n.modpow(exp, mod)` | Fast modular exponentiation |
| `n.isPrime()` | Primality test (trial division) |
| `n.digitSum()` | Sum of decimal digits |
| `n.digits()` | `[Int]` of decimal digits, MSB first |

**Float extensions (UFCS: `f.func()`):**

| Method | Description |
|---|---|
| `f.degrees()` | Radians → degrees |
| `f.radians()` | Degrees → radians |
| `f.normalize(lo, hi)` | Map to [0.0, 1.0] given lo/hi bounds |
| `f.mapRange(flo,fhi,tlo,thi)` | Map one range to another |

**Standalone functions:**

| Function | Description |
|---|---|
| `fib(n)` | nth Fibonacci number |
| `fibSeq(n)` | First n Fibonacci numbers as `[Int]` |
| `bin(n)` | Int → binary string (`"1010"`) |
| `hex(n)` | Int → hex string (`"ff"`) |
| `oct(n)` | Int → octal string |
| `divmod(n, d)` | `(quotient, remainder)` |
| `toRadians(deg)` | Float degrees → radians |
| `toDegrees(rad)` | Float radians → degrees |
| `lerp(a, b, t)` | Float linear interpolation |
| `lerpF(a, b, t)` | Int linear interpolation |
| `remap(v,fl,fh,tl,th)` | Map a value between ranges |
| `round(f)` | Float → nearest Int |
| `smoothstep(t)` | Smooth Hermite curve (3t²-2t³) |
| `sign(x)` | Float sign: -1.0, 0.0, or 1.0 |
| `isPrime(n)` | Standalone primality test |
| `primes(n)` | All primes ≤ n (Sieve of Eratosthenes) |
| `collatz(n)` | Collatz sequence from n to 1 |

---

### `std/string` — String utilities

```nova
import super.std.string
```

| Method | Description |
|---|---|
| `s.split(Char)` | Split by a single character |
| `s.padLeft(n, c)` | Left-pad to width n with char c |
| `s.padRight(n, c)` | Right-pad to width n with char c |
| `s.center(n, c)` | Center with char c padding |
| `s.count(sub)` | Count substring occurrences |
| `s.countChar(c)` | Count occurrences of a character |
| `s.indexOfChar(c)` | First index of char, or -1 |
| `s.isDigit()` | All chars are `0-9` |
| `s.isAlpha()` | All chars are alphabetic |
| `s.isAlphanumeric()` | All chars are letters or digits |
| `s.capitalize()` | First character uppercased |
| `s.title()` | Each word capitalized |
| `s.removeChar(c)` | Delete all occurrences of c |
| `s.replaceChar(old, new)` | Replace char old with char new |
| `s.lines()` | Split on newlines |
| `s.words()` | Split on spaces, drops empty strings |
| `s.truncate(n, suffix)` | Cut to n chars, append suffix if cut |
| `s.slugify()` | Lowercase, spaces→hyphens, strip specials |
| `s.wrap(width)` | Word-wrap at column width |
| `s.between(left, right)` | Extract text between two delimiters |
| `s.stripPrefix(p)` | Remove prefix if present |
| `s.stripSuffix(s)` | Remove suffix if present |

---

### `std/option` — Option combinators

```nova
import super.std.option
```

Extends the built-in `Option(T)` (which has `.isSome()` and `.unwrap()`):

| Method | Description |
|---|---|
| `opt.isNone()` | True when holding no value |
| `opt.orDefault(v)` | Unwrap or return fallback |
| `opt.orDoFn(f)` | Unwrap or lazily compute fallback |
| `opt.orError(msg)` | Unwrap or print msg and exit |
| `opt.map(f)` | Transform inner value: `Some(f(v))` or `None` |
| `opt.flatMap(f)` | Chain an Option-returning function |
| `opt.filter(pred)` | Keep Some only if pred holds |
| `opt.zip(other)` | Combine two Options into `Option((A,B))` |
| `opt.toList()` | `[v]` if Some, else `[]` |
| `opt.inspect(f)` | Run side-effect if Some, pass through |

---

### `std/iter` — Lazy iterator

```nova
import super.std.iter
```

**Constructors:**

| Constructor | Description |
|---|---|
| `Iter::fromRange(start, end)` | Integers `[start, end)` |
| `Iter::fromVec(list)` | Iterate over a list |
| `Iter::fromFn(fn)` | Custom pull function |
| `Iter::enumerate(iter)` | Pairs `(index, value)` |
| `Iter::repeat(v)` | Infinite stream of v |
| `Iter::generate(f)` | Infinite stream from `f()` |

**Transformers (lazy):**

| Method | Description |
|---|---|
| `.map(f)` | Apply f to each element |
| `.filter(pred)` | Keep elements where pred is true |
| `.take(n)` | First n elements |
| `.drop(n)` | Skip first n elements |
| `.takeWhile(pred)` | Take while pred holds |
| `.dropWhile(pred)` | Skip while pred holds, then yield rest |
| `.flatMap(f)` | Map then flatten one level |
| `.zip(other)` | Pair elements from two iterators |
| `.chain(other)` | Append another iterator |

**Consumers (eager):**

| Method | Description |
|---|---|
| `.collect()` | Gather into a list |
| `.show()` | Print each element |
| `.count()` | Number of elements |
| `.sum()` | Sum of Int elements |
| `.sumF()` | Sum of Float elements |
| `.reduce(f, init)` | Fold with accumulator |
| `.fold(f, init)` | Alias for reduce |
| `.any(pred)` | True if any element passes |
| `.all(pred)` | True if all elements pass |
| `.find(pred)` | First element passing pred |
| `.last()` | Last element |
| `.nth(n)` | nth element (0-indexed) |
| `.forEach(f)` | Side-effect on each element |

---

### `std/hashmap` — Generic hash map

```nova
import super.std.hashmap
```

| Method | Description |
|---|---|
| `HashMap::default()` | Empty map with 16-bucket initial capacity |
| `HashMap::fromPairs(list)` | Build from `[(K,V)]` |
| `.insert(k, v)` | Insert or update |
| `.get(k)` → `Option(V)` | Look up by key |
| `.delete(k)` | Remove by key |
| `.has(k)` | Check membership |
| `.size()` | Entry count |
| `.isEmpty()` | True when empty |
| `.clear()` | Remove all entries |
| `.getOrDefault(k, v)` | Look up with fallback |
| `.entries()` | `[(K,V)]` list |
| `.keys()` | `[K]` list |
| `.values()` | `[V]` list |
| `.forEach(f)` | Side-effect on each `(k,v)` |
| `.merge(other)` | Insert all entries from other |
| `.mapValues(f)` | New map with values transformed |
| `.filterKeys(pred)` | New map keeping matching keys |
| `.filterValues(pred)` | New map keeping matching values |
| `.increment(k)` | Increment Int value (count map helper) |
| `.update(k, default, f)` | Update value with function |
| `.toSortedPairs()` | Entries sorted by key string |

---

### `std/set` — Generic set

```nova
import super.std.set
```

| Method | Description |
|---|---|
| `Set::empty()` | Empty set |
| `Set::singleton(v)` | Set with one element |
| `Set::fromList(list)` | Build from list (deduplicates) |
| `.add(v)` | Insert a value |
| `.remove(v)` | Remove a value |
| `.has(v)` | Check membership |
| `.size()` | Number of elements |
| `.isEmpty()` | True when empty |
| `.toList()` | All elements as a list |
| `.union(other)` | A ∪ B |
| `.intersection(other)` | A ∩ B |
| `.difference(other)` | A \ B |
| `.isSubset(other)` | True if self ⊆ other |
| `.isSuperset(other)` | True if self ⊇ other |
| `.isDisjoint(other)` | True if self ∩ other == ∅ |
| `.forEach(f)` | Side-effect on each element |
| `.filter(pred)` | New set keeping matches |
| `.map(f)` | Transform elements (may reduce size) |

---

### `std/vec2` — 2D vector math

```nova
import super.std.vec2
```

| Constructor | Description |
|---|---|
| `Vec2::new(x, y)` | From two Floats |
| `Vec2::zero()` | (0.0, 0.0) |
| `Vec2::one()` | (1.0, 1.0) |
| `Vec2::up()` | (0.0, 1.0) |
| `Vec2::right()` | (1.0, 0.0) |
| `Vec2::fromAngle(rad)` | Unit vector at angle (radians) |

| Method | Description |
|---|---|
| `.add(v)`, `.sub(v)` | Component-wise add/subtract |
| `.scale(s)` | Multiply by scalar |
| `.negate()` | (-x, -y) |
| `.dot(v)` | Dot product |
| `.cross(v)` | 2D cross product (scalar) |
| `.length()` | Euclidean magnitude |
| `.lengthSq()` | Squared magnitude (avoids sqrt) |
| `.normalized()` | Unit vector |
| `.distance(v)` | Distance to other vector |
| `.distanceSq(v)` | Squared distance |
| `.angle()` | Angle of vector (radians) |
| `.angleTo(v)` | Angle between vectors |
| `.rotate(rad)` | Rotate by radians |
| `.lerp(v, t)` | Linear interpolation |
| `.reflect(normal)` | Reflect across a unit normal |
| `.perpendicular()` | 90° clockwise rotation |
| `.clampLength(max)` | Scale down if length exceeds max |
| `.equals(v)` | Component-wise equality |
| `.isZero()` | True if both components are 0.0 |

---

### `std/deque` — Double-ended queue

```nova
import super.std.deque
```

| Method | Description |
|---|---|
| `Deque::empty()` | Empty deque |
| `Deque::singleton(v)` | Deque with one element |
| `Deque::fromList(xs)` | Build from list |
| `.pushBack(v)` | Add to back (tail) |
| `.pushFront(v)` | Add to front (head) |
| `.popBack()` → `Option(T)` | Remove and return back element |
| `.popFront()` → `Option(T)` | Remove and return front element |
| `.peekBack()` → `Option(T)` | View back element |
| `.peekFront()` → `Option(T)` | View front element |
| `.len()` | Number of elements |
| `.isEmpty()` | True when empty |
| `.toList()` | Snapshot as list (front→back) |
| `.forEach(f)` | Side-effect on each element |
| `.map(f)` | New deque with f applied |
| `.filter(pred)` | New deque keeping matches |

---

### `std/tuple` — Pair and triple utilities

```nova
import super.std.tuple
```

| Method / Function | Description |
|---|---|
| `t.swap()` | Reverse the pair: (b,a) |
| `t.fst()` | First element |
| `t.snd()` | Second element |
| `t.mapFirst(f)` | Apply f to first, keep second |
| `t.mapSecond(f)` | Apply f to second, keep first |
| `t.both(f)` | Apply same f to both (requires A==B) |
| `t.toStrings()` | Both elements as `[String]` |
| `t.toList()` | `[a, b]` (requires A==B) |
| `pairs.unzip()` | `[(A,B)]` → `([A],[B])` |
| `pair(a, b)` | Convenience constructor |
| `triple(a, b, c)` | Convenience constructor |
| `fst(t)` | Standalone first element |
| `snd(t)` | Standalone second element |

---

### `std/io` — Input / output utilities

```nova
import super.std.io
```

| Function | Description |
|---|---|
| `prompt(msg)` | Print msg, return input line |
| `promptInt(msg)` | Prompt and parse Int → `Option(Int)` |
| `promptFloat(msg)` | Prompt and parse Float → `Option(Float)` |
| `promptYN(msg)` | Prompt for yes/no → Bool |
| `printSep(values, sep)` | Println values joined with separator |
| `eprintln(msg)` | Print `[error] msg` (stderr simulation) |
| `readLines(path)` | File → `[String]` |
| `writeLines(path, lines)` | `[String]` → file |
| `appendLine(path, line)` | Append one line to file |
| `linesOf(text)` | Split string on newlines |

---

### `std/functional` — Higher-order utilities

```nova
import super.std.functional
```

| Function | Description |
|---|---|
| `compose(f, g)` | `fn(x) -> f(g(x))` (right-to-left) |
| `pipe(f, g)` | `fn(x) -> g(f(x))` (left-to-right) |
| `flip(f)` | Swap the two arguments of a binary fn |
| `const_(v)` | Always return v, ignoring argument |
| `identity(x)` | Return x unchanged |
| `applyN(f, n, x)` | Apply f to x exactly n times |
| `applyWhile(f, pred, x)` | Apply f while pred(result) holds |
| `memoize(f)` | Cache results keyed by string of argument |
| `negate(pred)` | Logical NOT of a predicate |
| `both(p, q)` | True when both predicates hold |
| `either(p, q)` | True when at least one predicate holds |

---

### `std/ansi` — ANSI terminal colors

```nova
import super.std.ansi
```

Returns decorated strings for use with `print`/`println`:

```nova
println(Ansi::bold(Ansi::red("ERROR: something went wrong")))
println(Ansi::green("OK") + " test passed")
println(Ansi::rgb(255, 128, 0, "orange text"))
print(Ansi::moveTo(1, 1))
```

| Category | Functions |
|---|---|
| Styles | `bold`, `dim`, `italic`, `underline`, `blink`, `invert`, `strikethrough` |
| Foreground | `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` |
| Bright FG | `brightBlack` … `brightWhite` (same set, `bright` prefix) |
| Background | `bgBlack`, `bgRed`, `bgGreen`, `bgYellow`, `bgBlue`, `bgMagenta`, `bgCyan`, `bgWhite` |
| Bright BG | `bgBrightBlack` … `bgBrightWhite` |
| 256 / RGB | `color256(code, s)`, `bgColor256(code, s)`, `rgb(r,g,b,s)`, `bgRgb(r,g,b,s)` |
| Control | `reset()`, `clearScreen()`, `clearLine()`, `moveTo(row,col)`, `hideCursor()`, `showCursor()` |

---

### `std/color` — Named color tuples

```nova
import super.std.color
```

Provides `(Int,Int,Int)` RGB tuples for Raylib/TUI:
`red`, `green`, `blue`, `white`, `black`, `yellow`, `cyan`, `magenta`, and many more.
Helpers: `rgb(r,g,b)`, `lerpColor(a,b,t)`, `invert(c)`, `darken(c,f)`, `lighten(c,f)`.

---

### `std/tui` — Terminal UI

```nova
import super.std.tui
```

Raw-mode terminal rendering: `run(fn)`, `printAt(x,y,s)`, `clear()`, `flush()`, `size()`,
`fg(r,g,b)`, `bg(r,g,b)`, `resetColor()`, `drawBox(x,y,w,h)`, `getch()`, `poll()`.

---

### `std/widget` — TUI widget toolkit

```nova
import super.std.widget
```

Composable TUI widgets: `Button`, `Label`, `Panel`, `ProgressBar`, `Toggle`.
Each has `.draw()`, and interactive widgets have `.isClicked()` / `.isHovered()`.

---

### `std/list` — List utilities (auto-imported)

`list` is imported automatically by many std modules. Key additions:
`join(list, sep)`, `flatten(list)`, `zip(a, b)`, `unzip(pairs)`, `range(n)`,
and UFCS: `.filter(pred)`, `.map(f)`, `.sort()`, `.sortWith(cmp)`,
`.reduce(f, init)`, `.any(pred)`, `.all(pred)`, `.find(pred)`.
