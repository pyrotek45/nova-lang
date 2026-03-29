# Making Games with Nova and Raylib

Nova has first-class raylib integration for building 2D games. This guide covers everything
from your first window to a complete game architecture. For the full API reference, see
[raylib.md](raylib.md).

---

## Table of Contents

1. [Getting Started](#1-getting-started)
2. [The Game Loop](#2-the-game-loop)
3. [Movement and Delta Time](#3-movement-and-delta-time)
4. [Input Handling](#4-input-handling)
5. [Drawing Primitives](#5-drawing-primitives)
6. [Sprites and Pixel Art](#6-sprites-and-pixel-art)
7. [Audio: Sound Effects and Music](#7-audio-sound-effects-and-music)
8. [Collision Detection](#8-collision-detection)
9. [Game State Architecture](#9-game-state-architecture)
10. [Screen Management](#10-screen-management)
11. [UI and HUD](#11-ui-and-hud)
12. [Patterns for Nova Games](#12-patterns-for-nova-games)
13. [Performance Tips](#13-performance-tips)
14. [Common Pitfalls](#14-common-pitfalls)
15. [Complete Example: Pong](#15-complete-example-pong)

---

## 1. Getting Started

Create a file (e.g., `my_game.nv`) and run it with `nova run my_game.nv`.

```nova
module my_game

raylib::init("My First Game", 800, 600, 60)

while raylib::rendering() {
    raylib::clear((30, 30, 50))
    raylib::drawText("Hello, Gamedev!", 250, 280, 40, (255, 255, 255))
    raylib::drawFPS(10, 10)
}
```

`raylib::init` creates a window. `raylib::rendering()` processes one frame and returns
`false` when the window is closed. All drawing happens between consecutive
`rendering()` calls.

---

## 2. The Game Loop

Every game follows the same pattern: **Input → Update → Draw**.

```nova
module game_loop

raylib::init("Game", 800, 600, 60)

// State
let px = 400
let py = 300

while raylib::rendering() {
    let dt = raylib::getFrameTime()

    // 1. INPUT
    if raylib::isKeyPressed("KEY_RIGHT") { px += 5 }
    if raylib::isKeyPressed("KEY_LEFT")  { px -= 5 }
    if raylib::isKeyPressed("KEY_DOWN")  { py += 5 }
    if raylib::isKeyPressed("KEY_UP")    { py -= 5 }

    // 2. UPDATE (collision, AI, physics, etc.)
    if px < 0 { px = 0 }
    if px > 760 { px = 760 }

    // 3. DRAW
    raylib::clear((0, 0, 0))
    raylib::drawRectangle(px, py, 40, 40, (0, 200, 100))
    raylib::drawFPS(10, 10)
}
```

---

## 3. Movement and Delta Time

Use `raylib::getFrameTime()` (seconds since last frame) for smooth, frame-rate-independent
movement:

```nova
let speed = 200.0   // pixels per second
let px = 400.0
let py = 300.0

while raylib::rendering() {
    let dt = raylib::getFrameTime()

    if raylib::isKeyPressed("KEY_RIGHT") {
        px = px + speed * Cast::float(dt).unwrap()
    }

    // Convert to int for drawing
    let ix = Cast::int(px).unwrap()
    let iy = Cast::int(py).unwrap()

    raylib::clear((0, 0, 0))
    raylib::drawCircle(ix, iy, 20, (255, 100, 50))
}
```

> **Tip:** For simple games at 60 FPS, using integer positions with fixed pixel increments is
> perfectly fine. Delta time matters more for physics-heavy games or variable frame rates.

---

## 4. Input Handling

### Keyboard

| Function | Behaviour |
|---|---|
| `raylib::isKeyPressed("KEY_X")` | `true` while the key is held down |
| `raylib::isKeyReleased("KEY_X")` | `true` only the frame the key is released |
| `raylib::isKeyUp("KEY_X")` | `true` when the key is NOT pressed |
| `raylib::getKey()` | Returns `Option(String)` — the key pressed this frame |

Common key names: `"KEY_A"` – `"KEY_Z"`, `"KEY_0"` – `"KEY_9"`, `"KEY_UP"`, `"KEY_DOWN"`,
`"KEY_LEFT"`, `"KEY_RIGHT"`, `"KEY_SPACE"`, `"KEY_ENTER"`, `"KEY_ESCAPE"`,
`"KEY_LEFT_SHIFT"`, `"KEY_LEFT_CONTROL"`, `"KEY_TAB"`, `"KEY_BACKSPACE"`,
`"KEY_F1"` – `"KEY_F12"`.

### Mouse

```nova
let pos = raylib::mousePosition()   // (Int, Int) tuple
let mx = pos[0]
let my = pos[1]

if raylib::isMousePressed("MOUSE_BUTTON_LEFT") {
    // clicked!
}

let wheel = raylib::getMouseWheel()   // positive = scroll up
```

### Input pattern: action mapping

```nova
import super.std.core

fn isMovingRight() -> Bool {
    return raylib::isKeyPressed("KEY_RIGHT") || raylib::isKeyPressed("KEY_D")
}

fn isMovingLeft() -> Bool {
    return raylib::isKeyPressed("KEY_LEFT") || raylib::isKeyPressed("KEY_A")
}
```

---

## 5. Drawing Primitives

All draw functions take an RGB colour tuple `(Int, Int, Int)`.

```nova
// Background
raylib::clear((20, 20, 40))

// Shapes
raylib::drawRectangle(x, y, w, h, color)
raylib::drawRectangleLines(x, y, w, h, color)         // outline only
raylib::drawRoundedRectangle(x, y, w, h, 0.5, color)  // roundness 0.0–1.0
raylib::drawCircle(cx, cy, radius, color)
raylib::drawCircleLines(cx, cy, radius, color)
raylib::drawLine(x1, y1, x2, y2, color)
raylib::drawLineThick(x1, y1, x2, y2, 3.0, color)
raylib::drawTriangle(x1, y1, x2, y2, x3, y3, color)   // CCW order

// Text
raylib::drawText("Score: 100", 10, 10, 20, (255, 255, 255))
raylib::drawFPS(10, 560)

// Measure text width (for centering)
let w = raylib::measureText("Game Over", 40)
```

### Drawing order

Things drawn later appear on top. Draw background first, then world objects, then UI:

```nova
raylib::clear(bg_color)          // 1. background
drawWorld()                      // 2. game objects
drawUI()                         // 3. HUD on top
```

---

## 6. Sprites and Pixel Art

### Loading from file

```nova
let hero = raylib::loadSprite("assets/hero.png", 32, 1)
raylib::drawSprite(hero, px, py)
```

Parameters: `loadSprite(path, height, frameCount)`. For a single static image, use
`frameCount = 1`.

### Procedural sprites

Build pixel art programmatically:

```nova
fn makeCheckerboard(size: Int, c1: (Int,Int,Int), c2: (Int,Int,Int)) -> Sprite {
    let pixels = []: (Int, Int, Int)
    for let y = 0; y < size; y += 1 {
        for let x = 0; x < size; x += 1 {
            if (x + y) % 2 == 0 { pixels.push(c1) }
            else { pixels.push(c2) }
        }
    }
    return raylib::buildSprite(size, size, 1, pixels)
}

let checker = makeCheckerboard(8, (255, 0, 0), (0, 0, 255))
```

---

## 7. Audio: Sound Effects and Music

### Setup

Call `raylib::initAudio()` once before loading any audio:

```nova
raylib::init("Game", 800, 600, 60)
raylib::initAudio()
```

### Sound effects

Short clips loaded entirely into memory. Good for jumps, hits, UI clicks:

```nova
let jump_sfx = raylib::loadSound("assets/jump.wav")
let hit_sfx = raylib::loadSound("assets/hit.wav")

// In game loop:
if raylib::isKeyPressed("KEY_SPACE") {
    raylib::playSound(jump_sfx)
}

// Control volume and pitch per-sound:
raylib::setSoundVolume(jump_sfx, 0.8)
raylib::setSoundPitch(hit_sfx, 1.5)   // higher pitched
```

### Background music

Music is streamed from disk — ideal for long tracks. **You must call `updateMusic` every
frame:**

```nova
let bgm = raylib::loadMusic("assets/background.ogg")
raylib::setMusicLooping(bgm, true)
raylib::playMusic(bgm)

while raylib::rendering() {
    raylib::updateMusic(bgm)   // ← REQUIRED every frame

    // ... game logic and drawing ...
}

raylib::closeAudio()
```

### Music controls

```nova
raylib::pauseMusic(bgm)
raylib::resumeMusic(bgm)
raylib::stopMusic(bgm)
raylib::setMusicVolume(bgm, 0.5)
raylib::setMusicPitch(bgm, 1.0)

let total = raylib::getMusicLength(bgm)
let played = raylib::getMusicTimePlayed(bgm)
raylib::seekMusic(bgm, 0.0)   // restart from beginning

let isPlaying = raylib::isMusicPlaying(bgm)
```

### Audio cleanup

Always close the audio device when done:

```nova
raylib::closeAudio()
```

---

## 8. Collision Detection

Nova doesn't have built-in collision functions, but rectangle collision is simple:

### AABB (Axis-Aligned Bounding Box)

```nova
fn rectsOverlap(ax: Int, ay: Int, aw: Int, ah: Int,
                bx: Int, by: Int, bw: Int, bh: Int) -> Bool {
    return ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

// Usage:
if rectsOverlap(player_x, player_y, 40, 40, enemy_x, enemy_y, 30, 30) {
    // collision!
}
```

### Circle collision

```nova
fn circlesOverlap(x1: Int, y1: Int, r1: Int, x2: Int, y2: Int, r2: Int) -> Bool {
    let dx = x1 - x2
    let dy = y1 - y2
    let dist_sq = dx * dx + dy * dy
    let radii = r1 + r2
    return dist_sq < radii * radii
}
```

### Point in rectangle

```nova
fn pointInRect(px: Int, py: Int, rx: Int, ry: Int, rw: Int, rh: Int) -> Bool {
    return px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
}

// Mouse click detection:
let mouse = raylib::mousePosition()
if raylib::isMousePressed("MOUSE_BUTTON_LEFT") {
    if pointInRect(mouse[0], mouse[1], btn_x, btn_y, btn_w, btn_h) {
        // button clicked!
    }
}
```

---

## 9. Game State Architecture

### Struct-based state

Put all game state in a struct:

```nova
struct GameState {
    player_x: Int,
    player_y: Int,
    score: Int,
    lives: Int,
    enemies: [(Int, Int)],
    running: Bool,
}

fn GameStateNew() -> GameState {
    return GameState {
        player_x: 400,
        player_y: 500,
        score: 0,
        lives: 3,
        enemies: []: (Int, Int),
        running: true,
    }
}
```

### Update and draw as extends functions

```nova
fn extends update(state: GameState) {
    if raylib::isKeyPressed("KEY_LEFT")  { state.player_x -= 5 }
    if raylib::isKeyPressed("KEY_RIGHT") { state.player_x += 5 }

    // clamp
    if state.player_x < 0 { state.player_x = 0 }
    if state.player_x > 760 { state.player_x = 760 }
}

fn extends draw(state: GameState) {
    raylib::clear((0, 0, 0))
    raylib::drawRectangle(state.player_x, state.player_y, 40, 20, (0, 255, 0))
    raylib::drawText("Score: " + Cast::string(state.score), 10, 10, 20, (255, 255, 255))
    raylib::drawText("Lives: " + Cast::string(state.lives), 10, 40, 20, (255, 100, 100))
}
```

---

## 10. Screen Management

Use an enum to model game screens:

```nova
enum Screen {
    Title,
    Playing,
    Paused,
    GameOver
}

let current_screen = Screen::Title()

while raylib::rendering() {
    match current_screen {
        Title() => {
            raylib::clear((20, 20, 40))
            raylib::drawText("PRESS ENTER TO START", 200, 280, 30, (255, 255, 255))
            if raylib::isKeyPressed("KEY_ENTER") {
                current_screen = Screen::Playing()
            }
        }
        Playing() => {
            // game logic...
            if raylib::isKeyPressed("KEY_ESCAPE") {
                current_screen = Screen::Paused()
            }
        }
        Paused() => {
            raylib::clear((40, 40, 40))
            raylib::drawText("PAUSED", 340, 280, 40, (255, 255, 0))
            if raylib::isKeyPressed("KEY_ESCAPE") {
                current_screen = Screen::Playing()
            }
        }
        GameOver() => {
            raylib::clear((80, 0, 0))
            raylib::drawText("GAME OVER", 280, 280, 50, (255, 255, 255))
            if raylib::isKeyPressed("KEY_ENTER") {
                current_screen = Screen::Title()
            }
        }
    }
}
```

---

## 11. UI and HUD

### Health bar

```nova
fn drawHealthBar(x: Int, y: Int, w: Int, h: Int, current: Int, max: Int) {
    // Background
    raylib::drawRectangle(x, y, w, h, (60, 60, 60))
    // Fill
    let fill_w = (w * current) / max
    let color = if current * 3 < max { (255, 0, 0) }
                elif current * 3 < max * 2 { (255, 200, 0) }
                else { (0, 255, 0) }
    raylib::drawRectangle(x, y, fill_w, h, color)
    // Border
    raylib::drawRectangleLines(x, y, w, h, (255, 255, 255))
}

drawHealthBar(10, 560, 200, 20, 75, 100)
```

### Button

```nova
fn drawButton(x: Int, y: Int, w: Int, h: Int, text: String, hover: Bool) {
    let bg = if hover { (80, 120, 200) } else { (50, 80, 150) }
    raylib::drawRoundedRectangle(x, y, w, h, 0.3, bg)
    let tw = raylib::measureText(text, 20)
    raylib::drawText(text, x + (w - tw) / 2, y + (h - 20) / 2, 20, (255, 255, 255))
}

// Usage with mouse hover detection:
let mouse = raylib::mousePosition()
let hover = pointInRect(mouse[0], mouse[1], 300, 400, 200, 50)
drawButton(300, 400, 200, 50, "Start Game", hover)
```

---

## 12. Patterns for Nova Games

### Entity as struct + extends

```nova
struct Bullet {
    x: Int,
    y: Int,
    speed: Int,
    active: Bool,
}

fn extends update(b: Bullet) {
    b.y -= b.speed
    if b.y < 0 { b.active = false }
}

fn extends draw(b: Bullet) {
    if b.active {
        raylib::drawCircle(b.x, b.y, 4, (255, 255, 0))
    }
}
```

### Entity list management

```nova
import super.std.list

let bullets = []: Bullet

// Spawn
bullets.push(Bullet { x: px, y: py, speed: 8, active: true })

// Update all
for b in bullets { b.update() }

// Draw all
for b in bullets { b.draw() }

// Remove inactive (rebuild list)
bullets = bullets.filter(|b: Bullet| b.active)
```

### Timer pattern

```nova
import super.std.core

struct Timer {
    remaining: Int,
    interval: Int,
}

fn TimerNew(ms: Int) -> Timer {
    return Timer { remaining: ms, interval: ms }
}

fn extends tick(t: Timer, dt_ms: Int) -> Bool {
    t.remaining -= dt_ms
    if t.remaining <= 0 {
        t.remaining = t.interval
        return true
    }
    return false
}

// Usage:
let spawn_timer = TimerNew(1000)   // every 1 second
// In game loop:
let dt_ms = Cast::int(raylib::getFrameTime() * 1000.0).unwrap()
if spawn_timer.tick(dt_ms) {
    // spawn enemy!
}
```

---

## 13. Performance Tips

- **Minimize allocations in the game loop.** Create lists and structs outside the loop when
  possible. Reuse them.
- **Use `clone()` sparingly.** It deep-copies everything. For hot paths, share references
  instead.
- **Avoid string concatenation in tight loops.** Each `+` creates a new string on the heap.
  Use `format()` for complex strings.
- **Keep entity lists flat.** Lists of simple structs are cache-friendly and fast to iterate.
- **Use `setTargetFPS`** to cap the frame rate and save CPU.

---

## 14. Common Pitfalls

| Pitfall | Solution |
|---|---|
| Music stops playing | Call `raylib::updateMusic(id)` every frame |
| Forgot `raylib::clear()` | Ghost images from previous frames; always clear first |
| Triangle not showing | Vertices must be in counter-clockwise order |
| Audio functions fail | Call `raylib::initAudio()` before loading any audio |
| Colours look wrong | Values are 0–255 per channel: `(R, G, B)` |
| `loadSprite` path wrong | Path is relative to where you run `nova` |
| Game runs too fast | Set FPS with `raylib::init(title, w, h, 60)` or `setTargetFPS` |
| Entity list grows forever | Filter out dead entities: `list.filter(\|e\| e.active)` |
| State shared unexpectedly | Struct assignment aliases — use `clone()` for copies |
| Missing `closeAudio()` | Audio resources may leak; call before exit |

---

## 15. Complete Example: Pong

Here's a minimal but complete Pong game demonstrating the patterns above:

```nova
module pong

import super.std.core

// --- Constants ---
let SCREEN_W = 800
let SCREEN_H = 600
let PADDLE_W = 15
let PADDLE_H = 80
let BALL_SIZE = 10
let PADDLE_SPEED = 6
let BALL_SPEED = 4

// --- State ---
struct PongState {
    p1_y: Int,
    p2_y: Int,
    ball_x: Int,
    ball_y: Int,
    ball_dx: Int,
    ball_dy: Int,
    score1: Int,
    score2: Int,
}

fn PongNew() -> PongState {
    return PongState {
        p1_y: SCREEN_H / 2 - PADDLE_H / 2,
        p2_y: SCREEN_H / 2 - PADDLE_H / 2,
        ball_x: SCREEN_W / 2,
        ball_y: SCREEN_H / 2,
        ball_dx: BALL_SPEED,
        ball_dy: BALL_SPEED,
        score1: 0,
        score2: 0,
    }
}

fn extends update(s: PongState) {
    // Player 1 (W/S)
    if raylib::isKeyPressed("KEY_W") { s.p1_y -= PADDLE_SPEED }
    if raylib::isKeyPressed("KEY_S") { s.p1_y += PADDLE_SPEED }

    // Player 2 (Up/Down)
    if raylib::isKeyPressed("KEY_UP")   { s.p2_y -= PADDLE_SPEED }
    if raylib::isKeyPressed("KEY_DOWN") { s.p2_y += PADDLE_SPEED }

    // Clamp paddles
    if s.p1_y < 0 { s.p1_y = 0 }
    if s.p1_y > SCREEN_H - PADDLE_H { s.p1_y = SCREEN_H - PADDLE_H }
    if s.p2_y < 0 { s.p2_y = 0 }
    if s.p2_y > SCREEN_H - PADDLE_H { s.p2_y = SCREEN_H - PADDLE_H }

    // Move ball
    s.ball_x += s.ball_dx
    s.ball_y += s.ball_dy

    // Bounce off top/bottom
    if s.ball_y <= 0 || s.ball_y >= SCREEN_H - BALL_SIZE {
        s.ball_dy = 0 - s.ball_dy
    }

    // Paddle collision (left paddle)
    if s.ball_x <= 30 && s.ball_y + BALL_SIZE >= s.p1_y && s.ball_y <= s.p1_y + PADDLE_H {
        s.ball_dx = 0 - s.ball_dx
        s.ball_x = 31
    }

    // Paddle collision (right paddle)
    if s.ball_x >= SCREEN_W - 30 - BALL_SIZE && s.ball_y + BALL_SIZE >= s.p2_y && s.ball_y <= s.p2_y + PADDLE_H {
        s.ball_dx = 0 - s.ball_dx
        s.ball_x = SCREEN_W - 31 - BALL_SIZE
    }

    // Score
    if s.ball_x < 0 {
        s.score2 += 1
        s.ball_x = SCREEN_W / 2
        s.ball_y = SCREEN_H / 2
    }
    if s.ball_x > SCREEN_W {
        s.score1 += 1
        s.ball_x = SCREEN_W / 2
        s.ball_y = SCREEN_H / 2
    }
}

fn extends draw(s: PongState) {
    raylib::clear((0, 0, 0))

    // Centre line
    for let y = 0; y < SCREEN_H; y += 20 {
        raylib::drawRectangle(SCREEN_W / 2 - 1, y, 2, 10, (60, 60, 60))
    }

    // Paddles
    raylib::drawRectangle(15, s.p1_y, PADDLE_W, PADDLE_H, (200, 200, 200))
    raylib::drawRectangle(SCREEN_W - 30, s.p2_y, PADDLE_W, PADDLE_H, (200, 200, 200))

    // Ball
    raylib::drawRectangle(s.ball_x, s.ball_y, BALL_SIZE, BALL_SIZE, (255, 255, 0))

    // Scores
    raylib::drawText(Cast::string(s.score1), SCREEN_W / 2 - 60, 20, 40, (255, 255, 255))
    raylib::drawText(Cast::string(s.score2), SCREEN_W / 2 + 30, 20, 40, (255, 255, 255))

    raylib::drawFPS(10, SCREEN_H - 25)
}

// --- Main ---
raylib::init("Pong", SCREEN_W, SCREEN_H, 60)

let game = PongNew()

while raylib::rendering() {
    game.update()
    game.draw()
}
```
