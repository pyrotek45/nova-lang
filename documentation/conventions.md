# Nova Style & Conventions Guide

> This guide defines the naming, formatting, and design conventions used
> throughout the Nova standard library and recommended for all Nova code.

---

## 1. Naming Conventions

### Types (structs, enums, type aliases)

- **PascalCase** — every word capitalized, no underscores.
- Generic parameters: single uppercase letter (`T`, `K`, `V`, `A`).

```nv
struct Vec2 { x: Float, y: Float }
struct Entity(T) { id: Int, data: T }
struct HashMap(K, V) { ... }
type updatable = Dyn(T = update: fn($T, Float))
```

### Variables and parameters

- **camelCase** — first word lowercase, subsequent words capitalized.

```nv
let playerHealth = 100
let maxSpeed = 5.0
fn greet(userName: String) { ... }
```

### Functions and methods

- **camelCase** — verbs or verb-phrases preferred.

```nv
fn calculateDamage(base: Int, armor: Int) -> Int { ... }
fn extends toString(self: Vec2) -> String { ... }
fn extends isEmpty(self: Deque($T)) -> Bool { ... }
```

### Module names

- **snake_case** — lowercase, underscores for separation.
- Module names match the file name (without `.nv`).
- **Exception**: Modules that extend a built-in type may use the type's
  PascalCase name to avoid variable-name clashes (e.g., `module List`
  for `std/list.nv`, since `list` is a common variable name).

```nv
module my_game          // file: my_game.nv
module test_closures    // file: test_closures.nv
```

### Constants and level markers

- **UPPER_CASE** — for static values that act as constants.
- Implemented as static methods on an empty struct.

```nv
struct Log {}
fn extends(Log) TRACE() -> Int { return 0 }
fn extends(Log) DEBUG() -> Int { return 1 }
```

---

## 2. Constructors

### Primary constructor: `new`

Every struct that needs construction should provide `Type::new(...)`.
This is the standard, expected entry point.

```nv
fn extends(Vec2) new(x: Float, y: Float) -> Vec2 { ... }
let v = Vec2::new(1.0, 2.0)
```

### Named constructors for alternative creation

Use descriptive names for alternative constructors:

| Pattern       | When to use                        | Example                    |
|---------------|------------------------------------|----------------------------|
| `new`         | Primary constructor                | `Vec2::new(1.0, 2.0)`     |
| `empty`       | Zero-element / blank state         | `Set::empty()`             |
| `from___`     | Convert from another format        | `HashMap::fromPairs(list)` |
| `with___`     | Primary + config override          | `Logger::withLevel(n, 2)`  |
| Descriptive   | Semantically distinct alternatives | `Timer::repeating(1.0)`    |

**Avoid**: `default` as a constructor name — use `new` for the primary,
or `empty` for a zero-state alternative.

---

## 3. Method Naming Patterns

### Predicates (return Bool)

- Prefix with `is` or `has`:

```nv
fn extends isEmpty(self: Deque($T)) -> Bool { ... }
fn extends isDone(self: Timer) -> Bool { ... }
fn extends isRunning(self: Timer) -> Bool { ... }
fn extends has(self: Set($T), value: $T) -> Bool { ... }
fn extends isClicked(self: Button) -> Bool { ... }
```

### Mutators

- Use imperative verbs:

```nv
fn extends add(self: Set($T), value: $T) { ... }
fn extends clear(self: Updater) { ... }
fn extends reset(self: Timer) { ... }
fn extends push(self: Deque($T), value: $T) { ... }
```

### Accessors / converters

- `get___` for lookups that may fail (return `Option`).
- `to___` for type conversions.
- `count` for size/length (not `size` or `length`—use `.len()` for the
  built-in list/string length).

```nv
fn extends getById(self: EntityWorld($T), id: Int) -> Option(Entity($T)) { ... }
fn extends toString(self: Vec2) -> String { ... }
fn extends toList(self: Set($T)) -> [$T] { ... }
fn extends count(self: Updater) -> Int { ... }
```

### Update methods

- Game objects that animate over time use `update(self, dt: Float)`.
- Return `Bool` if the caller needs to know about state changes
  (e.g., `Timer.update(dt) -> Bool` fires when done).

```nv
fn extends update(self: Emitter, dt: Float) { ... }
fn extends update(self: Timer, dt: Float) -> Bool { ... }
fn extends update(self: Tween, dt: Float) -> Float { ... }
```

### Draw methods

- `draw(self)` for basic rendering.
- `drawAt(self, x, y, w, h)` for container-positioned rendering.
- `drawOffset(self, camX, camY)` for camera-aware rendering.

---

## 4. Formatting

### Indentation

- **4 spaces** per indent level. No tabs.

### Braces

