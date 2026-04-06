# Nova Reference

Complete API reference for Nova's built-in functions, standard library,
and raylib bindings.

For language features, syntax, imports, and CLI usage, see the
[Tutorial](tutorial.md).

---

## Table of Contents

1. [Built-in Functions](#1-built-in-functions)
   — [Output](#output) · [Type Inspection](#type-inspection) · [Option Handling](#option-handling) · [Type Conversion](#type-conversion) · [Hashing](#hashing) · [String Functions](#string-functions-string) · [Char Functions](#char-functions-char) · [Float Functions](#float-functions-float) · [List Functions](#list-functions) · [I/O](#io) · [Random](#random) · [Time](#time) · [Terminal](#terminal) · [Regex](#regex) · [Program](#program) · [Data Serialization](#data-serialization)
2. [Standard Library](#2-standard-library)
   - [`std/core`](#stdcore--foundation) — Box, Maybe, Result, range, Gen
   - [`std/math`](#stdmath--extended-mathematics) — Int/Float extensions, primes, fib
   - [`std/string`](#stdstring--string-utilities) — pad, slug, wrap, between
   - [`std/list`](#stdlist--list-utilities) — sort, zip, chunk, group, filter, map
   - [`std/option`](#stdoption--option-combinators) — orDefault, map, flatMap, filter
   - [`std/maybe`](#stdmaybe--maybe-type) — Just/Nothing algebraic type
   - [`std/result`](#stdresult--error-handling) — Ok/Err, map, andThen
   - [`std/iter`](#stditer--lazy-iterators) — lazy map, filter, take, collect
   - [`std/functional`](#stdfunctional--higher-order-utilities) — compose, pipe, memoize
   - [`std/tuple`](#stdtuple--pair-and-triple) — swap, fst, snd, unzip
   - [`std/hashmap`](#stdhashmap--hash-map) — insert, get, merge, mapValues
   - [`std/set`](#stdset--set) — union, intersection, difference
   - [`std/vec2`](#stdvec2--2d-vector-math) — add, dot, normalize, lerp
   - [`std/deque`](#stddeque--double-ended-queue) — pushBack, pushFront, pop
   - [`std/io`](#stdio--file-and-console-io) — prompt, readLines, writeLines
   - [`std/ansi`](#stdansi--ansi-terminal-colours) — bold, red, rgb, clearScreen
   - [`std/color`](#stdcolor--named-colour-tuples) — named RGB tuples, lerp, darken
   - [`std/tui`](#stdtui--terminal-ui) — printAt, drawBox, poll
   - [`std/widget`](#stdwidget--gui-widget-toolkit) — Button, Label, Panel, Toggle, ProgressBar, Checkbox, Slider, TextField, RadioGroup, Dropdown
   - [`std/container`](#stdcontainer--layout-containers) — VBox, HBox, GridBox, PanelBox, MarginBox (Godot-style)
   - [`std/plot`](#stdplot--charts--graphs-raylib) — line, bar, scatter, pie charts
   - [`std/timer`](#stdtimer--game-timers) — cooldown, repeating, once
   - [`std/tween`](#stdtween--interpolation-and-easing) — easeIn, easeOut, bounce, elastic
   - [`std/input`](#stdinput--action-based-input) — InputMap, bindKey, axis
   - [`std/camera`](#stdcamera--2d-camera) — follow, shake, zoom, worldToScreen
   - [`std/physics`](#stdphysics--2d-physics) — Body2D, AABB, Circle, raycasting
   - [`std/entity`](#stdentity--generic-entity-system) — Entity(T), EntityWorld(T), spawn, query, signals
   - [`std/scene`](#stdscene--scene-management) — SceneManager, push, pop, switch, signals
   - [`std/grid`](#stdgrid--2d-grid-and-tilemap) — Grid(T), get, set, bfs, draw, printGrid (terminal)
   - [`std/signal`](#stdsignal--signals) — Signal(T), VoidSignal, SignalBus
   - [`std/noise`](#stdnoise--procedural-noise) — fbm, ridged, domain warp
   - [`std/particle`](#stdparticle--particle-system) — Emitter, Particle, presets (fountain, fire, snow)
   - [`std/log`](#stdlog--structured-logging) — Logger, levels, colored output, tables, timers
   - [`std/datetime`](#stddatetime--date-and-time) — DateTime, Stopwatch, formatting
   - [`std/gameloop`](#stdgameloop--automatic-update-system) — Updater, Dyn-based tick-all pattern
3. [Raylib API](#3-raylib-api)

---

## 1. Built-in Functions

These functions are available without any imports.

### Output

| Signature | Description |
|---|---|
| `print(a) -> Void` | Print a value to stdout. |
| `println(a) -> Void` | Print a value followed by a newline. |

### Type Inspection

| Signature | Description |
|---|---|
| `typeof(a) -> String` | Return the type of a value as a string. |
| `clone(a) -> a` | Create a deep copy of a value. |

### Option Handling

| Signature | Description |
|---|---|
| `Some(a) -> Option(a)` | Wrap a value in an Option. |
| `None(T) -> Option(T)` | Create an empty Option of a given type. |
| `isSome(Option(a)) -> Bool` | Check if an Option contains a value. |
| `unwrap(Option(a)) -> a` | Extract the value. Panics if None. |

### Type Conversion

| Signature | Description |
|---|---|
| `Cast::string(a) -> String` | Convert any value to a String. |
| `Cast::int(a) -> Option(Int)` | Convert a value to Int. Returns None on failure. |
| `Cast::float(a) -> Option(Float)` | Convert a value to Float. Returns None on failure. |
| `Cast::charToInt(Char) -> Int` | Get the Unicode codepoint of a character. |
| `toStr(a) -> String` | Alias for `Cast::string`. |
| `toInt(a) -> Option(Int)` | Alias for `Cast::int`. |
| `chr(Int) -> Char` | Convert an integer (codepoint) to a character. |

### Hashing

| Signature | Description |
|---|---|
| `hash(a) -> Int` | Deterministic non-negative hash (FNV-1a for strings, Knuth for ints/chars). |

### String Functions (`String::`)

| Signature | Description |
|---|---|
| `strlen(String) -> Int` | Length of a string. |
| `strToChars(String) -> [Char]` | String to list of characters. |
| `charsToStr([Char]) -> String` | List of characters to string. |
| `String::contains(String, String) -> Bool` | Check if a string contains a substring. |
| `String::startsWith(String, String) -> Bool` | Check if a string starts with a prefix. |
| `String::endsWith(String, String) -> Bool` | Check if a string ends with a suffix. |
| `String::trim(String) -> String` | Remove leading and trailing whitespace. |
| `String::trimStart(String) -> String` | Remove leading whitespace. |
| `String::trimEnd(String) -> String` | Remove trailing whitespace. |
| `String::toUpper(String) -> String` | Convert to uppercase. |
| `String::toLower(String) -> String` | Convert to lowercase. |
| `String::replace(String, String, String) -> String` | Replace all occurrences of a substring. |
| `String::substring(String, Int, Int) -> String` | Extract substring by start index and length. |
| `String::indexOf(String, String) -> Option(Int)` | First index of a substring, or None. |
| `String::repeat(String, Int) -> String` | Repeat a string N times. |
| `String::reverse(String) -> String` | Reverse the characters. |
| `String::isEmpty(String) -> Bool` | Check if empty. |
| `String::charAt(String, Int) -> Option(Char)` | Character at index, or None. |
| `String::split(String, String) -> [String]` | Split by a delimiter. |
| `join([String], String) -> String` | Join strings with a separator. |
| `String::toInt(String) -> Option(Int)` | Parse a string as an integer. |

### Char Functions (`Char::`)

| Signature | Description |
|---|---|
| `Char::isAlpha(Char) -> Bool` | Alphabetic? |
| `Char::isDigit(Char) -> Bool` | Digit? |
| `Char::isWhitespace(Char) -> Bool` | Whitespace? |
| `Char::isAlphanumeric(Char) -> Bool` | Alphanumeric? |
| `Char::toUpper(Char) -> Char` | To uppercase. |
| `Char::toLower(Char) -> Char` | To lowercase. |
| `Char::isUpper(Char) -> Bool` | Uppercase? |
| `Char::isLower(Char) -> Bool` | Lowercase? |
| `Char::toInt(Char) -> Int` | Unicode codepoint. |

### Float Functions (`Float::`)

| Signature | Description |
|---|---|
| `Float::floor(Float) -> Float` | Round down. |
| `Float::ceil(Float) -> Float` | Round up. |
| `Float::round(Float) -> Float` | Round to nearest. |
| `Float::abs(Float) -> Float` | Absolute value. |
| `Float::sqrt(Float) -> Float` | Square root. |
| `Float::sin(Float) -> Float` | Sine (radians). |
| `Float::cos(Float) -> Float` | Cosine (radians). |
| `Float::tan(Float) -> Float` | Tangent (radians). |
| `Float::atan2(Float, Float) -> Float` | Two-argument arctangent. |
| `Float::log(Float) -> Float` | Natural logarithm. |
| `Float::log10(Float) -> Float` | Base-10 logarithm. |
| `Float::pow(Float, Float) -> Float` | Raise to a power. |
| `Float::exp(Float) -> Float` | e raised to a power. |
| `Float::min(Float, Float) -> Float` | Minimum of two floats. |
| `Float::max(Float, Float) -> Float` | Maximum of two floats. |
| `Float::clamp(Float, Float, Float) -> Float` | Clamp between min and max. |
| `Float::isNan(Float) -> Bool` | Is NaN? |
| `Float::isInfinite(Float) -> Bool` | Is infinite? |
| `Float::pi() -> Float` | The constant π. |
| `Float::e() -> Float` | The constant e. |

### List Functions

| Signature | Description |
|---|---|
| `len([a]) -> Int` | Length of a list. |
| `push([a], a) -> Void` | Append an element. |
| `pop([a]) -> Option(a)` | Remove and return the last element. |
| `remove([a], Int) -> Void` | Remove element at index. |
| `insert([a], Int, a) -> Void` | Insert element at index. |
| `swap([a], Int, Int) -> Void` | Swap two elements by index. |
| `clear([a]) -> Void` | Remove all elements. |
| `set([a], Int, a) -> Void` | Set element at index. |

### I/O

| Signature | Description |
|---|---|
| `readln() -> String` | Read a line from stdin. |
| `readFile(String) -> String` | Read file contents. |
| `writeFile(String, String) -> Void` | Write to file (creates or overwrites). |
| `appendFile(String, String) -> Void` | Append to file. |
| `fileExists(String) -> Bool` | Check if file exists. |
| `printf(String, [String]) -> Void` | Print formatted string (`{}` placeholders). |
| `format(String, [String]) -> String` | Format string without printing. |

### Random

| Signature | Description |
|---|---|
| `random(Int, Int) -> Int` | Random integer in range (inclusive). |
| `randomFloat(Float, Float) -> Float` | Random float in range. |
| `randomBool() -> Bool` | Random boolean. |

### Time

| Signature | Description |
|---|---|
| `sleep(Int) -> Void` | Pause for milliseconds. |
| `now() -> Int` | Current time in milliseconds since Unix epoch. |
| `nowSec() -> Int` | Current time in seconds since Unix epoch. |

### Terminal

| Signature | Description |
|---|---|
| `terminal::clearScreen() -> Void` | Clear the terminal screen. |
| `terminal::hideCursor() -> Void` | Hide the cursor. |
| `terminal::showCursor() -> Void` | Show the cursor. |
| `terminal::rawmode(Bool) -> Void` | Enable/disable raw mode. |
| `terminal::getch() -> Option(Char)` | Read a single character (no newline wait). |
| `terminal::rawread(Int) -> Option(Char)` | Read a character in raw mode with timeout (ms). |
| `terminal::moveTo(Int, Int) -> Void` | Move cursor to (column, row), 0-based. |
| `terminal::getSize() -> (Int, Int)` | Terminal size as (width, height). |
| `terminal::setForeground(Int, Int, Int) -> Void` | Set text foreground (R, G, B). |
| `terminal::setBackground(Int, Int, Int) -> Void` | Set text background (R, G, B). |
| `terminal::resetColor() -> Void` | Reset colours to defaults. |
| `terminal::print(String) -> Void` | Write string without trailing newline. |
| `terminal::flush() -> Void` | Flush stdout. |
| `terminal::enableMouse() -> Void` | Enable mouse event capture. |
| `terminal::disableMouse() -> Void` | Disable mouse event capture. |

### Regex

| Signature | Description |
|---|---|
| `Regex::matches(String, String) -> Bool` | Test whether a string matches a regex pattern. |
| `Regex::first(String, String) -> Option((Int, Int, String))` | First match: (start, end, text). |
| `Regex::captures(String, String) -> [String]` | All capture groups from a match. |

### Program

| Signature | Description |
|---|---|
| `exit() -> Void` | Terminate the program. |
| `error() -> Void` | Trigger a runtime error and halt. |
| `todo() -> T` | Placeholder for unimplemented code. Compiles as any return type. |
| `unreachable() -> T` | Marks unreachable code. Compiles as any return type. |
| `terminal::args() -> Option([String])` | Command-line arguments after the script name. Returns `None` if no extra arguments. |

### Data Serialization

Save and load any Nova value as JSON.  The JSON embeds `_type` tags so values
round-trip with full type fidelity.  Closures and functions cannot be serialized.

| Signature | Description |
|---|---|
| `Data::save(path: String, value: T) -> Bool` | Serialize `value` to a JSON file. Returns `true` on success. |
| `Data::load(path: String) -> Option(T)` | Deserialize a value from a JSON file. Returns `None` on failure. Needs `@[T: Type]`. |
| `Data::toJson(value: T) -> String` | Serialize `value` to a JSON string. |
| `Data::fromJson(json: String) -> Option(T)` | Deserialize a value from a JSON string. Returns `None` on failure. Needs `@[T: Type]`. |

---

## 2. Standard Library

All modules live in `nova-lang/std/`. Import with `import super.std.<name>`
(dot-path relative to your file). See the [Tutorial — Imports](tutorial.md#20-imports-and-the-standard-library) for
full syntax details including GitHub imports.

---

### `std/core` — Foundation

```rust
import super.std.core
```

The core module is the "batteries included" import. One line gives you `Box`,
`Gen`, `Maybe`, `Result`, range helpers, and Option bridge functions.

#### Box(T) — Mutable Shared Wrapper

Nova closures capture primitives **by value**. `Box(T)` wraps a value on the
heap so multiple closures can read and write the same state through `.value`.

```rust
let counter = Box(0)
let inc = || { counter.value += 1 }
inc()
inc()
println(Cast::string(counter.value))  // "2"
```

| Method | Description |
|---|---|
| `Box(value)` | Wrap any value. Access via `.value`. |
| `.toString()` | Returns `Cast::string(self.value)`. |
| `.show()` | Prints the value with `println`. |

#### Gen(start) — Stateful Counter

Returns a closure that yields successive integers starting from `start`.

```rust
let id = Gen(0)
println(Cast::string(id()))  // "0"
println(Cast::string(id()))  // "1"
println(Cast::string(id()))  // "2"
```

#### Maybe(A) — Algebraic Optional

A heap-allocated enum with two variants: `Just(value)` and `Nothing()`.
Unlike the built-in `Option(T)` (a VM stack sentinel), `Maybe` lives on the
heap and supports pattern matching.

```rust
let x = Maybe::Just(42)
let y = Maybe::Nothing() @[A: Int]

match x {
    Just(v)   => { println(Cast::string(v)) }   // "42"
    Nothing() => { println("empty") }
}
```

| Method | Signature | Description |
|---|---|---|
| `Maybe::Just(v)` | `$A -> Maybe($A)` | Wrap a value |
| `Maybe::Nothing()` | `-> Maybe($A)` | Empty (needs `@[A: Type]` annotation) |
| `.isJust()` | `-> Bool` | True if Just |
| `.isNothing()` | `-> Bool` | True if Nothing |
| `.unwrap()` | `-> $A` | Extract value (panics on Nothing) |
| `.orDefault(v)` | `($A) -> $A` | Value, or fallback if Nothing |
| `.orDoFn(f)` | `(fn()->$A) -> $A` | Value, or lazy fallback |
| `.map(f)` | `(fn($A)->$B) -> Maybe($B)` | Transform inner value |
| `.toString()` | `-> String` | `"Just(42)"` or `"Nothing"` |
| `.toOption()` | `-> Option($A)` | Convert to built-in Option |

```rust
let m = Maybe::Just(10)
let doubled = m.map(|n: Int| n * 2)       // Just(20)
let fallback = m.orDefault(0)             // 10
println(doubled.toString())               // "Just(20)"
```

#### Result(A, B) — Error Handling

A heap-allocated enum for operations that can fail: `Ok(value)` or `Err(error)`.

```rust
fn divide(a: Int, b: Int) -> Result(Int, String) {
    if b == 0 { return Result::Err("division by zero") @[A: Int] }
    return Result::Ok(a / b) @[B: String]
}

let r = divide(10, 3)
println(Cast::string(r.unwrap()))  // "3"

let bad = divide(1, 0)
println(Cast::string(bad.orDefault(-1)))  // "-1"
```

| Method | Signature | Description |
|---|---|---|
| `Result::Ok(v)` | `$A -> Result($A,$B)` | Wrap a success value |
| `Result::Err(e)` | `$B -> Result($A,$B)` | Wrap an error |
| `.isOk()` / `.isErr()` | `-> Bool` | Test which variant |
| `.unwrap()` | `-> $A` | Extract value (panics on Err) |
| `.unwrapErr()` | `-> $B` | Extract error (panics on Ok) |
| `.orDefault(v)` | `($A) -> $A` | Value, or fallback |
| `.orDoFn(f)` | `(fn()->$A) -> $A` | Value, or lazy fallback |
| `.map(f)` | `(fn($A)->$C) -> Result($C,$B)` | Transform success value |
| `.mapErr(f)` | `(fn($B)->$C) -> Result($A,$C)` | Transform error |
| `.toString()` | `-> String` | `"Ok(3)"` or `"Err(msg)"` |
| `.toOption()` | `-> Option($A)` | Ok→Some, Err→None |

#### Option Bridges

Convert the built-in `Option(T)` to `Maybe` or `Result`:

```rust
Some(42).toMaybe()            // Maybe::Just(42)
None(Int).toMaybe()           // Maybe::Nothing()

Some(10).toResult("missing")  // Result::Ok(10)
None(Int).toResult("missing") // Result::Err("missing")
```

#### Range Helpers

Build integer lists for loops and comprehensions.

```rust
range(5)          // [0, 1, 2, 3, 4]
range(2, 6)       // [2, 3, 4, 5]
3.to(7)           // [3, 4, 5, 6]      (UFCS)
5.iota()          // [0, 1, 2, 3, 4]   (UFCS)
0.toStep(10, 3)   // [0, 3, 6, 9]      (UFCS)
```

| Function | Description |
|---|---|
| `range(n)` | `[0, 1, ..., n-1]` |
| `range(start, end)` | `[start, ..., end-1]` |
| `n.to(end)` | `[n, ..., end-1]` (UFCS) |
| `n.iota()` | `[0, ..., n-1]` (UFCS) |
| `start.toStep(end, step)` | `[start, start+step, ..., < end]` (UFCS) |

---

### `std/math` — Extended Mathematics

```rust
import super.std.math
```

Extends Int and Float with UFCS methods. Also provides standalone math functions.

#### Int Extensions

```rust
(-5).abs()          // 5
3.pow(4)            // 81
2.sqrt()            // 1.414...
10.clamp(0, 8)      // 8
5.factorial()        // 120
12.gcd(8)            // 4
12.lcm(8)            // 24
6.isEven()           // true
7.isOdd()            // true
(-3).sign()          // -1
2.modpow(10, 1000)   // 24  (2^10 % 1000)
17.isPrime()         // true
1234.digitSum()      // 10
1234.digits()        // [1, 2, 3, 4]
3.min(5)             // 3
3.max(5)             // 5
```

| Method | Description |
|---|---|
| `n.abs()` | Absolute value |
| `n.pow(exp)` | Integer exponentiation |
| `n.sqrt()` → Float | Square root |
| `n.exp()` → Float | e^n |
| `n.clamp(lo, hi)` | Clamp into range |
| `n.factorial()` | n! |
| `n.gcd(other)` / `n.lcm(other)` | GCD / LCM |
| `n.isEven()` / `n.isOdd()` | Parity check |
| `n.sign()` | -1, 0, or 1 |
| `n.modpow(exp, mod)` | Fast modular exponentiation |
| `n.isPrime()` | Primality test (trial division) |
| `n.digitSum()` | Sum of decimal digits |
| `n.digits()` | List of decimal digits `[Int]` |
| `n.min(other)` / `n.max(other)` | Smaller / larger |

#### Float Extensions

```rust
90.0.radians()              // 1.5707...  (π/2)
Float::pi().degrees()       // 180.0
5.0.normalize(0.0, 10.0)    // 0.5
5.0.mapRange(0.0, 10.0, 0.0, 100.0)  // 50.0
```

| Method | Description |
|---|---|
| `f.radians()` | Degrees → radians |
| `f.degrees()` | Radians → degrees |
| `f.normalize(lo, hi)` | Map to [0.0, 1.0] within range |
| `f.mapRange(fromLo, fromHi, toLo, toHi)` | Remap between ranges |

#### Standalone Functions

```rust
fib(10)       // 55
fibSeq(6)     // [0, 1, 1, 2, 3, 5]
bin(42)       // "101010"
hex(255)      // "ff"
oct(8)        // "10"
divmod(17, 5) // (3, 2)
lerp(0.0, 10.0, 0.5)  // 5.0
smoothstep(0.5)        // 0.5  (3t²-2t³)
primes(20)   // [2, 3, 5, 7, 11, 13, 17, 19]
collatz(6)   // [6, 3, 10, 5, 16, 8, 4, 2, 1]
```

| Function | Description |
|---|---|
| `fib(n)` / `fibSeq(n)` | nth Fibonacci / first n Fibonacci numbers |
| `bin(n)` / `hex(n)` / `oct(n)` | Int → binary / hex / octal string |
| `divmod(n, d)` | `(quotient, remainder)` |
| `toRadians(deg)` / `toDegrees(rad)` | Degree ↔ radian conversion |
| `lerp(a, b, t)` | Float linear interpolation (t ∈ [0, 1]) |
| `lerpF(a, b, t)` | Int linear interpolation (floors result) |
| `remap(v, fromLo, fromHi, toLo, toHi)` | Remap value between ranges |
| `round(f)` | Float → nearest Int |
| `smoothstep(t)` | Smooth Hermite curve (3t²−2t³) |
| `sign(x)` | Float sign: -1.0, 0.0, or 1.0 |
| `isPrime(n)` | Standalone primality test |
| `primes(n)` | All primes ≤ n (Sieve of Eratosthenes) |
| `collatz(n)` | Collatz sequence from n to 1 |

---

### `std/string` — String Utilities

```rust
import super.std.string
```

Extends strings with padding, searching, classification, and transformation.

```rust
"hello".padLeft(10, '.')      // ".....hello"
"hi".padRight(8, '-')         // "hi------"
"yes".center(9, '=')          // "===yes==="
"banana".count("an")          // 2
"hello".countChar('l')        // 2
"hello".indexOfChar('l')      // 2
"12345".isDigit()             // true
"Hello".isAlpha()             // true
"hello".capitalize()          // "Hello"
"hello world".title()         // "Hello World"
"a-b-c".removeChar('-')       // "abc"
"a.b.c".replaceChar('.', '/')  // "a/b/c"
"one\ntwo\nthree".lines()     // ["one", "two", "three"]
"  the  quick  fox ".words()  // ["the", "quick", "fox"]
"hello world".truncate(7, "...")  // "hell..."
"Hello World".slugify()        // "hello-world"
"this is a long text".wrap(10) // "this is a\nlong text"
"[hello]".between("[", "]")    // Some("hello")
"foobar".stripPrefix("foo")    // "bar"
"foobar".stripSuffix("bar")    // "foo"
"ab".split('b')                // ["a", ""]
```

| Method | Description |
|---|---|
| `s.split(Char)` | Split by a single character |
| `s.padLeft(n, c)` / `s.padRight(n, c)` | Pad to width with char |
| `s.center(n, c)` | Center with padding char |
| `s.count(sub)` | Count non-overlapping substring matches |
| `s.countChar(c)` | Count character occurrences |
| `s.indexOfChar(c)` | First index of char, or -1 |
| `s.isDigit()` / `s.isAlpha()` / `s.isAlphanumeric()` | Character class checks |
| `s.capitalize()` | Uppercase first character |
| `s.title()` | Capitalize each word |
| `s.removeChar(c)` | Delete all occurrences of char |
| `s.replaceChar(old, new)` | Replace all occurrences of a char |
| `s.lines()` | Split on `'\n'` |
| `s.words()` | Split on spaces, drop empty strings |
| `s.truncate(n, suffix)` | Cut with suffix (e.g. `"..."`) |
| `s.slugify()` | Lowercase, spaces→hyphens, strip non-alphanumeric |
| `s.wrap(width)` | Word-wrap at column width |
| `s.between(left, right)` | Extract text between delimiters → `Option(String)` |
| `s.stripPrefix(p)` / `s.stripSuffix(s)` | Remove prefix / suffix if present |

---

### `std/list` — List Utilities

```rust
import super.std.list
```

A comprehensive set of functional-style operations on Nova's built-in list type.
All use UFCS (dot notation).

#### Transforming

```rust
[1, 2, 3].map(|x: Int| x * 2)             // [2, 4, 6]
[1, 2, 3, 4].filter(|x: Int| x > 2)       // [3, 4]
[[1,2],[3,4]].flatten()                     // [1, 2, 3, 4]
[1, 2, 3].flatmap(|x: Int| [x, x * 10])   // [1, 10, 2, 20, 3, 30]
[1, 2, 3].foreach(|x: Int| println(Cast::string(x)))
[1, 2, 3].reverse()                        // [3, 2, 1]
[1, 2, 2, 3, 3].unique()                   // [1, 2, 3]
```

#### Sorting

```rust
[3, 1, 4, 1, 5].quicksort()               // [1, 1, 3, 4, 5]
[3, 1, 2].bubblesort()                     // [1, 2, 3]
[3, 1, 2].sortWith(|a: Int, b: Int| a > b) // [1, 2, 3]
```

#### Searching and Querying

```rust
[10, 20, 30].indexOf(20)                    // 1
[1, 2, 3].contains(2)                       // true
[1, 2, 3, 4].find(|x: Int| x > 2)          // Some(3)
[1, 2, 3, 4].count(|x: Int| x > 2)         // 2
[true, true, true].all()                     // true
[false, true, false].any()                   // true
[1, 2, 3].anyWith(|x: Int| x == 2)          // true
[2, 4, 6].allWith(|x: Int| x % 2 == 0)     // true
[1, 2, 3].last()                             // Some(3)
[1, 2, 3].isEmpty()                          // false
```

#### Slicing and Chunking

```rust
[1, 2, 3, 4, 5].slice(1, 4)               // [2, 3, 4]
[1, 2, 3, 4, 5].take(3)                    // [1, 2, 3]
[1, 2, 3, 4, 5].drop(2)                    // [3, 4, 5]
[1, 2, 3, 4, 5, 6].chunk(2)               // [[1,2],[3,4],[5,6]]
[1, 2, 3, 4].windows(3)                    // [[1,2,3],[2,3,4]]
[1, 2, 3, 4, 5].takeWhile(|x: Int| x < 4) // [1, 2, 3]
[1, 2, 3, 4, 5].dropWhile(|x: Int| x < 3) // [3, 4, 5]
```

#### Aggregation

```rust
[1, 2, 3, 4].sum()     // 10
[1, 2, 3, 4].product()  // 24
[3, 1, 4].max()         // 4
[3, 1, 4].min()         // 1
[1.0, 2.5, 3.0].sum()   // 6.5  (Float overload)
```

#### Combining and Pairing

```rust
[1, 2, 3].concat([4, 5])                    // [1, 2, 3, 4, 5]
[1, 2, 3].append([4, 5])                    // [1, 2, 3, 4, 5]
[1, 2, 3].zip(["a", "b", "c"])              // [(1,"a"),(2,"b"),(3,"c")]
[(1,"a"),(2,"b")].unzip()                    // ([1,2],["a","b"])
[1, 2, 3].enumerate()                       // [(1,0),(2,1),(3,2)]
[1, 2, 3].intersperse(0)                    // [1, 0, 2, 0, 3]
```

#### Folding and Reducing

```rust
[1, 2, 3].foldl(|a: Int, b: Int| a + b)      // 6
[1, 2, 3].foldr(|a: Int, b: Int| a - b)      // 2  (1-(2-3))
[1, 2, 3, 4].reduce(|acc: Int, x: Int, i: Int| acc + x, 0) // 10
```

#### Grouping and Partitioning

```rust
[1, 2, 3, 4, 5, 6].partition(|x: Int| x % 2 == 0)
    // ([2, 4, 6], [1, 3, 5])

["apple","avocado","banana","blueberry"].groupBy(|s: String| s[0])
    // [('a',["apple","avocado"]),('b',["banana","blueberry"])]

[1, 2, 2, 3, 3, 3].group()
    // [(1,1),(2,2),(3,3)]   -- (value, count)
```

#### Advanced

```rust
[1, 2, 3].indices()                   // [0, 1, 2]
[1, 2, 3, 4, 5].shuffle()             // random order
[1, 2, 3, 4, 5].get(2)                // Some(3)
[1, 2, 3, 4, 5].get(99)               // None(Int)
[1, 2, 3].dropFirst(|x: Int| x == 2)  // [1, 3]
[1, 2, 3].dropIndex(1)                // [1, 3]
[1, 2, 3, 4, 5].truncate(2)           // [1, 2, 3]  (removes last 2)
```

#### Bitmask

Filter lists with a boolean mask:

```rust
let data = [10, 20, 30, 40, 50]
let mask = data.bitmask(|x: Int| x > 25)  // Bitmask{data:[0,0,1,1,1]}
let selected = data.selection(mask)         // [30, 40, 50]
let inv = mask.inverse()                    // Bitmask{data:[1,1,0,0,0]}
let excluded = data.selection(inv)          // [10, 20]
```

---

### `std/option` — Option Combinators

```rust
import super.std.option
```

Extends the built-in `Option(T)` (which already has `.isSome()` and `.unwrap()`)
with ergonomic chainable combinators.

```rust
let x: Option(Int) = None(Int)

// Fallbacks
x.orDefault(42)               // 42
x.orDoFn(|| 99)               // 99 (lazy — closure only called if None)
x.orError("value is missing") // prints msg and exits if None

// Querying
x.isNone()                    // true

// Transforming
Some(3).map(|n: Int| n * 2)         // Some(6)
None(Int).map(|n: Int| n * 2)       // None(Int)

// Chaining
Some(4).flatMap(|n: Int| if n > 3 { Some(n) } else { None(Int) })
    // Some(4)

// Filtering
Some(10).filter(|n: Int| n > 5)   // Some(10)
Some(3).filter(|n: Int| n > 5)    // None(Int)

// Combining
Some(1).zip(Some("a"))   // Some((1,"a"))
Some(1).zip(None(String)) // None

// Converting
Some(7).toList()    // [7]
None(Int).toList()  // []

// Side-effects
Some(42).inspect(|n: Int| println(Cast::string(n))).orDefault(0)
    // prints "42", returns 42
```

| Method | Description |
|---|---|
| `opt.isNone()` | True if None |
| `opt.orDefault(v)` | Unwrap or return fallback |
| `opt.orDoFn(f)` | Unwrap or lazily compute fallback |
| `opt.orError(msg)` | Unwrap or print msg and exit |
| `opt.map(f)` | Transform inner value if Some |
| `opt.flatMap(f)` | Chain Option-returning function |
| `opt.filter(pred)` | Keep Some only if pred holds |
| `opt.zip(other)` | Combine two Options → `Option((A,B))` |
| `opt.toList()` | `[v]` if Some, `[]` if None |
| `opt.inspect(f)` | Run side-effect if Some, pass through |

---

### `std/maybe` — Maybe Type

```rust
import super.std.maybe
```

> **Note:** `Maybe` is also available through `std/core`. This standalone
> module exists for when you only need `Maybe` without the rest of core.

See [`std/core`](#stdcore--foundation) for full Maybe documentation.

---

### `std/result` — Error Handling

```rust
import super.std.result
```

> **Note:** `Result` is also available through `std/core`. This standalone
> module exists for when you only need `Result` without the rest of core.

See [`std/core`](#stdcore--foundation) for full Result documentation.

---

### `std/iter` — Lazy Iterators

```rust
import super.std.iter
```

Lazy iterators evaluate one element at a time. Build a pipeline of
transformations, then consume at the end. Nothing runs until you call a
consumer like `.collect()`.

#### Constructing

```rust
Iter::fromRange(0, 5)      // 0, 1, 2, 3, 4
Iter::fromVec([10,20,30])  // 10, 20, 30
Iter::repeat(42)           // 42, 42, 42, ... (infinite)
Iter::generate(Gen(0))     // 0, 1, 2, 3, ...  (infinite)
```

| Constructor | Description |
|---|---|
| `Iter::fromRange(start, end)` | Integers `[start, end)` |
| `Iter::fromVec(list)` | Iterate over a list |
| `Iter::fromFn(f)` | Custom pull function `fn() -> Option(T)` |
| `Iter::enumerate(iter)` | Wrap with `(index, value)` pairs |
| `Iter::repeat(v)` | Infinite stream of `v` |
| `Iter::generate(f)` | Infinite stream from `f()` calls |

#### Transforming (lazy)

```rust
Iter::fromRange(0, 10)
    .filter(|x: Int| x % 2 == 0)  // keep even
    .map(|x: Int| x * x)           // square
    .take(3)                        // first 3
    .collect()                      // [0, 4, 16]
```

| Transformer | Description |
|---|---|
| `.map(f)` | Apply f to each element |
| `.filter(pred)` | Keep elements where pred is true |
| `.take(n)` | First n elements |
| `.drop(n)` | Skip first n elements |
| `.takeWhile(pred)` | Take while pred holds |
| `.dropWhile(pred)` | Skip while pred holds |
| `.flatMap(f)` | Map then flatten one level |
| `.zip(other)` | Pair elements from two iterators |
| `.chain(other)` | Append another iterator |

#### Consuming (eager)

```rust
Iter::fromRange(1, 6).sum()          // 15
Iter::fromRange(1, 6).count()        // 5
Iter::fromRange(1, 100)
    .find(|x: Int| x > 50)          // Some(51)
Iter::fromRange(1, 6)
    .reduce(|acc: Int, x: Int| acc + x, 0)  // 15
Iter::fromVec(["a","b","c"]).nth(1)  // Some("b")
Iter::fromRange(1, 6).last()         // Some(5)
```

| Consumer | Description |
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
| `.forEach(f)` | Run side-effect on each |

---

### `std/functional` — Higher-Order Utilities

```rust
import super.std.functional
```

Tools for working with Nova's first-class closures.

```rust
// Composition
let double  = |x: Int| x * 2
let addOne  = |x: Int| x + 1
let doubleThenAdd = pipe(double, addOne)
doubleThenAdd(3)  // 7  (3*2 = 6, 6+1 = 7)

let addThenDouble = compose(double, addOne)
addThenDouble(3)  // 8  (3+1 = 4, 4*2 = 8)

// Flip arguments
let sub = |a: Int, b: Int| a - b
let flipped = flip(sub)
flipped(3, 10)  // 7  (10 - 3)

// Apply repeatedly
applyN(|x: Int| x * 2, 4, 1)   // 16  (1→2→4→8→16)
applyWhile(|x: Int| x * 2, |x: Int| x < 100, 1)  // 128

// Constants and identity
let always5 = const_(5) @[B: String]
always5("anything")  // 5
identity(42)         // 42

// Predicate combinators
let isEven = |x: Int| x % 2 == 0
let isPositive = |x: Int| x > 0
let evenAndPositive = both(isEven, isPositive)
evenAndPositive(4)    // true
evenAndPositive(-2)   // false

let notEven = negate(isEven)
notEven(3)  // true

// Memoization
let fastFib = memoize(fib)
fastFib(30)  // cached after first call
```

| Function | Description |
|---|---|
| `compose(f, g)` | `fn(x) -> f(g(x))` (right-to-left) |
| `pipe(f, g)` | `fn(x) -> g(f(x))` (left-to-right) |
| `flip(f)` | Swap two arguments of a binary fn |
| `const_(v)` | Always return v |
| `identity(x)` | Return x unchanged |
| `applyN(f, n, x)` | Apply f to x exactly n times |
| `applyWhile(f, pred, x)` | Apply f while pred holds |
| `memoize(f)` | Cache results (String key cache) |
| `negate(pred)` | `fn(x) -> !pred(x)` |
| `both(p, q)` | `fn(x) -> p(x) && q(x)` |
| `either(p, q)` | `fn(x) -> p(x) \|\| q(x)` |

---

### `std/tuple` — Pair and Triple

```rust
import super.std.tuple
```

Utilities for working with tuples.

```rust
let p = (1, "hello")
p.fst()               // 1
p.snd()               // "hello"
p.swap()              // ("hello", 1)
p.mapFirst(|x: Int| x + 10)    // (11, "hello")
p.mapSecond(|s: String| s.len())  // (1, 5)

let nums = (3, 7)
nums.both(|n: Int| n * 2)   // (6, 14)
nums.toList()                // [3, 7]

// Convenience constructors
pair(1, 2)        // (1, 2)
triple(1, 2, 3)   // (1, 2, 3)

// Standalone
fst((10, 20))     // 10
snd((10, 20))     // 20

// Unzip a list of pairs
[(1,"a"),(2,"b"),(3,"c")].unzip()  // ([1,2,3],["a","b","c"])
```

| Method | Description |
|---|---|
| `t.swap()` | `(b, a)` |
| `t.fst()` / `t.snd()` | First / second element |
| `t.mapFirst(f)` / `t.mapSecond(f)` | Transform one element |
| `t.both(f)` | Apply f to both (when A==B) |
| `t.toStrings()` | `[String, String]` |
| `t.toList()` | `[a, b]` (when A==B) |
| `pairs.unzip()` | `[(A,B)]` → `([A], [B])` |
| `pair(a, b)` / `triple(a, b, c)` | Convenience constructors |

---

### `std/hashmap` — Hash Map

```rust
import super.std.hashmap
```

O(1) average-case hash map using bucket chaining. Automatically resizes at 75% load.

```rust
// Create and populate
let m = HashMap::new() @[K: String, V: Int]
m.insert("apples", 5)
m.insert("bananas", 3)

// Lookup
m.get("apples")           // Some(5)
m.get("oranges")          // None(Int)
m.getOrDefault("oranges", 0)  // 0
m.has("apples")           // true

// Modify
m.delete("bananas")
m.size()                  // 1

// Counting pattern
let freq = HashMap::new() @[K: String, V: Int]
freq.increment("hello")
freq.increment("hello")
freq.increment("world")
freq.get("hello")         // Some(2)

// Iterate
m.forEach(|k: String, v: Int| {
    println(k + " = " + Cast::string(v))
})

// Collections
m.keys()       // ["apples"]
m.values()     // [5]
m.entries()    // [("apples", 5)]

// Build from pairs
let m2 = HashMap::fromPairs([("x", 1), ("y", 2)]) @[K: String, V: Int]

// Filter and merge
let big = m.filterValues(|v: Int| v > 2)
m.merge(m2)  // m now has entries from m2 too

// Update with function
m.update("apples", 0, |v: Int| v + 10)  // apples = 15
```

| Method | Description |
|---|---|
| `HashMap::new()` | Empty map (16-bucket initial) |
| `HashMap::fromPairs(list)` | Build from `[(K,V)]` |
| `.insert(k, v)` | Insert or update |
| `.get(k)` → `Option(V)` | Lookup |
| `.getOrDefault(k, v)` | Lookup with fallback |
| `.delete(k)` | Remove entry |
| `.has(k)` | Key exists? |
| `.size()` / `.isEmpty()` | Count / empty check |
| `.clear()` | Remove all entries |
| `.keys()` / `.values()` / `.entries()` | Collection views |
| `.forEach(f)` | Iterate `(k, v)` pairs |
| `.merge(other)` | Insert all from other (other wins on conflict) |
| `.filterKeys(pred)` / `.filterValues(pred)` | Filter into new map |
| `.increment(k)` | Increment Int value (counting helper) |
| `.update(k, default, f)` | Update with function |
| `.toString()` | `"{k1 => v1, k2 => v2}"` |

---

### `std/set` — Set

```rust
import super.std.set
```

A generic set backed by `HashMap`. Values must be hashable.

```rust
let s = Set::fromList([1, 2, 3, 2, 1]) @[T: Int]
s.size()         // 3
s.has(2)         // true
s.add(4)
s.remove(1)

let a = Set::fromList([1, 2, 3]) @[T: Int]
let b = Set::fromList([2, 3, 4]) @[T: Int]

a.union(b).toList()         // [1, 2, 3, 4]  (order may vary)
a.intersection(b).toList()  // [2, 3]
a.difference(b).toList()    // [1]
a.isSubset(b)               // false
a.isDisjoint(b)             // false

// Higher-order
let evens = a.filter(|n: Int| n % 2 == 0)   // {2}
let doubled = a.map(|n: Int| n * 2)          // {2, 4, 6}
a.forEach(|n: Int| println(Cast::string(n)))

println(a.toString())  // "{1, 2, 3}"
```

| Method | Description |
|---|---|
| `Set::empty()` / `Set::singleton(v)` / `Set::fromList(list)` | Construct |
| `.add(v)` / `.remove(v)` | Mutate |
| `.has(v)` | Membership test |
| `.size()` / `.isEmpty()` | Count |
| `.toList()` | All elements as list |
| `.union(other)` / `.intersection(other)` / `.difference(other)` | Set operations |
| `.isSubset(other)` / `.isSuperset(other)` / `.isDisjoint(other)` | Comparisons |
| `.forEach(f)` / `.filter(pred)` / `.map(f)` | Higher-order |
| `.toString()` | `"{a, b, c}"` |

---

### `std/vec2` — 2D Vector Math

```rust
import super.std.vec2
```

A 2D floating-point vector for game math and physics.

```rust
let a = Vec2::new(3.0, 4.0)
let b = Vec2::new(1.0, 0.0)

a.length()              // 5.0
a.normalized()          // Vec2(0.6, 0.8)
a.add(b)                // Vec2(4.0, 4.0)
a.sub(b)                // Vec2(2.0, 4.0)
a.scale(2.0)            // Vec2(6.0, 8.0)
a.dot(b)                // 3.0
a.cross(b)              // -4.0
a.distance(b)           // ~4.47
a.angle()               // 0.927... radians
a.rotate(Float::pi())   // Vec2(-3.0, -4.0)
a.lerp(b, 0.5)          // Vec2(2.0, 2.0)
a.reflect(Vec2::up())   // reflect across y-axis
a.perpendicular()       // Vec2(4.0, -3.0)
a.clampLength(3.0)      // scaled down to length 3
a.negate()              // Vec2(-3.0, -4.0)
a.abs()                 // Vec2(3.0, 4.0)

Vec2::zero()            // (0, 0)
Vec2::one()             // (1, 1)
Vec2::fromAngle(1.57)   // unit vector at ~90°
```

| Method | Description |
|---|---|
| `Vec2::new(x, y)` / `::zero()` / `::one()` / `::up()` / `::right()` | Constructors |
| `::fromAngle(rad)` | Unit vector at angle |
| `.add(v)` / `.sub(v)` | Component-wise arithmetic |
| `.scale(s)` / `.negate()` / `.abs()` | Scalar operations |
| `.dot(v)` / `.cross(v)` | Dot product / 2D cross (scalar) |
| `.length()` / `.lengthSq()` | Magnitude |
| `.normalized()` | Unit vector |
| `.distance(v)` / `.distanceSq(v)` | Distance |
| `.angle()` / `.angleTo(v)` | Angle (radians) |
| `.rotate(rad)` | Rotate by radians |
| `.lerp(v, t)` | Linear interpolation |
| `.reflect(normal)` | Reflect across normal |
| `.perpendicular()` | 90° clockwise rotation |
| `.clampLength(max)` | Scale down if too long |
| `.equals(v)` / `.isZero()` | Equality |
| `.toString()` | `"(3.0, 4.0)"` |

---

### `std/deque` — Double-Ended Queue

```rust
import super.std.deque
```

A double-ended queue with efficient push/pop at both ends.

```rust
let d = Deque::fromList([1, 2, 3]) @[T: Int]

d.pushBack(4)       // [1, 2, 3, 4]
d.pushFront(0)      // [0, 1, 2, 3, 4]
d.popFront()        // Some(0)  → [1, 2, 3, 4]
d.popBack()         // Some(4)  → [1, 2, 3]
d.peekFront()       // Some(1)
d.peekBack()        // Some(3)
d.len()             // 3
d.isEmpty()         // false
d.toList()          // [1, 2, 3]

// Higher-order
let doubled = d.map(|x: Int| x * 2)        // Deque[2, 4, 6]
let big = d.filter(|x: Int| x > 1)         // Deque[2, 3]
d.forEach(|x: Int| println(Cast::string(x)))

println(d.toString())  // "Deque[1, 2, 3]"
```

| Method | Description |
|---|---|
| `Deque::empty()` / `::singleton(v)` / `::fromList(xs)` | Construct |
| `.pushBack(v)` / `.pushFront(v)` | Add to back / front |
| `.popBack()` / `.popFront()` | Remove from back / front → `Option` |
| `.peekBack()` / `.peekFront()` | View without removing → `Option` |
| `.len()` / `.isEmpty()` | Size |
| `.toList()` | Snapshot as list |
| `.forEach(f)` / `.map(f)` / `.filter(pred)` | Higher-order |
| `.toString()` | `"Deque[a, b, c]"` |

---

### `std/io` — File and Console I/O

```rust
import super.std.io
```

Convenience wrappers around Nova's built-in I/O functions.

```rust
// Interactive prompts
let name = prompt("Enter your name: ")
let age = promptInt("Enter age: ")       // Option(Int)
let ok = promptYN("Continue? (y/n) ")    // Bool

// Print with separator
printSep(["one", "two", "three"], ", ")  // "one, two, three\n"

// Error output
eprintln("something went wrong")  // "[error] something went wrong"

// File I/O
writeLines("output.txt", ["line 1", "line 2"])
let lines = readLines("output.txt")  // ["line 1", "line 2"]
appendLine("output.txt", "line 3")

// String splitting
let text = "hello\nworld"
linesOf(text)  // ["hello", "world"]
```

| Function | Description |
|---|---|
| `prompt(msg)` | Print msg, return input line |
| `promptInt(msg)` | Prompt and parse Int → `Option(Int)` |
| `promptFloat(msg)` | Prompt and parse Float → `Option(Float)` |
| `promptYN(msg)` | Prompt for yes/no → `Bool` |
| `printSep(values, sep)` | Print values joined with separator |
| `eprintln(msg)` | Print `[error] msg` |
| `readLines(path)` | File → `[String]` |
| `writeLines(path, lines)` | `[String]` → file |
| `appendLine(path, line)` | Append one line to file |
| `linesOf(text)` | Split string on newlines |

---

### `std/ansi` — ANSI Terminal Colours

```rust
import super.std.ansi
```

Wrap strings with ANSI escape codes for coloured terminal output.

```rust
println(bold("important"))
println(red("error: something failed"))
println(green("success!"))
println(italic(cyan("stylish")))
println(rgb(255, 128, 0, "orange text"))
println(bgRgb(0, 0, 128, "blue background"))
println(color256(196, "bright red"))
```

| Category | Functions |
|---|---|
| Styles | `bold`, `dim`, `italic`, `underline`, `blink`, `invert`, `strikethrough` |
| Foreground | `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` |
| Bright FG | `brightBlack` … `brightWhite` |
| Background | `bgBlack` … `bgWhite`, `bgBrightBlack` … `bgBrightWhite` |
| 256 / RGB | `color256(code, s)`, `bgColor256(code, s)`, `rgb(r,g,b,s)`, `bgRgb(r,g,b,s)` |
| Control | `reset()`, `clearScreen()`, `clearLine()`, `moveTo(row,col)`, `hideCursor()`, `showCursor()` |

---

### `std/color` — Named Colour Tuples

```rust
import super.std.color
```

Named RGB tuples and colour manipulation for use with raylib or terminal.

```rust
let c = red          // (255, 0, 0)
let mixed = lerpColor(red, blue, 0.5)  // purple-ish
let dark = darken(green, 0.5)          // half-brightness green
let light = lighten(blue, 0.3)         // lighter blue
let inv = invert(white)                // (0, 0, 0)
let custom = rgb(128, 64, 200)         // (128, 64, 200)
```

| Name / Function | Description |
|---|---|
| `red`, `green`, `blue`, `white`, `black` | Primary colours `(R,G,B)` |
| `yellow`, `cyan`, `magenta` | Secondary colours |
| `orange`, `purple`, `pink`, `brown`, `gray` | Extended palette |
| `rgb(r, g, b)` | Construct an RGB tuple |
| `lerpColor(a, b, t)` | Interpolate between two colours (t = 0.0–1.0) |
| `invert(c)` | Invert an RGB colour |
| `darken(c, f)` | Darken by factor (0.0–1.0) |
| `lighten(c, f)` | Lighten by factor (0.0–1.0) |

---

### `std/tui` — Terminal UI

```rust
import super.std.tui
```

Lightweight terminal-mode UI functions. Use for text-based games and tools.

```rust
import super.std.tui

fn main() {
    run(|| {
        clear()
        fg(255, 200, 50)
        printAt(10, 5, "Hello, TUI!")
        resetColor()
        drawBox(5, 3, 20, 5)
        flush()
        getch()  // wait for keypress
    })
}
```

| Function | Description |
|---|---|
| `run(fn)` | Start TUI app (sets up raw mode, cleans up on exit) |
| `printAt(x, y, s)` | Print string at column x, row y |
| `clear()` | Clear the terminal |
| `flush()` | Flush output buffer |
| `size()` | Terminal `(width, height)` |
| `fg(r, g, b)` / `bg(r, g, b)` | Set foreground / background colour |
| `resetColor()` | Reset to defaults |
| `drawBox(x, y, w, h)` | Draw Unicode box outline |
| `getch()` | Blocking single-character read |
| `poll()` | Non-blocking character read → `Option(Char)` |

---

### `std/widget` — GUI Widget Toolkit

```rust
import super.std.widget
import super.std.signal
```

Higher-level widgets for raylib game UIs.  Interactive widgets emit
**Godot-style signals** so you can wire up reactions once and keep the
game loop clean.  Every widget has a `drawAt(x, y, w, h)` method for
integration with layout containers (see `std/container`).

| Widget | Description |
|---|---|
| `Button` | Clickable button with label and position |
| `Label` | Static text display |
| `Panel` | Rectangular container with border |
| `ProgressBar` | Horizontal progress indicator |
| `Toggle` | On/off toggle switch |
| `Checkbox` | Check/uncheck box with label |
| `Slider` | Draggable horizontal slider (Float value) |
| `TextField` | Single-line text input with cursor |
| `RadioGroup` | Mutually exclusive radio buttons |
| `Dropdown` | Expandable drop-down menu |

#### Core Widgets

| Method | Description |
|---|---|
| `.draw()` | Render the widget |
| `.drawAt(x, y, w, h)` | Render at arbitrary position (for containers) |
| `.isClicked()` | True if widget was clicked (also emits signals) |
| `.isHovered()` | True if cursor is over widget |
| `Toggle.isOn()` | Current on/off state |
| `ProgressBar.setValue(v)` | Set value and emit `onValueChanged` |

#### New Widgets

**Checkbox** — check/uncheck with label:

```rust
let cb = Checkbox::new(100, 200, 20, "Enable sound")
cb.onToggle.connect(|v: Bool| { println("Sound: " + Cast::string(v)) })

// In game loop:
cb.isClicked()
cb.draw()
println(Cast::string(cb.checked))  // true or false
```

**Slider** — horizontal draggable slider:

```rust
let vol = Slider::new(100, 300, 200, 0.0, 1.0, 0.5)  // x, y, w, min, max, value
vol.onChanged.connect(|v: Float| { println("Volume: " + Cast::string(v)) })

// In game loop:
vol.update()   // handles mouse drag
vol.draw()
```

**TextField** — single-line text input:

```rust
let name = TextField::new(100, 400, 200, 30)
name.placeholder = "Enter name..."
name.onSubmit.connect(|| { println("Submitted: " + name.text) })

// In game loop:
name.update()   // handles keyboard input, cursor, backspace
name.draw()
```

**RadioGroup** — mutually exclusive options:

```rust
let diff = RadioGroup::new(100, 500, ["Easy", "Normal", "Hard"], 1)  // default index 1
diff.onChanged.connect(|idx: Int| { println("Difficulty: " + Cast::string(idx)) })

// In game loop:
diff.isClicked()
diff.draw()
println(Cast::string(diff.selected))  // 0, 1, or 2
```

**Dropdown** — expandable selection menu:

```rust
let weapon = Dropdown::new(100, 600, 160, 30, ["Sword", "Bow", "Staff"])
weapon.onChanged.connect(|idx: Int| { println("Weapon: " + Cast::string(idx)) })

// In game loop:
weapon.update()
weapon.draw()
```

#### Widget Signals

| Widget | Signal | Type | Fires when |
|---|---|---|---|
| `Button` | `onClick` | `VoidSignal` | `isClicked()` detects a click |
| `Button` | `onHover` | `VoidSignal` | `isHovered()` detects cursor over the button |
| `Toggle` | `onToggle` | `Signal(Bool)` | `isClicked()` flips state — carries new on/off value |
| `ProgressBar` | `onValueChanged` | `Signal(Float)` | `setValue(v)` is called — carries the new value |
| `Checkbox` | `onToggle` | `Signal(Bool)` | `isClicked()` flips checked state |
| `Slider` | `onChanged` | `Signal(Float)` | `update()` detects value change via drag |
| `TextField` | `onSubmit` | `VoidSignal` | Enter key is pressed |
| `TextField` | `onChanged` | `Signal(String)` | Text content changes |
| `RadioGroup` | `onChanged` | `Signal(Int)` | `isClicked()` selects a different option |
| `Dropdown` | `onChanged` | `Signal(Int)` | User selects a different item |

#### Container Integration

All widgets can be wrapped into a `Slot` for use with layout containers:

```rust
import super.std.container

let btn = Button::new(0, 0, 160, 40, "OK")
let slot = Slot::new(
    |x: Int, y: Int, w: Int, h: Int| { btn.drawAt(x, y, w, h) },
    fn() -> (Int, Int) { return (160, 40) }
)

let vbox = VBox::new(50, 50, 200)
vbox.add(slot)
vbox.draw()
```

---

### `std/container` — Layout Containers

```rust
import super.std.container
```

Godot-style layout containers that automatically position child **slots**.
Each slot is a pair of closures — `draw(x, y, w, h)` and `preferredSize() -> (Int, Int)` —
so containers stay generic and work with any widget type.

#### Slot — Universal Wrapper

```rust
let slot = Slot::new(
    |x: Int, y: Int, w: Int, h: Int| {
        raylib::drawRectangle(x, y, w, h, (200, 200, 200))
    },
    fn() -> (Int, Int) { return (100, 30) }
)

slot.draw(10, 20, 100, 30)     // invoke drawFn
let sz = slot.preferredSize()   // invoke sizeFn → (100, 30)
```

| Method | Description |
|---|---|
| `Slot::new(drawFn, sizeFn)` | Create a slot |
| `.draw(x, y, w, h)` | Invoke the draw closure |
| `.preferredSize()` → `(Int, Int)` | Invoke the size closure |

#### VBox — Vertical Container

Stacks children top-to-bottom with spacing and padding.

```rust
let vbox = VBox::new(50, 50, 300)   // x, y, width
vbox.spacing = 8
vbox.padding = 10
vbox.add(slot1)
vbox.add(slot2)
vbox.draw()

let h = vbox.totalHeight()
```

| Method | Description |
|---|---|
| `VBox::new(x, y, w)` | Create vertical container |
| `.add(slot)` | Append a slot |
| `.draw()` | Lay out and draw all children |
| `.totalHeight()` → `Int` | `padding*2 + sum(heights) + (n-1)*spacing` |

#### HBox — Horizontal Container

Stacks children left-to-right.

```rust
let hbox = HBox::new(50, 50, 60)   // x, y, height
hbox.spacing = 5
hbox.add(slotA)
hbox.add(slotB)
hbox.draw()

let w = hbox.totalWidth()
```

| Method | Description |
|---|---|
| `HBox::new(x, y, h)` | Create horizontal container |
| `.add(slot)` | Append a slot |
| `.draw()` | Lay out and draw all children |
| `.totalWidth()` → `Int` | `padding*2 + sum(widths) + (n-1)*spacing` |

#### GridBox — Grid Container

Arranges children in rows/columns with fixed cell sizes.

```rust
let grid = GridBox::new(50, 50, 4, 80, 40)  // x, y, cols, cellW, cellH
grid.spacingX = 5
grid.spacingY = 5
grid.padding = 10
for let i = 0; i < 12; i += 1 {
    grid.add(mySlot)
}
grid.draw()
```

| Method | Description |
|---|---|
| `GridBox::new(x, y, cols, cellW, cellH)` | Create grid |
| `.add(slot)` | Append a slot (fills left-to-right, top-to-bottom) |
| `.draw()` | Lay out and draw all children |
| `.totalWidth()` → `Int` | `padding*2 + cols*cellW + (cols-1)*spacingX` |
| `.totalHeight()` → `Int` | `padding*2 + rows*cellH + (rows-1)*spacingY` |

#### PanelBox — Background + Border + Children

```rust
let panel = PanelBox::new(100, 100, 300, 200)
panel.bgColor = color::darkGray()
panel.borderColor = color::gray()
panel.padding = 10
panel.add(slot)
panel.draw()
```

| Method | Description |
|---|---|
| `PanelBox::new(x, y, w, h)` | Create panel |
| `.add(slot)` | Add child slot |
| `.draw()` | Draw background, border, then children in VBox layout |

#### MarginBox — Margin Wrapper

Wraps a single child with configurable margins.

```rust
let inner = Slot::new(myDrawFn, mySizeFn)
let margin = MarginBox::new(inner, 10, 20, 10, 20)  // top, right, bottom, left
margin.draw(0, 0, 300, 200)

let ps = margin.preferredSize()  // child size + margins
```

| Method | Description |
|---|---|
| `MarginBox::new(child, top, right, bottom, left)` | Create margin wrapper |
| `.draw(x, y, w, h)` | Draw child at inset position |
| `.preferredSize()` → `(Int, Int)` | Child preferred size + margins |
| `.toSlot()` → `Slot` | Wrap as a Slot for nesting in other containers |

---

### `std/plot` — Charts & Graphs (Raylib)

```rust
import super.std.plot
```

Requires an active raylib window. Draw charts by creating a `PlotArea` that
maps data coordinates to screen pixels.

```rust
raylib::init("Chart Demo", 800, 600, 30)
let data = [3.0, 7.0, 2.0, 9.0, 5.0]
let area = PlotArea::auto(50, 50, 700, 400, data)

while raylib::rendering() {
    raylib::clear((25, 25, 30))
    area.drawGrid(5, 4, (50, 50, 55))
    area.drawAxes((150, 150, 150))
    area.barChart(data, (60, 120, 220))
    area.lineChart(data, (220, 60, 60))
    area.drawTitle("My Chart", 20, (230, 230, 230))
}
```

#### PlotArea Construction

| Constructor | Description |
|---|---|
| `PlotArea::new(x, y, w, h, xMin, xMax, yMin, yMax)` | Manual bounds |
| `PlotArea::auto(x, y, w, h, data)` | Auto-range from `[Float]` |
| `PlotArea::square(x, y, size, data)` | Square auto-range |

#### Coordinate Conversion

| Method | Description |
|---|---|
| `.toScreen(dataX, dataY)` → `(Int, Int)` | Data → pixel |
| `.toData(screenX, screenY)` → `(Float, Float)` | Pixel → data |

#### Charts (extends PlotArea)

| Method | Description |
|---|---|
| `.lineChart(data, color)` | Connected line chart |
| `.lineChartThick(data, thickness, color)` | Thick line chart |
| `.barChart(data, color)` | Vertical bar chart |
| `.barChartLabeled(data, labels, color, labelColor)` | Bars with x-axis labels |
| `.scatter(points, size, color)` | Scatter plot from `[(Float,Float)]` |
| `.scatterSized(points, sizes, color)` | Variable-size scatter |
| `.fillArea(data, color)` | Filled area chart |
| `.hLine(y, color)` / `.vLine(x, color)` | Reference lines |

#### Decorations (extends PlotArea)

| Method | Description |
|---|---|
| `.drawAxes(color)` | X/Y axes at data origin |
| `.drawGrid(cols, rows, color)` | Background grid |
| `.drawBorder(color)` | Outline rectangle |
| `.drawXLabels(labels, fontSize, color)` | X-axis labels |
| `.drawYLabels(steps, fontSize, color)` | Y-axis tick labels |
| `.drawTitle(title, fontSize, color)` | Centered title |

#### Standalone

| Function | Description |
|---|---|
| `drawPieChart(cx, cy, radius, data, colors)` | Filled pie chart |
| `drawPieChartLabeled(cx, cy, radius, data, labels, colors, labelColor, fontSize)` | Labeled pie chart |

---

### `std/timer` — Game Timers

```rust
import super.std.timer
```

Frame-rate-independent timers for game loops.

```rust
let shoot = Timer::cooldown(0.5)  // can fire every 0.5s

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    shoot.update(dt)

    if raylib::isKeyPressed("Space") && shoot.ready() {
        // fire! (ready() resets the timer automatically)
    }

    // Progress bar: shoot.progress() goes from 0.0 to 1.0
}
```

| Constructor | Behaviour |
|---|---|
| `Timer::cooldown(s)` | `.ready()` fires after `s` seconds; auto-resets |
| `Timer::repeating(s)` | `.ready()` fires every `s` seconds; auto-resets |
| `Timer::once(s)` | `.isDone()` fires once after `s` seconds |

| Method | Description |
|---|---|
| `.update(dt)` | Advance by dt seconds |
| `.ready()` | True if elapsed ≥ duration (resets cooldown/repeating) |
| `.isDone()` | True if elapsed ≥ duration (does NOT reset) |
| `.progress()` | 0.0 → 1.0 fraction of current cycle |
| `.activate()` | Manually arm the timer |
| `.reset()` | Reset elapsed to 0 |

---

### `std/tween` — Interpolation and Easing

```rust
import super.std.tween
```

Animate values from start to end with various easing curves.

```rust
let slide = Tween::easeOut(0.0, 400.0, 1.0)  // 0→400 in 1 second

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    let x = slide.update(dt)
    raylib::drawRectangle(Cast::int(x).unwrap(), 200, 50, 50, (255, 100, 50))

    if slide.isDone() {
        slide.ping()  // reverse direction
    }
}
```

| Constructor | Easing |
|---|---|
| `Tween::linear(start, end, duration)` | Constant speed |
| `Tween::easeIn(start, end, duration)` | Accelerating (quadratic) |
| `Tween::easeOut(start, end, duration)` | Decelerating (quadratic) |
| `Tween::smooth(start, end, duration)` | Ease-in-out |
| `Tween::easeOutBounce(start, end, duration)` | Bouncy landing |
| `Tween::easeOutElastic(start, end, duration)` | Spring overshoot |
| `Tween::easeOutBack(start, end, duration)` | Snappy overshoot |
| `Tween::sineInOut(start, end, duration)` | Gentle sine wave |

| Method | Description |
|---|---|
| `.update(dt)` | Advance and return current value |
| `.isDone()` | True when reached end |
| `.ping()` | Reverse direction (start ↔ end) |
| `.reset()` | Restart from beginning |
| `.value()` | Current value without advancing |

---

### `std/input` — Action-Based Input

```rust
import super.std.input
```

Map named actions to keys and mouse buttons for cleaner game input.

```rust
let keys = InputMap::new()
keys.bindKey("jump", "Space")
keys.bindKey("left", "Left")
keys.bindKey("right", "Right")
keys.bindMouse("shoot", "Left")

while raylib::rendering() {
    if keys.isPressed("jump") {
        // fires once on first press
    }
    if keys.isHeld("left") {
        // true every frame while held
    }
    let h = keys.axis("left", "right")   // -1.0, 0.0, or 1.0

    let pos = InputMap::mousePos()        // (Int, Int)
    let lastKey = InputMap::lastKey()     // Option(String)
}
```

| Method | Description |
|---|---|
| `InputMap::new()` | Create empty input map |
| `.bindKey(action, key)` | Bind keyboard key to action |
| `.bindMouse(action, button)` | Bind mouse button to action |
| `.isHeld(action)` | True while held |
| `.isPressed(action)` | True on first press (fires once) |
| `.isReleased(action)` | True on release frame |
| `.axis(neg, pos)` | -1.0, 0.0, or 1.0 |
| `InputMap::mousePos()` | Screen coordinates `(Int, Int)` |
| `InputMap::lastKey()` | Last key pressed → `Option(String)` |

---

### `std/camera` — 2D Camera

```rust
import super.std.camera
```

A 2D camera with smooth following, screen shake, and zoom.

```rust
let cam = Camera2D::new(800, 600)
cam.setZoom(1.5)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    cam.follow(player.pos, 5.0, dt)  // smooth follow
    cam.update(dt)

    // Draw in world space (automatically offset by camera)
    cam.drawRect(100, 100, 50, 50, (255, 0, 0))
    cam.drawCircle(200, 200, 20, (0, 255, 0))
    cam.drawLine(0, 0, 300, 300, (100, 100, 255))

    // Screen shake on hit
    if hit { cam.shake(10.0, 0.3) }

    // Coordinate conversion
    let worldPos = cam.screenToWorld(mouseVec)
    let visible = cam.isVisible(enemyX, enemyY, 50)
}
```

| Method | Description |
|---|---|
| `Camera2D::new(w, h)` | Create for screen size |
| `.follow(pos, speed, dt)` | Smooth-follow a Vec2 |
| `.shake(intensity, duration)` | Screen shake effect |
| `.setZoom(z)` | Set zoom (1.0 = normal) |
| `.update(dt)` | Advance shake decay |
| `.drawRect(x, y, w, h, c)` | Draw rect in world space |
| `.drawCircle(x, y, r, c)` | Draw circle in world space |
| `.drawLine(x1, y1, x2, y2, c)` | Draw line in world space |
| `.screenToWorld(v)` / `.worldToScreen(v)` | Coordinate conversion |
| `.isVisible(x, y, margin)` | Frustum-cull test |

---

### `std/physics` — 2D Physics

```rust
import super.std.physics
```

Simple 2D physics bodies, collision shapes, and raycasting.

```rust
// Create a physics body
let ball = Body2D::new(100.0, 50.0, 1.0)
ball.restitution = 0.8  // bouncy

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    ball.applyGravity(500.0, dt)
    ball.update(dt)

    // AABB collision
    let ground = AABB::new(0.0, 500.0, 800.0, 20.0)
    pushOutAABB(ball, 10.0, 10.0, ground)

    // Circle collision
    let c1 = Circle::new(100.0, 200.0, 25.0)
    let c2 = Circle::new(150.0, 200.0, 25.0)
    resolveCircle(c1, 25.0, c2, 25.0)

    // Raycasting
    let ray = Ray2::new(0.0, 250.0, 1.0, 0.0)
    let hit = ray.castAABB(ground)  // HitInfo
}
```

| Type / Function | Description |
|---|---|
| `Body2D::new(x, y, mass)` | Moveable physics body with velocity |
| `.applyGravity(g, dt)` | Apply downward acceleration |
| `.update(dt)` | Integrate velocity → position |
| `.restitution` | Bounce factor (0.0–1.0) |
| `AABB::new(x, y, w, h)` | Axis-aligned bounding box |
| `Circle::new(x, y, r)` | Circle shape |
| `pushOutAABB(body, hw, hh, aabb)` | Push body out of AABB |
| `resolveCircle(a, ra, b, rb)` | Circle-circle resolution |
| `resolveAABB(a, b)` | AABB-AABB resolution |
| `Ray2::new(x, y, dx, dy)` | Ray for casting |
| `ray.castAABB(aabb)` | Raycast against AABB → HitInfo |

---

### `std/entity` — Generic Entity System

```rust
import super.std.entity
import super.std.vec2
import super.std.signal
```

A lightweight **generic** entity system for games.  `Entity(T)` takes a type
parameter for its user data, so you can attach any struct, Int, Float, or
String to an entity — no casts required.

```rust
// Simple float data (like the old entity):
let world = EntityWorld::new() @[T: Float]
let player = world.spawn(100.0, 200.0, "player", 0.0)

// Custom struct data:
struct EnemyData { hp: Int, speed: Float }
let world = EntityWorld::new() @[T: EnemyData]
let e = world.spawn(300.0, 200.0, "enemy", EnemyData { hp: 50, speed: 120.0 })
e.data.hp -= 10  // direct mutation

// Query by tag
let enemies = world.query("enemy")

// Listen for spawns via signals
world.onSpawn.connect(|id: Int| {
    println("Entity " + Cast::string(id) + " spawned!")
})

// Update all entities (pos += vel * dt) and purge dead
world.update(dt)
```

**Entity(T) fields:** `id: Int`, `pos: Vec2`, `vel: Vec2`, `size: Vec2`,
`tag: String`, `alive: Bool`, `data: T`.

#### EntityWorld(T) Methods

| Method | Description |
|---|---|
| `EntityWorld::new() @[T: Type]` | Create entity manager (specify data type) |
| `.spawn(x, y, tag, data)` → `Entity(T)` | Create entity at position with user data |
| `.spawnFull(x, y, vx, vy, w, h, tag, data)` → `Entity(T)` | Full control over all fields |
| `.kill(id)` | Mark entity dead by id |
| `.killAll(tag)` | Mark all entities with tag dead |
| `.query(tag)` → `[Entity(T)]` | Get living entities with matching tag |
| `.all()` → `[Entity(T)]` | Get all living entities |
| `.getById(id)` → `Option` | Lookup entity by id |
| `.forEach(fn)` | Iterate ALL living entities |
| `.forEachTagged(tag, fn)` | Iterate entities with matching tag |
| `.count()` → `Int` | Total number of living entities |
| `.countAlive(tag)` → `Int` | Count living entities with tag |
| `.update(dt)` | Integrate velocity and purge dead |
| `.clear()` | Kill everything |

#### Entity(T) Helpers

| Method | Description |
|---|---|
| `e.overlapsAABB(other)` → `Bool` | Axis-aligned bounding box overlap |
| `e.overlapCircle(other)` → `Bool` | Circle-based overlap (uses size.x/2) |
| `e.center()` → `Vec2` | Center point of the entity |
| `e.drawRect(color)` | Draw filled rectangle (raylib) |
| `e.drawCircle(color)` | Draw filled circle (raylib) |

#### Built-in Signals

| Signal | Type | Fires when |
|---|---|---|
| `world.onSpawn` | `Signal(Int)` | After `spawn()` or `spawnFull()` — carries entity id |
| `world.onKill` | `Signal(Int)` | After `kill()` or `killAll()` — carries entity id |
| `world.onClear` | `VoidSignal` | After `clear()` |

---

### `std/scene` — Scene Management

```rust
import super.std.scene
import super.std.signal
```

Manage game scenes (menus, gameplay, pause screens) with a stack-based system.
The SceneManager fires **VoidSignal** events on every transition.

```rust
let menu = Scene::new(
    |dt: Float| { /* update menu */ },
    || { /* draw menu */ }
)
let game = Scene::new(
    |dt: Float| { /* update game */ },
    || { /* draw game */ }
)

let mgr = SceneManager::new(menu)

// Listen for transitions
mgr.onSwitch.connect(|| { println("Scene switched!") })

// Switch scenes
mgr.switch(game)   // replaces current, clears stack, fires onSwitch

// Overlay (pause menu on top of game)
mgr.push(menu)     // game still exists underneath, fires onPush
mgr.pop()          // return to game, fires onPop

// In game loop:
mgr.update(dt)
mgr.draw()
```

| Method | Description |
|---|---|
| `Scene::new(updateFn, drawFn)` | Create scene from two closures |
| `Scene::empty()` | No-op placeholder scene |
| `SceneManager::empty()` | Create with no scene |
| `SceneManager::new(scene)` | Create with initial scene |
| `.switch(scene)` | Replace current, clear stack |
| `.push(scene)` | Push over current (pause menus) |
| `.pop()` | Return to previous scene |
| `.update(dt)` | Tick current scene |
| `.draw()` | Draw current scene |
| `.has()` → `Bool` | True if there is an active scene |
| `.depth()` → `Int` | Number of scenes on the stack |

#### Built-in Signals

| Signal | Type | Fires when |
|---|---|---|
| `mgr.onSwitch` | `VoidSignal` | After `switch()` is called |
| `mgr.onPush` | `VoidSignal` | After `push()` is called |
| `mgr.onPop` | `VoidSignal` | After `pop()` is called |

---

### `std/grid` — 2D Grid and Tilemap

```rust
import super.std.grid
```

`Grid(T)` is a **generic** fixed-size 2D grid backed by a flat array.
Coordinates are `(col, row)` = `(x, y)`. Specify the element type
with `@[T: Type]` at construction.

```rust
let g = Grid::new(10, 10, 0) @[T: Int]
g.set(5, 5, 1)
g.get(5, 5)        // 1
g.inBounds(10, 10)  // false
g.width()           // 10
g.height()          // 10
```

#### Construction and Access

| Method | Description |
|---|---|
| `Grid::new(cols, rows, default) @[T: Type]` | Create grid filled with default |
| `.get(col, row)` | Read cell value |
| `.set(col, row, value)` | Write cell value |
| `.fill(value)` | Set all cells |
| `.fillRect(x, y, w, h, value)` | Fill rectangular region |
| `.inBounds(col, row)` | Check if coordinates are valid |
| `.width()` / `.height()` | Grid dimensions |

#### Neighbours and Pathfinding

```rust
g.neighbors4(5, 5)  // [(5,4),(5,6),(6,5),(4,5)]  — N/S/E/W
g.neighbors8(5, 5)  // all 8 surrounding cells

// BFS pathfinding: find path from (0,0) to (9,9) through walkable cells
let path = g.bfs(0, 0, 9, 9, |v: Int| v == 0)  // [(Int,Int)] or []
```

| Method | Description |
|---|---|
| `.neighbors4(col, row)` | 4 cardinal neighbours `[(Int,Int)]` |
| `.neighbors8(col, row)` | 8 surrounding neighbours `[(Int,Int)]` |
| `.forEach(fn(col, row, value))` | Iterate all cells |
| `.bfs(sx, sy, gx, gy, passable)` | BFS pathfinding → `[(Int,Int)]` |

#### Drawing (Raylib)

For graphical games with a raylib window:

```rust
g.draw(0, 0, 32, |v: Int| if v == 1 { (255,0,0) } else { (40,40,40) })
g.drawLines(0, 0, 32, (80, 80, 80))
g.drawLabels(0, 0, 32, |v: Int| Cast::string(v), 12, (200,200,200))
```

| Method | Description |
|---|---|
| `.draw(x, y, cellSize, colorFn)` | Draw coloured cells |
| `.drawLines(x, y, cellSize, color)` | Draw grid lines |
| `.drawLabels(x, y, cellSize, strFn, fontSize, color)` | Draw text labels |

#### Terminal Drawing (no raylib needed)

For text-based games and debugging — works in any terminal:

```rust
// Simple character grid
g.printGrid(|v: Int| if v == 1 { "#" } else { "." })
// ..#..
// .###.
// ..#..

// Padded numeric labels
g.printGridLabels(|v: Int| Cast::string(v), 3)
//   0   0   1   0   0
//   0   1   1   1   0
//   0   0   1   0   0

// Boxed with title
g.printGridBoxed(|v: Int| if v == 1 { "#" } else { "." }, "Map")
// ┌── Map ──┐
// │. . # . .│
// │. # # # .│
// │. . # . .│
// └─────────┘
```

| Method | Description |
|---|---|
| `.printGrid(charFn)` | Print grid with single-char cells |
| `.printGridLabels(strFn, cellWidth)` | Print grid with padded labels |
| `.printGridBoxed(charFn, title)` | Print grid with Unicode box border and title |

---

### `std/signal` — Signals

```rust
import super.std.signal
```

Godot-style signal system for decoupled communication.  An emitter
defines signals; listeners `connect()` callbacks.  When the signal
fires, every connected callback runs.

Three flavours:

| Type | Purpose |
|---|---|
| `Signal(T)` | Typed signal — carries a payload of type `T` |
| `VoidSignal` | Fire-and-forget — no payload |
| `SignalBus` | Named registry of void signals (global event bus) |

#### Signal(T) — Typed Signal

```rust
import super.std.signal
import super.std.core    // Box

let onDamage = Signal::new() @[T: Int]
let hp = Box(100)

onDamage.connect(|dmg: Int| {
    hp.value -= dmg
})

onDamage.emit(25)   // hp.value == 75
onDamage.emit(10)   // hp.value == 65
```

| Method | Description |
|---|---|
| `Signal::new() @[T: Type]` | Create a typed signal |
| `.connect(f: fn(T))` | Add a listener callback |
| `.emit(payload: T)` | Fire all listeners with payload |
| `.clear()` | Remove all listeners |
| `.count()` → `Int` | Number of connected listeners |

#### VoidSignal — No Payload

```rust
let onReady = VoidSignal::new()
onReady.connect(|| { println("Ready!") })
onReady.emit()   // "Ready!"
```

| Method | Description |
|---|---|
| `VoidSignal::new()` | Create a void signal |
| `.connect(f: fn())` | Add a listener callback |
| `.emit()` | Fire all listeners |
| `.clear()` | Remove all listeners |
| `.count()` → `Int` | Number of connected listeners |

#### SignalBus — Named Event Bus

```rust
let bus = SignalBus::new()
bus.register("game_over")
bus.register("level_up")

bus.connect("game_over", || { println("Game Over!") })
bus.connect("level_up",  || { println("Level Up!") })

bus.emit("game_over")   // "Game Over!"
bus.emit("level_up")    // "Level Up!"
```

| Method | Description |
|---|---|
| `SignalBus::new()` | Create an empty signal bus |
| `.register(name: String)` | Register a named signal |
| `.connect(name, f: fn())` | Connect listener to named signal |
| `.emit(name: String)` | Fire all listeners on named signal |
| `.has(name)` → `Bool` | True if signal name is registered |
| `.clear(name: String)` | Remove listeners on one signal |
| `.clearAll()` | Remove all signals and listeners |
| `.signalCount(name)` → `Int` | Number of listeners on one signal |

> **When to use which?**
> - `Signal(T)` — event carries data (damage amount, new position, toggle state).
> - `VoidSignal` — event is a pure notification (game over, scene changed).
> - `SignalBus` — many unrelated systems need a shared event bus by string name.

---

### `std/noise` — Procedural Noise

```rust
import super.std.noise
```

Pure-math procedural noise for terrain generation and visual effects.
No dependencies — works in both raylib and terminal programs.

```rust
// Basic noise
let n = valueNoise(1.5, 2.3, 42)     // 0.0–1.0
let s = smoothNoise(1.5, 2.3, 42)    // smoother version

// Fractal Brownian motion — layered noise for terrain
let h = fbm(x * 0.01, y * 0.01, 42, 6, 2.0, 0.5)

// Ridged noise — mountain ridges
let r = ridged(x * 0.01, y * 0.01, 42, 6)

// Domain warping — swirling organic patterns
let w = domain(x * 0.01, y * 0.01, 42, 2.0)

// Map noise to colour
let color = noiseToColor(h, (0, 50, 200), (255, 255, 255))
```

| Function | Signature | Description |
|---|---|---|
| `noiseHash(x, y, seed)` | `(Int, Int, Int) -> Float` | Deterministic hash → `[0.0, 1.0)` |
| `valueNoise(x, y, seed)` | `(Float, Float, Int) -> Float` | Bilinear value noise |
| `smoothNoise(x, y, seed)` | `(Float, Float, Int) -> Float` | Smoothstep-interpolated noise |
| `fbm(x, y, seed, octaves, lacunarity, gain)` | `(Float, Float, Int, Int, Float, Float) -> Float` | Fractal Brownian motion |
| `ridged(x, y, seed, octaves)` | `(Float, Float, Int, Int) -> Float` | Ridged multifractal |
| `domain(x, y, seed, strength)` | `(Float, Float, Int, Float) -> Float` | Domain-warped fbm |
| `noiseToColor(n, lo, hi)` | `(Float, (Int,Int,Int), (Int,Int,Int)) -> (Int,Int,Int)` | Lerp between colours |

---

### `std/particle` — Particle System

```rust
import super.std.particle
```

Simple particle system for game effects — explosions, fire, rain, snow,
sparks, fountains.  Each `Emitter` manages a pool of `Particle` structs,
handles spawning, aging, physics, and rendering.

#### Particle Struct

```rust
struct Particle {
    x: Float, y: Float,       // position
    vx: Float, vy: Float,     // velocity
    life: Float,               // remaining life (seconds)
    maxLife: Float,            // initial life (for ratio)
    size: Float,               // draw radius
    r: Int, g: Int, b: Int,   // colour
}
```

| Method | Description |
|---|---|
| `Particle::new(x, y, vx, vy, life, size, r, g, b)` | Create a particle |
| `.lifeRatio()` → `Float` | `life / maxLife` (1.0 → 0.0 as it ages) |

#### Emitter — Configurable Particle Source

```rust
let em = Emitter::new(400.0, 300.0)
em.minSpeed = 50.0
em.maxSpeed = 200.0
em.minLife = 0.5
em.maxLife = 2.0
em.gravity = 300.0

// In game loop:
em.emit(5)          // spawn 5 particles at emitter position
em.update(dt)       // age, move, remove dead particles
em.draw()           // render all alive particles as circles
```

| Constructor | Description |
|---|---|
| `Emitter::new(x, y)` | Create emitter at position |

| Configuration Field | Default | Description |
|---|---|---|
| `minSpeed` / `maxSpeed` | `50.0` / `200.0` | Speed range |
| `minLife` / `maxLife` | `0.5` / `2.0` | Lifetime range (seconds) |
| `minSize` / `maxSize` | `2.0` / `6.0` | Particle size range |
| `gravity` | `0.0` | Downward acceleration |
| `r`, `g`, `b` | `255, 200, 50` | Particle colour |
| `r2`, `g2`, `b2` | `-1, -1, -1` | End colour (-1 = same as start) |
| `maxParticles` | `500` | Pool limit |
| `angleMin` / `angleMax` | `0.0` / `6.2832` | Emission angle range (radians) |
| `fadeOut` | `true` | Fade alpha as life decreases |

| Method | Description |
|---|---|
| `.emit(count)` | Spawn particles at emitter (x, y) |
| `.emitAt(x, y, count)` | Spawn particles at custom position |
| `.update(dt)` | Age, apply gravity, prune dead |
| `.draw()` | Render all alive particles |
| `.count()` → `Int` | Number of alive particles |
| `.clear()` | Remove all particles |
| `.setColor(r, g, b)` | Set start colour |
| `.setColorRange(r1,g1,b1, r2,g2,b2)` | Set start → end colour gradient |

#### Presets

| Preset | Description |
|---|---|
| `Emitter::fountain(x, y)` | Upward stream with gravity |
| `Emitter::explosion(x, y)` | Burst in all directions |
| `Emitter::fire(x, y)` | Flickering upward flame |
| `Emitter::snow(x, y)` | Gentle downward drift |
| `Emitter::sparks(x, y)` | Fast bright sparks |

```rust
// One-shot explosion
let boom = Emitter::explosion(player.pos.x, player.pos.y)
boom.emit(50)

// Continuous fire
let fire = Emitter::fire(torch.x, torch.y)
// In loop:
fire.emit(3)
fire.update(dt)
fire.draw()
```

---

### `std/log` — Structured Logging

```rust
import super.std.log
```

A full-featured logging system with coloured output, level filtering, and
debugging utilities designed for game development.

#### Log Levels

| Constant | Value | Colour |
|---|---|---|
| `Log::TRACE` | 0 | dim grey |
| `Log::DEBUG` | 1 | cyan |
| `Log::INFO` | 2 | green |
| `Log::WARN` | 3 | yellow |
| `Log::ERROR` | 4 | bold red |
| `Log::OFF` | 5 | (silent) |

Only messages **at or above** the logger's level are printed.

#### Logger

```rust
let log = Logger::new("MyGame")       // INFO level, colours on
log.info("Player spawned")
log.warn("Low health")
log.error("Collision failed")
log.debug("pos = " + Cast::string(x)) // hidden at INFO level
```

| Constructor | Description |
|---|---|
| `Logger::new(name)` | INFO level, colours enabled |
| `Logger::withLevel(name, level)` | Custom level |
| `Logger::all(name)` | TRACE level (show everything) |
| `Logger::quiet(name)` | ERROR only |
| `Logger::silent(name)` | OFF (suppress all) |

| Method | Description |
|---|---|
| `.trace(msg)` `.debug(msg)` `.info(msg)` `.warn(msg)` `.error(msg)` | Log at level |
| `.setLevel(l)` `.getLevel()` | Change/read level |
| `.setColors(on)` | Toggle ANSI colours |
| `.sep()` | Print a separator line |
| `.header(title)` | Print a boxed header |
| `.dump(label, value)` | Print `label = value` at INFO |
| `.table(headers, rows)` | Print an ASCII table |
| `.assert(cond, msg)` | Log ERROR if `cond` is false |
| `.timer(label) → Int` | Start a timer (returns `now()`) |
| `.elapsed(label, start)` | Print ms elapsed since `start` |
| `.group(label)` / `.groupEnd()` | Indented output grouping |
| `.count(label)` | Increment + print a named counter |
| `.resetCount(label)` | Reset a named counter |

---

### `std/datetime` — Date and Time

```rust
import super.std.datetime
```

Date/time utilities built on the native `now()` (ms) and `nowSec()` builtins.

#### DateTime

```rust
let dt = DateTime::now()
println(dt.format())        // "2026-04-05 14:30:22"
println(dt.weekdayName())   // "Sunday"
```

| Constructor | Description |
|---|---|
| `DateTime::now()` | Current UTC date & time |
| `DateTime::fromEpoch(ms)` | From milliseconds since Unix epoch |

| Method | Return | Description |
|---|---|---|
| `.year()` `.month()` `.day()` | `Int` | Date components |
| `.hour()` `.minute()` `.second()` `.millis()` | `Int` | Time components |
| `.weekday()` | `Int` | 0=Sunday .. 6=Saturday |
| `.weekdayName()` | `String` | e.g. `"Monday"` |
| `.monthName()` | `String` | e.g. `"January"` |
| `.format()` | `String` | `"YYYY-MM-DD HH:MM:SS"` |
| `.formatDate()` | `String` | `"YYYY-MM-DD"` |
| `.formatTime()` | `String` | `"HH:MM:SS"` |
| `.toEpoch()` | `Int` | Milliseconds since epoch |
| `.diffMs(other)` | `Int` | Difference in ms |
| `.diffSec(other)` | `Float` | Difference in seconds |

#### Stopwatch

```rust
let sw = Stopwatch::start()
// ... do work ...
println(Cast::string(sw.elapsedMs()) + " ms")
```

| Method | Return | Description |
|---|---|---|
| `Stopwatch::start()` | `Stopwatch` | Create a running stopwatch |
| `.elapsedMs()` | `Int` | Milliseconds since start |
| `.elapsedSec()` | `Float` | Seconds since start |
| `.reset()` | | Restart from now |
| `.lap()` | `Int` | Ms since last lap, resets lap counter |

---

### `std/gameloop` — Automatic Update System

```rust
import super.std.gameloop
```

Collects game objects with `update(self, Float)` function fields into a single
list, then updates them all with one call. Uses Nova's `Dyn` structural typing.

#### How It Works

Any struct with an `update: fn(Self, Float)` field matches the `updatable` Dyn type:

```rust
type updatable = Dyn(T = update: fn($T, Float))
```

```rust
struct Spinner {
    angle: Float,
    speed: Float,
    update: fn(Spinner, Float),
}

let spinner = Spinner {
    angle: 0.0,
    speed: 90.0,
    update: fn(self: Spinner, dt: Float) {
        self.angle = self.angle + self.speed * dt
    },
}

let u = Updater::new()
u.add(spinner)

// In your game loop:
u.tick(dt)  // spinner.angle advances automatically!
```

| Method | Description |
|---|---|
| `Updater::new()` | Create an empty updater |
| `.add(obj)` | Register any object with an `update` field |
| `.tick(dt)` | Call `update(dt)` on everything |
| `.count()` | Number of registered objects |
| `.clear()` | Remove all objects |

> **Tip**: For existing std types (Timer, Tween) that use `extends` methods
> rather than function fields, create a thin adapter struct:
> ```rust
> struct ManagedTimer {
>     timer: Timer,
>     update: fn(ManagedTimer, Float),
> }
> let mt = ManagedTimer {
>     timer: Timer::repeating(2.0),
>     update: fn(self: ManagedTimer, dt: Float) { self.timer.update(dt) },
> }
> updater.add(mt)
> ```

---

## 3. Raylib API

Nova's raylib bindings provide 2D game development with window management, drawing,
input, sprites, and audio.

### Quick Start

```rust
raylib::init("My Game", 800, 600, 60)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    raylib::clear((20, 20, 40))
    raylib::drawText("Hello!", 300, 270, 36, (255, 255, 255))
    raylib::drawFPS(10, 10)
}
```

### Colours

All colour arguments are `(Int, Int, Int)` RGB tuples, values 0–255.
Use `import super.std.color` for named constants.

### Window & Timing

| Signature | Description |
|---|---|
| `raylib::init(String, Int, Int, Int) -> Void` | Create window (title, width, height, fps). |
| `raylib::rendering() -> Bool` | Process a frame. Returns false when window closes. |
| `raylib::getScreenWidth() -> Int` | Window width. |
| `raylib::getScreenHeight() -> Int` | Window height. |
| `raylib::setTargetFPS(Int) -> Void` | Change target FPS. |
| `raylib::getFPS() -> Int` | Current FPS. |
| `raylib::getTime() -> Float` | Seconds since init. |
| `raylib::getFrameTime() -> Float` | Delta time for the last frame. |
| `raylib::sleep(Int) -> Void` | Pause (milliseconds). |

### Drawing

| Signature | Description |
|---|---|
| `raylib::clear((Int,Int,Int)) -> Void` | Clear background. |
| `raylib::drawText(String, Int, Int, Int, (Int,Int,Int)) -> Void` | Draw text. |
| `raylib::drawFPS(Int, Int) -> Void` | Draw FPS counter. |
| `raylib::measureText(String, Int) -> Int` | Pixel width of text. |
| `raylib::drawRectangle(Int, Int, Int, Int, (Int,Int,Int)) -> Void` | Filled rectangle. |
| `raylib::drawRectangleLines(Int, Int, Int, Int, (Int,Int,Int)) -> Void` | Rectangle outline. |
| `raylib::drawRoundedRectangle(Int, Int, Int, Int, Float, (Int,Int,Int)) -> Void` | Rounded rectangle. |
| `raylib::drawCircle(Int, Int, Int, (Int,Int,Int)) -> Void` | Filled circle. |
| `raylib::drawCircleLines(Int, Int, Int, (Int,Int,Int)) -> Void` | Circle outline. |
| `raylib::drawLine(Int, Int, Int, Int, (Int,Int,Int)) -> Void` | 1-pixel line. |
| `raylib::drawLineThick(Int, Int, Int, Int, Float, (Int,Int,Int)) -> Void` | Thick line. |
| `raylib::drawTriangle(Int, Int, Int, Int, Int, Int, (Int,Int,Int)) -> Void` | Filled triangle (CCW). |

### Sprites

| Signature | Description |
|---|---|
| `raylib::loadSprite(String, Int, Int) -> Sprite` | Load from file (path, height, frameCount). |
| `raylib::buildSprite(Int, Int, Int, [(Int,Int,Int)]) -> Sprite` | Build from pixel data (w, h, frames, pixels). |
| `raylib::drawSprite(Sprite, Int, Int) -> Void` | Draw sprite at position. |
| `raylib::drawSpriteFrame(Sprite, Int, Int, Int) -> Void` | Draw specific animation frame. |

### Keyboard

| Signature | Description |
|---|---|
| `raylib::getKey() -> Option(String)` | Key pressed this frame. |
| `raylib::isKeyPressed(String) -> Bool` | Key held down? (true every frame while held) |
| `raylib::isKeyDown(String) -> Bool` | Alias for `isKeyPressed`. |
| `raylib::isKeyJustPressed(String) -> Bool` | Key just went down this frame? (fires once) |
| `raylib::isKeyReleased(String) -> Bool` | Key released this frame? |
| `raylib::isKeyUp(String) -> Bool` | Key not pressed? |

Key names: `"A"`–`"Z"`, `"0"`–`"9"`, `"Space"`, `"Enter"`, `"Escape"`,
`"Up"`, `"Down"`, `"Left"`, `"Right"`, `"LeftShift"`, `"LeftControl"`,
`"Tab"`, `"Backspace"`, `"F1"`–`"F12"`.

### Mouse

| Signature | Description |
|---|---|
| `raylib::mousePosition() -> (Int, Int)` | Current mouse position. |
| `raylib::isMousePressed(String) -> Bool` | Button just pressed this frame? (fires once) |
| `raylib::isMouseDown(String) -> Bool` | Button held down? (true every frame while held) |
| `raylib::isMouseReleased(String) -> Bool` | Button released this frame? |
| `raylib::getMouseWheel() -> Float` | Wheel movement (positive = up). |

Button names: `"Left"`, `"Right"`, `"Middle"`.

> **`isMousePressed` vs `isMouseDown`:** Use `isMousePressed` for single
> clicks (button, menu). Use `isMouseDown` for continuous actions (dragging,
> holding to shoot).

### Audio

Call `raylib::initAudio()` once before loading any audio. Call `raylib::closeAudio()`
before exit.

| Signature | Description |
|---|---|
| `raylib::initAudio() -> Void` | Initialize audio device. |
| `raylib::closeAudio() -> Void` | Close audio device. |
| `raylib::setMasterVolume(Float) -> Void` | Master volume (0.0–1.0). |
| `raylib::loadSound(String) -> Int` | Load sound (.wav, .ogg, .mp3) → ID. |
| `raylib::playSound(Int) -> Void` | Play sound. |
| `raylib::stopSound(Int) -> Void` | Stop sound. |
| `raylib::pauseSound(Int) -> Void` | Pause sound. |
| `raylib::resumeSound(Int) -> Void` | Resume sound. |
| `raylib::isSoundPlaying(Int) -> Bool` | Is sound playing? |
| `raylib::setSoundVolume(Int, Float) -> Void` | Sound volume (0.0–1.0). |
| `raylib::setSoundPitch(Int, Float) -> Void` | Sound pitch (1.0 = normal). |
| `raylib::loadMusic(String) -> Int` | Load music stream → ID. |
| `raylib::playMusic(Int) -> Void` | Start music. |
| `raylib::updateMusic(Int) -> Void` | Update music buffer — **call every frame**. |
| `raylib::stopMusic(Int) -> Void` | Stop music. |
| `raylib::pauseMusic(Int) -> Void` | Pause music. |
| `raylib::resumeMusic(Int) -> Void` | Resume music. |
| `raylib::isMusicPlaying(Int) -> Bool` | Is music playing? |
| `raylib::setMusicVolume(Int, Float) -> Void` | Music volume (0.0–1.0). |
| `raylib::setMusicPitch(Int, Float) -> Void` | Music pitch (1.0 = normal). |
| `raylib::getMusicLength(Int) -> Float` | Total duration (seconds). |
| `raylib::getMusicTimePlayed(Int) -> Float` | Elapsed play time (seconds). |
| `raylib::seekMusic(Int, Float) -> Void` | Seek to position (seconds). |
| `raylib::setMusicLooping(Int, Bool) -> Void` | Enable/disable looping. |
