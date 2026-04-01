# Nova Standard Library Reference

Complete reference for all modules in `nova-lang/std/`.
Import any module with: `import super.std.<name>`

---

## Table of Contents

| Module | Purpose |
|---|---|
| [core](#core) | Fundamental utilities |
| [math](#math) | Extended math functions |
| [string](#string) | String manipulation |
| [list](#list) | List utilities |
| [option](#option) | Option(T) combinators |
| [maybe](#maybe) | Maybe type helpers |
| [result](#result) | Result/error handling |
| [iter](#iter) | Lazy iteration and combinators |
| [functional](#functional) | Higher-order function tools |
| [tuple](#tuple) | Pair and tuple helpers |
| [hashmap](#hashmap) | HashMap utilities |
| [set](#set) | Set(T) backed by HashMap |
| [io](#io) | File and console I/O |
| [tui](#tui) | Terminal UI components |
| [ansi](#ansi) | ANSI escape sequences |
| [color](#color) | Named color constants |
| [vec2](#vec2) | 2D vector math |
| [deque](#deque) | Double-ended queue |
| [widget](#widget) | Raylib UI widgets |
| [plot](#plot) | Chart and plot drawing (raylib) |
| [timer](#timer) | Game timers and cooldowns |
| [tween](#tween) | Interpolation and easing |
| [input](#input) | Named action input mapping |
| [camera](#camera) | 2D camera (pan/zoom/shake) |
| [physics](#physics) | 2D physics and collision |
| [entity](#entity) | Lightweight entity system |
| [scene](#scene) | Scene and state management |
| [grid](#grid) | 2D grid, tilemap, BFS |
| [noise](#noise) | Procedural noise generation |

---

## core

`import super.std.core`

Fundamental helpers used across other modules.

| Function | Signature | Description |
|---|---|---|
| `clamp` | `(v, lo, hi: Float) -> Float` | Clamp value to [lo, hi] |
| `clampI` | `(v, lo, hi: Int) -> Int` | Clamp integer |
| `min` | `(a, b: Float) -> Float` | Minimum of two values |
| `max` | `(a, b: Float) -> Float` | Maximum of two values |
| `sign` | `(x: Float) -> Float` | -1, 0, or 1 |
| `lerp` | `(a, b, t: Float) -> Float` | Linear interpolation |
| `mapRange` | `(v, inLo, inHi, outLo, outHi: Float) -> Float` | Remap a value between ranges |
| `swapF` | `(a, b: Float) -> (Float, Float)` | Swap two floats |
| `swapI` | `(a, b: Int) -> (Int, Int)` | Swap two ints |

---

## math

`import super.std.math`

Extended math beyond built-in functions.

| Function | Signature | Description |
|---|---|---|
| `isPrime` | `(n: Int) -> Bool` | Primality test |
| `gcd` | `(a, b: Int) -> Int` | Greatest common divisor |
| `lcm` | `(a, b: Int) -> Int` | Least common multiple |
| `digitSum` | `(n: Int) -> Int` | Sum of decimal digits |
| `digits` | `(n: Int) -> [Int]` | List of digits |
| `fibSeq` | `(n: Int) -> [Int]` | First n Fibonacci numbers |
| `primes` | `(n: Int) -> [Int]` | Primes up to n (sieve) |
| `collatz` | `(n: Int) -> [Int]` | Collatz sequence |
| `oct` | `(n: Int) -> String` | Int to octal string |
| `lerpF` | `(a, b, t: Float) -> Float` | Linear interpolation |
| `smoothstep` | `(lo, hi, t: Float) -> Float` | Smooth Hermite interpolation |
| `Float::degrees` | `(self: Float) -> Float` | Radians → degrees |
| `Float::radians` | `(self: Float) -> Float` | Degrees → radians |
| `Float::normalize` | `(self, lo, hi: Float) -> Float` | Normalize to [0,1] |
| `Float::mapRange` | `(self, il, ih, ol, oh: Float) -> Float` | Remap value |

---

## string

`import super.std.string`

String manipulation utilities.

| Function | Signature | Description |
|---|---|---|
| `countChar` | `(s: String, c: Char) -> Int` | Count occurrences of a character |
| `indexOfChar` | `(s: String, c: Char) -> Option(Int)` | First index of character |
| `replaceChar` | `(s: String, from, to: Char) -> String` | Replace all occurrences |
| `truncate` | `(s: String, max: Int, suffix: String) -> String` | Cut string with suffix |
| `slugify` | `(s: String) -> String` | Convert to URL slug |
| `wrap` | `(s: String, width: Int) -> [String]` | Word-wrap to list of lines |
| `between` | `(s, open, close: String) -> Option(String)` | Extract substring between delimiters |
| `stripPrefix` | `(s, prefix: String) -> String` | Remove leading prefix |
| `stripSuffix` | `(s, suffix: String) -> String` | Remove trailing suffix |
| `String::repeat` | `(self: String, n: Int) -> String` | Repeat string n times |
| `String::center` | `(self: String, width: Int) -> String` | Center in field |
| `String::isPalindrome` | `(self: String) -> Bool` | Palindrome check |

---

## list

`import super.std.list`

Utilities for working with lists.

| Function | Signature | Description |
|---|---|---|
| `List::range` | `(start, end: Int) -> [Int]` | Integer range list |
| `List::rangeStep` | `(start, end, step: Int) -> [Int]` | Stepped range |
| `List::repeat` | `(v: T, n: Int) -> [T]` | Repeat a value n times |
| `List::zip` | `([A], [B]) -> [(A, B)]` | Zip two lists |
| `List::unzip` | `([(A, B)]) -> ([A], [B])` | Unzip pairs |
| `List::flatten` | `([[T]]) -> [T]` | Flatten one level |
| `List::chunk` | `([T], n: Int) -> [[T]]` | Split into chunks of size n |
| `List::unique` | `([T]) -> [T]` | Remove duplicates (preserving order) |
| `List::sum` | `([Int]) -> Int` | Sum of integers |
| `List::sumF` | `([Float]) -> Float` | Sum of floats |
| `List::min` | `([Float]) -> Float` | Minimum value |
| `List::max` | `([Float]) -> Float` | Maximum value |
| `List::mean` | `([Float]) -> Float` | Arithmetic mean |
| `List::sortAsc` | `([Float]) -> [Float]` | Sort ascending |
| `List::sortDesc` | `([Float]) -> [Float]` | Sort descending |
| `List::sortBy` | `([T], fn(T)->Float) -> [T]` | Sort by key function |
| `List::take` | `([T], n: Int) -> [T]` | First n elements |
| `List::drop` | `([T], n: Int) -> [T]` | Skip first n elements |
| `List::last` | `([T]) -> Option(T)` | Last element |
| `List::find` | `([T], fn(T)->Bool) -> Option(T)` | First matching element |
| `List::count` | `([T], fn(T)->Bool) -> Int` | Count matching elements |
| `List::any` | `([T], fn(T)->Bool) -> Bool` | True if any match |
| `List::all` | `([T], fn(T)->Bool) -> Bool` | True if all match |
| `List::partition` | `([T], fn(T)->Bool) -> ([T],[T])` | Split into matching/not |
| `List::groupBy` | `([T], fn(T)->String) -> HashMap` | Group by key |
| `List::frequencies` | `([T]) -> HashMap` | Count occurrences |
| `List::rotate` | `([T], n: Int) -> [T]` | Rotate left by n |
| `List::interleave` | `([T], [T]) -> [T]` | Alternating elements |

---

## option

`import super.std.option`

Combinators for `Option(T)` values.

| Function | Signature | Description |
|---|---|---|
| `Option::map` | `(self, fn(T)->U) -> Option(U)` | Transform the inner value |
| `Option::flatMap` | `(self, fn(T)->Option(U)) -> Option(U)` | Chain options |
| `Option::filter` | `(self, fn(T)->Bool) -> Option(T)` | Discard if predicate is false |
| `Option::zip` | `(Option(A), Option(B)) -> Option((A,B))` | Combine two options |
| `Option::toList` | `(self) -> [T]` | Convert to list (0 or 1 element) |
| `Option::inspect` | `(self, fn(T)) -> Option(T)` | Side-effect on value, pass through |
| `Option::unwrapOr` | `(self, default: T) -> T` | Value or default |

---

## maybe

`import super.std.maybe`

Alternative to Option using `Just(T)` / `Nothing`.

| Function | Signature | Description |
|---|---|---|
| `Maybe::just` | `(v: T) -> Maybe(T)` | Wrap a value |
| `Maybe::nothing` | `() -> Maybe(T)` | Empty maybe |
| `m.isJust` | `() -> Bool` | True if has value |
| `m.isNothing` | `() -> Bool` | True if empty |
| `m.fromMaybe` | `(default: T) -> T` | Value or default |

---

## result

`import super.std.result`

Error-handling type — `Ok(T)` / `Err(String)`.

| Function | Signature | Description |
|---|---|---|
| `Result::ok` | `(v: T) -> Result(T)` | Success |
| `Result::err` | `(msg: String) -> Result(T)` | Failure |
| `r.isOk` | `() -> Bool` | True if success |
| `r.isErr` | `() -> Bool` | True if failure |
| `r.unwrap` | `() -> T` | Value or panic |
| `r.unwrapOr` | `(default: T) -> T` | Value or default |
| `r.map` | `(fn(T)->U) -> Result(U)` | Transform success value |
| `r.mapErr` | `(fn(String)->String) -> Result(T)` | Transform error |
| `r.andThen` | `(fn(T)->Result(U)) -> Result(U)` | Chain results |

---

## iter

`import super.std.iter`

Lazy iterator builder for transforming and consuming lists.

```nova
Iter::from(myList)
    .filter(fn(x: Int) -> Bool { x > 0 })
    .map(fn(x: Int) -> Int { x * 2 })
    .take(5)
    .toList()
```

| Function | Signature | Description |
|---|---|---|
| `Iter::from` | `([T]) -> Iter(T)` | Create iterator from list |
| `Iter::range` | `(start, end: Int) -> Iter(Int)` | Integer range |
| `Iter::repeat` | `(v: T, n: Int) -> Iter(T)` | Repeat value n times |
| `Iter::generate` | `(fn()->T, n: Int) -> Iter(T)` | Produce n values from fn |
| `.filter` | `(fn(T)->Bool) -> Iter(T)` | Keep matching elements |
| `.map` | `(fn(T)->U) -> Iter(U)` | Transform each element |
| `.flatMap` | `(fn(T)->[U]) -> Iter(U)` | Map then flatten |
| `.take` | `(n: Int) -> Iter(T)` | Limit to n elements |
| `.drop` | `(n: Int) -> Iter(T)` | Skip first n elements |
| `.takeWhile` | `(fn(T)->Bool) -> Iter(T)` | Take while predicate holds |
| `.dropWhile` | `(fn(T)->Bool) -> Iter(T)` | Drop while predicate holds |
| `.zip` | `(Iter(U)) -> Iter((T,U))` | Zip two iterators |
| `.chain` | `(Iter(T)) -> Iter(T)` | Concatenate two iterators |
| `.toList` | `() -> [T]` | Collect into list |
| `.forEach` | `(fn(T))` | Consume with side-effect |
| `.count` | `() -> Int` | Count elements |
| `.sum` | `() -> Int` | Sum integers |
| `.sumF` | `() -> Float` | Sum floats |
| `.reduce` | `(T, fn(T,T)->T) -> T` | Left fold with initial |
| `.any` | `(fn(T)->Bool) -> Bool` | True if any match |
| `.all` | `(fn(T)->Bool) -> Bool` | True if all match |
| `.find` | `(fn(T)->Bool) -> Option(T)` | First matching element |
| `.last` | `() -> Option(T)` | Last element |
| `.nth` | `(n: Int) -> Option(T)` | Nth element |

---

## functional

`import super.std.functional`

Higher-order function combinators.

| Function | Signature | Description |
|---|---|---|
| `compose` | `(fn(B)->C, fn(A)->B) -> fn(A)->C` | Function composition (g∘f) |
| `pipe` | `(fn(A)->B, fn(B)->C) -> fn(A)->C` | Function pipeline (f then g) |
| `flip` | `(fn(A,B)->C) -> fn(B,A)->C` | Flip argument order |
| `const_` | `(v: T) -> fn(Any)->T` | Constant function |
| `identity` | `(v: T) -> T` | Identity function |
| `applyN` | `(fn(T)->T, T, n: Int) -> T` | Apply function n times |
| `applyWhile` | `(fn(T)->T, fn(T)->Bool, T) -> T` | Apply while predicate holds |
| `memoize` | `(fn(Int)->T) -> fn(Int)->T` | Cache results by Int key |
| `negate` | `(fn(T)->Bool) -> fn(T)->Bool` | Invert a predicate |
| `both` | `(fn(T)->Bool, fn(T)->Bool) -> fn(T)->Bool` | AND two predicates |
| `either` | `(fn(T)->Bool, fn(T)->Bool) -> fn(T)->Bool` | OR two predicates |

---

## tuple

`import super.std.tuple`

Pair and tuple helpers.

| Function | Signature | Description |
|---|---|---|
| `pair` | `(A, B) -> (A, B)` | Create a pair |
| `triple` | `(A, B, C) -> (A, B, C)` | Create a triple |
| `fst` | `((A, B)) -> A` | First element |
| `snd` | `((A, B)) -> B` | Second element |
| `both` | `((A,A), fn(A)->B) -> (B,B)` | Apply fn to both elements |
| `toList` | `((T,T)) -> [T]` | Pair to list |
| `unzip` | `([(A,B)]) -> ([A],[B])` | Unzip list of pairs |
| `swap` | `((A, B)) -> (B, A)` | Swap pair |
| `mapFst` | `((A,B), fn(A)->C) -> (C,B)` | Transform first |
| `mapSnd` | `((A,B), fn(B)->C) -> (A,C)` | Transform second |

---

## hashmap

`import super.std.hashmap`

Utilities for the built-in HashMap type.

| Function | Signature | Description |
|---|---|---|
| `HashMap::new` | `() -> HashMap` | Create empty map |
| `HashMap::fromPairs` | `([(String,T)]) -> HashMap` | Build from list of pairs |
| `m.get` | `(key: String) -> Option(T)` | Look up a key |
| `m.set` | `(key: String, value: T)` | Insert or update |
| `m.remove` | `(key: String)` | Delete a key |
| `m.containsKey` | `(key: String) -> Bool` | Key existence check |
| `m.keys` | `() -> [String]` | All keys |
| `m.values` | `() -> [T]` | All values |
| `m.pairs` | `() -> [(String, T)]` | All key-value pairs |
| `m.len` | `() -> Int` | Number of entries |
| `m.merge` | `(other: HashMap) -> HashMap` | Merge (other wins on conflict) |
| `m.mapValues` | `(fn(T)->U) -> HashMap` | Transform all values |
| `m.filterKeys` | `(fn(String)->Bool) -> HashMap` | Keep matching keys |
| `m.filterValues` | `(fn(T)->Bool) -> HashMap` | Keep matching values |
| `m.increment` | `(key: String)` | Increment integer value by 1 |
| `m.update` | `(key: String, fn(T)->T)` | Update value in place |
| `m.toSortedPairs` | `() -> [(String, T)]` | Pairs sorted by key |

---

## set

`import super.std.set`

Set(T) data structure backed by HashMap (keys only, no values).

| Function | Signature | Description |
|---|---|---|
| `Set::new` | `() -> Set` | Empty set |
| `Set::fromList` | `([String]) -> Set` | Build from list |
| `s.add` | `(v: String)` | Add an element |
| `s.remove` | `(v: String)` | Remove an element |
| `s.contains` | `(v: String) -> Bool` | Membership test |
| `s.len` | `() -> Int` | Number of elements |
| `s.toList` | `() -> [String]` | Convert to list |
| `s.union` | `(other: Set) -> Set` | Set union |
| `s.intersect` | `(other: Set) -> Set` | Set intersection |
| `s.difference` | `(other: Set) -> Set` | Set difference |
| `s.isSubset` | `(other: Set) -> Bool` | True if s ⊆ other |

---

## io

`import super.std.io`

File and console I/O beyond the built-ins.

| Function | Signature | Description |
|---|---|---|
| `readFile` | `(path: String) -> String` | Read entire file as string |
| `writeFile` | `(path, content: String)` | Write string to file |
| `appendLine` | `(path, line: String)` | Append line to file |
| `linesOf` | `(path: String) -> [String]` | Read file as list of lines |
| `prompt` | `(msg: String) -> String` | Print prompt, read input |
| `promptYN` | `(msg: String) -> Bool` | y/n question, returns Bool |
| `eprintln` | `(msg: String)` | Print to stderr |

---

## tui

`import super.std.tui`

Terminal UI helpers for building text-mode interfaces.

| Function | Signature | Description |
|---|---|---|
| `clearScreen` | `()` | Clear the terminal |
| `moveCursor` | `(row, col: Int)` | ANSI cursor position |
| `hideCursor` | `()` | Hide terminal cursor |
| `showCursor` | `()` | Show terminal cursor |
| `drawBox` | `(row, col, h, w: Int, title: String)` | ASCII box |
| `drawProgressBar` | `(row, col, width: Int, pct: Float)` | Text progress bar |
| `drawTable` | `(rows: [[String]], headers: [String])` | Formatted table |
| `colorText` | `(s: String, color: String) -> String` | ANSI colored string |

---

## ansi

`import super.std.ansi`

ANSI escape sequence constants and helpers.

| Function / Constant | Description |
|---|---|
| `Ansi::reset` | Reset all attributes |
| `Ansi::bold`, `dim`, `italic`, `underline` | Text style codes |
| `Ansi::fg(r,g,b)` | 24-bit foreground RGB |
| `Ansi::bg(r,g,b)` | 24-bit background RGB |
| `Ansi::black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` | Standard foreground colors |
| `Ansi::bgBlack`, `bgRed`, … `bgWhite` | Standard background colors |
| `Ansi::up(n)`, `down(n)`, `left(n)`, `right(n)` | Cursor movement |
| `Ansi::moveTo(row, col)` | Absolute cursor position |
| `Ansi::clearLine` | Erase current line |
| `Ansi::clearScreen` | Erase entire screen |
| `Ansi::wrap(s, code)` | Wrap a string with an ANSI code |

---

## color

`import super.std.color`

Named color constants as `(Int,Int,Int)` tuples for use with raylib.

All standard web colors are defined: `Color::red`, `Color::green`, `Color::blue`,
`Color::white`, `Color::black`, `Color::yellow`, `Color::cyan`, `Color::magenta`,
`Color::orange`, `Color::purple`, `Color::pink`, `Color::brown`, `Color::gray`,
`Color::lightGray`, `Color::darkGray`, `Color::gold`, `Color::skyBlue`,
`Color::lime`, `Color::maroon`, `Color::darkGreen`, `Color::darkBlue`, etc.

| Function | Signature | Description |
|---|---|---|
| `Color::rgb` | `(r,g,b: Int) -> (Int,Int,Int)` | Build a color tuple |
| `Color::lerp` | `(a, b: (Int,Int,Int), t: Float) -> (Int,Int,Int)` | Interpolate colors |
| `Color::fade` | `(c: (Int,Int,Int), alpha: Float) -> (Int,Int,Int)` | Apply transparency |
| `Color::grayscale` | `(c: (Int,Int,Int)) -> (Int,Int,Int)` | Convert to gray |

---

## vec2

`import super.std.vec2`

2D vector math for game development.

```nova
let v = Vec2::new(3.0, 4.0)
let len = v.length()   // 5.0
```

| Constructor | Signature | Description |
|---|---|---|
| `Vec2::new` | `(x, y: Float) -> Vec2` | Create vector |
| `Vec2::zero` | `() -> Vec2` | (0, 0) |
| `Vec2::one` | `() -> Vec2` | (1, 1) |
| `Vec2::up` | `() -> Vec2` | (0, -1) |
| `Vec2::down` | `() -> Vec2` | (0, 1) |
| `Vec2::left` | `() -> Vec2` | (-1, 0) |
| `Vec2::right` | `() -> Vec2` | (1, 0) |
| `Vec2::fromAngle` | `(radians: Float) -> Vec2` | Unit vector from angle |

| Method | Signature | Description |
|---|---|---|
| `v.length` | `() -> Float` | Magnitude |
| `v.lengthSq` | `() -> Float` | Squared magnitude |
| `v.normalize` | `() -> Vec2` | Unit vector |
| `v.dot` | `(other: Vec2) -> Float` | Dot product |
| `v.cross` | `(other: Vec2) -> Float` | 2D cross product (scalar) |
| `v.add` | `(other: Vec2) -> Vec2` | Addition |
| `v.sub` | `(other: Vec2) -> Vec2` | Subtraction |
| `v.scale` | `(s: Float) -> Vec2` | Scalar multiply |
| `v.negate` | `() -> Vec2` | Negate |
| `v.lerp` | `(other: Vec2, t: Float) -> Vec2` | Linear interpolation |
| `v.distTo` | `(other: Vec2) -> Float` | Distance |
| `v.distToSq` | `(other: Vec2) -> Float` | Squared distance |
| `v.rotate` | `(angle: Float) -> Vec2` | Rotate by radians |
| `v.reflect` | `(normal: Vec2) -> Vec2` | Reflect off normal |
| `v.perpendicular` | `() -> Vec2` | 90° rotation |
| `v.clampLength` | `(max: Float) -> Vec2` | Cap magnitude |
| `v.angle` | `() -> Float` | Angle in radians |
| `v.angleTo` | `(other: Vec2) -> Float` | Angle to other vector |
| `v.toString` | `() -> String` | "(x, y)" |
| `v.toInt` | `() -> (Int, Int)` | Truncated to int pair |

---

## deque

`import super.std.deque`

Double-ended queue backed by a list with compaction.

```nova
let dq = Deque::new()
dq.pushBack(1)
dq.pushFront(0)
let front = dq.popFront()  // Some(0)
```

| Function | Signature | Description |
|---|---|---|
| `Deque::new` | `() -> Deque` | Empty deque |
| `Deque::fromList` | `([T]) -> Deque` | Create from list |
| `d.pushBack` | `(v: T)` | Add to back |
| `d.pushFront` | `(v: T)` | Add to front |
| `d.popFront` | `() -> Option(T)` | Remove from front |
| `d.popBack` | `() -> Option(T)` | Remove from back |
| `d.peekFront` | `() -> Option(T)` | Front without removing |
| `d.peekBack` | `() -> Option(T)` | Back without removing |
| `d.len` | `() -> Int` | Number of elements |
| `d.isEmpty` | `() -> Bool` | True if empty |
| `d.toList` | `() -> [T]` | Convert to list |
| `d.clear` | `()` | Remove all elements |

---

## widget

`import super.std.widget`

Raylib GUI widget components.

| Function | Signature | Description |
|---|---|---|
| `Button::new` | `(x,y,w,h: Int, label: String) -> Button` | Create a button |
| `b.draw` | `(color, hoverColor, textColor: (Int,Int,Int))` | Draw button |
| `b.isClicked` | `() -> Bool` | True when mouse released over it |
| `b.isHovered` | `() -> Bool` | True when mouse over it |
| `Slider::new` | `(x,y,w: Int, min,max,value: Float) -> Slider` | Create a slider |
| `sl.draw` | `(color, knobColor: (Int,Int,Int))` | Draw slider |
| `sl.update` | `()` | Handle mouse input |
| `sl.value` | `() -> Float` | Current value |
| `Label::draw` | `(x,y: Int, text: String, size: Int, color)` | Draw label text |
| `Panel::new` | `(x,y,w,h: Int) -> Panel` | Rectangular panel |
| `p.draw` | `(bgColor, borderColor: (Int,Int,Int))` | Draw panel |

---

## plot

`import super.std.plot`

2D chart drawing using raylib. Call draw functions inside `raylib::rendering()`.

```nova
let area = PlotArea::auto(50, 50, 400, 300, data)
area.drawGrid(10, 8, (40,40,40))
area.drawAxes((200,200,200))
area.lineChartThick(data, 2.0, (80,200,120))
area.drawTitle("My Chart", 18, (255,255,255))
```

| Constructor | Signature | Description |
|---|---|---|
| `PlotArea::new` | `(x,y,w,h, xMin,xMax,yMin,yMax: Float) -> PlotArea` | Manual bounds |
| `PlotArea::auto` | `(x,y,w,h: Int, data: [Float]) -> PlotArea` | Auto y-bounds from data |
| `PlotArea::square` | `(x,y,size: Int, data: [Float]) -> PlotArea` | Square auto-bounds |

| Method | Signature | Description |
|---|---|---|
| `a.toScreen` | `(dx, dy: Float) -> (Int, Int)` | Data → screen coords |
| `a.toData` | `(px, py: Int) -> (Float, Float)` | Screen → data coords |
| `a.drawAxes` | `(color)` | Draw x/y axes |
| `a.drawGrid` | `(cols, rows: Int, color)` | Draw background grid |
| `a.drawBorder` | `(color)` | Draw border rectangle |
| `a.drawTitle` | `(title: String, fontSize: Int, color)` | Centered title above plot |
| `a.drawXLabels` | `(labels: [String], fontSize: Int, color)` | X-axis labels |
| `a.drawYLabels` | `(steps: Int, fontSize: Int, color)` | Y-axis value labels |
| `a.lineChart` | `(data: [Float], color)` | Line chart |
| `a.lineChartThick` | `(data: [Float], thickness: Float, color)` | Thick line chart |
| `a.barChart` | `(data: [Float], color)` | Bar chart |
| `a.barChartLabeled` | `(data, labels: [String], barColor, labelColor)` | Labeled bar chart |
| `a.scatter` | `(points: [(Float,Float)], radius: Int, color)` | Scatter plot |
| `a.scatterSized` | `(points, sizes: [Int], color)` | Per-point radius scatter |
| `a.hLine` | `(dataY: Float, color)` | Horizontal reference line |
| `a.vLine` | `(dataX: Float, color)` | Vertical reference line |
| `a.fillArea` | `(data: [Float], color)` | Filled area chart |

---

## timer

`import super.std.timer`

Game timers and cooldowns.

```nova
let jump = Timer::cooldown(0.3)
// each frame:
jump.update(dt)
if raylib::isKeyPressed("Space") && jump.ready() {
    // jump!
}
```

| Constructor | Signature | Description |
|---|---|---|
| `Timer::once` | `(duration: Float) -> Timer` | One-shot timer |
| `Timer::repeating` | `(duration: Float) -> Timer` | Auto-resetting timer |
| `Timer::cooldown` | `(duration: Float) -> Timer` | Starts as done/ready |

| Method | Signature | Description |
|---|---|---|
| `t.update` | `(dt: Float) -> Bool` | Advance; returns true when fires |
| `t.isDone` | `() -> Bool` | True when elapsed |
| `t.isRunning` | `() -> Bool` | True if counting |
| `t.progress` | `() -> Float` | 0.0..1.0 normalized |
| `t.elapsed` | `() -> Float` | Seconds since reset |
| `t.remaining` | `() -> Float` | Seconds left |
| `t.reset` | `()` | Restart timer |
| `t.stop` | `()` | Pause timer |
| `t.start` | `()` | Resume timer |
| `t.setDuration` | `(d: Float)` | Change duration |
| `t.activate` | `()` | Reset a cooldown to start counting |
| `t.ready` | `() -> Bool` | True if done; auto-resets |

---

## tween

`import super.std.tween`

Value interpolation with easing functions.

```nova
let tween = Tween::smooth(0.0, 100.0, 1.5)
// each frame:
let x = tween.update(dt)
```

| Easing Function | Description |
|---|---|
| `Ease::linear(t)` | No easing |
| `Ease::easeIn(t)` | Quadratic ease in |
| `Ease::easeOut(t)` | Quadratic ease out |
| `Ease::easeInOut(t)` | Quadratic ease in-out |
| `Ease::easeInCubic(t)` | Cubic ease in |
| `Ease::easeOutCubic(t)` | Cubic ease out |
| `Ease::easeInOutCubic(t)` | Cubic ease in-out |
| `Ease::sineIn(t)` | Sine ease in |
| `Ease::sineOut(t)` | Sine ease out |
| `Ease::sineInOut(t)` | Sine ease in-out |
| `Ease::easeInBack(t)` | Overshoot ease in |
| `Ease::easeOutBack(t)` | Overshoot ease out |
| `Ease::easeOutBounce(t)` | Bounce ease out |
| `Ease::easeInBounce(t)` | Bounce ease in |
| `Ease::easeOutElastic(t)` | Elastic ease out |
| `Ease::easeInElastic(t)` | Elastic ease in |

| Constructor | Signature | Description |
|---|---|---|
| `Tween::new` | `(from, to, dur: Float, easeFn) -> Tween` | Custom easing |
| `Tween::linear` | `(from, to, dur: Float) -> Tween` | Linear tween |
| `Tween::smooth` | `(from, to, dur: Float) -> Tween` | Ease-in-out quadratic |

| Method | Signature | Description |
|---|---|---|
| `t.update` | `(dt: Float) -> Float` | Advance and return value |
| `t.value` | `() -> Float` | Current value (no advance) |
| `t.progress` | `() -> Float` | 0.0..1.0 normalized |
| `t.isDone` | `() -> Bool` | True when complete |
| `t.remaining` | `() -> Float` | Seconds left |
| `t.reset` | `()` | Restart from beginning |
| `t.restart` | `(from, to: Float)` | New endpoints + restart |
| `t.ping` | `()` | Reverse direction (ping-pong) |
| `lerpF` | `(a, b, t: Float) -> Float` | Float interpolation helper |
| `lerpI` | `(a, b: Int, t: Float) -> Int` | Int interpolation helper |

---

## input

`import super.std.input`

Named action input mapping over raylib keyboard and mouse.

```nova
let map = InputMap::new()
map.bindKey("jump", "Space")
map.bindKey("left", "A")
map.bindKey("right", "D")
map.bindMouse("fire", "Left")

// in game loop:
let dx = map.axis("left", "right")
if map.isPressed("jump") { /* ... */ }
```

| Constructor | Signature | Description |
|---|---|---|
| `InputMap::new` | `() -> InputMap` | Create empty input map |

| Method | Signature | Description |
|---|---|---|
| `m.bindKey` | `(action, key: String)` | Bind action to key name |
| `m.bindMouse` | `(action, button: String)` | Bind action to mouse button |
| `m.unbind` | `(action: String)` | Remove all bindings for action |
| `m.isPressed` | `(action: String) -> Bool` | Just triggered this frame |
| `m.isHeld` | `(action: String) -> Bool` | Held every frame |
| `m.isReleased` | `(action: String) -> Bool` | Just released this frame |
| `m.axis` | `(neg, pos: String) -> Float` | -1, 0, or 1 directional |
| `m.axis2D` | `(left,right,up,down) -> (Float,Float)` | 2D movement axis |
| `InputMap::mouseX` | `() -> Int` | Mouse X position |
| `InputMap::mouseY` | `() -> Int` | Mouse Y position |
| `InputMap::mousePos` | `() -> (Int,Int)` | Mouse position tuple |
| `InputMap::mouseWheel` | `() -> Float` | Scroll wheel delta |
| `InputMap::mousePressed` | `(button: String) -> Bool` | Direct mouse button check |
| `InputMap::anyKey` | `() -> Bool` | True if any key pressed |
| `InputMap::lastKey` | `() -> Option(String)` | Last pressed key name |

Key names: `"A"`–`"Z"`, `"0"`–`"9"`, `"Space"`, `"Enter"`, `"Escape"`, `"Up"`, `"Down"`, `"Left"`, `"Right"`, `"Tab"`, `"Backspace"`, `"F1"`–`"F12"`, etc.  
Mouse buttons: `"Left"`, `"Right"`, `"Middle"`

---

## camera

`import super.std.camera`

2D camera with pan, zoom, follow, and screen shake.

```nova
let cam = Camera2D::new(800, 600)

while raylib::rendering() {
    cam.update(raylib::getFrameTime())
    cam.follow(player.pos, 5.0, dt)
    cam.drawRect(player.pos.x, player.pos.y, 32.0, 32.0, (0,255,0))
}
```

| Constructor | Signature | Description |
|---|---|---|
| `Camera2D::new` | `(screenW, screenH: Int) -> Camera2D` | Centered on world origin |
| `Camera2D::at` | `(x, y: Float, screenW, screenH: Int) -> Camera2D` | Explicit position |

| Method | Signature | Description |
|---|---|---|
| `cam.update` | `(dt: Float)` | Advance shake effect |
| `cam.worldToScreen` | `(wp: Vec2) -> Vec2` | World → screen coords |
| `cam.screenToWorld` | `(sp: Vec2) -> Vec2` | Screen → world coords |
| `cam.follow` | `(target: Vec2, speed, dt: Float)` | Lerp toward target |
| `cam.snapTo` | `(target: Vec2)` | Instant center on target |
| `cam.pan` | `(dx, dy: Float)` | Offset camera position |
| `cam.setZoom` | `(z: Float)` | Set zoom level |
| `cam.zoomBy` | `(factor: Float)` | Multiply zoom |
| `cam.setRotation` | `(degrees: Float)` | Set rotation angle |
| `cam.shake` | `(magnitude, duration: Float)` | Start screen shake |
| `cam.drawRect` | `(wx,wy,w,h: Float, color)` | Draw rect in world space |
| `cam.drawCircle` | `(wx,wy,radius: Float, color)` | Draw circle in world space |
| `cam.drawText` | `(text, wx,wy: Float, size: Int, color)` | Draw text in world space |
| `cam.worldBounds` | `() -> (Float,Float,Float,Float)` | View rect in world space |
| `cam.isVisible` | `(wx,wy,margin: Float) -> Bool` | Visibility check |

---

## physics

`import super.std.physics`

2D physics primitives — AABB, circles, rigid bodies, and raycasting.

```nova
let body = Body2D::new(100.0, 200.0, 1.0)
body.applyGravity(980.0, dt)
body.update(dt)
let box = body.aabb(32.0, 32.0)
let pen = box.overlap(wall)
```

| Struct | Fields | Description |
|---|---|---|
| `AABB` | `pos: Vec2, size: Vec2` | Axis-aligned bounding box |
| `Circle` | `pos: Vec2, radius: Float` | Circle collider |
| `Body2D` | `pos, vel, acc: Vec2, mass, restitution, friction: Float` | Physics body |
| `Ray2` | `origin, dir: Vec2` | Ray for casting |
| `HitInfo` | `hit: Bool, point, normal: Vec2, t: Float` | Raycast result |

| Function | Signature | Description |
|---|---|---|
| `AABB::new` | `(x,y,w,h: Float) -> AABB` | Top-left + size |
| `AABB::centered` | `(cx,cy,w,h: Float) -> AABB` | Centered AABB |
| `box.center` | `() -> Vec2` | Center point |
| `box.overlaps` | `(other: AABB) -> Bool` | Overlap test |
| `box.overlap` | `(other: AABB) -> Vec2` | Penetration vector |
| `box.containsPoint` | `(p: Vec2) -> Bool` | Point inside? |
| `box.overlapCircle` | `(c: Circle) -> Bool` | AABB vs circle |
| `Circle::new` | `(x,y,r: Float) -> Circle` | Create circle |
| `Body2D::new` | `(x,y,mass: Float) -> Body2D` | Create body |
| `b.applyForce` | `(fx,fy: Float)` | Continuous force |
| `b.applyImpulse` | `(ix,iy: Float)` | Instant velocity change |
| `b.applyGravity` | `(g,dt: Float)` | Gravity per frame |
| `b.update` | `(dt: Float)` | Integrate motion |
| `b.stop` | `()` | Zero velocity |
| `b.aabb` | `(w,h: Float) -> AABB` | Centered AABB from body |
| `b.circle` | `(r: Float) -> Circle` | Circle from body |
| `resolveAABB` | `(a,b: Body2D, aW,aH,bW,bH: Float)` | Resolve body vs body |
| `resolveCircle` | `(a,b: Body2D, ar,br: Float)` | Circle vs circle |
| `pushOutAABB` | `(body: Body2D, bW,bH: Float, wall: AABB) -> Vec2` | Push out of static wall |
| `Ray2::new` | `(ox,oy,dx,dy: Float) -> Ray2` | Create ray |
| `ray.castAABB` | `(box: AABB) -> HitInfo` | Slab-method ray cast |

---

## entity

`import super.std.entity`

Lightweight entity system for game objects.

```nova
let world = EntityWorld::new()
let player = world.spawn(400.0, 300.0, "player")
player.vel.x = 100.0

// each frame:
world.update(dt)
world.forEachTagged("enemy", fn(e: Entity) {
    e.entityDrawRect((255, 0, 0))
})
```

| Struct | Key Fields | Description |
|---|---|---|
| `Entity` | `id, pos, vel, size: Vec2, tag: String, alive: Bool, data: Float` | Game object |
| `EntityWorld` | internal | Manages all entities |

| Method | Signature | Description |
|---|---|---|
| `EntityWorld::new` | `() -> EntityWorld` | Create empty world |
| `w.spawn` | `(x,y: Float, tag: String) -> Entity` | Create entity |
| `w.spawnFull` | `(x,y,vx,vy,w,h: Float, tag: String, data: Float) -> Entity` | Full constructor |
| `w.kill` | `(id: Int)` | Mark entity dead |
| `w.killAll` | `(tag: String)` | Kill all with tag |
| `w.update` | `(dt: Float)` | Integrate + prune dead |
| `w.count` | `() -> Int` | Total live entities |
| `w.countAlive` | `(tag: String) -> Int` | Live entities with tag |
| `w.query` | `(tag: String) -> [Entity]` | Get live entities by tag |
| `w.all` | `() -> [Entity]` | All live entities |
| `w.getById` | `(id: Int) -> Option(Entity)` | Look up by id |
| `w.forEach` | `(fn(Entity))` | Iterate all live |
| `w.forEachTagged` | `(tag: String, fn(Entity))` | Iterate by tag |
| `w.clear` | `()` | Remove all entities |
| `e.overlapsAABB` | `(other: Entity) -> Bool` | AABB collision check |
| `e.overlapCircle` | `(other: Entity) -> Bool` | Circle collision check |
| `e.center` | `() -> Vec2` | Center of entity |
| `e.entityDrawRect` | `(color)` | Draw as rectangle |
| `e.entityDrawCircle` | `(color)` | Draw as circle |

---

## scene

`import super.std.scene`

Scene-based state management for games (title screen, gameplay, pause, etc.).

```nova
let titleScene = Scene::new(
    fn(dt: Float) { /* update */ },
    fn()          { /* draw   */ }
)

let mgr = SceneManager::new(titleScene)

while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

| Constructor | Signature | Description |
|---|---|---|
| `Scene::new` | `(updateFn: fn(Float), drawFn: fn()) -> Scene` | Create scene |
| `Scene::empty` | `() -> Scene` | No-op placeholder |
| `SceneManager::new` | `(initial: Scene) -> SceneManager` | Manager with first scene |
| `SceneManager::empty` | `() -> SceneManager` | Empty manager |

| Method | Signature | Description |
|---|---|---|
| `mgr.update` | `(dt: Float)` | Call current scene update |
| `mgr.draw` | `()` | Call current scene draw |
| `mgr.switch` | `(scene: Scene)` | Replace current + clear stack |
| `mgr.push` | `(scene: Scene)` | Push scene (pause current) |
| `mgr.pop` | `()` | Return to previous scene |
| `mgr.has` | `() -> Bool` | True if any scene active |
| `mgr.depth` | `() -> Int` | Stack depth |

---

## grid

`import super.std.grid`

Fixed-size 2D grid with pathfinding and drawing.

```nova
let g = Grid::new(20, 15, 0)
g.set(5, 3, 1)                   // set a wall
let path = g.bfs(0,0, 19,14, fn(v: Any) -> Bool { v == 0 })
g.draw(0, 0, 32, fn(v: Any) -> (Int,Int,Int) {
    if v == 1 { (100,100,100) } else { (30,30,30) }
})
```

| Constructor | Signature | Description |
|---|---|---|
| `Grid::new` | `(cols, rows: Int, default: Any) -> Grid` | Create filled grid |

| Method | Signature | Description |
|---|---|---|
| `g.width` | `() -> Int` | Number of columns |
| `g.height` | `() -> Int` | Number of rows |
| `g.inBounds` | `(col, row: Int) -> Bool` | Bounds check |
| `g.get` | `(col, row: Int) -> Any` | Get cell value |
| `g.set` | `(col, row: Int, value: Any)` | Set cell value |
| `g.fill` | `(value: Any)` | Fill all cells |
| `g.fillRect` | `(x,y,w,h: Int, value: Any)` | Fill a rectangular region |
| `g.neighbors4` | `(col, row: Int) -> [(Int,Int)]` | 4-connected neighbors |
| `g.neighbors8` | `(col, row: Int) -> [(Int,Int)]` | 8-connected neighbors |
| `g.forEach` | `(fn(col,row: Int, value: Any))` | Iterate all cells |
| `g.bfs` | `(sCol,sRow,gCol,gRow: Int, passable: fn(Any)->Bool) -> [(Int,Int)]` | BFS shortest path |
| `g.draw` | `(screenX,screenY,cellSize: Int, colorFn: fn(Any)->(Int,Int,Int))` | Draw filled cells |
| `g.drawLines` | `(screenX,screenY,cellSize: Int, color)` | Draw grid lines |
| `g.drawLabels` | `(screenX,screenY,cellSize: Int, strFn,fontSize: Int, color)` | Draw text labels |

---

## noise

`import super.std.noise`

Procedural noise for terrain, textures, and particle effects.

```nova
let h = fbm(x * 0.01, y * 0.01, 42, 6, 2.0, 0.5)
let c = noiseToColor(h, (0,80,200), (200,240,150))
```

| Function | Signature | Description |
|---|---|---|
| `hash` | `(ix, iy, seed: Int) -> Float` | Deterministic hash [0,1) |
| `valueNoise` | `(x, y: Float, seed: Int) -> Float` | Bilinear value noise |
| `smoothNoise` | `(x, y: Float, seed: Int) -> Float` | Smoothstep value noise |
| `fbm` | `(x,y: Float, seed,octaves: Int, lacunarity,gain: Float) -> Float` | Fractal Brownian motion |
| `ridged` | `(x,y: Float, seed,octaves: Int) -> Float` | Ridged multifractal |
| `domain` | `(x,y: Float, seed: Int, strength: Float) -> Float` | Domain-warped fbm |
| `noiseToColor` | `(n: Float, lo,hi: (Int,Int,Int)) -> (Int,Int,Int)` | Lerp color by noise |

**Recommended parameters for `fbm`:**

| Effect | octaves | lacunarity | gain |
|---|---|---|---|
| Smooth terrain | 4 | 2.0 | 0.5 |
| Detailed terrain | 8 | 2.0 | 0.5 |
| Clouds | 5 | 2.0 | 0.6 |
| Turbulence | 6 | 2.0 | 0.4 |

---

*Generated for Nova standard library — see `documentation/game_engine_guide.md` for a complete game building tutorial.*
