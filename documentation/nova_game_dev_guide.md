# How to Write Games in Nova

The definitive guide to building games with Nova and its game-development standard library.
This document replaces `game_engine_guide.md` and `game_guide.md`.

---

## Table of Contents

1. [Quick Start — Your First Window](#1-quick-start)
2. [Project Structure](#2-project-structure)
3. [⚠️ Critical Rules You Must Know](#3-critical-rules)
   - [3.1 Box-wrap all mutable scalars captured by closures](#31-box-wrap-mutable-scalars)
   - [3.2 Entity movement — manual vs `world.update(dt)`](#32-entity-movement)
   - [3.3 Forward declarations for mutually-calling functions](#33-forward-declarations)
   - [3.4 `elif` vs `else if`](#34-elif-syntax)
4. [Scene Management](#4-scene-management)
5. [Entity System](#5-entity-system)
6. [Input Handling](#6-input-handling)
7. [Physics and Collision](#7-physics-and-collision)
8. [Camera](#8-camera)
9. [Timers](#9-timers)
10. [Tweens](#10-tweens)
11. [Vec2 Math](#11-vec2-math)
12. [Tilemaps with Grid](#12-tilemaps-with-grid)
13. [Procedural Generation with Noise](#13-noise)
14. [Sprites and Pixel Art](#14-sprites)
15. [Audio](#15-audio)
16. [HUD and UI](#16-hud-and-ui)
17. [Advanced Patterns](#17-advanced-patterns)
    - [17.1 Enum-based entity kinds](#171-enum-based-entities)
    - [17.2 Vtable dispatch with `->`](#172-vtable-dispatch)
    - [17.3 Object pooling](#173-object-pooling)
    - [17.4 Spatial grid (broad-phase collision)](#174-spatial-grid)
    - [17.5 Screen-stack without SceneManager](#175-screen-stack)
18. [Tips and Tricks](#18-tips-and-tricks)
    - [18.1 Frame animation](#181-frame-animation)
    - [18.2 Screen shake (manual)](#182-screen-shake)
    - [18.3 Hit flash](#183-hit-flash)
    - [18.4 Wave spawner](#184-wave-spawner)
    - [18.5 Floating score popups](#185-score-popups)
    - [18.6 Particle burst](#186-particles)
    - [18.7 Debug overlay](#187-debug-overlay)
    - [18.8 Integer lerp and clamp](#188-lerp-clamp)
19. [Performance Tips](#19-performance-tips)
20. [Complete Example — Breakout](#20-example-breakout)
21. [Complete Example — Top-Down Shooter](#21-example-shooter)
22. [Quick Reference Tables](#22-quick-reference)

---

## 1. Quick Start — Your First Window {#1-quick-start}

Create a file (e.g., `my_game.nv`) and run it with `nova run my_game.nv`.

```nova
raylib::init("My Game", 800, 600, 60)

while raylib::rendering() {
    let dt = raylib::getFrameTime()   // seconds since last frame (Float)
    raylib::clear((20, 20, 40))
    raylib::drawText("Hello, Nova!", 280, 270, 36, (255, 255, 255))
    raylib::drawFPS(10, 10)
}
```

`raylib::rendering()` returns `true` each frame and handles:
- Clearing the background before each call
- Polling input events
- Presenting the rendered frame
- Returning `false` when the window is closed

The game loop follows **Input → Update → Draw**:

```nova
while raylib::rendering() {
    let dt = raylib::getFrameTime()

    // 1. INPUT — read keys, mouse, gamepad

    // 2. UPDATE — move entities, run AI, resolve collisions

    // 3. DRAW — call all raylib::draw* functions
    raylib::clear((0, 0, 0))
}
```

---

## 2. Project Structure {#2-project-structure}

Recommended layout for a Nova game:

```
my_game/
  main.nv           ← entry point: raylib::init + SceneManager loop
  scenes/
    title.nv        ← makeTitleScene()
    gameplay.nv     ← makeGameplayScene()
    gameover.nv     ← makeGameOverScene()
  entities/
    player.nv       ← player spawn / update helpers
    enemy.nv        ← enemy AI helpers
  assets/           ← sprites, sounds (loaded by raylib)
```

For single-file games (common for small projects), keep everything in one file — scene
factory functions and forward declarations handle ordering.

---

## 3. ⚠️ Critical Rules You Must Know {#3-critical-rules}

These are the most common sources of silent bugs in Nova games. Read this section
before writing a single line of game logic.

---

### 3.1 Box-wrap all mutable scalars captured by closures {#31-box-wrap-mutable-scalars}

> **The #1 Nova game-dev gotcha.**

Nova closures capture **plain scalars (`Int`, `Float`, `Bool`) by value** — a snapshot
is taken at the moment the closure is created. Any mutations made inside the closure are
invisible to the next frame.

Heap objects (`Vec2`, `EntityWorld`, `SceneManager`, `InputMap`, `Tween`, `Timer`,
`Camera2D`, user-defined structs) are captured **by reference** — mutations persist
automatically.

**Wrong — scalar captured by value, mutations are lost:**
```nova
let score = 0         // plain Int
let lives = 3

let update = fn(dt: Float) {
    score += 10       // ← writes to a COPY; never seen outside this call
    lives -= 1        // ← same — always resets to 0 and 3 next frame
}
```

**Right — wrap mutable scalars in `Box(T)`:**
```nova
let score = Box(0)    // heap-allocated Int
let lives = Box(3)

let update = fn(dt: Float) {
    score.value += 10   // ← mutates the heap cell; persists across frames
    lives.value -= 1
}

// Read anywhere:
raylib::drawText("Score: " + Cast::string(score.value), 10, 10, 20, (255,255,255))
```

**Checklist — wrap in `Box` if ALL of these are true:**
1. The variable holds a scalar (`Int`, `Float`, `Bool`)
2. The variable is declared outside a closure
3. The variable needs to be mutated inside the closure

Heap objects (`Vec2`, `EntityWorld`, etc.) never need `Box` — they already live on
the heap and are shared by reference.

---

### 3.2 Entity movement — manual vs `world.update(dt)` {#32-entity-movement}

`EntityWorld::update(dt)` does two things:
1. **Integrates velocity**: `pos += vel * dt` for every entity
2. **Purges dead entities**: removes all entities where `alive == false`

If you call `world.update(dt)` with a real `dt`, every entity's velocity is applied.
If you instead move entities **manually** in `forEachTagged` (setting `pos` directly),
you should call `world.update(0.0)` — this only runs the dead-entity purge, not the
double-integration.

```nova
// ✅ Pattern A — let world.update handle movement
// Set vel fields, then call update with real dt
world.forEachTagged("enemy", fn(e: Entity) {
    e.vel.x = dirX * SPEED
    e.vel.y = dirY * SPEED
})
world.update(dt)   // integrates pos += vel * dt, purges dead

// ✅ Pattern B — move manually, use update only for cleanup
// Integrate pos yourself, then call update with 0.0
world.forEachTagged("bullet", fn(b: Entity) {
    b.pos.x += b.vel.x * dt    // ← manual integration
    b.pos.y += b.vel.y * dt
    b.data += dt               // age/timer in data field
    if b.data > BULLET_LIFETIME { b.alive = false }
})
world.update(0.0)  // ← only purges dead, does NOT double-integrate
```

> **Rule of thumb:** Pick ONE approach per entity type and stick to it.
> Never call `world.update(dt)` with a real dt AND also manually increment `pos` in the
> same frame — that will move everything twice.

---

### 3.3 Forward declarations for mutually-calling functions {#33-forward-declarations}

Scene factory functions often reference each other in a cycle
(`makeMenuScene` calls `makePlayScene`, which calls `makeMenuScene` on game-over).
Nova's parser needs to know the signatures before it sees the bodies.

**Syntax**: write the signature with **no body and no `{}`**:

```nova
fn makeMenuScene() -> Scene         // ← forward declaration (no body!)
fn makePlayScene() -> Scene
fn makeGameOverScene() -> Scene

// Bodies can now appear in any order
fn makeMenuScene() -> Scene {
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("start") { mgr.switch(makePlayScene()) }
        },
        fn() { raylib::drawText("Press Start", 300, 280, 24, (255,255,255)) }
    )
}

fn makePlayScene() -> Scene {
    return Scene::new(
        fn(dt: Float) {
            if playerDead.value { mgr.switch(makeGameOverScene()) }
        },
        fn() { /* draw world */ }
    )
}

fn makeGameOverScene() -> Scene {
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("restart") { mgr.switch(makeMenuScene()) }
        },
        fn() { raylib::drawText("Game Over", 300, 280, 36, (255,100,100)) }
    )
}
```

The parser detects the absence of `{` and registers the line as a forward declaration.

---

### 3.4 `elif` syntax {#34-elif-syntax}

Nova uses `elif` (not `else if`) for chained conditionals in **statement** context.
In **expression** context (assigned to `let`), only `if { } else { }` pairs are valid
— no chaining.

```nova
// ✅ Statement context — use elif
if hp > 60 {
    color = (0, 255, 0)
} elif hp > 30 {
    color = (255, 200, 0)
} else {
    color = (255, 0, 0)
}

// ✅ Expression context — only single if/else
let color = if hp > 60 { (0, 255, 0) } else { (255, 0, 0) }

// ❌ Wrong — else if (not a keyword)
} else if hp > 30 {   // parse error

// ❌ Wrong — elif in expression chain
let color = if hp > 60 { green } elif hp > 30 { yellow } else { red }  // parse error
```

---

## 4. Scene Management {#4-scene-management}

Scenes decouple game logic into self-contained states (title, gameplay, pause,
game-over). Each scene holds all its own state via closures — no global variables needed.

```nova
import super.std.scene

// A Scene is just two closures: update(dt) and draw()
fn makeGameplayScene(mgr: SceneManager) -> Scene {
    // ── local state ───────────────────────────────────────────
    let world   = EntityWorld::new()
    let keys    = InputMap::new()
    let score   = Box(0)          // ← scalar: Box-wrap!
    let paused  = Box(false)      // ← scalar: Box-wrap!

    // ... set up entities, key bindings ...

    // ── update closure ────────────────────────────────────────
    let update = fn(dt: Float) {
        if keys.isPressed("pause") {
            mgr.push(makePauseScene(mgr))
        }
        // ... game logic ...
    }

    // ── draw closure ─────────────────────────────────────────
    let draw = fn() {
        raylib::clear((10, 10, 20))
        // ... draw world ...
        raylib::drawText("Score: " + Cast::string(score.value), 10, 10, 20, (255,255,255))
    }

    return Scene::new(update, draw)
}

// Main loop
raylib::init("My Game", 800, 600, 60)
let mgr = SceneManager::empty()
mgr.switch(makeTitleScene(mgr))
while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

### Scene transitions

| Method | Effect |
|---|---|
| `mgr.switch(scene)` | Replace current scene; clears the whole stack |
| `mgr.push(scene)` | Push over current scene (pause menu, dialog box) |
| `mgr.pop()` | Return to the previous scene |

`mgr.push` is ideal for pause menus — the gameplay scene is preserved underneath.
`mgr.pop` resumes it without re-creating anything.

### SceneManager initialisation

```nova
// Option A — start empty, switch to first scene
let mgr = SceneManager::empty()
mgr.switch(makeTitle(mgr))

// Option B — start with a scene already loaded
let mgr = SceneManager::new(makeTitle(mgr))
```

---

## 5. Entity System {#5-entity-system}

`EntityWorld` is a tag-based entity manager. Entities are simple structs mutated in place.

### Setup and spawn

```nova
import super.std.entity
import super.std.vec2

let world = EntityWorld::new()

let player = world.spawn(400.0, 300.0, "player")
player.size = Vec2::new(32.0, 32.0)

for let i = 0; i < 5; i += 1 {
    let e = world.spawn(Cast::float(i * 120 + 50).unwrap(), 80.0, "enemy")
    e.size = Vec2::new(24.0, 24.0)
    e.data = 3.0   // health stored in data field
}
```

### Entity fields

| Field | Type | Purpose |
|---|---|---|
| `id` | `Int` | Unique auto-assigned identifier |
| `pos` | `Vec2` | World-space top-left position |
| `vel` | `Vec2` | Velocity in units/second |
| `size` | `Vec2` | Width × height (for AABB, drawing) |
| `tag` | `String` | Category: `"player"`, `"enemy"`, `"bullet"` … |
| `alive` | `Bool` | Set to `false` to destroy on next `update` |
| `data` | `Float` | General-purpose slot: health, angle, age, … |

### Querying and iterating

```nova
// Get a list of entities with a specific tag
let pList = world.query("player")
let player = pList[0]

// Iterate all entities with a tag — mutations take effect immediately
world.forEachTagged("enemy", fn(e: Entity) {
    e.pos.x += e.vel.x * dt
    e.pos.y += e.vel.y * dt
})

// Iterate ALL entities
world.forEach(fn(e: Entity) {
    // ...
})

// Count alive entities with a tag
let count = world.countAlive("enemy")
```

### Collision detection

```nova
// AABB overlap between two entities
if bullet.overlapsAABB(enemy) {
    bullet.alive = false
    enemy.data -= 1.0
    if enemy.data <= 0.0 { enemy.alive = false }
}

// Center of an entity (convenience)
let cx = e.center().x    // pos.x + size.x / 2
```

### Built-in draw helpers

```nova
world.forEachTagged("player", fn(e: Entity) {
    e.entityDrawRect((60, 200, 100))          // filled rectangle at pos/size
})
world.forEachTagged("enemy", fn(e: Entity) {
    e.entityDrawRect((220, 60, 60))
})
world.forEachTagged("bullet", fn(e: Entity) {
    e.entityDrawCircle((255, 230, 0))         // circle at center, radius size.x/2
})
```

### Update and purge

```nova
world.update(dt)     // integrates pos += vel*dt, then removes dead entities
world.update(0.0)    // only removes dead entities (no movement integration)
```

---

## 6. Input Handling {#6-input-handling}

### Raw raylib input

| Function | Behaviour |
|---|---|
| `raylib::isKeyPressed("A")` | `true` while key is held |
| `raylib::isKeyReleased("A")` | `true` only the frame the key is released |
| `raylib::isKeyUp("A")` | `true` when key is NOT held |
| `raylib::isMousePressed("Left")` | `true` while mouse button is held |
| `raylib::isMouseReleased("Left")` | `true` only the frame button is released |
| `raylib::mousePosition()` | `(Int, Int)` screen coordinates |
| `raylib::getMouseWheel()` | `Float` — positive = scroll up |

Common key names: `"A"`–`"Z"`, `"0"`–`"9"`, `"Space"`, `"Enter"`, `"Escape"`,
`"Up"`, `"Down"`, `"Left"`, `"Right"`, `"LeftShift"`, `"LeftControl"`, `"F1"`–`"F12"`.

### InputMap — decoupled action bindings

`InputMap` decouples game logic from raw key names and lets you support multiple
bindings per action and runtime rebinding.

```nova
import super.std.input

let keys = InputMap::new()
keys.bindKey("left",   "A")
keys.bindKey("right",  "D")
keys.bindKey("up",     "W")
keys.bindKey("down",   "S")
keys.bindKey("jump",   "Space")
keys.bindKey("fire",   "J")
keys.bindMouse("aim",  "Left")

// Inside the game loop:
let dx = keys.axis("left", "right")   // -1.0, 0.0, or 1.0
let dy = keys.axis("up",   "down")

if keys.isHeld("fire")     { /* fire while held */        }
if keys.isPressed("jump")  { /* fire once per press */    }
if keys.isReleased("fire") { /* fire on button release */ }

let (mx, my) = InputMap::mousePos()   // screen coordinates
```

### Runtime rebinding

```nova
let newKey = InputMap::lastKey()   // Option(String)
if newKey.isSome() {
    keys.bindKey("jump", newKey.unwrap())
}
```

---

## 7. Physics and Collision {#7-physics-and-collision}

The `physics` module provides `Body2D` for moveable objects and `AABB` / `Circle`
for shapes.

### Basic gravity + floor bounce

```nova
import super.std.physics

let ball = Body2D::new(400.0, 100.0, 1.0)
ball.restitution = 0.7

let floor = AABB::new(0.0, 560.0, 800.0, 40.0)
let wallL = AABB::new(-20.0, 0.0, 20.0, 600.0)
let wallR = AABB::new(800.0, 0.0, 20.0, 600.0)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    ball.applyGravity(600.0, dt)
    ball.update(dt)
    pushOutAABB(ball, 20.0, 20.0, floor)
    pushOutAABB(ball, 20.0, 20.0, wallL)
    pushOutAABB(ball, 20.0, 20.0, wallR)
    raylib::drawCircle(Cast::int(ball.pos.x).unwrap(),
                       Cast::int(ball.pos.y).unwrap(), 10, (255, 100, 0))
}
```

### Body vs body resolution

```nova
// Ball-to-ball collision
for let i = 0; i < balls.len(); i += 1 {
    for let j = i + 1; j < balls.len(); j += 1 {
        resolveCircle(balls[i], 12.0, balls[j], 12.0)
    }
}

// AABB vs AABB push-out (two dynamic bodies)
resolveAABB(bodyA, bodyB)
```

### AABB overlap from entity positions

```nova
// Manual rectangle overlap check (useful for entity vs tilemap)
fn rectsOverlap(ax: Float, ay: Float, aw: Float, ah: Float,
                bx: Float, by: Float, bw: Float, bh: Float) -> Bool {
    return ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

// AABB::new(x, y, w, h) — for static geometry
let wall = AABB::new(100.0, 400.0, 200.0, 20.0)
let pen  = AABB::new(player.pos.x, player.pos.y, player.size.x, player.size.y)
            .overlap(wall)
// pen is (Float, Float): how much to push out
if pen.x != 0.0 || pen.y != 0.0 {
    player.pos.x += pen.x
    player.pos.y += pen.y
    if pen.y < 0.0 { player.vel.y = 0.0 }   // landed
    if pen.x != 0.0 { player.vel.x = 0.0 }  // hit wall
}
```

### Raycasting

```nova
let ray = Ray2::new(px, py, dirX, dirY)   // dir should be normalised
let hit = ray.castAABB(wall)
if hit.hit {
    // hit.point  — world-space intersection point
    // hit.normal — surface normal
    // hit.t      — parametric distance
}
```

### Restitution guide

| Value | Effect |
|---|---|
| `0.0` | Rigid — no bounce |
| `0.5–0.8` | Bouncy |
| `1.0` | Perfectly elastic — no energy loss |

---

## 8. Camera {#8-camera}

`Camera2D` maps world coordinates to screen pixels with zoom, smooth-follow, and
screen-shake built in.

```nova
import super.std.camera

let cam = Camera2D::new(800, 600)  // screen width, height
cam.setZoom(1.5)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    cam.update(dt)

    // Smooth-follow a target (lerp speed 6.0)
    cam.follow(player.pos, 6.0, dt)

    // Draw world objects through the camera transform
    world.forEach(fn(e: Entity) {
        cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (200, 200, 200))
    })

    // Screen shake on explosion
    cam.shake(12.0, 0.4)   // intensity=12.0, duration=0.4s
}
```

### Coordinate conversion

```nova
// Screen → world (for mouse targeting)
let (mx, my)   = InputMap::mousePos()
let worldMouse = cam.screenToWorld(Vec2::new(Cast::float(mx).unwrap(),
                                             Cast::float(my).unwrap()))

// World → screen (for HUD anchored to world objects)
let screenPos = cam.worldToScreen(entity.pos)
```

### Culling off-screen entities

```nova
world.forEach(fn(e: Entity) {
    if cam.isVisible(e.pos.x, e.pos.y, 64.0) {   // 64-pixel margin
        cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, color)
    }
})
```

### Camera draw helpers

```nova
cam.drawRect(x, y, w, h, color)         // rectangle in world space
cam.drawCircle(x, y, radius, color)     // circle in world space
cam.drawLine(x1, y1, x2, y2, color)     // line in world space
```

---

## 9. Timers {#9-timers}

```nova
import super.std.timer

// Three timer modes:
let fireRate   = Timer::cooldown(0.15)   // repeats: ready after 150ms
let bossTimer  = Timer::once(30.0)       // fires once after 30s
let blinkTimer = Timer::repeating(0.5)  // fires every 500ms, auto-resets
```

### Usage

```nova
while raylib::rendering() {
    let dt = raylib::getFrameTime()

    // Shooting cooldown
    fireRate.update(dt)
    if keys.isHeld("fire") && fireRate.ready() {
        spawnBullet(player.pos)
        // ready() resets the timer automatically on cooldown/repeating timers
    }

    // Win condition — fires once
    bossTimer.update(dt)
    if bossTimer.isDone() {
        triggerBossEscape()
    }

    // Blinking effect
    blinkTimer.update(dt)
    if blinkTimer.progress() > 0.5 {
        drawHealthBar()
    }
}
```

### Timer method reference

| Method | Description |
|---|---|
| `update(dt)` | Advance timer by `dt` seconds |
| `ready()` | `true` if elapsed ≥ duration; resets elapsed on `cooldown`/`repeating` |
| `isDone()` | `true` if elapsed ≥ duration (does NOT reset) |
| `progress()` | `0.0` → `1.0` fraction of current cycle |
| `activate()` | Manually arm a `cooldown` timer (e.g., coyote time) |
| `reset()` | Reset elapsed to 0 |

---

## 10. Tweens {#10-tweens}

Tweens animate a scalar value from `start` to `end` over a `duration`.

```nova
import super.std.tween

let openDoor   = Tween::smooth(0.0, 80.0, 0.6)      // smooth ease-in-out
let fadeIn     = Tween::linear(0.0, 255.0, 1.0)     // linear
let flashAlpha = Tween::easeOut(255.0, 0.0, 0.3)    // ease-out

while raylib::rendering() {
    let dt = raylib::getFrameTime()

    let doorY = openDoor.update(dt)     // returns current Float value
    let alpha = fadeIn.update(dt)

    // Ping-pong for pulsing effects
    if glowTween.isDone() { glowTween.ping() }
    let glow = glowTween.update(dt)
}
```

### Tween method reference

| Method | Description |
|---|---|
| `update(dt)` | Advance and return current value |
| `isDone()` | `true` when value has reached `end` |
| `ping()` | Reverse direction (start ↔ end swap) |
| `reset()` | Restart from beginning |
| `value()` | Current value without advancing |

### Easing cheatsheet

| Constructor | Feel |
|---|---|
| `Tween::linear(s, e, d)` | Constant speed — mechanical |
| `Tween::easeIn(s, e, d)` | Starts slow, ends fast — building up |
| `Tween::easeOut(s, e, d)` | Starts fast, ends slow — landing/settling |
| `Tween::smooth(s, e, d)` | Smooth start and end — UI animations |
| `Tween::easeOutBounce(s, e, d)` | Bouncy landing |
| `Tween::easeOutElastic(s, e, d)` | Spring overshoot |
| `Tween::easeOutBack(s, e, d)` | Slight overshoot — snappy buttons |
| `Tween::sineInOut(s, e, d)` | Gentle sine wave — breathing, floating |

---

## 11. Vec2 Math {#11-vec2-math}

`Vec2` is a heap-allocated 2D float vector captured by reference in closures.

```nova
import super.std.vec2

let v = Vec2::new(3.0, 4.0)

let len  = v.length()                    // 5.0
let norm = v.normalized()               // (0.6, 0.8)
let sc   = v.scale(2.0)                 // (6.0, 8.0)
let sum  = v.add(Vec2::new(1.0, 2.0))   // (4.0, 6.0)
let diff = v.sub(Vec2::new(1.0, 2.0))   // (2.0, 2.0)
let dot  = v.dot(Vec2::new(1.0, 0.0))   // 3.0
let dist = v.distance(Vec2::new(0.0, 0.0)) // 5.0

// Directional helpers
let angle = v.angle()                   // atan2(y, x) in radians
let fromA = Vec2::fromAngle(1.5708)     // unit vector from angle
let refl  = v.reflect(normal)          // reflect off a surface normal
```

### Aim direction toward a target

```nova
let toTarget = target.pos.sub(e.pos)    // vector pointing at target
if toTarget.length() > 0.5 {
    let move = toTarget.normalized().scale(SPEED * dt)
    e.pos.x += move.x
    e.pos.y += move.y
}
```

### Fire a bullet toward the mouse

```nova
let (mx, my) = InputMap::mousePos()
let worldMouse = cam.screenToWorld(Vec2::new(Cast::float(mx).unwrap(),
                                             Cast::float(my).unwrap()))
let dir = worldMouse.sub(player.pos).normalized()
let b = world.spawn(player.pos.x, player.pos.y, "bullet")
b.vel.x = dir.x * BULLET_SPEED
b.vel.y = dir.y * BULLET_SPEED
```

---

## 12. Tilemaps with Grid {#12-tilemaps-with-grid}

```nova
import super.std.grid

let MAP_W = 30
let MAP_H = 20
let TILE  = 32

let map = Grid::new(MAP_W, MAP_H, 0)   // width, height, default value

// Fill operations
map.fillRect(0, 0, MAP_W, MAP_H, 1)          // fill with walls
map.fillRect(1, 1, MAP_W-2, MAP_H-2, 0)     // hollow interior

// Get / set
map.set(x, y, value)
let v = map.get(x, y)

// Pathfinding (BFS)
let path = map.bfs(startX, startY, goalX, goalY, fn(v: Any) -> Bool { v == 0 })

// Draw
let colorMap = fn(v: Any) -> (Int, Int, Int) {
    if v == 1 { return (80, 80, 90) }    // wall
    return (50, 50, 55)                  // floor
}
map.draw(0, 0, TILE, colorMap)
map.drawLines(0, 0, TILE, (30, 30, 35))   // grid lines
```

### Snapping to grid

```nova
fn worldToTile(wx: Float, wy: Float, tileSize: Int) -> (Int, Int) {
    return (Cast::int(wx).unwrap() / tileSize,
            Cast::int(wy).unwrap() / tileSize)
}

fn tileCenter(tx: Int, ty: Int, tileSize: Int) -> Vec2 {
    let ts = Cast::float(tileSize).unwrap()
    return Vec2::new(Cast::float(tx).unwrap() * ts + ts / 2.0,
                     Cast::float(ty).unwrap() * ts + ts / 2.0)
}
```

---

## 13. Procedural Generation with Noise {#13-noise}

```nova
import super.std.noise

// Terrain heightmap
for let row = 0; row < MAP_H; row += 1 {
    for let col = 0; col < MAP_W; col += 1 {
        let nx = Cast::float(col).unwrap() * 0.08
        let ny = Cast::float(row).unwrap() * 0.08
        let h  = fbm(nx, ny, SEED, 5, 2.0, 0.5)
        map.set(col, row, if h > 0.6 { 1 } else { 0 })
    }
}
```

### Noise recipe guide

| Effect | Function | Scale | Notes |
|---|---|---|---|
| Terrain heights | `fbm` | 0.005–0.02 | octaves=6, lac=2, gain=0.5 |
| Caves | `smoothNoise` | 0.04–0.1 | threshold ~0.5 |
| Mountain ridges | `ridged` | 0.008 | octaves=5 |
| Cloud texture | `domain` | 0.002–0.006 | strength=1.5 |
| Grass variation | `valueNoise` | 0.2–0.5 | direct use |
| Particle turbulence | `fbm` | per-particle | octaves=3, gain=0.6 |

### Visualise noise as colour

```nova
let c = noiseToColor(h, (20, 80, 200), (200, 230, 120))
raylib::drawRectangle(col * TILE, row * TILE, TILE, TILE, c)
```

---

## 14. Sprites and Pixel Art {#14-sprites}

### Load from file

```nova
let hero = raylib::loadSprite("assets/hero.png", 32, 1)
raylib::drawSprite(hero, px, py)
```

`loadSprite(path, height, frameCount)` — for a static image use `frameCount = 1`.

### Procedural pixel art

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
```

### Animated sprites

```nova
let walkSprite = raylib::loadSprite("assets/walk.png", 32, 4)  // 4 frames
let frame = Box(0)
let frameTimer = Timer::cooldown(0.1)   // 10 FPS animation

while raylib::rendering() {
    frameTimer.update(raylib::getFrameTime())
    if frameTimer.ready() {
        frame.value = (frame.value + 1) % 4
    }
    raylib::drawSpriteFrame(walkSprite, frame.value, px, py)
}
```

---

## 15. Audio {#15-audio}

### Setup

Call `raylib::initAudio()` once before loading any audio files.

```nova
raylib::init("Game", 800, 600, 60)
raylib::initAudio()
```

### Sound effects

```nova
let jumpSfx = raylib::loadSound("assets/jump.wav")
let hitSfx  = raylib::loadSound("assets/hit.wav")

// Play
raylib::playSound(jumpSfx)

// Adjust per-sound
raylib::setSoundVolume(jumpSfx, 0.8)
raylib::setSoundPitch(hitSfx, 1.5)
```

### Background music

```nova
let bgm = raylib::loadMusic("assets/bgm.ogg")
raylib::setMusicLooping(bgm, true)
raylib::playMusic(bgm)

while raylib::rendering() {
    raylib::updateMusic(bgm)   // ← REQUIRED every frame or music stops
    // ...
}

raylib::closeAudio()   // always call before exit
```

### Music controls

```nova
raylib::pauseMusic(bgm)
raylib::resumeMusic(bgm)
raylib::stopMusic(bgm)
raylib::setMusicVolume(bgm, 0.5)
let played = raylib::getMusicTimePlayed(bgm)
raylib::seekMusic(bgm, 0.0)
```

---

## 16. HUD and UI {#16-hud-and-ui}

### Drawing primitives

```nova
raylib::clear((20, 20, 40))                                          // background
raylib::drawRectangle(x, y, w, h, color)                            // filled rect
raylib::drawRectangleLines(x, y, w, h, color)                       // outline
raylib::drawRoundedRectangle(x, y, w, h, 0.3, color)               // rounded
raylib::drawCircle(cx, cy, r, color)                                 // filled circle
raylib::drawLine(x1, y1, x2, y2, color)                             // line
raylib::drawLineThick(x1, y1, x2, y2, 3.0, color)                  // thick line
raylib::drawTriangle(x1, y1, x2, y2, x3, y3, color)                // CCW winding
raylib::drawText("Score: 100", 10, 10, 20, (255, 255, 255))        // text
raylib::drawFPS(10, 560)                                             // FPS counter
let w = raylib::measureText("Game Over", 40)                        // text width
```

All colours are `(Int, Int, Int)` RGB tuples, values 0–255.

> **Draw order**: things drawn later appear on top. Always draw background first,
> then world, then HUD/UI.

### Health bar

```nova
fn drawHealthBar(x: Int, y: Int, w: Int, h: Int, current: Int, maxHp: Int) {
    raylib::drawRectangle(x, y, w, h, (60, 60, 60))
    let fillW = (w * current) / maxHp
    let color = if current * 3 < maxHp       { (255, 0, 0)    }
                elif current * 3 < maxHp * 2 { (255, 200, 0)  }
                else                          { (0, 255, 0)    }
    raylib::drawRectangle(x, y, fillW, h, color)
    raylib::drawRectangleLines(x, y, w, h, (255, 255, 255))
}
```

### Centred button

```nova
fn drawButton(x: Int, y: Int, w: Int, h: Int, text: String, hover: Bool) {
    let bg = if hover { (80, 120, 200) } else { (50, 80, 150) }
    raylib::drawRoundedRectangle(x, y, w, h, 0.3, bg)
    let tw = raylib::measureText(text, 20)
    raylib::drawText(text, x + (w - tw) / 2, y + (h - 20) / 2, 20, (255, 255, 255))
}

let mouse = raylib::mousePosition()
let hov = mouse[0] >= btnX && mouse[0] <= btnX+btnW &&
          mouse[1] >= btnY && mouse[1] <= btnY+btnH
drawButton(btnX, btnY, btnW, btnH, "Play", hov)
```

### Score text (centered horizontally)

```nova
let txt = "Score: " + Cast::string(score.value)
let tw  = raylib::measureText(txt, 28)
raylib::drawText(txt, (SCREEN_W - tw) / 2, 20, 28, (255, 230, 80))
```

---

## 17. Advanced Patterns {#17-advanced-patterns}

### 17.1 Enum-based entity kinds {#171-enum-based-entities}

For games that need more than one type-specific data slot per entity, encode the
entity kind in an enum with associated data:

```nova
enum Tag { Player, Enemy: Int, Pickup: Int, Bullet }

struct Entity {
    x: Int, y: Int, w: Int, h: Int,
    vx: Int, vy: Int,
    tag: Tag,
    active: Bool,
}

fn mkEnemy(x: Int, y: Int, hp: Int) -> Entity {
    return Entity { x: x, y: y, w: 24, h: 24, vx: -2, vy: 0,
                    tag: Tag::Enemy(hp), active: true }
}

// Collision response
match a.tag {
    Bullet() => {
        match b.tag {
            Enemy(hp) => {
                a.active = false
                if hp <= 1 { b.active = false }
                else { b.tag = Tag::Enemy(hp - 1) }
                score.value += 10
            }
            _ => {}
        }
    }
    _ => {}
}
```

### 17.2 Vtable dispatch with `->` {#172-vtable-dispatch}

Store a `draw` or `update` closure **as a field** on each struct.
The `->` operator calls it, dispatching to the entity's own implementation at runtime:

```nova
struct PlayerEntity {
    x: Int, y: Int, hp: Int,
    draw: fn(PlayerEntity),
}

fn makePlayer(x: Int, y: Int) -> PlayerEntity {
    return PlayerEntity {
        x: x, y: y, hp: 3,
        draw: fn(self: PlayerEntity) {
            raylib::drawRectangle(self.x, self.y, 32, 32, (50, 200, 255))
        }
    }
}

// Define a Dyn type to accept any entity that has a draw field
type drawable = Dyn(T = draw: fn($T))

// Single draw loop — no match, no if/else chains
for item in allDrawables { item->draw() }
```

Adding a new entity type means writing one new struct and constructor function.
**No existing code changes.**

### 17.3 Object pooling {#173-object-pooling}

Pre-allocate a fixed pool; reuse slots with an `active` flag to avoid GC pressure:

```nova
struct Bullet { x: Int, y: Int, vx: Int, vy: Int, active: Int }

let bulletPool = []: Bullet
for let i = 0; i < 256; i += 1 {
    bulletPool.push(Bullet { x: 0, y: 0, vx: 0, vy: -8, active: 0 })
}

fn spawnBullet(px: Int, py: Int, vx: Int, vy: Int) {
    for b in bulletPool {
        if b.active == 0 {
            b.x = px; b.y = py; b.vx = vx; b.vy = vy; b.active = 1
            return
        }
    }
}

fn updateBullets(dt: Float) {
    for b in bulletPool {
        if b.active == 1 {
            b.x += Cast::int(Cast::float(b.vx).unwrap() * dt).unwrap()
            b.y += Cast::int(Cast::float(b.vy).unwrap() * dt).unwrap()
            if b.x < 0 || b.x > 800 || b.y < 0 || b.y > 600 { b.active = 0 }
        }
    }
}
```

### 17.4 Spatial grid (broad-phase collision) {#174-spatial-grid}

O(n²) bullet-vs-enemy checks become O(n) with a spatial hash:

```nova
// Divide world into cells; insert enemies; query per bullet
fn insertIntoGrid(grid: [[Int]], e: Entity, cellSize: Int, cols: Int) {
    let ci = (Cast::int(e.pos.y).unwrap() / cellSize) * cols +
              Cast::int(e.pos.x).unwrap() / cellSize
    if ci >= 0 && ci < grid.len() { grid[ci].push(e.id) }
}

fn nearbyIds(grid: [[Int]], wx: Float, wy: Float, cellSize: Int, cols: Int, rows: Int) -> [Int] {
    let result = []: Int
    let col = Cast::int(wx).unwrap() / cellSize
    let row = Cast::int(wy).unwrap() / cellSize
    for let dr = -1; dr <= 1; dr += 1 {
        for let dc = -1; dc <= 1; dc += 1 {
            let r = row + dr; let c = col + dc
            if r >= 0 && r < rows && c >= 0 && c < cols {
                for id in grid[r * cols + c] { result.push(id) }
            }
        }
    }
    return result
}
```

### 17.5 Screen-stack without SceneManager {#175-screen-stack}

For simple games that do not need `SceneManager`, model screens as an enum stack:

```nova
enum Screen { MainMenu, Playing, Paused, GameOver: Int }

let stack = []: Screen

fn pushScreen(s: Screen) { stack.push(s) }
fn popScreen() { if stack.len() > 0 { stack.pop() } }
fn currentScreen() -> Option(Screen) {
    if stack.len() == 0 { return None(Screen) }
    return Some(stack[stack.len() - 1])
}

pushScreen(Screen::MainMenu())

while raylib::rendering() {
    if let s = currentScreen() {
        match s {
            Playing() => { updateGame(); drawWorld() }
            Paused()  => { drawWorld(); drawPauseOverlay() }
            MainMenu() => { drawMainMenu() }
            GameOver(score) => { drawGameOver(score) }
        }
    }
}
```

---

## 18. Tips and Tricks {#18-tips-and-tricks}

### 18.1 Frame animation {#181-frame-animation}

```nova
let frame   = Box(0)
let counter = Box(0)
let TICKS_PER_FRAME = 8
let FRAME_COUNT     = 4

// In update:
counter.value += 1
if counter.value >= TICKS_PER_FRAME {
    counter.value = 0
    frame.value = (frame.value + 1) % FRAME_COUNT
}

// In draw:
raylib::drawSpriteFrame(sprite, frame.value, px, py)
```

Or use a `Timer::cooldown` to drive frame advancement frame-rate independently.

### 18.2 Screen shake (manual) {#182-screen-shake}

```nova
let shakeIntensity = Box(0.0)
let shakeDuration  = Box(0.0)

fn triggerShake(intensity: Float, duration: Float) {
    shakeIntensity.value = intensity
    shakeDuration.value  = duration
}

// In update:
if shakeDuration.value > 0.0 {
    shakeDuration.value -= dt
}

// In draw — offset all world draws by shake amount:
let ox = if shakeDuration.value > 0.0 {
    Cast::int(shakeIntensity.value * (Cast::float(raylib::getFPS()).unwrap() % 7.0 - 3.0) / 7.0).unwrap()
} else { 0 }
```

Or use `Camera2D::shake(intensity, duration)` which handles this automatically.

### 18.3 Hit flash {#183-hit-flash}

```nova
let flashTimer = Tween::linear(1.0, 0.0, 0.4)   // alpha/intensity fades out

// On hit:
// flashTimer.reset()

// In draw:
let alpha = Cast::int(flashTimer.value() * 255.0).unwrap()
if alpha > 10 {
    raylib::drawRectangle(px, py, pw, ph, (255, 255, alpha))
}
flashTimer.update(dt)
```

### 18.4 Wave spawner {#184-wave-spawner}

```nova
let wave      = Box(1)
let waveTimer = Timer::repeating(5.0)   // new wave every 5s

// In update:
waveTimer.update(dt)
if waveTimer.ready() {
    wave.value += 1
    let count = wave.value * 3
    for let i = 0; i < count; i += 1 {
        spawnEnemy()
    }
}
```

### 18.5 Floating score popups {#185-score-popups}

```nova
struct Popup { x: Float, y: Float, text: String, life: Float, active: Bool }
let popups = []: Popup

fn spawnPopup(x: Float, y: Float, pts: Int) {
    popups.push(Popup { x: x, y: y, text: "+" + Cast::string(pts),
                        life: 1.0, active: true })
}

// In update:
for p in popups {
    if p.active {
        p.y -= 40.0 * dt
        p.life -= dt
        if p.life <= 0.0 { p.active = false }
    }
}
popups = popups.filter(): |p: Popup| p.active

// In draw:
for p in popups {
    let alpha = Cast::int(p.life * 255.0).unwrap()
    raylib::drawText(p.text, Cast::int(p.x).unwrap(), Cast::int(p.y).unwrap(),
                     18, (255, 230, alpha))
}
```

### 18.6 Particle burst {#186-particles}

```nova
struct Particle { x: Float, y: Float, vx: Float, vy: Float, life: Float, active: Bool }
let particles = []: Particle
for let i = 0; i < 256; i += 1 {
    particles.push(Particle { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, life: 0.0, active: false })
}

fn burst(cx: Float, cy: Float, count: Int) {
    let spawned = Box(0)
    for p in particles {
        if !p.active && spawned.value < count {
            p.x  = cx; p.y = cy
            p.vx = Cast::float((spawned.value * 7) % 11 - 5).unwrap() * 30.0
            p.vy = Cast::float((spawned.value * 3) % 9 - 6).unwrap() * 30.0
            p.life = 0.5
            p.active = true
            spawned.value += 1
        }
    }
}

// In update:
for p in particles {
    if p.active {
        p.x += p.vx * dt; p.y += p.vy * dt
        p.life -= dt
        if p.life <= 0.0 { p.active = false }
    }
}

// In draw:
for p in particles {
    if p.active {
        let br = Cast::int(p.life * 510.0).unwrap()
        raylib::drawCircle(Cast::int(p.x).unwrap(), Cast::int(p.y).unwrap(), 3, (br, br, br))
    }
}
```

### 18.7 Debug overlay {#187-debug-overlay}

```nova
let debugOn = Box(false)

// In update:
if raylib::isKeyReleased("F3") { debugOn.value = !debugOn.value }

// In draw (after normal draw):
if debugOn.value {
    world.forEachTagged("enemy", fn(e: Entity) {
        raylib::drawRectangleLines(Cast::int(e.pos.x).unwrap(),
                                   Cast::int(e.pos.y).unwrap(),
                                   Cast::int(e.size.x).unwrap(),
                                   Cast::int(e.size.y).unwrap(), (0, 255, 0))
    })
    raylib::drawText("Enemies: " + Cast::string(world.countAlive("enemy")), 10, 40, 16, (0, 255, 0))
    raylib::drawText("FPS: " + Cast::string(raylib::getFPS()), 10, 60, 16, (0, 255, 0))
}
```

### 18.8 Integer lerp and clamp {#188-lerp-clamp}

```nova
fn lerp(a: Int, b: Int, t: Int, tmax: Int) -> Int {
    return a + (b - a) * t / tmax
}

fn clamp(v: Int, lo: Int, hi: Int) -> Int {
    if v < lo { return lo }
    if v > hi { return hi }
    return v
}

// Smooth health-bar display
displayHp = lerp(displayHp, realHp, 1, 4)

// Keep player in bounds
player.x = clamp(player.x, 0, SCREEN_W - playerW)
```

---

## 19. Performance Tips {#19-performance-tips}

| Problem | Solution |
|---|---|
| GC spikes from spawning bullets | Object pool — reuse slots with `active` flag |
| O(n²) bullet-enemy collision | Spatial grid — insert enemies, query per bullet |
| Temporary list allocated each frame | Pre-allocate outside loop, call `clear()` inside |
| Off-screen entities still drawn | Cull with `cam.isVisible(x, y, margin)` |
| Large tilemap drawn per-tile each frame | Pre-render to texture; blit each frame |
| Per-pixel noise generation is slow | Pre-generate into a `Grid`; cache as colours |
| `Cast::string` in tight draw loop | Cache the string; only regenerate when value changes |
| Music stops mid-game | Ensure `raylib::updateMusic(id)` is called every frame |

---

## 20. Complete Example — Breakout {#20-example-breakout}

A full Breakout game demonstrating: `SceneManager`, `EntityWorld`, `InputMap`,
`Timer`, `Tween`, `Vec2`, `Box`-wrapped mutable scalars, forward declarations,
and manual ball physics.

```nova
module breakout

import super.std.scene
import super.std.entity
import super.std.input
import super.std.timer
import super.std.tween
import super.std.vec2

// ── Constants ─────────────────────────────────────────────────────
let W          = 800
let H          = 600
let PADDLE_W   = 100
let PADDLE_H   = 16
let PADDLE_Y   = H - 48
let BALL_R     = 10.0
let BALL_SPEED = 340.0
let MAX_LIVES  = 3
let BRICK_COLS = 10
let BRICK_ROWS = 5
let BRICK_W    = 64
let BRICK_H    = 22
let BRICK_PAD  = 4
let BRICK_OFF_X = (W - BRICK_COLS * (BRICK_W + BRICK_PAD)) / 2
let BRICK_OFF_Y = 60
let TOTAL_BRICKS = BRICK_COLS * BRICK_ROWS

// ── Forward declarations ──────────────────────────────────────────
fn makeMenuScene() -> Scene
fn makePlayScene() -> Scene
fn makeGameOverScene() -> Scene
fn makeWinScene() -> Scene

// ── Shared state ──────────────────────────────────────────────────
let mgr        = SceneManager::empty()
let paddleX    = Box(W / 2 - PADDLE_W / 2)
let score      = Box(0)
let lives      = Box(MAX_LIVES)
let hitCount   = Box(0)
let ballOnBoard = Box(true)
let ballVel    = Vec2::new(BALL_SPEED * 0.6, -BALL_SPEED * 0.8)
let world      = EntityWorld::new()
let scoreFlash = Tween::linear(255.0, 0.0, 0.4)
let keys       = InputMap::new()

// ── Scene: Menu ───────────────────────────────────────────────────
fn makeMenuScene() -> Scene {
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("fire") { mgr.switch(makePlayScene()) }
        },
        fn() {
            raylib::clear((10, 20, 40))
            let t1 = "BREAKOUT"
            raylib::drawText(t1, (W - raylib::measureText(t1, 60)) / 2, 180, 60, (100, 200, 255))
            raylib::drawText("SPACE to play", (W - raylib::measureText("SPACE to play", 22)) / 2,
                             310, 22, (180, 180, 180))
        }
    )
}

// ── Scene: Play ───────────────────────────────────────────────────
fn makePlayScene() -> Scene {
    // Reset per-game state
    score.value     = 0
    lives.value     = MAX_LIVES
    hitCount.value  = 0
    ballOnBoard.value = true
    paddleX.value   = W / 2 - PADDLE_W / 2
    ballVel.x       = BALL_SPEED * 0.6
    ballVel.y       = -BALL_SPEED * 0.8

    // Clear old entities
    world.forEach(fn(e: Entity) { e.alive = false })
    world.update(0.0)

    // Spawn ball
    let ball = world.spawn(Cast::float(W / 2).unwrap(),
                           Cast::float(H / 2).unwrap(), "ball")
    ball.size = Vec2::new(BALL_R * 2.0, BALL_R * 2.0)

    // Spawn bricks
    for let row = 0; row < BRICK_ROWS; row += 1 {
        for let col = 0; col < BRICK_COLS; col += 1 {
            let bx = Cast::float(BRICK_OFF_X + col * (BRICK_W + BRICK_PAD)).unwrap()
            let by = Cast::float(BRICK_OFF_Y + row * (BRICK_H + BRICK_PAD)).unwrap()
            let b = world.spawn(bx, by, "brick")
            b.size = Vec2::new(Cast::float(BRICK_W).unwrap(), Cast::float(BRICK_H).unwrap())
            b.data = Cast::float(row + 1).unwrap()  // health (row 0 = 1 hit, row 4 = 5 hits)
        }
    }

    keys.bindKey("left",  "Left")
    keys.bindKey("right", "Right")
    keys.bindKey("fire",  "Space")

    return Scene::new(
        fn(dt: Float) {
            // Paddle movement
            let dx = keys.axis("left", "right")
            paddleX.value = Cast::int(Cast::float(paddleX.value).unwrap() + dx * 360.0 * dt).unwrap()
            if paddleX.value < 0 { paddleX.value = 0 }
            if paddleX.value > W - PADDLE_W { paddleX.value = W - PADDLE_W }

            // Ball movement (manual integration)
            let balls = world.query("ball")
            if balls.len() > 0 && ballOnBoard.value {
                let b = balls[0]
                b.pos.x = b.pos.x + ballVel.x * dt
                b.pos.y = b.pos.y + ballVel.y * dt

                // Wall bounces
                if b.pos.x <= 0.0 { b.pos.x = 0.0; ballVel.x = -ballVel.x }
                if b.pos.x + BALL_R * 2.0 >= Cast::float(W).unwrap() {
                    b.pos.x = Cast::float(W).unwrap() - BALL_R * 2.0
                    ballVel.x = -ballVel.x
                }
                if b.pos.y <= 0.0 { b.pos.y = 0.0; ballVel.y = -ballVel.y }

                // Paddle collision
                let px = Cast::float(paddleX.value).unwrap()
                let py = Cast::float(PADDLE_Y).unwrap()
                if b.pos.y + BALL_R * 2.0 >= py && b.pos.y < py + Cast::float(PADDLE_H).unwrap() {
                    if b.pos.x + BALL_R * 2.0 > px && b.pos.x < px + Cast::float(PADDLE_W).unwrap() {
                        ballVel.y = -Float::abs(ballVel.y)
                        let relX = (b.pos.x + BALL_R - px) / Cast::float(PADDLE_W).unwrap() - 0.5
                        ballVel.x = relX * BALL_SPEED * 2.0
                    }
                }

                // Ball fell off bottom
                if b.pos.y > Cast::float(H + 20).unwrap() {
                    lives.value -= 1
                    ballOnBoard.value = false
                    if lives.value <= 0 { mgr.switch(makeGameOverScene()) }
                    else {
                        b.pos.x = Cast::float(W / 2).unwrap()
                        b.pos.y = Cast::float(H / 2).unwrap()
                        ballVel.x = BALL_SPEED * 0.6
                        ballVel.y = -BALL_SPEED * 0.8
                        ballOnBoard.value = true
                    }
                }

                // Brick collision
                world.forEachTagged("brick", fn(br: Entity) {
                    if b.pos.x + BALL_R * 2.0 > br.pos.x &&
                       b.pos.x < br.pos.x + br.size.x &&
                       b.pos.y + BALL_R * 2.0 > br.pos.y &&
                       b.pos.y < br.pos.y + br.size.y {
                        let overlapL = b.pos.x + BALL_R * 2.0 - br.pos.x
                        let overlapR = br.pos.x + br.size.x - b.pos.x
                        let overlapT = b.pos.y + BALL_R * 2.0 - br.pos.y
                        let overlapB = br.pos.y + br.size.y - b.pos.y
                        let minH = if overlapL < overlapR { overlapL } else { overlapR }
                        let minV = if overlapT < overlapB { overlapT } else { overlapB }
                        if minH < minV { ballVel.x = -ballVel.x }
                        else { ballVel.y = -ballVel.y }
                        br.data -= 1.0
                        if br.data <= 0.0 { br.alive = false }
                        score.value += 10
                        hitCount.value += 1
                        scoreFlash.reset()
                        if hitCount.value >= TOTAL_BRICKS { mgr.switch(makeWinScene()) }
                    }
                })
            }

            world.update(0.0)   // purge dead bricks only
        },
        fn() {
            raylib::clear((8, 12, 28))

            // Bricks
            world.forEachTagged("brick", fn(br: Entity) {
                let hp = Cast::int(br.data).unwrap()
                let r = 60 + hp * 35; let g = 40 + hp * 20
                raylib::drawRectangle(Cast::int(br.pos.x).unwrap(), Cast::int(br.pos.y).unwrap(),
                                      BRICK_W - 2, BRICK_H - 2, (r, g, 60))
            })

            // Ball
            world.forEachTagged("ball", fn(b: Entity) {
                raylib::drawCircle(Cast::int(b.pos.x + BALL_R).unwrap(),
                                   Cast::int(b.pos.y + BALL_R).unwrap(),
                                   Cast::int(BALL_R).unwrap(), (255, 230, 80))
            })

            // Paddle
            raylib::drawRoundedRectangle(paddleX.value, PADDLE_Y, PADDLE_W, PADDLE_H,
                                         0.4, (80, 180, 255))

            // HUD
            raylib::drawText("Score: " + Cast::string(score.value), 10, 10, 20, (255, 255, 255))
            for let i = 0; i < lives.value; i += 1 {
                raylib::drawCircle(W - 20 - i * 24, 18, 8, (255, 80, 80))
            }

            // Score flash
            let fa = Cast::int(scoreFlash.value()).unwrap()
            if fa > 5 {
                raylib::drawText("+10", paddleX.value + PADDLE_W / 2 - 15, PADDLE_Y - 28,
                                 20, (255, 230, fa))
            }
            scoreFlash.update(raylib::getFrameTime())
        }
    )
}

// ── Scene: Game Over ──────────────────────────────────────────────
fn makeGameOverScene() -> Scene {
    let finalScore = score.value
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("fire") { mgr.switch(makeMenuScene()) }
        },
        fn() {
            raylib::clear((30, 0, 0))
            raylib::drawText("GAME OVER", (W - raylib::measureText("GAME OVER", 60)) / 2,
                             200, 60, (255, 80, 80))
            raylib::drawText("Score: " + Cast::string(finalScore),
                             (W - raylib::measureText("Score: " + Cast::string(finalScore), 28)) / 2,
                             290, 28, (255, 255, 255))
            raylib::drawText("SPACE to menu",
                             (W - raylib::measureText("SPACE to menu", 20)) / 2,
                             350, 20, (160, 160, 160))
        }
    )
}

// ── Scene: Win ────────────────────────────────────────────────────
fn makeWinScene() -> Scene {
    let finalScore = score.value
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("fire") { mgr.switch(makeMenuScene()) }
        },
        fn() {
            raylib::clear((0, 30, 0))
            raylib::drawText("YOU WIN!", (W - raylib::measureText("YOU WIN!", 60)) / 2,
                             200, 60, (80, 255, 120))
            raylib::drawText("Score: " + Cast::string(finalScore),
                             (W - raylib::measureText("Score: " + Cast::string(finalScore), 28)) / 2,
                             290, 28, (255, 255, 255))
            raylib::drawText("SPACE to menu",
                             (W - raylib::measureText("SPACE to menu", 20)) / 2,
                             350, 20, (160, 160, 160))
        }
    )
}

// ── Main ──────────────────────────────────────────────────────────
raylib::init("Breakout", W, H, 60)
mgr.switch(makeMenuScene())
while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

---

## 21. Complete Example — Top-Down Shooter {#21-example-shooter}

A full top-down shooter demonstrating: `SceneManager`, `EntityWorld`, `Camera2D`,
`InputMap`, `Timer`, `Tween`, wave system, bullet movement via `forEachTagged`,
and `Box`-wrapped mutable scalars.

```nova
module shooter

import super.std.scene
import super.std.entity
import super.std.input
import super.std.camera
import super.std.timer
import super.std.tween
import super.std.vec2

// ── Constants ─────────────────────────────────────────────────────
let SW = 900
let SH = 600
let PLAYER_SPEED   = 200.0
let BULLET_SPEED   = 500.0
let BULLET_LIFETIME = 2.0
let ENEMY_BASE_SPEED = 60.0
let PLAYER_HP_MAX  = 5
let WAVE_INTERVAL  = 8.0

// ── Forward declarations ──────────────────────────────────────────
fn makeMenuScene() -> Scene
fn makePlayScene() -> Scene
fn makeGameOverScene() -> Scene

// ── Shared state ──────────────────────────────────────────────────
let mgr         = SceneManager::empty()
let world       = EntityWorld::new()
let cam         = Camera2D::new(SW, SH)
let keys        = InputMap::new()
let score       = Box(0)
let wave        = Box(1)
let playerHp    = Box(PLAYER_HP_MAX)
let playerAlive = Box(true)
let fireTimer   = Timer::cooldown(0.12)
let waveTimer   = Timer::repeating(WAVE_INTERVAL)
let hitFlash    = Tween::linear(180.0, 0.0, 0.35)

// ── Scene: Menu ───────────────────────────────────────────────────
fn makeMenuScene() -> Scene {
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("fire") { mgr.switch(makePlayScene()) }
        },
        fn() {
            raylib::clear((10, 10, 24))
            raylib::drawText("TOP-DOWN SHOOTER",
                             (SW - raylib::measureText("TOP-DOWN SHOOTER", 48)) / 2,
                             180, 48, (100, 200, 255))
            raylib::drawText("WASD to move  •  Mouse to aim  •  Click to fire",
                             (SW - raylib::measureText("WASD to move  •  Mouse to aim  •  Click to fire", 18)) / 2,
                             280, 18, (160, 160, 160))
            raylib::drawText("SPACE to start",
                             (SW - raylib::measureText("SPACE to start", 22)) / 2,
                             340, 22, (220, 220, 80))
        }
    )
}

// ── Scene: Play ───────────────────────────────────────────────────
fn makePlayScene() -> Scene {
    score.value  = 0
    wave.value   = 1
    playerHp.value    = PLAYER_HP_MAX
    playerAlive.value = true

    world.forEach(fn(e: Entity) { e.alive = false })
    world.update(0.0)

    let player = world.spawn(Cast::float(SW / 2).unwrap(), Cast::float(SH / 2).unwrap(), "player")
    player.size = Vec2::new(20.0, 20.0)

    keys.bindKey("up",    "W")
    keys.bindKey("down",  "S")
    keys.bindKey("left",  "A")
    keys.bindKey("right", "D")
    keys.bindKey("fire",  "Space")
    keys.bindMouse("shoot", "Left")

    fn spawnWave() {
        let count = wave.value * 3 + 2
        for let i = 0; i < count; i += 1 {
            let side = i % 4
            let ex = if side == 0 { -40.0 }
                     elif side == 1 { Cast::float(SW + 40).unwrap() }
                     elif side == 2 { Cast::float(i * 80 % SW).unwrap() }
                     else { Cast::float(i * 60 % SW).unwrap() }
            let ey = if side == 2 { -40.0 }
                     elif side == 3 { Cast::float(SH + 40).unwrap() }
                     elif side == 0 { Cast::float(i * 70 % SH).unwrap() }
                     else { Cast::float(i * 50 % SH).unwrap() }
            let e = world.spawn(ex, ey, "enemy")
            e.size = Vec2::new(22.0, 22.0)
            e.data = ENEMY_BASE_SPEED + Cast::float(wave.value).unwrap() * 10.0
        }
    }

    spawnWave()

    return Scene::new(
        fn(dt: Float) {
            if !playerAlive.value { return }

            // Player movement
            let pList = world.query("player")
            if pList.len() == 0 { return }
            let p = pList[0]
            let dx = keys.axis("left", "right")
            let dy = keys.axis("up", "down")
            p.pos.x += dx * PLAYER_SPEED * dt
            p.pos.y += dy * PLAYER_SPEED * dt

            // Shooting
            fireTimer.update(dt)
            if (keys.isHeld("shoot") || keys.isHeld("fire")) && fireTimer.ready() {
                let (mx, my) = InputMap::mousePos()
                let wm = cam.screenToWorld(Vec2::new(Cast::float(mx).unwrap(),
                                                     Cast::float(my).unwrap()))
                let dir = wm.sub(p.pos)
                if dir.length() > 1.0 {
                    let nd = dir.normalized()
                    let b = world.spawn(p.pos.x, p.pos.y, "bullet")
                    b.vel.x = nd.x * BULLET_SPEED
                    b.vel.y = nd.y * BULLET_SPEED
                    b.size  = Vec2::new(6.0, 6.0)
                    b.data  = 0.0
                }
            }

            // Move bullets manually
            world.forEachTagged("bullet", fn(b: Entity) {
                b.pos.x += b.vel.x * dt
                b.pos.y += b.vel.y * dt
                b.data  += dt
                if b.data > BULLET_LIFETIME { b.alive = false }
            })

            // Move enemies manually (chase player)
            let pPos = p.pos
            world.forEachTagged("enemy", fn(e: Entity) {
                let toPlayer = pPos.sub(e.center())
                if toPlayer.length() > 0.5 {
                    let spd = e.data
                    let move = toPlayer.normalized().scale(spd * dt)
                    e.pos.x += move.x
                    e.pos.y += move.y
                }
            })

            // Bullet vs enemy
            world.forEachTagged("bullet", fn(b: Entity) {
                world.forEachTagged("enemy", fn(e: Entity) {
                    if b.overlapsAABB(e) {
                        b.alive = false
                        e.alive = false
                        score.value += 10 * wave.value
                        cam.shake(4.0, 0.1)
                    }
                })
            })

            // Enemy vs player
            world.forEachTagged("enemy", fn(e: Entity) {
                if e.overlapsAABB(p) {
                    e.alive = false
                    playerHp.value -= 1
                    hitFlash.reset()
                    cam.shake(8.0, 0.2)
                    if playerHp.value <= 0 {
                        playerAlive.value = false
                        mgr.switch(makeGameOverScene())
                    }
                }
            })

            // Wave progression
            waveTimer.update(dt)
            if waveTimer.ready() || world.countAlive("enemy") == 0 {
                wave.value += 1
                spawnWave()
            }

            world.update(0.0)   // purge dead, no velocity integration
            cam.update(dt)
            cam.follow(p.pos, 5.0, dt)
        },
        fn() {
            raylib::clear((10, 10, 20))

            // Player
            world.forEachTagged("player", fn(e: Entity) {
                let fa = Cast::int(hitFlash.value()).unwrap()
                let col = if fa > 10 { (255, fa, fa) } else { (60, 220, 100) }
                cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, col)
            })
            hitFlash.update(raylib::getFrameTime())

            // Enemies
            world.forEachTagged("enemy", fn(e: Entity) {
                cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (200, 60, 60))
            })

            // Bullets
            world.forEachTagged("bullet", fn(b: Entity) {
                cam.drawCircle(b.pos.x + 3.0, b.pos.y + 3.0, 3.0, (255, 240, 80))
            })

            // HUD (screen-space — do not go through camera)
            raylib::drawText("Score: " + Cast::string(score.value), 10, 10, 22, (255, 255, 255))
            raylib::drawText("Wave:  " + Cast::string(wave.value),  10, 36, 18, (200, 200, 80))
            for let i = 0; i < playerHp.value; i += 1 {
                raylib::drawRectangle(SW - 20 - i * 22, 12, 16, 16, (255, 80, 80))
            }
        }
    )
}

// ── Scene: Game Over ──────────────────────────────────────────────
fn makeGameOverScene() -> Scene {
    let finalScore = score.value
    let finalWave  = wave.value
    return Scene::new(
        fn(dt: Float) {
            if keys.isPressed("fire") || keys.isHeld("shoot") {
                mgr.switch(makeMenuScene())
            }
        },
        fn() {
            raylib::clear((20, 0, 0))
            raylib::drawText("GAME OVER",
                             (SW - raylib::measureText("GAME OVER", 60)) / 2,
                             180, 60, (255, 80, 80))
            raylib::drawText("Score: " + Cast::string(finalScore),
                             (SW - raylib::measureText("Score: " + Cast::string(finalScore), 28)) / 2,
                             270, 28, (255, 255, 255))
            raylib::drawText("Wave:  " + Cast::string(finalWave),
                             (SW - raylib::measureText("Wave:  " + Cast::string(finalWave), 22)) / 2,
                             310, 22, (200, 200, 80))
            raylib::drawText("Click or SPACE to retry",
                             (SW - raylib::measureText("Click or SPACE to retry", 18)) / 2,
                             380, 18, (160, 160, 160))
        }
    )
}

// ── Main ──────────────────────────────────────────────────────────
raylib::init("Top-Down Shooter", SW, SH, 60)
mgr.switch(makeMenuScene())
while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

---

## 22. Quick Reference Tables {#22-quick-reference}

### Critical rules at a glance

| Rule | Correct | Wrong |
|---|---|---|
| Mutable scalar in closure | `let x = Box(0)` → `x.value += 1` | `let x = 0` → `x += 1` |
| Entity update + purge only | `world.update(0.0)` | `world.update(dt)` (double-integrates) |
| Forward declaration | `fn foo() -> T` (no body) | `fn foo() -> T {}` (empty body, different) |
| Chained if in statement | `if ... { } elif ... { }` | `if ... { } else if ... { }` |
| Chained if in expression | `let x = if cond { a } else { b }` | `let x = if c { a } elif c2 { b } ...` |

### SceneManager

| Method | Effect |
|---|---|
| `SceneManager::empty()` | Create empty manager (no scene) |
| `SceneManager::new(scene)` | Create with initial scene |
| `mgr.switch(scene)` | Replace current, clear stack |
| `mgr.push(scene)` | Push over current (pause menus) |
| `mgr.pop()` | Return to previous scene |
| `mgr.update(dt)` | Tick current scene |
| `mgr.draw()` | Draw current scene |

### EntityWorld

| Method | Effect |
|---|---|
| `world.spawn(x, y, tag)` | Create entity at position with tag |
| `world.query(tag)` | Return `[Entity]` with matching tag |
| `world.forEachTagged(tag, fn)` | Iterate entities with tag (mutable) |
| `world.forEach(fn)` | Iterate ALL entities |
| `world.countAlive(tag)` | Count living entities with tag |
| `world.update(dt)` | `pos += vel*dt` for all; purge dead |
| `world.update(0.0)` | Purge dead only (no movement) |
| `e.alive = false` | Mark entity for removal |
| `e.overlapsAABB(other)` | AABB collision test |
| `e.center()` | `Vec2` at centre of entity |
| `e.entityDrawRect(color)` | Draw filled rect at pos/size |
| `e.entityDrawCircle(color)` | Draw circle at centre, radius size.x/2 |

### Timer

| Constructor | Behaviour |
|---|---|
| `Timer::cooldown(s)` | `ready()` fires after `s` seconds; resets |
| `Timer::repeating(s)` | `ready()` fires every `s` seconds; auto-resets |
| `Timer::once(s)` | `isDone()` fires once after `s` seconds |

### Tween constructors

| Constructor | Easing |
|---|---|
| `Tween::linear(s, e, d)` | Constant |
| `Tween::easeIn(s, e, d)` | Accelerating |
| `Tween::easeOut(s, e, d)` | Decelerating |
| `Tween::smooth(s, e, d)` | Ease-in-out |
| `Tween::easeOutBounce(s, e, d)` | Bouncy |
| `Tween::easeOutElastic(s, e, d)` | Spring |

### Camera2D

| Method | Description |
|---|---|
| `Camera2D::new(w, h)` | Create camera for screen size |
| `cam.follow(pos, speed, dt)` | Smooth-follow a `Vec2` |
| `cam.shake(intensity, duration)` | Screen shake |
| `cam.setZoom(z)` | Set zoom (1.0 = normal) |
| `cam.update(dt)` | Advance shake decay |
| `cam.drawRect(x, y, w, h, c)` | Draw rect in world space |
| `cam.drawCircle(x, y, r, c)` | Draw circle in world space |
| `cam.screenToWorld(v)` | Convert screen Vec2 to world Vec2 |
| `cam.worldToScreen(v)` | Convert world Vec2 to screen Vec2 |
| `cam.isVisible(x, y, margin)` | Frustum-cull test |

### Common pitfalls

| Symptom | Cause | Fix |
|---|---|---|
| Game state resets every frame | Mutable scalar captured by value | Wrap in `Box(T)` |
| Entities don't move | `world.update(0.0)` + no manual integration | Add `e.pos += e.vel * dt` in `forEachTagged` |
| Entities move twice as fast | `world.update(dt)` AND manual integration | Pick one; use `world.update(0.0)` when moving manually |
| Parse error on `else if` | `else if` is not valid syntax | Use `elif` |
| Forward dec not working | Wrote `fn foo() -> T {}` (empty braces = definition) | Omit the `{}` entirely |
| Music stops playing | `updateMusic` not called | Call `raylib::updateMusic(id)` every frame |
| Audio functions fail | `initAudio` not called | Call `raylib::initAudio()` after `raylib::init(...)` |
| Triangle wrong winding | Vertices in clockwise order | Use counter-clockwise order |

---

*For raw raylib API: see `documentation/raylib.md`*
*For std module API details: see `documentation/std.md`*
*For the Nova language reference: see `documentation/how_to_write_nova.md`*
