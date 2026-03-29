# Building Terminal Apps with Nova

Nova includes a built-in `terminal` module for creating interactive terminal applications —
everything from simple prompts to full-screen TUI apps with colours, cursor control, and
keyboard input.

---

## Table of Contents

1. [Quick Start](#1-quick-start)
2. [Terminal Functions Reference](#2-terminal-functions-reference)
3. [Raw Mode and Key Input](#3-raw-mode-and-key-input)
4. [Colours and Styling](#4-colours-and-styling)
5. [Cursor Movement and Screen Layout](#5-cursor-movement-and-screen-layout)
6. [Building a Menu System](#6-building-a-menu-system)
7. [Building a Game Loop](#7-building-a-game-loop)
8. [Patterns and Best Practices](#8-patterns-and-best-practices)
9. [Common Pitfalls](#9-common-pitfalls)

---

## 1. Quick Start

The terminal functions are available without any imports:

```nova
module hello

terminal::clearScreen()
terminal::moveTo(0, 0)
terminal::print("Hello from Nova!")
terminal::flush()
sleep(2000)
```

For interactive apps, you'll typically use raw mode:

```nova
module interactive

terminal::rawmode(true)
terminal::hideCursor()
terminal::clearScreen()

terminal::moveTo(10, 5)
terminal::print("Press 'q' to quit")
terminal::flush()

let running = true
while running {
    if let ch = terminal::rawread(100) {
        if ch == 'q' { running = false }
    }
}

terminal::showCursor()
terminal::rawmode(false)
terminal::clearScreen()
```

---

## 2. Terminal Functions Reference

| Function | Description |
|---|---|
| `terminal::clearScreen()` | Clear the entire screen. |
| `terminal::hideCursor()` | Hide the blinking cursor. |
| `terminal::showCursor()` | Restore the cursor. |
| `terminal::rawmode(Bool)` | Enable/disable raw mode (no line buffering, no echo). |
| `terminal::getch() -> Option(Char)` | Read a single character (blocking). |
| `terminal::rawread(Int) -> Option(Char)` | Read a char with timeout in milliseconds. Returns `None` on timeout. |
| `terminal::moveTo(Int, Int)` | Move cursor to (column, row), 0-based. |
| `terminal::getSize() -> (Int, Int)` | Get terminal size as (width, height). |
| `terminal::setForeground(Int, Int, Int)` | Set text colour (R, G, B). |
| `terminal::setBackground(Int, Int, Int)` | Set background colour (R, G, B). |
| `terminal::resetColor()` | Reset to default colours. |
| `terminal::print(String)` | Write text without newline. |
| `terminal::flush()` | Flush the output buffer. |
| `terminal::enableMouse()` | Enable mouse event capture. |
| `terminal::disableMouse()` | Disable mouse event capture. |
| `terminal::args() -> Option([String])` | Get command-line arguments. |

---

## 3. Raw Mode and Key Input

Normal terminal mode buffers input until Enter is pressed. **Raw mode** delivers each
keypress immediately, without echoing it to the screen. This is essential for interactive
apps.

```nova
terminal::rawmode(true)    // enter raw mode

// rawread returns None if no key within the timeout
if let ch = terminal::rawread(50) {
    // ch is a Char
    if ch == 'w' { /* handle up */ }
    if ch == 's' { /* handle down */ }
}

terminal::rawmode(false)   // always restore on exit
```

**Important:** Always restore normal mode before exiting. If your program crashes in raw
mode, the terminal may be left in a broken state. Use `terminal::rawmode(false)` in your
cleanup path.

### Reading arrow keys

Arrow keys produce escape sequences. `rawread` returns the first character of the sequence.
You may need to read multiple characters:

```nova
if let ch = terminal::rawread(50) {
    if Cast::charToInt(ch) == 27 {   // ESC byte
        if let ch2 = terminal::rawread(50) {
            if ch2 == '[' {
                if let ch3 = terminal::rawread(50) {
                    if ch3 == 'A' { /* UP */ }
                    if ch3 == 'B' { /* DOWN */ }
                    if ch3 == 'C' { /* RIGHT */ }
                    if ch3 == 'D' { /* LEFT */ }
                }
            }
        }
    }
}
```

---

## 4. Colours and Styling

Set foreground and background colours using RGB values (0–255):

```nova
terminal::setForeground(255, 0, 0)      // red text
terminal::print("Error!")
terminal::resetColor()

terminal::setForeground(0, 255, 0)      // green text
terminal::setBackground(0, 0, 0)        // black background
terminal::print("Success")
terminal::resetColor()
terminal::flush()
```

### Colour helper pattern

Define colour constants for consistency:

```nova
fn setRed()   { terminal::setForeground(255, 80, 80) }
fn setGreen() { terminal::setForeground(80, 255, 80) }
fn setBlue()  { terminal::setForeground(80, 80, 255) }
fn setWhite() { terminal::setForeground(255, 255, 255) }
fn setGray()  { terminal::setForeground(128, 128, 128) }
```

---

## 5. Cursor Movement and Screen Layout

Use `moveTo(col, row)` to position text anywhere on screen. Coordinates are 0-based:

```nova
let size = terminal::getSize()
let width = size[0]
let height = size[1]

// Draw a border
for let x = 0; x < width; x += 1 {
    terminal::moveTo(x, 0)
    terminal::print("-")
    terminal::moveTo(x, height - 1)
    terminal::print("-")
}

for let y = 0; y < height; y += 1 {
    terminal::moveTo(0, y)
    terminal::print("|")
    terminal::moveTo(width - 1, y)
    terminal::print("|")
}

terminal::flush()
```

### Centering text

```nova
fn printCentered(text: String, row: Int) {
    let size = terminal::getSize()
    let col = (size[0] - strlen(text)) / 2
    terminal::moveTo(col, row)
    terminal::print(text)
}

printCentered("Welcome to My App", 10)
terminal::flush()
```

---

## 6. Building a Menu System

A simple numbered menu:

```nova
module menu_app

fn showMenu(options: [String]) -> Int {
    terminal::clearScreen()
    terminal::moveTo(0, 0)

    terminal::setForeground(0, 200, 255)
    terminal::print("=== Main Menu ===\n")
    terminal::resetColor()

    for let i = 0; i < options.len(); i += 1 {
        terminal::print("  " + Cast::string(i + 1) + ") " + options[i] + "\n")
    }

    terminal::print("\nChoose: ")
    terminal::flush()

    let input = readln()
    if let n = Cast::int(input) {
        if n >= 1 && n <= options.len() {
            return n
        }
    }
    return -1
}

let choice = showMenu(["New Game", "Load Game", "Settings", "Quit"])

if choice == 1 { println("Starting new game...") }
elif choice == 4 { println("Goodbye!") }
else { println("You chose option " + Cast::string(choice)) }
```

### Using the std/tui.nv SceneManager

Nova's standard library includes a TUI scene manager:

```nova
module tui_demo

import super.std.tui

struct GameState { score: Int, name: String }

let scenes = []: Menu(GameState)

scenes.push(Menu {
    name: "main",
    screen: fn(state: GameState) -> String {
        return "Welcome, " + state.name + "!\nScore: " + Cast::string(state.score)
    },
    Items: [
        Item { name: "Play", kind: "goto",
               trigger: fn(state: GameState) -> String { return "game" } },
        Item { name: "Quit", kind: "goto",
               trigger: fn(state: GameState) -> String { exit(); return "" } }
    ]
})

let sm = SceneManager {
    currentScene: "main",
    state: GameState { score: 0, name: "Player" },
    scenes: scenes
}

sm.show()
```

---

## 7. Building a Game Loop

For real-time terminal games, use raw mode with a timed read:

```nova
module terminal_game

terminal::rawmode(true)
terminal::hideCursor()

let px = 10
let py = 5
let running = true

while running {
    // Input
    if let ch = terminal::rawread(50) {
        if ch == 'q' { running = false }
        if ch == 'w' { py -= 1 }
        if ch == 's' { py += 1 }
        if ch == 'a' { px -= 1 }
        if ch == 'd' { px += 1 }
    }

    // Clamp to screen
    let size = terminal::getSize()
    if px < 0 { px = 0 }
    if py < 0 { py = 0 }
    if px >= size[0] { px = size[0] - 1 }
    if py >= size[1] { py = size[1] - 1 }

    // Render
    terminal::clearScreen()
    terminal::moveTo(0, 0)
    terminal::setForeground(128, 128, 128)
    terminal::print("WASD to move, Q to quit")

    terminal::moveTo(px, py)
    terminal::setForeground(0, 255, 0)
    terminal::print("@")

    terminal::resetColor()
    terminal::flush()
}

terminal::showCursor()
terminal::rawmode(false)
terminal::clearScreen()
```

### Frame timing

For consistent frame rates, use `now()` and `sleep()`:

```nova
let target_ms = 16   // ~60 FPS
let last = now()

while running {
    let dt = now() - last
    last = now()

    // ... update & render ...

    let elapsed = now() - last
    if elapsed < target_ms {
        sleep(target_ms - elapsed)
    }
}
```

---

## 8. Patterns and Best Practices

### Always clean up raw mode

Wrap your app in a setup/teardown pattern:

```nova
fn run() {
    terminal::rawmode(true)
    terminal::hideCursor()
    // ... your app logic ...
    terminal::showCursor()
    terminal::rawmode(false)
}

run()
```

### Double-buffering with strings

Instead of many `moveTo`/`print` calls, build a frame string and print it all at once:

```nova
fn renderFrame(width: Int, height: Int, entities: [(Int, Int, Char)]) {
    terminal::moveTo(0, 0)
    for entity in entities {
        terminal::moveTo(entity[0], entity[1])
        terminal::print(Cast::string(entity[2]))
    }
    terminal::flush()
}
```

### Use structs for game state

```nova
struct AppState {
    running: Bool,
    player_x: Int,
    player_y: Int,
    score: Int,
}
```

### Colour themes

```nova
struct Theme {
    bg: (Int, Int, Int),
    fg: (Int, Int, Int),
    accent: (Int, Int, Int),
}

let dark = Theme {
    bg: (20, 20, 30),
    fg: (200, 200, 200),
    accent: (100, 200, 255),
}
```

---

## 9. Common Pitfalls

| Pitfall | Solution |
|---|---|
| Terminal stuck in raw mode after crash | Run `reset` in your shell, or `stty sane` |
| Nothing appears on screen | Call `terminal::flush()` after printing |
| `rawread` blocks forever | Pass a timeout in ms: `rawread(50)` |
| Colours not resetting | Always call `terminal::resetColor()` before exiting |
| `moveTo` coordinates wrong | It's `(column, row)`, 0-based, not `(x, y)` from top-left |
| Arrow keys not detected | They produce 3-byte escape sequences — read all 3 bytes |
| Screen flickers | Clear and redraw the whole screen each frame, or redraw only changed parts |
