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
21. [Garbage Collector](#21-garbage-collector)
22. [Cast and Type Conversion](#22-cast-and-type-conversion)
23. [Iterators](#23-iterators)
24. [String Operations](#24-string-operations)
25. [The Fuzzer](#25-the-fuzzer)
26. [Quick Reference: Common Mistakes](#26-quick-reference-common-mistakes)

---

## 1. Module System

Every Nova source file must begin with a `module` declaration. This sets the file's namespace.

```nova
module my_module

// code goes here
```

Module names are used when other files import this file. There is no way to have code at the
top level without a `module` declaration -- the parser will reject the file.

To import another module:

```nova
module main

import super.std.core   // imports std/core.nv relative to the project root
import super.std.list
import super.std.iter
```

The `super` prefix means "go up one directory from this file." Standard library modules live
in the `std/` folder of the project.

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

> **Note:** Nova does not support forward declarations. A function must be defined before it
> is called. Mutual recursion is not directly supported.

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

`Option(T)` represents a value that may or may not be present.

```nova
let found: Option(Int) = Some(42)
let empty: Option(Int) = None(Int)

if found.isSome() {
    println(found.unwrap())   // 42
}
```

Use `if let` for ergonomic unwrapping:

```nova
if let value = found {
    println(value)
}
```

Standard library helpers (from `std/core.nv`):

```nova
import super.std.core

let x: Option(Int) = None(Int)
let v = x.orDefault(0)              // 0
let v2 = x.orDoFn(|| 99)            // 99
x.orError("Expected a value")       // prints error and exits if None
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

---

## 14. Tuples

Tuples are fixed-size, typed collections. Access elements by index:

```nova
let t = (42, "hello", true)
let n = t[0]    // Int: 42
let s = t[1]    // String: "hello"
let b = t[2]    // Bool: true
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
import super.std.string   // string manipulation
import super.std.math     // mathematical functions
import super.std.io       // io::prompt, io::readFile
import super.std.hashmap  // HashMap
import super.std.tuple    // tuple utilities
import super.std.tui      // terminal UI helpers
```

### Key std/core.nv exports

| Name | Description |
|---|---|
| `Box(T)` | Heap wrapper for mutable shared state |
| `Gen(start)` | Creates an integer generator starting at `start` |
| `range(n)` | Returns `[0, 1, ..., n-1]` |
| `range(start, end)` | Returns `[start, ..., end-1]` |
| `Maybe(T)` | `Just(value)` or `Nothing` |
| `Result(A, B)` | `Ok(value)` or `Err(error)` |
| `.orDefault(v)` | Unwrap Option or return default |
| `.orError(msg)` | Unwrap Option or print error and exit |
| `.isNone()` | True if Option is None |

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

## 21. Garbage Collector

Nova uses a hybrid garbage collector combining:

1. **Reference counting** -- objects whose reference count drops to zero are freed immediately.
2. **Mark-and-sweep** -- a periodic sweep collects objects involved in reference cycles.

This means:

- Most objects are freed promptly when they go out of scope.
- Circular structures (e.g. two structs referencing each other via `Box`) are still collected.
- No manual memory management is needed.

### Practical implications

- Short-lived allocations (temp strings, list intermediates) are cheap -- freed on scope exit.
- Closures that capture `Box` values extend those values' lifetimes.
- You do not need to worry about memory leaks in normal code.
- The GC introduces no latency spikes for typical workloads.

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

## 25. The Fuzzer

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

## 26. Quick Reference: Common Mistakes

| Mistake | Fix |
|---|---|
| `let x = []; x.push(1)` | `let x = []: Int; x.push(1)` |
| `x += 1` used as expression | Use `x = x + 1` in expressions; `+=` is statement-only |
| `fn extends f(x)` called as `f(x)` | Use `x.f()` (UFCS only) |
| `5 \|> myExtendsFn()` | Pipe only works with non-extends functions |
| `fn extends id(x: $A)` | Not supported -- use concrete type wrapper |
| `match x { 0 => ... }` | Literals not allowed in match -- use `if`/`elif` |
| `-10 % 3 == -1` | Wrong -- Nova uses Euclidean modulo: `-10 % 3 == 2` |
| `Cast::int(x)` used as `Int` | Returns `Option(Int)` -- must `.unwrap()` |
| `Cast::float(x)` used as `Float` | Returns `Option(Float)` -- must `.unwrap()` |
| Mutual recursion | Not supported -- refactor to single function |
| Naming a struct `Box` | Conflicts with built-in `Box` -- use a different name |
