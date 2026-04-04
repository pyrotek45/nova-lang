# Getting Started with Nova

This guide walks you through creating, structuring, and running a Nova project
from scratch. By the end you'll know how to use `nova init`, organise your
code with modules, pull in libraries, and run tests.

---

## Table of Contents

1. [Your First Project](#1-your-first-project)
2. [Project Structure](#2-project-structure)
3. [Running Your Code](#3-running-your-code)
4. [Adding Modules](#4-adding-modules)
5. [Importing Modules](#5-importing-modules)
6. [Using the Standard Library](#6-using-the-standard-library)
7. [Fetching Libraries from GitHub](#7-fetching-libraries-from-github)
8. [Writing Tests](#8-writing-tests)
9. [Running Tests](#9-running-tests)
10. [GitHub Imports in Source](#10-github-imports-in-source)
11. [Command-Line Arguments](#11-command-line-arguments)
12. [Common Errors and Fixes](#12-common-errors-and-fixes)
13. [What Next?](#13-what-next)

---

## 1. Your First Project

Create a new project with `nova init`:

```bash
nova init myapp
```

Output:

```
Created myapp/main.nv
Created myapp/libs/
Created myapp/tests/test_example.nv

Project 'myapp' is ready!
  cd myapp
  nova run
  nova test
```

That's it — you have a runnable project. Let's look at what it created.

---

## 2. Project Structure

A Nova project is any folder that contains a `main.nv` file. The `nova init`
command sets up the conventional layout:

```
myapp/
    main.nv              ← entry point (must declare: module main)
    libs/                ← shared modules and external dependencies
    tests/               ← test files (run with: nova test)
        test_example.nv  ← starter test
```

### What each piece does

| Path | Purpose |
|---|---|
| `main.nv` | The entry point. Nova runs this file when you type `nova run` from inside the project. |
| `libs/` | Put your shared modules here. Import them from `main.nv` with `import libs.modulename`. |
| `tests/` | Put test files here. Files must be named `test_*.nv`. Run them all with `nova test`. |

There is no config file, no `package.json`, no `Cargo.toml` — Nova uses a
simple convention: if a folder has a `main.nv`, it's a project.

### The generated `main.nv`

```nova
module main

fn main() {
    println("Hello from myapp!")
}

main()
```

Every Nova file starts with `module <name>`. The module name must be unique
across all files in the project — Nova uses it to prevent duplicate imports.

---

## 3. Running Your Code

### Run a project

From inside the project folder:

```bash
cd myapp
nova run
```

```
(detected Nova project: running main.nv)
Hello from myapp!
```

Nova looks for `main.nv` in the current directory and runs it automatically.

### Run a specific file

```bash
nova run path/to/file.nv
```

### Other commands

| Command | What it does |
|---|---|
| `nova run` | Compile and run (`main.nv` or specified file) |
| `nova check file.nv` | Type-check without running — catches errors fast |
| `nova time file.nv` | Run and print execution time |
| `nova dis file.nv` | Show the compiled bytecode |
| `nova dbg file.nv` | Run in debug mode |
| `nova install name repo/path` | Install a library into `libs/<name>/` |
| `nova remove name` | Remove a library from `libs/<name>/` |
| `nova repl` | Interactive REPL |
| `nova help` | Show all commands |

All commands that accept a file also accept `--git` to fetch from GitHub:

```bash
nova check --git pyrotek45/nova-lang/demo/fib.nv
nova time  --git pyrotek45/nova-lang/demo/fib.nv
nova dis   --git pyrotek45/nova-lang/demo/fib.nv
nova dbg   --git pyrotek45/nova-lang/demo/fib.nv
```

If you omit both `--git` and a file path, Nova auto-detects `main.nv` in the
current directory (project mode).

---

## 4. Adding Modules

As your project grows, split your code into modules inside `libs/`.

**Create `libs/math_utils.nv`:**

```nova
module math_utils

fn square(x: Int) -> Int {
    return x * x
}

fn clamp(x: Int, lo: Int, hi: Int) -> Int {
    if x < lo { return lo }
    if x > hi { return hi }
    return x
}
```

**Create `libs/greetings.nv`:**

```nova
module greetings

fn hello(name: String) -> String {
    return "Hello, " + name + "!"
}
```

Your project now looks like:

```
myapp/
    main.nv
    libs/
        math_utils.nv
        greetings.nv
    tests/
        test_example.nv
```

---

## 5. Importing Modules

### From `main.nv`

Import files from `libs/` using dot-path syntax:

```nova
module main
import libs.math_utils
import libs.greetings

println(square(5))             // 25
println(clamp(15, 0, 10))     // 10
println(hello("Nova"))         // Hello, Nova!
```

All imported functions, structs, and enums are available directly by name —
Nova flattens imports into the caller's scope (no prefix needed).

### How import paths work

Each `.` becomes a `/`, and `.nv` is appended automatically. Paths are
always **relative to the file containing the import statement**.

| Import statement | Resolved path (relative to importing file) |
|---|---|
| `import libs.math_utils` | `./libs/math_utils.nv` |
| `import helper` | `./helper.nv` |
| `import super.std.core` | `../std/core.nv` |
| `import super.super.std.list` | `../../std/list.nv` |

### The `super` keyword

`super` means "go up one directory" (like `..`). Use it when the file you
want is in a parent or sibling directory:

```nova
// From libs/math_utils.nv, import another file in libs/:
import helper              // → ./helper.nv (same directory)

// From libs/math_utils.nv, import from a sibling folder:
import super.data.records  // → ../data/records.nv
```

### String literal imports

If you prefer, you can use a string literal as the path:

```nova
import "libs/math_utils.nv"     // same as: import libs.math_utils
import "../std/core.nv"         // same as: import super.std.core
```

### Importing from deeper folders

If you have a nested file like `libs/utils/strings.nv`:

```nova
// From main.nv:
import libs.utils.strings

// From libs/math_utils.nv:
import utils.strings            // relative to libs/
```

---

## 6. Using the Standard Library

Nova ships with a standard library in the `std/` folder at the repository root.
To use it, the `std/` folder must be reachable from your source files via
relative `super` imports.

### If your project is inside the nova-lang repo

```
nova-lang/
    std/          ← standard library
    myapp/        ← your project
        main.nv
```

```nova
module main
import super.std.core      // → ../std/core.nv
import super.std.list      // → ../std/list.nv

let items = [3, 1, 4, 1, 5]
let sorted = items.sortWith(): |a: Int, b: Int| a < b
println(sorted)  // [1, 1, 3, 4, 5]
```

### If your project is standalone

Use `nova init --with` to fetch the standard library into your `libs/` folder:

```bash
nova init myapp --with pyrotek45/nova-lang/std
```

This downloads all `.nv` files from the `std/` folder on GitHub into `libs/`.
Then import locally:

```nova
module main
import libs.core
import libs.list
```

### Key standard library modules

| Module | What it gives you |
|---|---|
| `core` | `Box`, `range()`, `Maybe`, `Result`, Option helpers |
| `list` | `map`, `filter`, `reduce`, `sortWith`, `flatten`, `zip` |
| `iter` | Lazy iterators — `fromVec`, `map`, `filter`, `collect` |
| `string` | `split`, `padLeft`, `padRight`, `capitalize`, `lines` |
| `math` | `sqrt`, `pow`, `abs`, `floor`, trig functions |
| `io` | `prompt()`, `readLines()`, `writeLines()` |
| `hashmap` | O(1) `HashMap` |
| `grid` | Generic 2D grid — `Grid(T)` |
| `tui` | Terminal UI helpers |

See the full list in the [Reference](reference.md#3-standard-library).

---

## 7. Fetching Libraries from GitHub

### During project creation

Use `--with` to fetch an entire folder from a GitHub repository into `libs/`:

```bash
nova init mygame --with pyrotek45/nova-lang/std
```

The path format is `owner/repo/folder`. Nova uses the GitHub Contents API
to list all `.nv` files in that folder and downloads them.

You can use multiple `--with` flags:

```bash
nova init mygame --with pyrotek45/nova-lang/std --with myuser/myrepo/utils
```

Each `--with` can point to a different repository and folder. This lets you
pull libraries from multiple sources in one command.

> **⚠ Filename collisions:** All fetched files go into the same `libs/` folder.
> If two `--with` sources contain a file with the same name (e.g. both have
> `math.nv`), the second download will overwrite the first. Rename files if
> this happens.

### How it works

1. Nova calls the GitHub API to list files in the folder
2. Every `.nv` file is downloaded into `libs/`
3. You import them locally: `import libs.core`, `import libs.list`, etc.

### Installing libraries into an existing project

If you already have a project, use `nova install` to add libraries:

```bash
cd myapp
nova install std pyrotek45/nova-lang/std
```

This creates `libs/std/` and downloads all `.nv` files into it. Import with:

```nova
import libs.std.core
import libs.std.math
```

To remove a library:

```bash
nova remove std
```

### Running code from GitHub

You can also run a file directly from GitHub without downloading it:

```bash
nova run --git pyrotek45/nova-lang/demo/fib.nv
```

Or from a specific commit:

```bash
nova run --git pyrotek45/nova-lang/demo/fib.nv a1b2c3d
```

The `--git` flag works with all file-based commands:

```bash
nova check --git pyrotek45/nova-lang/demo/fib.nv   # type-check only
nova time  --git pyrotek45/nova-lang/demo/fib.nv   # run with timing
nova dis   --git pyrotek45/nova-lang/demo/fib.nv   # disassemble bytecode
nova dbg   --git pyrotek45/nova-lang/demo/fib.nv   # debug run
```

---

## 8. Writing Tests

### Test file convention

Test files live in the `tests/` directory and must be named `test_*.nv`:

```
myapp/
    tests/
        test_example.nv      ✓ will be found
        test_math.nv         ✓ will be found
        helper.nv            ✗ not a test file (no test_ prefix)
```

### Writing a test

A test file is a normal Nova program that uses `assert()` and ends with
a `PASS:` message:

```nova
module test_math
import super.libs.math_utils

// Test square
assert(square(0) == 0, "square of zero")
assert(square(3) == 9, "square of 3")
assert(square(-4) == 16, "square of negative")

// Test clamp
assert(clamp(5, 0, 10) == 5, "clamp in range")
assert(clamp(-1, 0, 10) == 0, "clamp below")
assert(clamp(15, 0, 10) == 10, "clamp above")

println("PASS: test_math")
```

**Important:** The import path from `tests/test_math.nv` to `libs/math_utils.nv`
uses `super` because the test file is one directory below the project root:

```
tests/test_math.nv  →  import super.libs.math_utils
                        (goes up to myapp/, then into libs/)
```

### Assert

`assert(condition, message)` is a built-in function. If the condition is false,
it halts the program with the message and a non-zero exit code.

```nova
assert(1 + 1 == 2, "basic math works")
assert(true != false, "booleans work")
```

### The PASS convention

End every test with `println("PASS: test_name")`. The test runner
(`tests/run_tests.sh`) checks for this output to determine success.
The built-in `nova test` command checks exit codes instead, so `PASS:` output
is optional for `nova test` but recommended for compatibility with both runners.

---

## 9. Running Tests

### With `nova test`

From inside your project folder:

```bash
cd myapp
nova test
```

Output:

```
========================================
  Nova Test Runner
========================================
Running 2 test files from tests/

  ✓ test_example
  ✓ test_math

========================================
  2 passed, 0 failed
========================================

All tests passed! ✓
```

### Specifying a test directory

```bash
nova test path/to/tests
```

### What nova test does

1. Scans the `tests/` directory (or the directory you specify)
2. Finds all files matching `test_*.nv`
3. Compiles and runs each one
4. Reports ✓ for pass (exit code 0) and ✗ for fail (compile/runtime error)
5. Prints a summary and exits with code 1 if any test failed

### Running the full test suite runner

The repository also includes a bash-based test runner that checks both
positive tests and rejection tests:

```bash
bash tests/run_tests.sh
```

---

## 10. GitHub Imports in Source

You can import a file directly from GitHub inside your Nova source code
using `import @`:

```nova
import @ "pyrotek45/nova-lang/std/core.nv"
```

The format is `"owner/repo/path/to/file.nv"`. Nova fetches from the `main`
branch by default.

### Locking to a commit

To ensure your code doesn't break if the repo changes, lock to a specific
commit hash with `!`:

```nova
import @ "pyrotek45/nova-lang/std/core.nv" ! "a1b2c3d4e5f6"
```

### When to use GitHub imports vs local imports

| Situation | Use |
|---|---|
| Prototyping, quick scripts | `import @` — no setup needed |
| New projects | `nova init --with` then `import libs.module` |
| Existing projects | `nova install name repo/path` then `import libs.name.module` |
| Reproducible builds | `import @` with `! "commit_hash"` |

> **Tip:** `import @` requires network access. For offline work, use
> `nova init --with` or `nova install` to download files locally.

---

## 11. Command-Line Arguments

Nova programs can receive command-line arguments using `terminal::args()`.

### Basic usage

```nova
module main

let args = terminal::args()   // Option([String])

if args.isSome() {
    let arglist = args.unwrap()
    for a in arglist {
        println("arg: " + a)
    }
} else {
    println("No arguments")
}
```

Run it:

```bash
nova run myapp.nv hello world 42
```

```
arg: hello
arg: world
arg: 42
```

### How it works

- `terminal::args()` returns `Option([String])`
- It gives you everything after `nova run file.nv` — your arguments only
- Returns `None` when no extra arguments are passed
- Quoted strings are preserved: `"foo bar"` becomes one argument

### Using `if let` for cleaner code

```nova
if let arglist = terminal::args() {
    println("First arg: " + arglist[0])
} else {
    println("Usage: nova run myapp.nv <name>")
}
```

### Parsing argument types

Arguments arrive as strings. Use `Cast::int` or `Cast::float` to convert:

```nova
if let arglist = terminal::args() {
    if let n = Cast::int(arglist[0]) {
        println("Got number: " + Cast::string(n))
    } else {
        println("Not a number: " + arglist[0])
    }
}
```

---

## 12. Common Errors and Fixes

### "No file specified and no main.nv found"

```
Error: no file specified and no main.nv found in the current directory.
  Usage: nova run <file.nv>
  Or create a project: nova init myproject
```

**Cause:** You ran `nova run` without a file, and there's no `main.nv` in the
current directory.

**Fix:** Either specify a file (`nova run myfile.nv`) or `cd` into a project
folder that has a `main.nv`.

---

### "Error Importing file"

```
Error Importing file
help: Could not find file: ./libs/missing.nv
  Import paths are relative to the current file's directory.
  Use `super` to go up a directory: import super.folder.module
  Check that the file exists and the path is spelled correctly.
```

**Cause:** The import path doesn't match any file on disk.

**Fix:**
- Check spelling: `import libs.math_utils` looks for `./libs/math_utils.nv`
- Remember that paths are relative to the *importing* file, not the project root
- If importing from `tests/`, you need `super`: `import super.libs.module`

---

### "Invalid GitHub path"

```
Invalid GitHub path
expected "owner/repo/path/to/file.nv", got "badpath"
```

**Cause:** The GitHub path needs at least three segments separated by `/`.

**Fix:** Use the format `owner/repo/path/to/file.nv`:
```nova
import @ "pyrotek45/nova-lang/std/core.nv"
```

---

### "GitHub import: could not fetch file"

```
Error: https://raw.githubusercontent.com/owner/repo/main/file.nv: status code 404
```

**Cause:** The file doesn't exist at that URL. Either the repo is private,
the path is wrong, or the commit hash doesn't exist.

**Fix:**
- Check that the repository is public
- Verify the file path exists in the repo
- If using `! "hash"`, make sure the commit hash is correct

---

### "directory already exists"

```
Error: directory 'myapp' already exists.
```

**Cause:** You ran `nova init myapp` but a folder called `myapp` already exists.

**Fix:** Choose a different name, or delete the existing folder first.

---

### Unknown command

```
Error: unknown command 'rn'.
  Run 'nova help' to see available commands.
```

**Cause:** Typo in the command name.

**Fix:** Check the command name. Available commands: `run`, `check`, `time`,
`dis`, `dbg`, `init`, `test`, `repl`, `help`.

---

## 13. What Next?

- **Learn the language:** Read the [Tutorial](tutorial.md) for a complete guide
  from basics to advanced patterns
- **Explore the standard library:** See the [Reference](reference.md) for API
  documentation
- **Try the demos:** Run `nova run demo/fib.nv` or explore `demo/` and `games/`
- **Start the REPL:** Type `nova repl` for interactive experimentation
- **Build a game:** Part II of the Tutorial covers game development with raylib
