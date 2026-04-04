# Nova Reference

Complete API reference for Nova's built-in functions, type system, standard library,
and raylib bindings.

---

## Table of Contents

1. [Built-in Types](#1-built-in-types)
2. [Built-in Functions](#2-built-in-functions)
3. [Standard Library](#3-standard-library)
4. [Raylib API](#4-raylib-api)
5. [Import System](#5-import-system)
6. [CLI Reference](#6-cli-reference)

---

## 1. Built-in Types

### Primitive Types

| Type | Description |
|---|---|
| `Int` | 64-bit signed integer. |
| `Float` | 64-bit IEEE 754 floating-point number. |
| `Bool` | Boolean value: `true` or `false`. |
| `String` | UTF-8 encoded text. Indexable: `"hello"[0]` → `'h'`. Supports negative indexing. |
| `Char` | Single Unicode character. |
| `Void` | Absence of a return value. |

### Composite Types

| Type | Syntax | Description |
|---|---|---|
| `Option(T)` | `Some(value)` / `None(T)` | An optional value — either present or absent. |
| `List` | `[T]` | A dynamically-sized sequence of elements of a single type. |
| `Tuple` | `(A, B, ...)` | A fixed-size, heterogeneous collection. |
| `Function` | `fn(A, B) -> R` | A callable value with typed parameters and a return type. |

### User-defined Types

| Type | Description |
|---|---|
| `Custom` | A user-defined struct or enum, optionally generic (e.g. `Pair(Int, String)`). |
| `Generic` | A type variable used in generic definitions, written `$T`. |

### Special Types

| Type | Description |
|---|---|
| `Dyn` | Structural constraint for duck-typed dispatch: `Dyn(T = field: Type + ...)`. |
| `None` | The absence of a value inside an `Option`. Written `None(T)` to specify the inner type. |

---

## 2. Built-in Functions

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

`terminal::args()` returns everything passed after `nova run file.nv`:

```bash
nova run myapp.nv hello world 42
```

```nova
let args = terminal::args()         // Some(["hello", "world", "42"])
if args.isSome() {
    let list = args.unwrap()
    println(list[0])                // "hello"
}
```

### Generic Annotation

When calling `todo()`, `unreachable()`, or other generic functions where the return
type cannot be inferred, use `@[T: Type]`:

```nova
return todo() @[T: String]
return unreachable() @[T: Int]
let map = HashMap::default() @[K: String, V: Int]
```

---

## 3. Standard Library

All modules live in `nova-lang/std/`. Import with `import super.std.<name>`
(dot-path relative to your file). See [Import System](#5-import-system) for
full syntax details including GitHub imports.

---

### `std/core` — Foundation

```nova
import super.std.core
```

| Function | Signature | Description |
|---|---|---|
| `clamp` | `(v, lo, hi: Float) -> Float` | Clamp value to [lo, hi] |
| `clampI` | `(v, lo, hi: Int) -> Int` | Clamp integer |
| `min` | `(a, b: Float) -> Float` | Minimum |
| `max` | `(a, b: Float) -> Float` | Maximum |
| `sign` | `(x: Float) -> Float` | -1, 0, or 1 |
| `lerp` | `(a, b, t: Float) -> Float` | Linear interpolation |
| `mapRange` | `(v, inLo, inHi, outLo, outHi: Float) -> Float` | Remap between ranges |
| `swapF` | `(a, b: Float) -> (Float, Float)` | Swap two floats |
| `swapI` | `(a, b: Int) -> (Int, Int)` | Swap two ints |

**Key types:**

| Name | Kind | Description |
|---|---|---|
| `Box(T)` | Struct | Heap wrapper for mutable shared state. Read/write via `.value`. |
| `Gen(start)` | Function | Stateful integer counter. |
| `Maybe(A)` | Enum | `Maybe::Just(v)` / `Maybe::Nothing()`. `.isJust()`, `.map(f)`, `.flatMap(f)`, `.orElse(v)`. |
| `Result(A, B)` | Enum | `Result::Ok(v)` / `Result::Err(e)`. `.isOk()`, `.map(f)`, `.mapErr(f)`, `.andThen(f)`. |
| `range(start, end)` | Function | Returns `[start, ..., end-1]`. |
| `n.to(end)` | Extension | Returns `[n, ..., end-1]` (UFCS). |
| `.orDefault(v)` | Extension | Unwrap Option/Maybe/Result or return default. |
| `.orError(msg)` | Extension | Unwrap Option or print error and exit. |
| `.isNone()` | Extension | True if Option is None. |
| `.toMaybe()` | Extension | Convert Option to Maybe. |
| `.toResult(err)` | Extension | Convert Option to Result. |

---

### `std/math` — Extended Mathematics

```nova
import super.std.math
```

**Int extensions (UFCS):**

| Method | Description |
|---|---|
| `n.min(other)` / `n.max(other)` | Smaller / larger of two |
| `n.abs()` | Absolute value |
| `n.pow(exp)` | Integer exponentiation |
| `n.sqrt()` → Float | Square root |
| `n.clamp(lo, hi)` | Clamp into range |
| `n.factorial()` | n! |
| `n.gcd(other)` / `n.lcm(other)` | GCD / LCM |
| `n.isEven()` / `n.isOdd()` | Parity check |
| `n.sign()` | -1, 0, or 1 |
| `n.modpow(exp, mod)` | Fast modular exponentiation |
| `n.isPrime()` | Primality test |
| `n.digitSum()` / `n.digits()` | Digit operations |

**Float extensions (UFCS):**

| Method | Description |
|---|---|
| `f.degrees()` / `f.radians()` | Radians ↔ degrees |
| `f.normalize(lo, hi)` | Map to [0.0, 1.0] |
| `f.mapRange(flo, fhi, tlo, thi)` | Remap between ranges |

**Standalone functions:**

| Function | Description |
|---|---|
| `fib(n)` / `fibSeq(n)` | nth Fibonacci / first n Fibonacci numbers |
| `bin(n)` / `hex(n)` / `oct(n)` | Int to binary / hex / octal string |
| `divmod(n, d)` | `(quotient, remainder)` |
| `toRadians(deg)` / `toDegrees(rad)` | Degree ↔ radian conversion |
| `lerp(a, b, t)` / `lerpF(a, b, t)` | Float / Int linear interpolation |
| `remap(v, fl, fh, tl, th)` | Remap between ranges |
| `round(f)` | Float → nearest Int |
| `smoothstep(t)` | Smooth Hermite curve (3t²−2t³) |
| `sign(x)` | Float sign |
| `isPrime(n)` / `primes(n)` | Primality / sieve |
| `collatz(n)` | Collatz sequence |

---

### `std/string` — String Utilities

```nova
import super.std.string
```

| Method | Description |
|---|---|
| `s.split(Char)` | Split by character |
| `s.padLeft(n, c)` / `s.padRight(n, c)` | Pad to width |
| `s.center(n, c)` | Center with padding |
| `s.count(sub)` / `s.countChar(c)` | Count substrings / characters |
| `s.indexOfChar(c)` | First index of char, or -1 |
| `s.isDigit()` / `s.isAlpha()` / `s.isAlphanumeric()` | Character class checks |
| `s.capitalize()` / `s.title()` | Capitalize first / each word |
| `s.removeChar(c)` / `s.replaceChar(old, new)` | Character mutation |
| `s.lines()` / `s.words()` | Split on newlines / spaces |
| `s.truncate(n, suffix)` | Cut with suffix |
| `s.slugify()` | URL-friendly slug |
| `s.wrap(width)` | Word-wrap at column width |
| `s.between(left, right)` | Extract between delimiters |
| `s.stripPrefix(p)` / `s.stripSuffix(s)` | Remove prefix / suffix |

---

### `std/list` — List Utilities

```nova
import super.std.list
```

| Function | Description |
|---|---|
| `List::range(start, end)` / `List::rangeStep(start, end, step)` | Integer range lists |
| `List::repeat(v, n)` | Repeat value n times |
| `List::zip(a, b)` / `List::unzip(pairs)` | Zip / unzip |
| `List::flatten(nested)` | Flatten one level |
| `List::chunk(list, n)` | Split into chunks |
| `List::unique(list)` | Remove duplicates |
| `List::sum(list)` / `List::sumF(list)` | Sum integers / floats |
| `List::min(list)` / `List::max(list)` / `List::mean(list)` | Aggregates |
| `List::sortAsc(list)` / `List::sortDesc(list)` / `List::sortBy(list, f)` | Sorting |
| `List::take(list, n)` / `List::drop(list, n)` | First n / skip n |
| `List::last(list)` | Last element (Option) |
| `List::find(list, pred)` / `List::count(list, pred)` | Search / count |
| `List::any(list, pred)` / `List::all(list, pred)` | Boolean checks |
| `List::partition(list, pred)` | Split into matches / non-matches |
| `List::groupBy(list, f)` / `List::frequencies(list)` | Grouping |
| `List::rotate(list, n)` / `List::interleave(a, b)` | Rotation / interleaving |

**UFCS methods:** `.filter(pred)`, `.map(f)`, `.sort()`, `.sortWith(cmp)`, `.reduce(f, init)`,
`.foreach(f)`, `.any(pred)`, `.all(pred)`, `.find(pred)`, `.join(sep)`, `.flatten()`,
`.concat(other)`, `.fill(v, n)`, `.bubblesort()`, `.enumerate()`, `.zip(other)`.

---

### `std/option` — Option Combinators

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
| `opt.map(f)` | Transform inner value |
| `opt.flatMap(f)` | Chain Option-returning function |
| `opt.filter(pred)` | Keep Some only if pred holds |
| `opt.zip(other)` | Combine two Options into `Option((A,B))` |
| `opt.toList()` | `[v]` if Some, else `[]` |
| `opt.inspect(f)` | Side-effect if Some, pass through |

---

### `std/maybe` — Maybe Type

```nova
import super.std.maybe
```

| Method | Description |
|---|---|
| `Maybe::just(v)` | Wrap a value |
| `Maybe::nothing()` | Empty maybe |
| `m.isJust()` / `m.isNothing()` | Test state |
| `m.fromMaybe(default)` | Value or default |

---

### `std/result` — Error Handling

```nova
import super.std.result
```

| Method | Description |
|---|---|
| `Result::ok(v)` / `Result::err(msg)` | Construct |
| `r.isOk()` / `r.isErr()` | Test state |
| `r.unwrap()` / `r.unwrapOr(default)` | Extract value |
| `r.map(f)` / `r.mapErr(f)` | Transform |
| `r.andThen(f)` | Chain results |

---

### `std/iter` — Lazy Iterators

```nova
import super.std.iter
```

**Constructors:** `Iter::fromVec(list)`, `Iter::fromRange(start, end)`, `Iter::fromFn(f)`,
`Iter::enumerate(iter)`, `Iter::repeat(v)`, `Iter::generate(f)`.

**Transformers (lazy):** `.map(f)`, `.filter(pred)`, `.take(n)`, `.drop(n)`,
`.takeWhile(pred)`, `.dropWhile(pred)`, `.flatMap(f)`, `.zip(other)`, `.chain(other)`.

**Consumers (eager):** `.collect()`, `.show()`, `.count()`, `.sum()`, `.sumF()`,
`.reduce(f, init)`, `.fold(f, init)`, `.any(pred)`, `.all(pred)`, `.find(pred)`,
`.last()`, `.nth(n)`, `.forEach(f)`.

---

### `std/functional` — Higher-Order Utilities

```nova
import super.std.functional
```

| Function | Description |
|---|---|
| `compose(f, g)` | `fn(x) -> f(g(x))` |
| `pipe(f, g)` | `fn(x) -> g(f(x))` |
| `flip(f)` | Swap two arguments of a binary fn |
| `const_(v)` | Always return v |
| `identity(x)` | Return x unchanged |
| `applyN(f, n, x)` | Apply f to x n times |
| `applyWhile(f, pred, x)` | Apply while pred holds |
| `memoize(f)` | Cache results |
| `negate(pred)` | Logical NOT of predicate |
| `both(p, q)` / `either(p, q)` | Combine predicates |

---

### `std/tuple` — Pair and Triple

```nova
import super.std.tuple
```

| Method | Description |
|---|---|
| `t.swap()` | Reverse pair: `(b, a)` |
| `t.fst()` / `t.snd()` | First / second element |
| `t.mapFirst(f)` / `t.mapSecond(f)` | Transform one element |
| `t.both(f)` | Apply f to both (requires A==B) |
| `t.toStrings()` / `t.toList()` | Conversion |
| `pairs.unzip()` | `[(A,B)]` → `([A], [B])` |
| `pair(a, b)` / `triple(a, b, c)` | Convenience constructors |

---

### `std/hashmap` — Hash Map

```nova
import super.std.hashmap
```

| Method | Description |
|---|---|
| `HashMap::default()` | Empty map (16-bucket initial) |
| `HashMap::fromPairs(list)` | Build from `[(K,V)]` |
| `.insert(k, v)` / `.delete(k)` | Mutate |
| `.get(k)` → `Option(V)` | Lookup |
| `.has(k)` / `.size()` / `.isEmpty()` | Query |
| `.clear()` | Remove all |
| `.getOrDefault(k, v)` | Lookup with fallback |
| `.entries()` / `.keys()` / `.values()` | Collection views |
| `.forEach(f)` | Iterate `(k, v)` pairs |
| `.merge(other)` | Insert all from other |
| `.mapValues(f)` / `.filterKeys(pred)` / `.filterValues(pred)` | Transform |
| `.increment(k)` | Increment Int value (counting helper) |
| `.update(k, default, f)` | Update with function |
| `.toSortedPairs()` | Entries sorted by key |

---

### `std/set` — Set

```nova
import super.std.set
```

| Method | Description |
|---|---|
| `Set::empty()` / `Set::singleton(v)` / `Set::fromList(list)` | Construct |
| `.add(v)` / `.remove(v)` | Mutate |
| `.has(v)` / `.size()` / `.isEmpty()` | Query |
| `.toList()` | All elements as list |
| `.union(other)` / `.intersection(other)` / `.difference(other)` | Set operations |
| `.isSubset(other)` / `.isSuperset(other)` / `.isDisjoint(other)` | Comparisons |
| `.forEach(f)` / `.filter(pred)` / `.map(f)` | Iteration |

---

### `std/vec2` — 2D Vector Math

```nova
import super.std.vec2
```

**Constructors:** `Vec2::new(x, y)`, `Vec2::zero()`, `Vec2::one()`, `Vec2::up()`,
`Vec2::right()`, `Vec2::fromAngle(rad)`.

| Method | Description |
|---|---|
| `.add(v)` / `.sub(v)` | Component-wise add / subtract |
| `.scale(s)` / `.negate()` | Scalar multiply / negate |
| `.dot(v)` / `.cross(v)` | Dot product / 2D cross (scalar) |
| `.length()` / `.lengthSq()` | Magnitude / squared magnitude |
| `.normalized()` | Unit vector |
| `.distance(v)` / `.distanceSq(v)` | Distance / squared distance |
| `.angle()` / `.angleTo(v)` | Angle of vector / between vectors |
| `.rotate(rad)` | Rotate by radians |
| `.lerp(v, t)` | Linear interpolation |
| `.reflect(normal)` | Reflect across unit normal |
| `.perpendicular()` | 90° clockwise rotation |
| `.clampLength(max)` | Scale down if too long |
| `.equals(v)` / `.isZero()` | Equality checks |

---

### `std/deque` — Double-Ended Queue

```nova
import super.std.deque
```

| Method | Description |
|---|---|
| `Deque::empty()` / `Deque::singleton(v)` / `Deque::fromList(xs)` | Construct |
| `.pushBack(v)` / `.pushFront(v)` | Add to back / front |
| `.popBack()` / `.popFront()` | Remove from back / front (Option) |
| `.peekBack()` / `.peekFront()` | View back / front (Option) |
| `.len()` / `.isEmpty()` | Size |
| `.toList()` / `.forEach(f)` / `.map(f)` / `.filter(pred)` | Iteration |

---

### `std/io` — File and Console I/O

```nova
import super.std.io
```

| Function | Description |
|---|---|
| `prompt(msg)` | Print msg, return input line |
| `promptInt(msg)` / `promptFloat(msg)` | Prompt and parse (Option) |
| `promptYN(msg)` | Prompt for yes/no → Bool |
| `printSep(values, sep)` | Print values joined with separator |
| `eprintln(msg)` | Print `[error] msg` |
| `readLines(path)` / `writeLines(path, lines)` | File line I/O |
| `appendLine(path, line)` | Append one line |
| `linesOf(text)` | Split string on newlines |

---

### `std/ansi` — ANSI Terminal Colours

```nova
import super.std.ansi
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

```nova
import super.std.color
```

| Function / Constant | Description |
|---|---|
| `red`, `green`, `blue`, `white`, `black` | Primary colour tuples `(Int,Int,Int)` |
| `yellow`, `cyan`, `magenta` | Secondary colour tuples |
| `orange`, `purple`, `pink`, `brown`, `gray` | Extended palette |
| `rgb(r, g, b)` | Construct an RGB tuple |
| `lerpColor(a, b, t)` | Interpolate between two colours (t = 0.0–1.0) |
| `invert(c)` | Invert an RGB colour |
| `darken(c, f)` | Darken by factor f (0.0–1.0) |
| `lighten(c, f)` | Lighten by factor f (0.0–1.0) |

---

### `std/tui` — Terminal UI

```nova
import super.std.tui
```

| Function | Description |
|---|---|
| `run(fn)` | Start a TUI app with a main function |
| `printAt(x, y, s)` | Print string at position (col, row) |
| `clear()` | Clear the terminal |
| `flush()` | Flush output buffer |
| `size()` | Get terminal size `(Int, Int)` |
| `fg(r, g, b)` | Set foreground colour (RGB) |
| `bg(r, g, b)` | Set background colour (RGB) |
| `resetColor()` | Reset terminal colours to default |
| `drawBox(x, y, w, h)` | Draw a box outline with Unicode characters |
| `getch()` | Read a single character (blocking) |
| `poll()` | Non-blocking character read |

---

### `std/widget` — TUI Widget Toolkit

```nova
import super.std.widget
```

| Type | Description |
|---|---|
| `Button` | Clickable button with label and position |
| `Label` | Static text display at a position |
| `Panel` | Rectangular container with border |
| `ProgressBar` | Horizontal progress indicator |
| `Toggle` | On/off toggle switch |

| Method | Description |
|---|---|
| `.draw()` | Render the widget to the terminal |
| `.isClicked()` | True if widget was clicked (interactive widgets) |
| `.isHovered()` | True if cursor is over widget (interactive widgets) |

---

### `std/plot` — Charts & Graphs (Raylib)

```nova
import super.std.plot
```

> Requires an active raylib window (`raylib::init(...)` + `while raylib::rendering() { ... }`).

#### PlotArea Struct

A `PlotArea` maps data coordinates to screen pixels.

| Constructor | Description |
|---|---|
| `PlotArea::new(x, y, w, h, xMin, xMax, yMin, yMax)` | Manual bounds |
| `PlotArea::auto(x, y, w, h, data: [Float])` | Auto-range from data |
| `PlotArea::square(x, y, size, data: [Float])` | Square auto-range |

#### Coordinate Conversion

| Method | Signature | Description |
|---|---|---|
| `toScreen` | `(Float, Float) -> (Int, Int)` | Data → pixel |
| `toData` | `(Int, Int) -> (Float, Float)` | Pixel → data |

#### Chart Drawing (extends PlotArea)

| Method | Signature | Description |
|---|---|---|
| `lineChart` | `([Float], (Int,Int,Int))` | Connected line from sequential data |
| `lineChartThick` | `([Float], Float, (Int,Int,Int))` | Thick line chart |
| `barChart` | `([Float], (Int,Int,Int))` | Vertical bars |
| `barChartLabeled` | `([Float], [String], color, labelColor)` | Bars + x-axis labels |
| `scatter` | `([(Float,Float)], Int, (Int,Int,Int))` | Scatter points |
| `scatterSized` | `([(Float,Float)], [Int], (Int,Int,Int))` | Variable-size scatter |
| `fillArea` | `([Float], (Int,Int,Int))` | Filled area chart |
| `hLine` | `(Float, (Int,Int,Int))` | Horizontal reference line |
| `vLine` | `(Float, (Int,Int,Int))` | Vertical reference line |

#### Decorations (extends PlotArea)

| Method | Description |
|---|---|
| `drawAxes(color)` | X/Y axes at data origin |
| `drawGrid(cols, rows, color)` | Background grid |
| `drawBorder(color)` | Outline rectangle |
| `drawXLabels(labels, fontSize, color)` | X-axis labels |
| `drawYLabels(steps, fontSize, color)` | Y-axis tick labels |
| `drawTitle(title, fontSize, color)` | Centered title above chart |

#### Standalone Functions

| Function | Signature | Description |
|---|---|---|
| `drawPieChart` | `(cx, cy, radius, [Float], [(Int,Int,Int)])` | Filled pie chart |
| `drawPieChartLabeled` | `(cx, cy, radius, [Float], [String], colors, labelColor, fontSize)` | Pie chart with text labels |

#### Example

```nova
import super.std.plot

raylib::init("Chart", 800, 600, 30)
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

See `demo/plotdemo.nv` for a comprehensive 7-chart demo.

---

### `std/timer` — Game Timers

```nova
import super.std.timer
```

| Constructor | Behaviour |
|---|---|
| `Timer::cooldown(s)` | `ready()` fires after `s` seconds; auto-resets |
| `Timer::repeating(s)` | `ready()` fires every `s` seconds; auto-resets |
| `Timer::once(s)` | `isDone()` fires once after `s` seconds |

| Method | Description |
|---|---|
| `update(dt)` | Advance by `dt` seconds |
| `ready()` | True if elapsed ≥ duration; resets on cooldown/repeating |
| `isDone()` | True if elapsed ≥ duration (does NOT reset) |
| `progress()` | 0.0 → 1.0 fraction of current cycle |
| `activate()` | Manually arm a timer |
| `reset()` | Reset elapsed to 0 |

---

### `std/tween` — Interpolation and Easing

```nova
import super.std.tween
```

| Constructor | Easing |
|---|---|
| `Tween::linear(start, end, duration)` | Constant speed |
| `Tween::easeIn(start, end, duration)` | Accelerating |
| `Tween::easeOut(start, end, duration)` | Decelerating |
| `Tween::smooth(start, end, duration)` | Ease-in-out |
| `Tween::easeOutBounce(start, end, duration)` | Bouncy landing |
| `Tween::easeOutElastic(start, end, duration)` | Spring overshoot |
| `Tween::easeOutBack(start, end, duration)` | Snappy overshoot |
| `Tween::sineInOut(start, end, duration)` | Gentle sine wave |

| Method | Description |
|---|---|
| `update(dt)` | Advance and return current value |
| `isDone()` | True when reached end |
| `ping()` | Reverse direction (start ↔ end) |
| `reset()` | Restart from beginning |
| `value()` | Current value without advancing |

---

### `std/input` — Action-Based Input

```nova
import super.std.input
```

| Method | Description |
|---|---|
| `InputMap::new()` | Create empty input map |
| `keys.bindKey(action, key)` | Bind a key to an action name |
| `keys.bindMouse(action, button)` | Bind a mouse button to an action |
| `keys.isHeld(action)` | True while held |
| `keys.isPressed(action)` | True on first press |
| `keys.isReleased(action)` | True on release frame |
| `keys.axis(neg, pos)` | -1.0, 0.0, or 1.0 |
| `InputMap::mousePos()` | Screen coordinates `(Int, Int)` |
| `InputMap::lastKey()` | `Option(String)` — last key pressed |

---

### `std/camera` — 2D Camera

```nova
import super.std.camera
```

| Method | Description |
|---|---|
| `Camera2D::new(w, h)` | Create for screen size |
| `cam.follow(pos, speed, dt)` | Smooth-follow a Vec2 |
| `cam.shake(intensity, duration)` | Screen shake |
| `cam.setZoom(z)` | Set zoom (1.0 = normal) |
| `cam.update(dt)` | Advance shake decay |
| `cam.drawRect(x, y, w, h, c)` | Draw rect in world space |
| `cam.drawCircle(x, y, r, c)` | Draw circle in world space |
| `cam.drawLine(x1, y1, x2, y2, c)` | Draw line in world space |
| `cam.screenToWorld(v)` | Screen Vec2 → world Vec2 |
| `cam.worldToScreen(v)` | World Vec2 → screen Vec2 |
| `cam.isVisible(x, y, margin)` | Frustum-cull test |

---

### `std/physics` — 2D Physics

```nova
import super.std.physics
```

| Type / Function | Description |
|---|---|
| `Body2D::new(x, y, mass)` | Moveable physics body |
| `body.applyGravity(g, dt)` | Apply downward force |
| `body.update(dt)` | Integrate velocity |
| `body.restitution` | Bounce factor (0.0–1.0) |
| `AABB::new(x, y, w, h)` | Axis-aligned bounding box |
| `Circle::new(x, y, r)` | Circle shape |
| `pushOutAABB(body, r, r, aabb)` | Push body out of AABB |
| `resolveCircle(a, ra, b, rb)` | Circle-circle resolution |
| `resolveAABB(a, b)` | AABB-AABB resolution |
| `Ray2::new(x, y, dx, dy)` | Ray for casting |
| `ray.castAABB(aabb)` | Raycast against AABB → HitInfo |

---

### `std/entity` — Entity System

```nova
import super.std.entity
```

| Method | Description |
|---|---|
| `EntityWorld::new()` | Create entity manager |
| `world.spawn(x, y, tag)` | Create entity at position with tag |
| `world.query(tag)` | Return `[Entity]` with matching tag |
| `world.forEachTagged(tag, fn)` | Iterate entities with tag (mutable) |
| `world.forEach(fn)` | Iterate ALL entities |
| `world.countAlive(tag)` | Count living entities with tag |
| `world.update(dt)` | `pos += vel*dt` for all; purge dead |
| `world.update(0.0)` | Purge dead only (no movement) |

**Entity fields:** `id: Int`, `pos: Vec2`, `vel: Vec2`, `size: Vec2`, `tag: String`,
`alive: Bool`, `data: Float`.

**Entity methods:** `e.overlapsAABB(other)`, `e.center()`, `e.entityDrawRect(color)`,
`e.entityDrawCircle(color)`.

---

### `std/scene` — Scene Management

```nova
import super.std.scene
```

| Method | Description |
|---|---|
| `SceneManager::empty()` | Create empty (no scene) |
| `SceneManager::new(scene)` | Create with initial scene |
| `mgr.switch(scene)` | Replace current, clear stack |
| `mgr.push(scene)` | Push over current (pause menus) |
| `mgr.pop()` | Return to previous scene |
| `mgr.update(dt)` | Tick current scene |
| `mgr.draw()` | Draw current scene |
| `Scene::new(updateFn, drawFn)` | Create scene from two closures |

---

### `std/grid` — 2D Grid and Tilemap

```nova
import super.std.grid
```

`Grid(T)` is a **generic** fixed-size 2D grid. Specify the element type
with `@[T: Type]` at construction.

```nova
let intGrid  = Grid::new(10, 10, 0)     @[T: Int]
let boolGrid = Grid::new(5, 5, false)   @[T: Bool]
let strGrid  = Grid::new(3, 3, ".")     @[T: String]
```

| Method | Description |
|---|---|
| `Grid::new(cols, rows, default) @[T: Type]` | Create `Grid(T)` filled with `default` |
| `grid.get(col, row) -> T` | Read cell value |
| `grid.set(col, row, value: T)` | Write cell value |
| `grid.fill(value: T)` | Set all cells to `value` |
| `grid.fillRect(x, y, w, h, value: T)` | Fill rectangular region |
| `grid.inBounds(col, row) -> Bool` | Check if coordinates are valid |
| `grid.cols() -> Int` / `grid.rows() -> Int` | Grid dimensions |
| `grid.neighbors4(col, row) -> [(Int, Int)]` | 4 cardinal neighbours |
| `grid.neighbors8(col, row) -> [(Int, Int)]` | 8 surrounding neighbours |
| `grid.forEach(fn(Int, Int, T))` | Iterate all cells with col, row, value |
| `grid.bfs(sx, sy, gx, gy, fn(T)->Bool) -> [(Int,Int)]` | BFS pathfinding |
| `grid.draw(ox, oy, tileSize, fn(T)->(Int,Int,Int))` | Draw with colour function |
| `grid.drawLines(ox, oy, tileSize, color)` | Draw grid lines |
| `grid.drawLabels(ox, oy, tileSize, fn(T)->String, fontSize, color)` | Draw text labels |

---

### `std/noise` — Procedural Noise

```nova
import super.std.noise
```

| Function | Description |
|---|---|
| `valueNoise(x, y, seed)` | Basic value noise |
| `smoothNoise(x, y, seed)` | Smoothed noise |
| `fbm(x, y, seed, octaves, lacunarity, gain)` | Fractal Brownian motion |
| `ridged(x, y, seed, octaves, lacunarity, gain)` | Ridged noise |
| `domain(x, y, seed, octaves, lacunarity, gain, strength)` | Domain-warped noise |
| `noiseToColor(value, colorA, colorB)` | Map noise to RGB colour |

---

## 4. Raylib API

Nova's raylib bindings provide 2D game development with window management, drawing,
input, sprites, and audio.

### Quick Start

```nova
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
| `raylib::isKeyPressed(String) -> Bool` | Key held down? |
| `raylib::isKeyReleased(String) -> Bool` | Key released this frame? |
| `raylib::isKeyUp(String) -> Bool` | Key not pressed? |

Key names: `"A"`–`"Z"`, `"0"`–`"9"`, `"Space"`, `"Enter"`, `"Escape"`,
`"Up"`, `"Down"`, `"Left"`, `"Right"`, `"LeftShift"`, `"LeftControl"`,
`"Tab"`, `"Backspace"`, `"F1"`–`"F12"`.

### Mouse

| Signature | Description |
|---|---|
| `raylib::mousePosition() -> (Int, Int)` | Current mouse position. |
| `raylib::isMousePressed(String) -> Bool` | Button held? |
| `raylib::isMouseReleased(String) -> Bool` | Button released this frame? |
| `raylib::getMouseWheel() -> Float` | Wheel movement (positive = up). |

Button names: `"Left"`, `"Right"`, `"Middle"`.

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

---

## 5. Import System

Every Nova file begins with `module <name>`. The module name is used for
deduplication — if a module has already been imported, subsequent imports
of the same module are silently skipped.

### Local Imports

| Syntax | Resolves to | Notes |
|---|---|---|
| `import helper` | `./helper.nv` | Same directory |
| `import libs.math` | `./libs/math.nv` | Subfolder |
| `import super.std.core` | `../std/core.nv` | `super` = go up one directory |
| `import super.super.std.grid` | `../../std/grid.nv` | Chain `super` to go up multiple |
| `import "libs/helper.nv"` | `./libs/helper.nv` | String literal — path used as-is |
| `import "../std/core.nv"` | `../std/core.nv` | String literal with parent traversal |

**Rules:**
- Dot-separated names: each `.` becomes `/`, `.nv` is appended automatically.
- `super` translates to `..` (parent directory).
- All paths are relative to the file containing the `import`.
- All imported symbols flatten into the caller's scope (no prefix needed).

### GitHub Imports

Fetch a file from a public GitHub repository:

```nova
import @ "owner/repo/path/to/file.nv"
```

Lock to a specific commit hash:

```nova
import @ "owner/repo/path/to/file.nv" ! "a1b2c3d"
```

| Part | Meaning |
|---|---|
| `@` | Signals a GitHub import |
| `"owner/repo/path"` | GitHub path: `owner/repo/filepath` |
| `! "hash"` | Optional: fetch from this exact commit instead of `main` |

The `module` declaration inside the fetched file determines the module name.
Duplicate detection works the same as local imports.

---

## 6. CLI Reference

### Commands

| Command | Description |
|---|---|
| `nova run file.nv` | Compile and run a file |
| `nova run` | Run `main.nv` in the current directory (project mode) |
| `nova run --git owner/repo/path.nv [commit]` | Fetch and run from GitHub |
| `nova check [file.nv]` | Typecheck only (no execution) |
| `nova check --git owner/repo/path.nv [commit]` | Typecheck a file from GitHub |
| `nova time [file.nv]` | Run and print execution time |
| `nova time --git owner/repo/path.nv [commit]` | Time a file from GitHub |
| `nova dbg [file.nv]` | Run in debug mode |
| `nova dbg --git owner/repo/path.nv [commit]` | Debug a file from GitHub |
| `nova dis [file.nv]` | Disassemble compiled bytecode |
| `nova dis --git owner/repo/path.nv [commit]` | Disassemble a file from GitHub |
| `nova init name [--with owner/repo/folder]` | Create a new project |
| `nova install name owner/repo/folder` | Install a library into `libs/<name>/` |
| `nova remove name` | Remove a library from `libs/<name>/` |
| `nova test [dir]` | Run all `test_*.nv` files in `tests/` (or given dir) |
| `nova repl` | Start interactive REPL |
| `nova help` | Show help |

All commands that accept `[file.nv]` also accept `--git` to fetch the file
from GitHub. They will auto-detect `main.nv` in the current directory if
the file argument is omitted.

### Project Structure

A Nova project is any folder with a `main.nv` file. No config file is needed.

```
myproject/
    main.nv          ← entry point (module main)
    libs/            ← shared modules
        math.nv
        helper.nv
    tests/           ← test files (run with: nova test)
        test_math.nv
```

- `nova init myproject` creates this structure with a hello-world template and starter test.
- `nova init myproject --with owner/repo/folder` also fetches all `.nv` files from a GitHub folder into `libs/`.
- `nova install name owner/repo/folder` fetches a library into `libs/<name>/` in an existing project.
- `nova remove name` removes `libs/<name>/`.
- `cd myproject && nova run` runs the project.
- `cd myproject && nova test` runs all tests.