- Opening brace `{` on the **same line** as the statement.
- Closing brace `}` on its own line, at the same indent as the opening keyword.

```nv
if x > 0 {
    println("positive")
} elif x == 0 {
    println("zero")
} else {
    println("negative")
}
```

### Single-line bodies

- Short function bodies may be written on one line:

```nv
fn extends(Ease) linear(t: Float) -> Float { return t }
fn extends(Vec2) zero() -> Vec2 { return Vec2::new(0.0, 0.0) }
```

### Semicolons

- Semicolons **separate** statements on the same line.
- Prefer newlines for readability. Use semicolons sparingly, mainly for:
  - Very short related statements: `let x = 0; let y = 0`
  - C-style for loops: `for let i = 0; i < n; i += 1 { ... }`
- Never end a line with a semicolon (it's not required).

### Comments

- Use `//` line comments. Nova supports `/* ... */` block comments.
- Module headers use a boxed banner:

```nv
// ============================================================
// std/module_name.nv  — Short description
// ============================================================
```

- Section dividers use `// ── Section Name ───────...`

---

## 5. File Structure

Standard order for a module file:

1. **Module declaration**: `module my_module`
2. **Imports**: `import super.std.vec2`
3. **Type aliases**: `type updatable = Dyn(...)`
4. **Struct definitions**: `struct MyType { ... }`
5. **Static constructors**: `fn extends(MyType) new(...) -> MyType { ... }`
6. **Instance methods**: `fn extends methodName(self: MyType, ...) { ... }`
7. **Module-level functions**: `fn helperName(...) { ... }`

---

## 6. Import Conventions

- From the standard library: `import super.std.module_name`
- Relative sibling file: `import module_name` (same directory)
- Parent directory: `import super.module_name`

---

## 7. Closure Style

### Short closures (bar syntax)

Use `|args| expr` for short inline callbacks:

```nv
list.map(|x: Int| x * 2)
list.filter(|x: Int| x > 0)
```

### Typed closures (fn syntax)

Use `fn(args) -> Type { body }` when a return type annotation is needed:

```nv
fn(x: Int) -> String { return Cast::string(x) }
```

### Zero-arg closures

```nv
let greet = || { println("hello") }
let getVal = fn() -> Int { return 42 }
```

> **Tip**: `||` closures cannot have `-> Type` annotations. If you need
> a typed return, use the `fn()` form instead.

---

## 8. Error Handling Conventions

- Functions that can fail return `Option(T)` or `Result(T, E)`.
- Use `unwrap()` only when failure is a programming error.
- Prefer `orDefault(fallback)` for safe fallbacks.

```nv
let val = Cast::int(input).orDefault(0)
```

---

## 9. Pipe Operator `|>`

The pipe operator passes the left-hand value as the **first argument** to
the function on the right. The function **must** be called with `()`.

```nv
// Correct:
5 |> double() |> println()

// Wrong (missing parentheses):
// 5 |> println     ← parser error

// Multi-arg piping:
10 |> add(5)        // calls add(10, 5)
```

The pipe is designed to make regular functions feel like left-to-right
method chains. Use it when chaining transformations:

```nv
let result = data
    |> parse()
    |> validate()
    |> transform()
```

---

## 10. Dyn Type Conventions

Use `Dyn` for **structural typing** — accepting any struct that has
specific fields:

```nv
type named = Dyn(T = name: String)
fn greet(obj: Dyn(T = name: String)) {
    println("Hello, " + obj.name)
}
```

Use `Dyn` with function fields for **vtable dispatch**:

```nv
type updatable = Dyn(T = update: fn($T, Float))
fn tickAll(items: [Dyn(T = update: fn($T, Float))], dt: Float) {
    for item in items { item->update(dt) }
}
```

Use `->` to dispatch through function fields on Dyn objects.

---

## Summary Table

| Element              | Convention        | Example                     |
|----------------------|-------------------|-----------------------------|
| Struct / Enum        | PascalCase        | `Entity`, `Vec2`, `HashMap` |
| Generic param        | Single uppercase  | `T`, `K`, `V`               |
| Variable / param     | camelCase         | `playerHealth`, `maxSpeed`   |
| Function / method    | camelCase         | `update`, `isEmpty`, `draw`  |
| Module               | snake_case        | `module my_game`             |
| Constant             | UPPER_CASE        | `Log::TRACE`, `Log::ERROR`   |
| Primary constructor  | `new`             | `Vec2::new(1.0, 2.0)`       |
| Empty constructor    | `empty`           | `Set::empty()`               |
| Conversion ctor      | `from___`         | `HashMap::fromPairs(list)`   |
| Predicate            | `is___` / `has`   | `isEmpty()`, `has(key)`      |
| Converter            | `to___`           | `toString()`, `toList()`     |
