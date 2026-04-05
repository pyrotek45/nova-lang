# Debugging Nova Programs

A practical guide to finding and fixing bugs in Nova — from the type checker
to the step debugger to the disassembler.

---

## Table of Contents

1. [Philosophy — Three Lines of Defence](#1-philosophy--three-lines-of-defence)
2. [First Line: `nova check`](#2-first-line-nova-check)
3. [Second Line: `nova dbg` — The Step Debugger](#3-second-line-nova-dbg--the-step-debugger)
   - [Launching](#launching)
   - [Screen Layout](#screen-layout)
   - [Controls Reference](#controls-reference)
   - [Play Mode](#play-mode)
   - [Navigating History](#navigating-history)
   - [Run to End and Rewind](#run-to-end-and-rewind)
   - [Step Over](#step-over)
   - [Reading the Heap/GC Bar](#reading-the-heapgc-bar)
   - [Reading the Stack](#reading-the-stack)
   - [Reading Variables](#reading-variables)
   - [Reading Bytecode](#reading-bytecode)
4. [Third Line: `nova dis` — The Disassembler](#4-third-line-nova-dis--the-disassembler)
   - [Launching](#launching-1)
   - [Reading the Output](#reading-the-output)
   - [Flow Arrows](#flow-arrows)
   - [Function Nesting](#function-nesting)
   - [Summary Section](#summary-section)
5. [Debugging Strategies](#5-debugging-strategies)
   - [Strategy 1: Narrow It Down](#strategy-1-narrow-it-down)
   - [Strategy 2: Watch the Stack](#strategy-2-watch-the-stack)
   - [Strategy 3: Track Variables](#strategy-3-track-variables)
   - [Strategy 4: Bisect with Play Mode](#strategy-4-bisect-with-play-mode)
   - [Strategy 5: Check the Disassembly](#strategy-5-check-the-disassembly)
6. [Common Mistakes and How to Spot Them](#6-common-mistakes-and-how-to-spot-them)
7. [Pitfalls](#7-pitfalls)
8. [Tips and Tricks](#8-tips-and-tricks)
9. [Quick Reference Card](#9-quick-reference-card)

---

## 1. Philosophy — Three Lines of Defence

Nova gives you three tools, each at a different level:

| Tool | Command | What it catches |
|---|---|---|
| **Type checker** | `nova check file.nv` | Type errors, missing fields, wrong argument counts, unreachable code — all before anything runs. |
| **Step debugger** | `nova dbg file.nv` | Runtime logic errors — wrong values, unexpected control flow, off-by-one loops, bad state. |
| **Disassembler** | `nova dis file.nv` | Compiler output — see exactly what instructions were generated, how jumps are wired, what the optimizer did. |

**Rule of thumb:** Always start with `check`. If it passes and the program still
misbehaves, reach for `dbg`. If you suspect the compiler itself is doing something
wrong, use `dis`.

---

## 2. First Line: `nova check`

```bash
nova check my_program.nv
```

This parses, resolves imports, and typechecks your entire program without executing
it. If everything is fine you'll see:

```
OK | Compile time: 12ms
```

If there are errors, Nova prints them with line numbers and clear descriptions:

```
Error at line 14: Type mismatch — expected Int, got String
```

### What `check` catches

- **Type mismatches** — passing a `String` where an `Int` is expected.
- **Missing struct fields** — forgetting a field in a constructor.
- **Wrong argument counts** — too many or too few arguments to a function.
- **Undefined variables and functions** — typos in names.
- **Missing return values** — a function declares a return type but a branch doesn't return.
- **Invalid pattern matches** — missing enum variants, duplicate defaults.
- **Void used as a value** — trying to store the result of a void function.
- **Import errors** — modules that can't be found.

### Tips for `check`

- **Run it constantly.** It's fast. Make it a habit after every edit.
- **Use project mode.** If you have a `main.nv`, just `cd` into the folder and
  run `nova check` with no arguments.
- **Check remote code.** `nova check --git owner/repo/file.nv` works too —
  useful for validating libraries before you install them.

---

## 3. Second Line: `nova dbg` — The Step Debugger

The debugger is a full-screen terminal UI (TUI) that lets you execute your
program one instruction at a time, inspect the stack and variables at every
step, and travel backwards through execution history.

### Launching

```bash
nova dbg my_program.nv        # local file
nova dbg                       # project mode (runs main.nv)
nova dbg --git owner/repo/f.nv # from GitHub
```

The debugger compiles the file, then opens the TUI. You start at instruction 0,
before anything has executed.

### Screen Layout

The debugger uses a **3-column layout**:

```
┌─────────────────────┬──────────────────────┬──────────────┐
│  Bytecode Listing   │  Stack               │  Variables   │
│                     │                      │              │
│    0 ALLOCGLOBAL +5 │  ─ Stack ─           │ ─ Variables ─│
│    1 INTEGER     0  │   (3 entries, off=2)  │  locals:     │
│  > 5 STOREGLOBAL g  │  > [  2] Int(42) (x) │   x = 42    │
│    6 GETGLOBAL   g  │    [  1] Fn@30 (foo) │  globals:    │
│    7 CALL           │    [  0] Fn@10 (main) │   g = 99    │
│   ...               │                      │              │
├─────────────────────┴──────────────────────┴──────────────┤
│  Output: Hello, world!                                    │
├───────────────────────────────────────────────────────────┤
│  Heap: 3 live / 16 slots (13 free) │ Stack: 5 │ GC …     │
├───────────────────────────────────────────────────────────┤
│  [↑/↓] step  [p] play  [r] run  [n] next  [?] help  [q] │
└───────────────────────────────────────────────────────────┘
```

- **Left column — Bytecode**: The full compiled instruction listing. The
  current instruction is marked with `>` and highlighted. The view auto-scrolls
  to keep the current instruction centered. Instructions show resolved names
  (`STOREGLOBAL  my_var`) instead of raw slot numbers.
- **Middle column — Stack**: All values currently on the stack, shown from
  top to bottom. Top-of-stack is marked with `>` and shown in green. Entries
  in the current local frame are marked with `•` and shown in white. Entries
  below the frame offset are shown in grey. Where possible, entries are
  annotated with their name (e.g. `Int(42) (x)`).
- **Right column — Variables**: Named local variables and their values, plus
  any globals that hold non-function values. This is the "human-readable"
  view — you don't need to decode the stack by hand.
- **Output**: The last 2 lines of program output from `print`/`println`.
- **Heap/GC bar**: Live object counts, heap capacity, GC threshold info, and
  callstack depth. See [Reading the Heap/GC Bar](#reading-the-heapgc-bar).
- **Header**: Step number (`Step 5/120`), instruction pointer (`IP:12`),
  callstack depth (`Depth:1`), and local frame offset (`Offset:3`).

### Controls Reference

| Key | Action |
|---|---|
| `↓` / `j` / `Space` | Step forward one instruction |
| `↑` / `k` | Step backward (browse history) |
| `u` / `d` | Scroll stack view up / down |
| `PgDn` | Step forward 20 instructions |
| `PgUp` | Step back 20 instructions |
| `Home` | Jump to the very first step |
| `End` | Jump to the latest step |
| `Tab` | Toggle between Stack view and Heap inspect view |
| `p` | Toggle play/pause (auto-step) |
| `+` / `=` | Speed up playback (decrease delay) |
| `-` / `_` | Slow down playback (increase delay) |
| `r` | Run to end (execute all remaining instructions) |
| `n` | Step over (run until callstack returns to current depth) |
| `:` | Enter command mode |
| `?` | Toggle the help screen |
| `q` / `Esc` | Quit the debugger |

### Command Mode

Press **`:`** to enter **command mode**. A command prompt appears at the bottom
of the screen. Type a command and press **Enter** to execute, or **Esc** to
cancel. If a command has an error (bad address, invalid number, unknown
command), the error is shown in red on the control bar and clears on the next
keypress.

| Command | Action |
|---|---|
| `:goto <addr>` | Jump to a bytecode address. If the address hasn't been executed yet, the debugger steps forward until it reaches it (up to 1M steps). |
| `:step <n>` | Execute *n* steps forward. |
| `:find <text>` | Search bytecode for text (matches against opname or operand). Wraps around from the current position. If the target instruction hasn't been executed yet, the debugger steps forward to reach it. |
| `:speed <ms>` | Set the playback speed in milliseconds (1–10000). |
| `:heap` | Switch to heap inspect view. |
| `:stack` | Switch to stack view. |
| `:help` | Show the help screen. |
| `:quit` | Quit the debugger. |

### Play Mode

Press **`p`** to start **play mode**. The debugger will automatically step
forward at a steady pace, like watching a recording of your program running.

- The header shows `▶ PLAYING (100ms)` with the current speed.
- Press **`+`** or **`=`** to speed up (decrease the delay between steps).
- Press **`-`** or **`_`** to slow down (increase the delay).
- The speed range is 10ms (very fast) to 2000ms (very slow). Default is 100ms.
- Press **`p`** again to pause.
- Pressing any navigation key (`↑`, `↓`, `PgUp`, `PgDn`, `Home`, `End`, `r`,
  `n`, `?`) automatically pauses play mode.
- Play mode stops automatically when the program finishes or hits an error.

**Replaying history:** If you've stepped forward, scrolled back in history, and
then press `p`, play mode replays through the already-recorded history first
before executing new instructions. This lets you "rewatch" a section of
execution at any speed.

### Navigating History

The debugger records a snapshot of the entire VM state at every instruction.
This means you can step backwards freely:

- **`↑`** moves one step back in history.
- **`↓`** moves one step forward. If you're at the latest snapshot and the
  program isn't finished, it executes the next instruction.
- **`PgUp`** / **`PgDn`** move 20 steps at a time.
- **`Home`** jumps to the very first step (the start of the program).
- **`End`** jumps to the latest recorded step.

You can go forward, go back, go forward again — history is never lost.

### Run to End and Rewind

Press **`r`** to execute all remaining instructions at full speed. Unlike
`play`, this doesn't animate — it just runs everything as fast as possible.

**Key feature:** Every single step is still recorded. After `r` finishes, you
can use `↑`, `PgUp`, and `Home` to scrub back through the entire execution.
This is the fastest way to get a full recording: press `r`, then rewind to
wherever you want to inspect.

### Step Over

Press **`n`** to "step over" a function call. This executes instructions until
the callstack returns to the same depth as when you pressed `n`. In practice:

- If the current instruction is about to call a function, `n` will run through
  the entire function body and stop when it returns.
- If you're not at a function call, `n` behaves like a single step forward.

Every instruction executed during step-over is recorded, so you can rewind into
the function body afterwards if you need to.

### Reading the Heap/GC Bar

Below the output area, the debugger shows a **heap/GC status bar** with live
memory information:

```
 Heap: 12 live / 64 slots (52 free) │ Stack: 7 │ GC next@128 base=5000 │ Calls: 2
```

During play mode a speed indicator is appended:

```
 Heap: 12 live / 64 slots (52 free) │ Stack: 7 │ GC next@128 base=5000 │ Calls: 2 │ Speed: 100ms
```

**Field meanings:**

| Field | Meaning |
|---|---|
| `Heap: N live` | Number of live (allocated) objects on the heap right now. |
| `/ M slots` | Total heap capacity — the heap array length. |
| `(K free)` | Number of slots on the free list, ready for reuse without growing the heap. |
| `Stack: N` | Current value-stack depth (number of entries). |
| `GC next@T` | The garbage collector will trigger its next collection when live objects reach this threshold. |
| `base=B` | The adaptive base threshold. The GC adjusts this between 5 000 and 1 000 000 depending on allocation pressure. |
| `LOCKED` | Shown when the GC is inhibited (lock depth > 0). Some native operations temporarily lock the GC to prevent collection mid-operation. If you see this stuck on, something may be wrong. |
| `Calls: D` | Current callstack depth (number of active function frames). |
| `Speed: Xms` | Only shown during play mode — the delay between auto-steps. |

**How to use it for memory debugging:**

- **Watch `live` climb**: If live objects grow without bound as your program
  runs, you have a leak. Use `↑` to scrub back and find the instruction where
  objects start accumulating.
- **Check `free`**: A high free count relative to live count means many objects
  were allocated and freed — normal for short-lived temporaries. A very low
  free count means the heap is nearly full and the next allocation will grow it.
- **GC `LOCKED` staying on**: If the GC stays locked for many steps, a native
  operation may be holding the lock too long, preventing collection.
- **`next@` vs `live`**: When live approaches `next@`, a GC cycle is imminent.
  Step forward and watch the live count drop after collection.
- **`base` changes**: The base threshold adapts over time. If it's growing
  toward 1 000 000, your program allocates heavily and the GC is backing off
  to avoid constant collection.

### Reading the Stack

The stack column shows every value currently on the VM stack:

```
─ Stack ─
 (5 entries, offset=2)
> [  4] Int(42) (result)
  [  3] Int(7) (y)
• [  2] Int(6) (x)
  [  1] Fn@30 (add)
  [  0] Fn@0 (main)
```

- **Index** `[N]` — the absolute position on the stack. Slot 0 is the bottom.
- **Value** — the type and value: `Int(42)`, `Float(3.14)`, `Bool(true)`,
  `Fn@30` (function at bytecode address 30), `Obj#5` (heap object #5).
- **Name** — if the slot corresponds to a known global or local variable,
  the name is shown in parentheses: `(x)`, `(main)`, `(add)`.
- **`>`** — marks the top of the stack (the value that will be used next).
- **`•`** — marks entries that belong to the current local frame
  (at or above the frame offset). These are your local variables.
- **Colors**: green = top-of-stack, white = local frame, grey = below offset.

### Reading Variables

The variables column shows the same information as the stack, but in
human-readable form:

```
─ Variables ─
 locals:
  x = 6
  y = 7
  result = 42
 globals:
  counter = 10
```

- **Locals** are variables in the current function scope.
- **Globals** that hold non-function values are shown separately (function
  globals like `main` or `add` are omitted since they're just code pointers).

### Reading Bytecode

The bytecode column shows the compiled instructions:

```
    0 ALLOCGLOBAL  +3
    3 INTEGER      42
   12 STOREGLOBAL  my_var
>  13 GETGLOBAL    my_var
   14 CALL
```

- **Address** — the byte offset in the program. Instructions vary in length
  (1–9 bytes), so addresses aren't sequential.
- **Opcode** — the instruction name: `STOREGLOBAL`, `CALL`, `JMP`, etc.
- **Operand** — resolved where possible. Instead of `global[2]`, you'll see
  the actual variable name like `my_var`. Instead of `native[1234]`, you'll
  see the native function name like `println`. String literals show their
  content (with control characters escaped).

---

## 4. Third Line: `nova dis` — The Disassembler

The disassembler shows you the compiled bytecode of your program without
running it. This is useful when you want to understand what the compiler
generated, verify that the optimizer is working, or trace control flow.

### Launching

```bash
nova dis my_program.nv         # local file
nova dis                        # project mode (disassembles main.nv)
nova dis --git owner/repo/f.nv  # from GitHub
```

### Reading the Output

The output is a color-coded instruction listing:

```
     0  ALLOCGLOBAL   +3           ; reserve 3 global slots
     1  FUNCTION      +25          ; ── begin function body ──
   │ 2  ALLOCLOCALS   +2
   │ 3  GET           x
   │ 4  GET           y
   │ 5  ADD
   │ 6  RET           (val)
     7  STOREGLOBAL   add          ; ── end function body ──
     8  INTEGER       42
     9  STOREGLOBAL   answer
    10  GETGLOBAL     add
    11  INTEGER       3
    12  INTEGER       7
    13  CALL
```

**Color categories:**

| Color | Category | Examples |
|---|---|---|
| Green | Memory | `STORE`, `GET`, `STOREGLOBAL`, `GETGLOBAL`, `ALLOCLOCALS` |
| Yellow | Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `NEG` |
| Red | Control flow | `JMP`, `JUMPIFFALSE`, `BJMP`, `CALL`, `RET` |
| Magenta | Comparisons | `EQ`, `NEQ`, `LT`, `GT`, `LTE`, `GTE` |
| Blue | I/O | `NATIVE` (print, println, etc.) |
| White | Stack ops | `POP`, `DUP`, `SWAP`, `CLONE` |
| Cyan | Data | `INTEGER`, `FLOAT`, `STRING`, `BOOL`, `CHAR` |

### Flow Arrows

Jump instructions are visualized with ASCII arrows in the left margin:

```
  ┌──  10  JUMPIFFALSE   +5       (if condition is false, skip to 15)
  │    11  INTEGER       1
  │    12  STOREGLOBAL   x
  │    13  JMP           +2        (skip else branch)
  └──  15  INTEGER       2
       16  STOREGLOBAL   x
```

Arrow colors indicate the type of jump:

| Color | Meaning |
|---|---|
| **Magenta** | Backward jump (loop — jumps to an earlier address) |
| **Yellow** | Conditional jump (`JUMPIFFALSE`) |
| **Cyan** | Forward jump (unconditional skip) |

Multiple arrows are assigned to separate columns so they don't overlap. This
makes nested loops and complex control flow readable at a glance.

### Function Nesting

Function and closure bodies are indented with `│` markers to show nesting:

```
     0  FUNCTION      +20
   │ 1  ALLOCLOCALS   +3
   │ 2  CLOSURE       +8
   ││3  GET           x
   ││4  GET           captured
   ││5  ADD
   ││6  RET           (val)
   │ 7  STORE         callback
   │ 8  RET           (val)
     9  STOREGLOBAL   make_adder
```

Each `│` represents one level of function nesting. This makes it easy to see
where closures begin and end.

### Summary Section

At the bottom, the disassembler prints:

- **Instruction count** — total number of instructions.
- **Globals table** — a map of global slot numbers to their names:
  ```
  Globals:
    [0] main
    [1] add
    [2] answer
  ```
- **Reading guide** — a legend explaining the arrow colors and categories.

---

## 5. Debugging Strategies

### Strategy 1: Narrow It Down

Don't debug your entire program at once. **Reduce the problem:**

1. **Comment out** sections until the bug disappears.
2. **Extract** the problematic code into a tiny test file.
3. **Hardcode** inputs instead of computing them.
4. Debug the minimal case, then apply the fix to your real program.

```rust
// debug_test.nv — minimal reproduction
module main

fn broken(x: Int) -> Int {
    return x * 2 + 1   // is this really returning what I expect?
}

println(Cast::string(broken(5)))  // should print 11
```

```bash
nova dbg debug_test.nv
```

### Strategy 2: Watch the Stack

If a function returns the wrong value, step through it in the debugger and
watch the stack column:

1. Open `nova dbg`.
2. Step to just before the function call.
3. Note the stack contents.
4. Step into the function (just press `↓`).
5. Watch each instruction push/pop values.
6. When you see a wrong value appear, you've found the bug.

The stack always tells the truth — it shows exactly what the VM is computing.

### Strategy 3: Track Variables

Use the **Variables column** (right panel) to monitor named locals:

1. Step into the function of interest.
2. Watch the `locals:` section update after each instruction.
3. Compare against what you expect at each step.

This is especially useful for loop bugs — you can watch a counter or
accumulator change on every iteration.

### Strategy 4: Bisect with Play Mode

For bugs that only appear after many steps:

1. Press **`r`** to run the program to completion.
2. Check the output — is it wrong?
3. Press **`Home`** to go back to the start.
4. Press **`p`** to play from the beginning, with speed set fast (`+` a few times).
5. Watch the variables and stack scroll by.
6. When you see something wrong, press **`p`** to pause.
7. Use **`↑`** and **`↓`** to step around the exact moment things go wrong.

**Alternative:** If you know roughly when the bug happens (e.g. after step 500),
use `PgDn` to skip forward quickly, then fine-tune with `↑`/`↓`.

### Strategy 5: Check the Disassembly

If the debugger shows correct logic but wrong results, the compiler may have
generated unexpected code. Use `nova dis` to check:

```bash
nova dis my_program.nv | less
```

Look for:
- **Missing instructions** — did the optimizer remove something it shouldn't have?
- **Wrong jump targets** — does a loop jump to the right address?
- **Wrong variable slots** — does `STORE x` and `GET x` refer to the same slot?
- **Function boundaries** — are `FUNCTION`/`RET` pairs matched correctly?

---

## 6. Common Mistakes and How to Spot Them

### Forgetting `return`

```rust
fn add(a: Int, b: Int) -> Int {
    let result = a + b
    // forgot: return result
}
```

**Symptom in debugger:** The function body executes but `RET` pops no value
off the stack (you'll see `RET` without `(val)`).

**Fix:** The type checker usually catches this, but if you're using `todo()` or
`unreachable()` as placeholders, double-check that every branch has a `return`.

### Modifying a shared reference

```rust
let a = [1, 2, 3]
let b = a          // b is NOT a copy — it's the same list
b.push(4)          // this also changes a!
```

**Symptom in debugger:** The Variables column shows `a` and `b` pointing to
the same `Obj#N`. When `b` changes, `a` changes too.

**Fix:** Use `clone(a)` to get an independent copy.

### Off-by-one in loops

```rust
let xs = [10, 20, 30]
for i = 0; i <= xs.len(); i = i + 1 {   // BUG: should be <, not <=
    println(Cast::string(xs[i]))
}
```

**Symptom in debugger:** The loop body executes one extra time. On the last
iteration, the index goes out of bounds.

**Fix:** Use `<` instead of `<=`, or better yet, use `for x in xs { ... }`.

### Wrong variable shadowing

```rust
let x = 10
fn foo() -> Int {
    let x = 20      // this is a NEW local x, not the global
    return x
}
```

**Symptom in debugger:** The Stack column shows two different `x` values at
different stack positions. The Variables column makes it clear which is which.

### Forgetting `module main`

Every Nova file needs a module declaration at the top. Without it, the
compiler will complain. This one is always caught by `nova check`.

### Passing wrong types to generic functions

```rust
let result = someGenericFn(42) @[T: String]  // T is String, but 42 is Int
```

**Symptom:** The type checker catches this at compile time.

### Using `None` without a type

```rust
let x = None    // ERROR: None needs a type — None(Int), None(String), etc.
```

**Fix:** Always specify the inner type: `None(Int)`, `None(String)`.

---

## 7. Pitfalls

### Pitfall: Bytecode addresses aren't line numbers

In the debugger, the left column shows bytecode addresses, not source line
numbers. A single line of Nova code may compile to many instructions. Don't
expect a 1:1 mapping.

**Tip:** Use the opcode names and operand labels to orient yourself. If you see
`STOREGLOBAL  my_var`, you know that's the assignment to `my_var`, regardless
of the address.

### Pitfall: The stack is the source of truth

The Variables column is a convenience view derived from the stack. If something
looks wrong in Variables but right in the Stack (or vice versa), trust the
Stack — it's the raw VM state.

### Pitfall: `Obj#N` tells you nothing about the value

Heap objects (structs, lists, strings stored as objects) are shown as `Obj#5`
on the stack. You can't see their contents directly in the debugger stack
column. Use the Variables column, which resolves named locals to their
display-formatted values. The **Heap/GC bar** shows aggregate stats — how many
live objects exist and when the next collection will trigger — so you can
correlate `Obj#N` references with overall heap activity.

### Pitfall: Play mode speed affects nothing but display

Play mode doesn't change execution speed or behavior. It's purely a display
feature — the program runs the same way whether you step manually or use play.

### Pitfall: Large programs and memory

The debugger records a snapshot at every step. For programs that run millions
of instructions, this can use significant memory. Use **`r`** (run to end)
judiciously — if you only need to debug a specific section, step to it manually
or use **`n`** (step over) to skip past function calls you don't care about.

### Pitfall: Closures capture by reference

Variables captured by closures are shared, not copied. If you modify a captured
variable after creating the closure, the closure sees the new value.

```rust
let x = 10
let f = fn() -> Int { return x }
x = 20
println(Cast::string(f()))  // prints 20, not 10
```

**In the debugger:** You'll see the closure's captured reference update when
`x` is reassigned.

---

## 8. Tips and Tricks

### Tip 1: Use `check` before `dbg`

Always run `nova check` first. If there are type errors, fix them before
opening the debugger. The debugger can't help with compile-time errors.

### Tip 2: Start small

Don't try to debug a 500-line program by stepping from instruction 0. Extract
the broken part into a small test file and debug that.

### Tip 3: Use `r` then rewind

For most bugs, the fastest workflow is:

1. `nova dbg file.nv`
2. Press `r` to run everything.
3. Check if the output is wrong.
4. Press `Home` to go to the start.
5. Use `PgDn` to jump forward in large steps.
6. Use `↑`/`↓` to find the exact instruction where things go wrong.

### Tip 4: Use `dis` to verify function structure

If a function isn't being called or returns the wrong thing, disassemble and
look for:
- Is the function body between `FUNCTION` and `RET`?
- Is `STOREGLOBAL` storing it to the right name?
- Is `CALL` or `DCALL` actually calling the right function?

### Tip 5: Print debugging still works

Sometimes the easiest approach is to add `println(...)` calls at key points:

```rust
println("before loop: x = " + Cast::string(x))
for i = 0; i < 10; i = i + 1 {
    println("  i = " + Cast::string(i) + ", sum = " + Cast::string(sum))
    sum = sum + i
}
println("after loop: sum = " + Cast::string(sum))
```

The debugger's Output section captures these prints, so you can use both
approaches simultaneously.

### Tip 6: Use `nova test` for regression prevention

Once you've fixed a bug, write a test file so it doesn't come back:

```bash
# tests/test_my_fix.nv
module main

fn broken_function(x: Int) -> Int {
    return x * 2 + 1
}

assert(broken_function(5) == 11)
assert(broken_function(0) == 1)
assert(broken_function(-3) == -5)
```

```bash
nova test
```

### Tip 7: The debugger works on remote files

You can debug code from GitHub without downloading it:

```bash
nova dbg --git pyrotek45/nova-lang/demo/fib.nv
```

This is handy for verifying that a library works correctly before installing it.

### Tip 8: Speed controls have two gears

The `+`/`-` speed controls adjust by 10ms when the speed is under 100ms,
and by 50ms when it's over 100ms. This gives you fine control at fast speeds
and large jumps at slow speeds. The range is 10ms to 2000ms.

### Tip 9: Step over for library code

When stepping through code that calls library functions, use `n` (step over)
to skip through the library internals. You'll see the result without having to
step through every instruction in the library function.

### Tip 10: Pipe into `less` for disassembly

The disassembler output can be long. Pipe it into a pager:

```bash
nova dis my_program.nv | less -R    # -R preserves ANSI colors
```

Or redirect to a file to search through later:

```bash
nova dis my_program.nv > dis_output.txt
```

(Note: the file version won't have colors since ANSI codes are stripped when
output isn't a terminal.)

---

## 9. Quick Reference Card

```
┌─────────────────────────────────────────────────────────┐
│                  NOVA DEBUGGING CHEAT SHEET              │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  CHECK     nova check file.nv     type errors           │
│  DEBUG     nova dbg file.nv       step debugger (TUI)   │
│  DISASM    nova dis file.nv       bytecode listing       │
│                                                         │
│  DEBUGGER KEYS                                          │
│  ──────────────                                         │
│  ↓ / j / Space    step forward                          │
│  ↑ / k            step backward                         │
│  u / d            scroll stack view up / down            │
│  PgDn / PgUp      jump 20 steps                         │
│  Home / End        first / latest step                   │
│  Tab               toggle stack / heap inspect           │
│  p                 play / pause                          │
│  + / =             faster                                │
│  - / _             slower                                │
│  r                 run to end (records all steps)        │
│  n                 step over (skip function body)        │
│  :                 command mode                           │
│  ?                 help                                  │
│  q / Esc           quit                                  │
│                                                         │
│  COMMANDS (press : first)                               │
│  ─────────────────────────                              │
│  :goto <addr>     jump to bytecode address              │
│  :step <n>        execute n steps forward                │
│  :find <text>     search bytecode for text              │
│  :speed <ms>      set playback speed (1-10000)          │
│  :heap / :stack   switch view                            │
│  :help / :quit    help / quit                            │
│                                                         │
│  DEBUGGER COLUMNS                                       │
│  ─────────────────                                      │
│  Left     Bytecode (> = current instruction)            │
│  Middle   Stack    (> = TOS, • = local frame)           │
│  Right    Variables (locals + globals)                   │
│                                                         │
│  HEAP/GC BAR                                            │
│  ────────────                                           │
│  live       allocated objects on the heap               │
│  slots      total heap capacity                         │
│  free       free-list slots ready for reuse             │
│  Stack      value-stack depth                           │
│  GC next@   collection threshold                        │
│  base=      adaptive base threshold (5K–1M)             │
│  LOCKED     GC inhibited (lock depth > 0)               │
│  Calls      callstack depth                             │
│  Speed      play mode delay (shown while playing)       │
│                                                         │
│  DISASSEMBLER ARROWS                                    │
│  ────────────────────                                   │
│  Magenta   backward jump (loop)                         │
│  Yellow    conditional jump (if/else)                    │
│  Cyan      forward jump (skip)                          │
│                                                         │
│  WORKFLOW                                               │
│  ────────                                               │
│  1. nova check     — fix type errors first              │
│  2. nova dbg       — step through logic                 │
│  3. nova dis       — inspect compiled output            │
│  4. nova test      — prevent regressions                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```
