# Nova Game Engine Guide

A complete guide to building games with Nova's game development standard library.

---

## Table of Contents

1. [Quick Start — Your First Window](#quick-start)
2. [Project Structure](#project-structure)
3. [Scene Management](#scene-management)
4. [Entity System](#entity-system)
5. [Input Handling](#input-handling)
6. [Physics and Collision](#physics-and-collision)
7. [Camera](#camera)
8. [Timers and Tweens](#timers-and-tweens)
9. [Tilemaps with Grid](#tilemaps-with-grid)
10. [Procedural Generation with Noise](#procedural-generation-with-noise)
11. [Plotting and Debugging](#plotting-and-debugging)
12. [Complete Example — Mini Platformer](#complete-example)
13. [Complete Example — Top-Down Shooter](#top-down-shooter)

---

## Quick Start — Your First Window {#quick-start}

Every Nova raylib game follows the same skeleton:

```nova
raylib::init("My Game", 800, 600, 60)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    // update
    // draw
    raylib::drawText("Hello, Nova!", 300, 280, 24, (255,255,255))
}
```

`raylib::rendering()` returns `true` each frame and handles:
- Clearing the background between calls
- Polling input events
- Presenting the frame
- Returning `false` when the window is closed

---

## Project Structure {#project-structure}

Recommended layout for a Nova game:

```
my_game/
  main.nv          ← entry point (raylib::init + scene manager loop)
  scenes/
    title.nv       ← TitleScene
    gameplay.nv    ← GameplayScene
    gameover.nv    ← GameOverScene
  entities/
    player.nv      ← player spawn / update helpers
    enemy.nv       ← enemy AI helpers
  assets/          ← sprites, sounds (loaded by raylib)
```

Each scene file defines a `makeXxxScene()` function that returns a `Scene`.

---

## Scene Management {#scene-management}

Scenes decouple your game logic into self-contained states (title, gameplay, pause, game-over). Each scene holds its own state via `Box(T)` closures so there is no shared global state.

```nova
import super.std.scene

fn makeTitleScene(mgr: SceneManager) -> Scene {
    let selected = Box::new(0)
    return Scene::new(
        fn(dt: Float) {
            if raylib::isKeyPressed("Down") { selected.set(selected.get() + 1) }
            if raylib::isKeyPressed("Up")   { selected.set(selected.get() - 1) }
            if raylib::isKeyPressed("Enter") {
                // switch to gameplay
                mgr.switch(makeGameplayScene(mgr))
            }
        },
        fn() {
            raylib::drawText("MY GAME", 300, 150, 48, (255,200,50))
            raylib::drawText("Press Enter to Play", 280, 260, 20, (200,200,200))
        }
    )
}

// Main entry point
raylib::init("My Game", 800, 600, 60)
let mgr = SceneManager::new(makeTitleScene(mgr))
while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

### Scene transitions

| Method | Effect |
|---|---|
| `mgr.switch(scene)` | Replace current scene; clears the entire stack |
| `mgr.push(scene)` | Push over current scene (pause menu, dialog) |
| `mgr.pop()` | Return to the previous scene |

```nova
// Pause menu example
if map.isPressed("pause") {
    mgr.push(makePauseScene(mgr))
}

// Inside PauseScene:
if map.isPressed("resume") {
    mgr.pop()   // returns to gameplay
}
```

---

## Entity System {#entity-system}

`EntityWorld` is a simple tag-based entity manager. Entities are plain structs with public fields — mutate them directly.

```nova
import super.std.entity
import super.std.vec2

let world = EntityWorld::new()

// Spawn a player
let player = world.spawn(400.0, 300.0, "player")
player.size = Vec2::new(32.0, 32.0)

// Spawn enemies
for let i = 0; i < 5; i += 1 {
    let e = world.spawn(Cast::float(i * 120 + 50).unwrap(), 100.0, "enemy")
    e.vel.x = -40.0
    e.size  = Vec2::new(24.0, 24.0)
    e.data  = 3.0   // health
}

// Game loop
while raylib::rendering() {
    let dt = raylib::getFrameTime()

    // Move player with arrow keys
    let p = world.query("player")[0]
    if !raylib::isKeyUp("Right") { p.vel.x =  150.0 }
    else if !raylib::isKeyUp("Left") { p.vel.x = -150.0 }
    else { p.vel.x = 0.0 }

    // Enemy AI: bounce off screen edges
    world.forEachTagged("enemy", fn(e: Entity) {
        if e.pos.x < 0.0 || e.pos.x > 760.0 { e.vel.x = -e.vel.x }
    })

    // Bullet vs enemy
    world.forEachTagged("bullet", fn(b: Entity) {
        world.forEachTagged("enemy", fn(e: Entity) {
            if b.overlapsAABB(e) {
                b.alive = false
                e.data = e.data - 1.0
                if e.data <= 0.0 { e.alive = false }
            }
        })
    })

    world.update(dt)

    // Draw
    world.forEachTagged("player", fn(e: Entity) { e.entityDrawRect((0, 200, 80)) })
    world.forEachTagged("enemy",  fn(e: Entity) { e.entityDrawRect((220, 50, 50)) })
    world.forEachTagged("bullet", fn(e: Entity) { e.entityDrawCircle((255,230,0)) })
}
```

### Entity fields reference

| Field | Type | Typical use |
|---|---|---|
| `id` | `Int` | Unique identifier (auto-assigned) |
| `pos` | `Vec2` | World-space position |
| `vel` | `Vec2` | Velocity (units/second) |
| `size` | `Vec2` | Width/height for AABB and draw |
| `tag` | `String` | Category (`"player"`, `"enemy"`, `"bullet"`) |
| `alive` | `Bool` | Set to `false` to destroy on next `update` |
| `data` | `Float` | General purpose (health, angle, timer, …) |

---

## Input Handling {#input-handling}

Decouple game logic from raw key names using `InputMap`.

```nova
import super.std.input

let keys = InputMap::new()
keys.bindKey("left",   "A")
keys.bindKey("right",  "D")
keys.bindKey("jump",   "Space")
keys.bindKey("fire",   "J")
keys.bindMouse("aim",  "Left")

// Game loop queries
let dx = keys.axis("left", "right")     // -1, 0, or 1
let (mx, my) = InputMap::mousePos()

if keys.isPressed("jump")  { /* one-time jump */ }
if keys.isHeld("fire")     { /* held fire */     }
if keys.isReleased("fire") { /* release event */ }
```

To support rebinding:
```nova
// Read a new key and reassign
let newKey = InputMap::lastKey()
if newKey.isSome() {
    keys.bindKey("jump", newKey.unwrap())
}
```

---

## Physics and Collision {#physics-and-collision}

Use `Body2D` for moveable objects and `AABB` / `Circle` for static shapes.

### Basic gravity + wall bounce

```nova
import super.std.physics
import super.std.vec2

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
    raylib::drawCircle(Cast::int(ball.pos.x).unwrap(), Cast::int(ball.pos.y).unwrap(), 10, (255,100,0))
}
```

### Body vs Body collision

```nova
for let i = 0; i < balls.len(); i += 1 {
    for let j = i + 1; j < balls.len(); j += 1 {
        resolveCircle(balls[i], 12.0, balls[j], 12.0)
    }
}
```

### Raycasting (line-of-sight)

```nova
let ray = Ray2::new(player.pos.x, player.pos.y,
                    dirX, dirY)  // dirX/Y should be normalized
let hit = ray.castAABB(wall)
if hit.hit {
    // hit.point is where the ray hit
    // hit.normal is the surface normal
    // hit.t is the distance (parametric)
}
```

---

## Camera {#camera}

The camera maps world coordinates to screen pixels, supporting zoom, follow, and shake.

```nova
import super.std.camera

let cam = Camera2D::new(800, 600)
cam.setZoom(1.5)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    cam.update(dt)

    // Smoothly follow the player
    cam.follow(player.pos, 6.0, dt)

    // Draw world objects using camera transform
    world.forEach(fn(e: Entity) {
        cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (200,200,200))
    })

    // Screen shake on explosion
    if explosion {
        cam.shake(12.0, 0.4)
    }
}
```

### World vs screen coordinates

```nova
// Convert mouse position to world coords for clicking on objects
let (mx, my) = InputMap::mousePos()
let worldMouse = cam.screenToWorld(Vec2::new(Cast::float(mx).unwrap(),
                                             Cast::float(my).unwrap()))
```

### Culling off-screen entities

```nova
world.forEach(fn(e: Entity) {
    if cam.isVisible(e.pos.x, e.pos.y, 64.0) {
        e.entityDrawRect(color)
    }
})
```

---

## Timers and Tweens {#timers-and-tweens}

### Timers

```nova
import super.std.timer

let fireRate  = Timer::cooldown(0.15)   // shoot every 150ms
let bossTimer = Timer::once(30.0)        // 30-second encounter timer
let blinkTimer= Timer::repeating(0.5)   // blink every half second

while raylib::rendering() {
    let dt = raylib::getFrameTime()

    // Shooting cooldown
    fireRate.update(dt)
    if keys.isHeld("fire") && fireRate.ready() {
        spawnBullet(player.pos)
    }

    // Win condition
    if bossTimer.update(dt) {
        // timer fired — boss music ends
    }

    // Blinking HP bar
    blinkTimer.update(dt)
    if !blinkTimer.isDone() || (blinkTimer.progress() > 0.5) {
        drawHealthBar()
    }
}
```

### Tweens

```nova
import super.std.tween

let openDoor  = Tween::smooth(0.0, 80.0, 0.6)   // door slides open 80px in 0.6s
let fadeIn    = Tween::new(0.0, 255.0, 1.0, fn(t: Float) -> Float {
    // custom easing: ease-out cubic
    let u = 1.0 - t
    return 1.0 - u * u * u
})
let flashAlpha = Tween::linear(255.0, 0.0, 0.3)

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    let doorY = openDoor.update(dt)
    let alpha  = fadeIn.update(dt)

    // Ping-pong a tween for a pulsing effect
    if glowTween.isDone() { glowTween.ping() }
    let glow = glowTween.update(dt)
}
```

### Easing cheatsheet

| Ease | Feel |
|---|---|
| `linear` | Constant speed — mechanical |
| `easeIn` | Starts slow, ends fast — building up |
| `easeOut` | Starts fast, ends slow — landing/settling |
| `easeInOut` | Smooth start and end — UI animations |
| `easeOutBounce` | Bouncy landing — jumping, popups |
| `easeOutElastic` | Spring overshoot — rubber-band effect |
| `easeOutBack` | Slight overshoot — snappy buttons |
| `sineInOut` | Gentle sine wave — breathing, floating |

---

## Tilemaps with Grid {#tilemaps-with-grid}

```nova
import super.std.grid

// Tile IDs: 0 = floor, 1 = wall, 2 = water
let MAP_W = 30
let MAP_H = 20
let TILE  = 32

let map = Grid::new(MAP_W, MAP_H, 0)

// Draw a room with walls
map.fillRect(0, 0, MAP_W, MAP_H, 1)    // fill with walls
map.fillRect(1, 1, MAP_W-2, MAP_H-2, 0) // hollow interior

// Find path from top-left to bottom-right
let path = map.bfs(1, 1, MAP_W-2, MAP_H-2, fn(v: Any) -> Bool { v == 0 })

// Draw
let colorMap = fn(v: Any) -> (Int, Int, Int) {
    if v == 1 { return (80, 80, 90) }     // wall
    if v == 2 { return (40, 100, 180) }   // water
    return (50, 50, 55)                    // floor
}

while raylib::rendering() {
    map.draw(0, 0, TILE, colorMap)
    map.drawLines(0, 0, TILE, (30, 30, 35))

    // Draw path
    for cell in path {
        raylib::drawRectangle(cell[0]*TILE + 10, cell[1]*TILE + 10, 12, 12, (255,220,0))
    }
}
```

### Snapping entities to grid

```nova
// World position → tile coordinate
fn worldToTile(wx: Float, wy: Float, tileSize: Int) -> (Int, Int) {
    return (Cast::int(wx).unwrap() / tileSize,
            Cast::int(wy).unwrap() / tileSize)
}

// Tile coordinate → world center
fn tileCenter(tx: Int, ty: Int, tileSize: Int) -> Vec2 {
    let ts = Cast::float(tileSize).unwrap()
    return Vec2::new(Cast::float(tx).unwrap() * ts + ts / 2.0,
                     Cast::float(ty).unwrap() * ts + ts / 2.0)
}
```

---

## Procedural Generation with Noise {#procedural-generation-with-noise}

```nova
import super.std.noise

// Generate a terrain heightmap
let SEED = 7

for let row = 0; row < MAP_H; row += 1 {
    for let col = 0; col < MAP_W; col += 1 {
        let nx = Cast::float(col).unwrap() * 0.08
        let ny = Cast::float(row).unwrap() * 0.08
        let h  = fbm(nx, ny, SEED, 5, 2.0, 0.5)
        let tile = if h > 0.6 { 1 }   // wall/mountain
                   else { 0 }          // floor/ocean
        map.set(col, row, tile)
    }
}

// Color terrain by height for a heightmap visualization
let heightColor = fn(v: Any) -> (Int, Int, Int) {
    // v is stored as Float in data field
    let h = Cast::float(v).unwrap()
    return noiseToColor(h, (20, 80, 200), (200, 230, 120))
}

// Domain-warped clouds
for let row = 0; row < SCREEN_H; row += 1 {
    for let col = 0; col < SCREEN_W; col += 1 {
        let n = domain(Cast::float(col).unwrap() * 0.003,
                       Cast::float(row).unwrap() * 0.003,
                       SEED, 1.5)
        let c = noiseToColor(n, (20,20,60), (200,220,255))
        raylib::drawRectangle(col, row, 1, 1, c)
    }
}
```

### Noise recipe guide

| Effect | Function | Scale | Settings |
|---|---|---|---|
| Terrain height | `fbm` | 0.005–0.02 | octaves=6, lac=2, gain=0.5 |
| Caves | `smoothNoise` | 0.04–0.1 | threshold ~0.5 |
| Mountain ridges | `ridged` | 0.008 | octaves=5 |
| Cloud texture | `domain` | 0.002–0.006 | strength=1.5 |
| Grass variation | `valueNoise` | 0.2–0.5 | direct use |
| Particle turbulence | `fbm` | per-particle | octaves=3, gain=0.6 |

---

## Plotting and Debugging {#plotting-and-debugging}

Plot any `[Float]` data in-game for debugging AI, physics, or performance:

```nova
import super.std.plot

let fpsHistory = []
let plotArea   = PlotArea::auto(10, 10, 250, 100, [0.0, 120.0])

while raylib::rendering() {
    let dt = raylib::getFrameTime()
    let fps = if dt > 0.0 { 1.0 / dt } else { 60.0 }
    fpsHistory.push(fps)
    if fpsHistory.len() > 120 { fpsHistory = fpsHistory.slice(1, fpsHistory.len()) }

    // Draw FPS graph
    let area = PlotArea::new(10, 10, 250, 100, 0.0,
                              Cast::float(fpsHistory.len()).unwrap(), 0.0, 120.0)
    area.drawGrid(12, 6, (30,30,30))
    area.drawAxes((80,80,80))
    area.lineChart(fpsHistory, (0,220,100))
    area.hLine(60.0, (255,200,0))   // target FPS reference
    area.drawTitle("FPS", 12, (200,200,200))
}
```

Other useful debug draws:
```nova
// Velocity vectors
world.forEach(fn(e: Entity) {
    let ex = Cast::int(e.pos.x + e.size.x / 2.0).unwrap()
    let ey = Cast::int(e.pos.y + e.size.y / 2.0).unwrap()
    let vx = Cast::int(e.pos.x + e.vel.x * 0.1 + e.size.x / 2.0).unwrap()
    let vy = Cast::int(e.pos.y + e.vel.y * 0.1 + e.size.y / 2.0).unwrap()
    raylib::drawLine(ex, ey, vx, vy, (255,255,0))
})

// AABB wireframes
world.forEach(fn(e: Entity) {
    raylib::drawRectangleLines(Cast::int(e.pos.x).unwrap(), Cast::int(e.pos.y).unwrap(),
                                Cast::int(e.size.x).unwrap(), Cast::int(e.size.y).unwrap(),
                                (0,255,0))
})
```

---

## Complete Example — Mini Platformer {#complete-example}

A self-contained platformer demonstrating scene, entity, input, physics, camera, timer, and tween together.

```nova
import super.std.scene
import super.std.entity
import super.std.input
import super.std.physics
import super.std.camera
import super.std.timer
import super.std.tween
import super.std.vec2

// ── Constants ────────────────────────────────────────────────

let SCREEN_W = 800
let SCREEN_H = 600
let GRAVITY   = 1200.0
let JUMP_SPEE = -600.0
let WALK_SPEE = 220.0
let TILE      = 32

// ── Level layout (1 = solid) ─────────────────────────────────

let LEVEL = [
    "################################",
    "#                              #",
    "#   P                          #",
    "#                              #",
    "#          ##########          #",
    "#                              #",
    "#    ###                       #",
    "#                              #",
    "##########              ########",
    "#                              #",
    "#                        E     #",
    "###############################",
]

// ── Gameplay scene factory ────────────────────────────────────

fn makeGameplay(mgr: SceneManager) -> Scene {
    // --- init world ---
    let world  = EntityWorld::new()
    let keys   = InputMap::new()
    let cam    = Camera2D::new(SCREEN_W, SCREEN_H)
    let jumpCd = Timer::cooldown(0.08)   // jump coyote time

    keys.bindKey("left",  "A")
    keys.bindKey("right", "D")
    keys.bindKey("jump",  "Space")

    // Parse level into grid + spawn entities
    let player = world.spawn(96.0, 80.0, "player")
    player.size = Vec2::new(Cast::float(TILE - 4).unwrap(), Cast::float(TILE - 2).unwrap())

    for let row = 0; row < LEVEL.len(); row += 1 {
        let line = LEVEL[row]
        for let col = 0; col < line.len(); col += 1 {
            let ch = line.charAt(col)
            if ch == 'E' {
                let e = world.spawn(Cast::float(col * TILE).unwrap(),
                                    Cast::float(row * TILE).unwrap(), "enemy")
                e.size = Vec2::new(28.0, 30.0)
                e.vel.x = -60.0
            }
        }
    }

    // Build wall AABB list from '#' tiles
    let walls = []
    for let row = 0; row < LEVEL.len(); row += 1 {
        let line = LEVEL[row]
        for let col = 0; col < line.len(); col += 1 {
            if line.charAt(col) == '#' {
                walls.push(AABB::new(
                    Cast::float(col * TILE).unwrap(),
                    Cast::float(row * TILE).unwrap(),
                    Cast::float(TILE).unwrap(),
                    Cast::float(TILE).unwrap()))
            }
        }
    }

    // Spawn-death tween for hit flash
    let hitFlash = Tween::linear(1.0, 0.0, 0.3)

    // --- update closure ---
    let update = fn(dt: Float) {
        let p = world.query("player")
        if p.len() == 0 { return }
        let player = p[0]

        // Input → velocity
        let dx = keys.axis("left", "right")
        player.vel.x = dx * WALK_SPEE

        // Gravity
        player.vel.y = player.vel.y + GRAVITY * dt

        // Jump
        jumpCd.update(dt)
        if keys.isPressed("jump") && jumpCd.ready() {
            player.vel.y = JUMP_SPEE
        }

        // Integrate position manually (physics module Body2D not used here for simplicity)
        player.pos.x = player.pos.x + player.vel.x * dt
        player.pos.y = player.pos.y + player.vel.y * dt

        // Wall collision
        for wall in walls {
            let pb = AABB::new(player.pos.x, player.pos.y, player.size.x, player.size.y)
            let pen = pb.overlap(wall)
            if pen.x != 0.0 || pen.y != 0.0 {
                player.pos.x = player.pos.x + pen.x
                player.pos.y = player.pos.y + pen.y
                if pen.y != 0.0 {
                    player.vel.y = 0.0
                    // landed — reset jump cooldown
                    if pen.y < 0.0 { jumpCd.activate() }
                }
                if pen.x != 0.0 { player.vel.x = 0.0 }
            }
        }

        // Enemy movement + wall bounce
        world.forEachTagged("enemy", fn(e: Entity) {
            e.pos.x = e.pos.x + e.vel.x * dt
            e.pos.y = e.pos.y + GRAVITY * dt * 0.5
            for wall in walls {
                let eb = AABB::new(e.pos.x, e.pos.y, e.size.x, e.size.y)
                let pen = eb.overlap(wall)
                if pen.x != 0.0 { e.vel.x = -e.vel.x; e.pos.x = e.pos.x + pen.x }
                if pen.y != 0.0 { e.vel.y = 0.0;      e.pos.y = e.pos.y + pen.y }
            }
        })

        // Player vs enemy collision
        world.forEachTagged("enemy", fn(e: Entity) {
            let pb = AABB::new(player.pos.x, player.pos.y, player.size.x, player.size.y)
            let eb = AABB::new(e.pos.x, e.pos.y, e.size.x, e.size.y)
            if pb.overlaps(eb) {
                // stomp from above
                if player.vel.y > 0.0 && player.pos.y + player.size.y < e.pos.y + e.size.y / 2.0 {
                    e.alive = false
                    player.vel.y = JUMP_SPEE * 0.6
                    cam.shake(6.0, 0.15)
                } else {
                    // hurt player
                    player.alive = false
                    mgr.switch(makeGameplay(mgr))  // restart
                }
            }
        })

        world.update(dt)

        // Camera follows player
        cam.update(dt)
        cam.follow(Vec2::new(player.pos.x + player.size.x / 2.0,
                              player.pos.y + player.size.y / 2.0), 7.0, dt)
    }

    // --- draw closure ---
    let draw = fn() {
        // Draw walls using camera
        for wall in walls {
            cam.drawRect(wall.pos.x, wall.pos.y, wall.size.x, wall.size.y, (100, 100, 120))
        }

        // Draw entities
        world.forEachTagged("player", fn(e: Entity) {
            cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (60, 200, 100))
        })
        world.forEachTagged("enemy", fn(e: Entity) {
            cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (220, 60, 60))
        })

        // HUD
        let alive = world.countAlive("enemy")
        raylib::drawText("Enemies: " + Cast::string(alive), 10, 10, 18, (255,255,255))
        raylib::drawText("WASD/Space to move", 10, SCREEN_H - 30, 14, (150,150,150))
    }

    return Scene::new(update, draw)
}

// ── Main ──────────────────────────────────────────────────────

raylib::init("Nova Platformer", SCREEN_W, SCREEN_H, 60)

let mgr = SceneManager::empty()
mgr.switch(makeGameplay(mgr))

while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

---

## Complete Example — Top-Down Shooter {#top-down-shooter}

```nova
import super.std.scene
import super.std.entity
import super.std.input
import super.std.camera
import super.std.timer
import super.std.noise
import super.std.vec2

let W = 900
let H = 600

fn makeShooterScene(mgr: SceneManager) -> Scene {
    let world    = EntityWorld::new()
    let keys     = InputMap::new()
    let cam      = Camera2D::new(W, H)
    let fireRate = Timer::cooldown(0.12)
    let spawnTmr = Timer::repeating(1.8)

    keys.bindKey("up",    "W")
    keys.bindKey("down",  "S")
    keys.bindKey("left",  "A")
    keys.bindKey("right", "D")
    keys.bindMouse("fire", "Left")

    let player = world.spawn(Cast::float(W).unwrap() / 2.0,
                              Cast::float(H).unwrap() / 2.0, "player")
    player.size = Vec2::new(20.0, 20.0)
    let score = Box::new(0)

    let update = fn(dt: Float) {
        let pList = world.query("player")
        if pList.len() == 0 { return }
        let p = pList[0]

        // Movement
        let dx = keys.axis("left", "right")
        let dy = keys.axis("up", "down")
        p.vel.x = dx * 200.0
        p.vel.y = dy * 200.0

        // Shooting: aim toward mouse
        fireRate.update(dt)
        if keys.isHeld("fire") && fireRate.ready() {
            let (mx, my) = InputMap::mousePos()
            let worldMouse = cam.screenToWorld(Vec2::new(Cast::float(mx).unwrap(), Cast::float(my).unwrap()))
            let dxM = worldMouse.x - p.pos.x
            let dyM = worldMouse.y - p.pos.y
            let dist = Float::sqrt(dxM*dxM + dyM*dyM)
            if dist > 0.0 {
                let b = world.spawn(p.pos.x, p.pos.y, "bullet")
                b.vel.x = dxM / dist * 600.0
                b.vel.y = dyM / dist * 600.0
                b.size  = Vec2::new(6.0, 6.0)
                b.data  = 1.5   // lifetime seconds
            }
        }

        // Bullet lifetime
        world.forEachTagged("bullet", fn(b: Entity) {
            b.data = b.data - dt
            if b.data <= 0.0 { b.alive = false }
        })

        // Enemy spawning
        if spawnTmr.update(dt) {
            let side = random::range(0, 4)
            let ex = if side == 0 { -30.0 }
                     else if side == 1 { Cast::float(W + 30).unwrap() }
                     else { Cast::float(random::range(0, W)).unwrap() }
            let ey = if side == 2 { -30.0 }
                     else if side == 3 { Cast::float(H + 30).unwrap() }
                     else { Cast::float(random::range(0, H)).unwrap() }
            let e = world.spawn(ex, ey, "enemy")
            e.size = Vec2::new(22.0, 22.0)
        }

        // Enemy AI: chase player
        world.forEachTagged("enemy", fn(e: Entity) {
            let dxE = p.pos.x - e.pos.x
            let dyE = p.pos.y - e.pos.y
            let dist = Float::sqrt(dxE*dxE + dyE*dyE)
            if dist > 0.0 {
                e.vel.x = dxE / dist * 90.0
                e.vel.y = dyE / dist * 90.0
            }
            // bullet hits enemy
            world.forEachTagged("bullet", fn(b: Entity) {
                if b.overlapsAABB(e) {
                    b.alive = false
                    e.alive = false
                    score.set(score.get() + 10)
                    cam.shake(5.0, 0.1)
                }
            })
            // enemy hits player
            if e.overlapsAABB(p) {
                p.alive = false
                mgr.switch(makeShooterScene(mgr))  // restart
            }
        })

        world.update(dt)
        cam.update(dt)
        cam.follow(Vec2::new(p.pos.x, p.pos.y), 5.0, dt)
    }

    let draw = fn() {
        // Procedural noise background
        // (in a real game, pre-render this to a texture)
        for let ty = 0; ty < H; ty += 16 {
            for let tx = 0; tx < W; tx += 16 {
                let n = smoothNoise(Cast::float(tx).unwrap() * 0.04,
                                    Cast::float(ty).unwrap() * 0.04, 1)
                let v = Cast::int(20.0 + n * 25.0).unwrap()
                raylib::drawRectangle(tx, ty, 16, 16, (v, v, v+5))
            }
        }

        world.forEachTagged("player", fn(e: Entity) { cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (80, 220, 120)) })
        world.forEachTagged("enemy",  fn(e: Entity) { cam.drawRect(e.pos.x, e.pos.y, e.size.x, e.size.y, (220, 70, 70)) })
        world.forEachTagged("bullet", fn(e: Entity) { cam.drawCircle(e.pos.x, e.pos.y, 4.0, (255, 240, 80)) })

        // HUD
        raylib::drawText("Score: " + Cast::string(score.get()), 10, 10, 22, (255,255,255))
        raylib::drawText("Enemies: " + Cast::string(world.countAlive("enemy")), 10, 36, 16, (200,150,150))
        raylib::drawText("WASD + Click to shoot", 10, H - 26, 13, (120,120,120))
    }

    return Scene::new(update, draw)
}

raylib::init("Nova Top-Down Shooter", W, H, 60)
let mgr = SceneManager::empty()
mgr.switch(makeShooterScene(mgr))
while raylib::rendering() {
    mgr.update(raylib::getFrameTime())
    mgr.draw()
}
```

---

## Tips and Best Practices

### Performance

- Call `world.update(dt)` once per frame — it integrates velocity and prunes dead entities in one pass.
- Use `cam.isVisible(wx, wy, margin)` to skip drawing off-screen entities.
- For large static tilemaps, draw once to a texture (using raylib sprite system) and blit each frame.
- `noise` functions are CPU-intensive for per-pixel maps; pre-generate into a Grid of colors.

### State management

- Store all scene-local state in `Box(T)` captures inside the `Scene::new(...)` call — no globals needed.
- Pass `mgr: SceneManager` into scene factories so scenes can trigger their own transitions.
- Use `mgr.push(pauseScene)` for overlays so the gameplay scene is preserved underneath.

### Entity design

- The `data: Float` field is your general-purpose slot — use it for health, angle, timer, ammo, etc.
- For entities that need multiple custom values, use the `tag` field to dispatch to different update paths.
- Keep entity lists small (< 1000). For particle effects with thousands of objects, use a plain `[Float]` array instead.

### Physics

- `pushOutAABB` is for one-sided (static wall) correction; `resolveAABB` is for body vs body.
- Apply gravity via `body.applyGravity(g, dt)` or directly via `vel.y += g * dt` — both work.
- Use `restitution = 0.0` for rigid walls, `0.5–0.8` for bouncy objects, `1.0` for perfectly elastic.

### Audio

- Sounds and music use the built-in `raylib::playSound`, `raylib::loadSound`, `raylib::loadMusic`, etc.
- Load assets before the main loop; store handles in Box(T) captures inside scenes.

---

*See `documentation/std.md` for the complete API reference for all standard library modules.*
