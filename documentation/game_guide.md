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
16. [Entity Systems](#16-entity-systems)
    - [16.1 The Problem with One Big Struct](#161-the-problem-with-one-big-struct)
    - [16.2 Enum-based Entity Kinds](#162-enum-based-entity-kinds)
    - [16.3 Shared Movement and Bounds](#163-shared-movement-and-bounds)
    - [16.4 Collision Detection](#164-collision-detection)
    - [16.5 Collision Response with Pattern Matching](#165-collision-response-with-pattern-matching)
    - [16.6 Per-type Drawing with the `->` Vtable Pattern](#166-per-type-drawing-with-the---vtable-pattern)
    - [16.7 Per-type Update with `->` Dispatch](#167-per-type-update-with---dispatch)
    - [16.8 Putting It All Together](#168-putting-it-all-together)
    - [16.9 Layered Draw Order](#169-layered-draw-order)
    - [16.10 Quick Reference](#1610-quick-reference)

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

---

## 16. Entity Systems

As games grow, individual structs for each entity type get unwieldy. An entity system
gives you a clean way to store many kinds of objects together, update them uniformly,
and dispatch drawing to each type's own logic.

---

### 16.1 The Problem with One Big Struct

The naive approach is a single struct with a field for everything:

```nova
struct Entity {
    kind: String,   // "player", "enemy", "bullet" ...
    x: Int,
    y: Int,
    hp: Int,
    speed: Int,
    value: Int,     // only used by pickups
    active: Bool,
}
```

This works for toy examples but quickly becomes a mess — most fields are irrelevant for
most entity types, and you need to check `kind` everywhere.

---

### 16.2 Enum-based Entity Kinds

Give each entity type its own data by storing the type-specific state in an enum:

```nova
enum EntityKind {
    Player,
    Enemy:  Int,   // carries current health
    Pickup: Int,   // carries point value
    Bullet,
}

struct Entity {
    x: Int,
    y: Int,
    w: Int,
    h: Int,
    vx: Int,
    vy: Int,
    kind: EntityKind,
    active: Bool,
}
```

Constructor helpers keep spawning readable:

```nova
fn mkPlayer(x: Int, y: Int) -> Entity {
    return Entity { x: x, y: y, w: 32, h: 32, vx: 0, vy: 0,
                    kind: EntityKind::Player(), active: true }
}

fn mkEnemy(x: Int, y: Int, hp: Int) -> Entity {
    return Entity { x: x, y: y, w: 24, h: 24, vx: -2, vy: 0,
                    kind: EntityKind::Enemy(hp), active: true }
}

fn mkPickup(x: Int, y: Int, pts: Int) -> Entity {
    return Entity { x: x, y: y, w: 16, h: 16, vx: 0, vy: 0,
                    kind: EntityKind::Pickup(pts), active: true }
}

fn mkBullet(x: Int, y: Int) -> Entity {
    return Entity { x: x, y: y, w: 4, h: 10, vx: 0, vy: -12,
                    kind: EntityKind::Bullet(), active: true }
}
```

---

### 16.3 Shared Movement and Bounds

Movement is the same for every entity — it just applies velocity:

```nova
fn extends move(e: Entity) {
    e.x += e.vx
    e.y += e.vy
}

fn extends keepInBounds(e: Entity, w: Int, h: Int) {
    if e.x < 0        { e.x = 0 }
    if e.y < 0        { e.y = 0 }
    if e.x + e.w > w  { e.x = w - e.w }
    if e.y + e.h > h  { e.y = h - e.h }
}
```

Updating the whole list in the game loop:

```nova
import super.std.list

let entities = []: Entity

// ... spawn some entities ...

// Update all
for e in entities {
    e.move()
}

// Remove dead entities
entities = entities.filter(): |e: Entity| e.active
```

---

### 16.4 Collision Detection

AABB (axis-aligned bounding box) overlap as an `extends` function means you can call it
naturally on any two entities:

```nova
fn rectsOverlap(ax: Int, ay: Int, aw: Int, ah: Int,
                bx: Int, by: Int, bw: Int, bh: Int) -> Bool {
    return ax < bx + bw
        && ax + aw > bx
        && ay < by + bh
        && ay + ah > by
}

fn extends overlaps(a: Entity, b: Entity) -> Bool {
    return rectsOverlap(a.x, a.y, a.w, a.h,
                        b.x, b.y, b.w, b.h)
}
```

Checking all pairs in the loop:

```nova
import super.std.core

let score = Box(0)

for let i = 0; i < entities.len(); i += 1 {
    for let j = i + 1; j < entities.len(); j += 1 {
        if entities[i].active && entities[j].active {
            if entities[i].overlaps(entities[j]) {
                handleCollision(entities[i], entities[j])
            }
        }
    }
}
```

---

### 16.5 Collision Response with Pattern Matching

`match` on the enum lets you handle each combination cleanly, with access to the
associated data:

```nova
fn handleCollision(a: Entity, b: Entity) {
    match a.kind {
        Bullet() => {
            match b.kind {
                Enemy(hp) => {
                    // bullet hits enemy
                    a.active = false
                    if hp <= 1 {
                        b.active = false
                    } else {
                        b.kind = EntityKind::Enemy(hp - 1)
                    }
                }
                Player() => {
                    // enemy bullet hits player — handle elsewhere
                }
                _ => { }
            }
        }
        Player() => {
            match b.kind {
                Pickup(pts) => {
                    score.value += pts
                    b.active = false
                }
                Enemy(hp) => {
                    // player touches enemy
                    a.active = false
                }
                _ => { }
            }
        }
        _ => { }
    }
}
```

Because the collision handler is symmetric, call it both ways for pairs that need it, or
normalise the order first (e.g. always put the Bullet first).

---

### 16.6 Per-type Drawing with the `->` Vtable Pattern

This is where Nova's `->` dispatch shines. Instead of a big `match` for drawing, each
entity type carries its own `draw` function as a field. The `->` operator calls it,
dispatching through the stored closure at runtime.

```nova
// Each entity type gets its own draw closure stored as a field.
// The type must have a field named exactly 'draw' with signature fn(Self).

struct PlayerEntity {
    x: Int,
    y: Int,
    hp: Int,
    draw: fn(PlayerEntity),
}

struct EnemyEntity {
    x: Int,
    y: Int,
    hp: Int,
    speed: Int,
    draw: fn(EnemyEntity),
}

struct BulletEntity {
    x: Int,
    y: Int,
    active: Bool,
    draw: fn(BulletEntity),
}
```

Construct each type with a concrete draw function baked in:

```nova
fn makePlayer(x: Int, y: Int) -> PlayerEntity {
    return PlayerEntity {
        x: x, y: y, hp: 3,
        draw: fn(self: PlayerEntity) {
            raylib::drawRectangle(self.x, self.y, 32, 32, (50, 200, 255))
            // draw a small health pip for each hp
            for i in 0..self.hp {
                raylib::drawRectangle(self.x + i * 12, self.y - 10, 10, 6, (0, 255, 100))
            }
        }
    }
}

fn makeEnemy(x: Int, y: Int) -> EnemyEntity {
    return EnemyEntity {
        x: x, y: y, hp: 2, speed: 2,
        draw: fn(self: EnemyEntity) {
            raylib::drawRectangle(self.x, self.y, 24, 24, (220, 60, 60))
        }
    }
}

fn makeBullet(x: Int, y: Int) -> BulletEntity {
    return BulletEntity {
        x: x, y: y, active: true,
        draw: fn(self: BulletEntity) {
            if self.active {
                raylib::drawRectangle(self.x, self.y, 4, 10, (255, 255, 80))
            }
        }
    }
}
```

Now define a `Dyn` type that matches anything with a `draw` field, and write a single
draw-all function:

```nova
type drawable = Dyn(T = draw: fn($T))

fn drawAll(items: [drawable]) {
    for item in items {
        item->draw()   // calls PlayerEntity::draw, EnemyEntity::draw, etc.
    }
}
```

In the game loop:

```nova
let player  = makePlayer(400, 500)
let enemies = [makeEnemy(100, 100), makeEnemy(300, 150)]
let bullets = []: BulletEntity

// All drawables in one call — each uses its own closure
while raylib::rendering() {
    raylib::clear((10, 10, 20))

    player->draw()
    for e in enemies { e->draw() }
    for b in bullets { b->draw() }
}
```

Or mix them all into a single `[drawable]` list if every entity type has the `draw` field:

```nova
// You can push any type that has a draw field into the same list
// (as long as the Dyn constraint is satisfied)
fn drawScene(players: [PlayerEntity], enemies: [EnemyEntity], bullets: [BulletEntity]) {
    for p in players { p->draw() }
    for e in enemies { e->draw() }
    for b in bullets { b->draw() }
}
```

The key point: **you never write a `match` to decide how to draw**. Each entity type knows
how to draw itself. Adding a new entity type means writing a new struct with its own
`draw` closure — existing code does not change.

---

### 16.7 Per-type Update with `->` Dispatch

The same pattern works for `update`:

```nova
struct PlayerEntity {
    x: Int, y: Int, hp: Int,
    draw:   fn(PlayerEntity),
    update: fn(PlayerEntity),
}

fn makePlayer(x: Int, y: Int) -> PlayerEntity {
    return PlayerEntity {
        x: x, y: y, hp: 3,
        draw: fn(self: PlayerEntity) {
            raylib::drawRectangle(self.x, self.y, 32, 32, (50, 200, 255))
        },
        update: fn(self: PlayerEntity) {
            if raylib::isKeyPressed("KEY_LEFT")  { self.x -= 4 }
            if raylib::isKeyPressed("KEY_RIGHT") { self.x += 4 }
            if raylib::isKeyPressed("KEY_UP")    { self.y -= 4 }
            if raylib::isKeyPressed("KEY_DOWN")  { self.y += 4 }
        }
    }
}

struct EnemyEntity {
    x: Int, y: Int, hp: Int, speed: Int,
    draw:   fn(EnemyEntity),
    update: fn(EnemyEntity),
}

fn makeEnemy(x: Int, y: Int, spd: Int) -> EnemyEntity {
    return EnemyEntity {
        x: x, y: y, hp: 2, speed: spd,
        draw: fn(self: EnemyEntity) {
            raylib::drawRectangle(self.x, self.y, 24, 24, (220, 60, 60))
        },
        update: fn(self: EnemyEntity) {
            self.x -= self.speed   // march left
            if self.x < -24 { self.x = 820 }
        }
    }
}

type updatable = Dyn(T = update: fn($T))

// Game loop:
while raylib::rendering() {
    player->update()
    for e in enemies { e->update() }
    for b in bullets { b->update() }

    raylib::clear((10, 10, 20))
    player->draw()
    for e in enemies { e->draw() }
    for b in bullets { b->draw() }
}
```

---

### 16.8 Putting It All Together

Here is a minimal but complete structure for a game with multiple entity types,
type-dispatched drawing, shared collision, and score:

```nova
module shooter
import super.std.list
import super.std.core

let SCREEN_W = 800
let SCREEN_H = 600

// ── Collision helper ───────────────────────────────────────────────
fn rectsOverlap(ax: Int, ay: Int, aw: Int, ah: Int,
                bx: Int, by: Int, bw: Int, bh: Int) -> Bool {
    return ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

// ── Entity kinds ───────────────────────────────────────────────────
enum Tag { Player, Enemy: Int, Bullet, Pickup: Int }

struct Entity {
    x: Int,
    y: Int,
    w: Int,
    h: Int,
    vx: Int,
    vy: Int,
    tag: Tag,
    active: Bool,
    draw: fn(Entity),
}

// ── Constructors ───────────────────────────────────────────────────
fn mkPlayer(x: Int, y: Int) -> Entity {
    return Entity {
        x: x, y: y, w: 32, h: 32, vx: 0, vy: 0,
        tag: Tag::Player(), active: true,
        draw: fn(self: Entity) {
            raylib::drawRectangle(self.x, self.y, self.w, self.h, (50, 200, 255))
        }
    }
}

fn mkEnemy(x: Int, y: Int, hp: Int) -> Entity {
    return Entity {
        x: x, y: y, w: 24, h: 24, vx: -1, vy: 0,
        tag: Tag::Enemy(hp), active: true,
        draw: fn(self: Entity) {
            match self.tag {
                Enemy(hp) => {
                    let r = 80 + hp * 40
                    raylib::drawRectangle(self.x, self.y, self.w, self.h, (r, 60, 60))
                }
                _ => { }
            }
        }
    }
}

fn mkBullet(x: Int, y: Int) -> Entity {
    return Entity {
        x: x, y: y, w: 4, h: 10, vx: 0, vy: -12,
        tag: Tag::Bullet(), active: true,
        draw: fn(self: Entity) {
            if self.active {
                raylib::drawRectangle(self.x, self.y, self.w, self.h, (255, 255, 80))
            }
        }
    }
}

// ── Shared logic ───────────────────────────────────────────────────
fn extends move(e: Entity) {
    e.x += e.vx
    e.y += e.vy
}

fn extends overlaps(a: Entity, b: Entity) -> Bool {
    return rectsOverlap(a.x, a.y, a.w, a.h, b.x, b.y, b.w, b.h)
}

let score = Box(0)

fn handleCollision(a: Entity, b: Entity) {
    match a.tag {
        Bullet() => {
            match b.tag {
                Enemy(hp) => {
                    a.active = false
                    if hp <= 1 { b.active = false }
                    else { b.tag = Tag::Enemy(hp - 1) }
                    score.value += 10
                }
                _ => { }
            }
        }
        _ => { }
    }
}

// ── Game ───────────────────────────────────────────────────────────
raylib::init("Shooter", SCREEN_W, SCREEN_H, 60)

let player  = mkPlayer(SCREEN_W / 2, SCREEN_H - 60)
let enemies = [mkEnemy(100, 80, 1), mkEnemy(300, 80, 2), mkEnemy(500, 80, 3)]
let bullets = []: Entity

let fire_cooldown = Box(0)

while raylib::rendering() {
    // Input
    if raylib::isKeyPressed("KEY_LEFT")  { player.x -= 5 }
    if raylib::isKeyPressed("KEY_RIGHT") { player.x += 5 }
    if raylib::isKeyPressed("KEY_SPACE") && fire_cooldown.value <= 0 {
        bullets.push(mkBullet(player.x + 14, player.y - 10))
        fire_cooldown.value = 12
    }
    if fire_cooldown.value > 0 { fire_cooldown.value -= 1 }

    // Move
    for b in bullets { b.move() }
    for e in enemies { e.move() }

    // Collisions — bullets vs enemies
    for b in bullets {
        for e in enemies {
            if b.active && e.active && b.overlaps(e) {
                handleCollision(b, e)
            }
        }
    }

    // Cull dead entities
    bullets = bullets.filter(): |b: Entity| b.active
    enemies = enemies.filter(): |e: Entity| e.active

    // Draw — each entity calls its own closure via ->
    raylib::clear((10, 10, 20))
    player->draw()
    for e in enemies { e->draw() }
    for b in bullets { b->draw() }

    raylib::drawText("Score: " + Cast::string(score.value), 10, 10, 24, (255, 255, 255))
    raylib::drawFPS(10, SCREEN_H - 25)
}
```

The draw loop has no `match`, no `if kind == ...`. Each entity draws itself. Adding a
new entity type — a shield powerup, a boss, an explosion effect — means writing one new
constructor function with its own `draw` closure. Nothing else changes.

---

### 16.9 Layered Draw Order

When entities need to draw in layers (background effects behind enemies, UI on top),
keep separate lists per layer and draw them in order:

```nova
// Spawn into the right layer
let layer_bg      = []: Entity   // background effects, floor tiles
let layer_world   = []: Entity   // enemies, pickups, projectiles
let layer_player  = []: Entity   // player (always above enemies)
let layer_ui      = []: Entity   // health bars, text popups

// Draw in order — later layers appear on top
raylib::clear((10, 10, 20))
for e in layer_bg     { e->draw() }
for e in layer_world  { e->draw() }
for e in layer_player { e->draw() }
for e in layer_ui     { e->draw() }
```

---

### 16.10 Quick Reference

| Pattern | How |
|---|---|
| Multiple entity types | `enum Tag` with associated data |
| Per-type draw | `draw: fn(Self)` field + `item->draw()` |
| Per-type update | `update: fn(Self)` field + `item->update()` |
| Accept any drawable | `Dyn(T = draw: fn($T))` |
| Shared movement | `fn extends move(e: Entity)` |
| AABB collision | `fn extends overlaps(a: Entity, b: Entity) -> Bool` |
| Collision response | `match a.tag { ... }` |
| Kill dead entities | `list.filter(): \|e\| e.active` |
| Layered draw order | Separate lists per layer, drawn in sequence |
