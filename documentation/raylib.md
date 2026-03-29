# Raylib Guide for Nova

Nova has first-class raylib integration for creating 2D games and graphical applications.
All raylib functions live in the `raylib` module, which is available without any imports.

---

## Quick Start

```nova
module my_game

// Create a window: title, width, height, target FPS
raylib::init("My Game", 800, 600, 60)

// Main game loop
while raylib::rendering() {
    // Clear the screen with a color (R, G, B)
    raylib::clear((30, 30, 30))

    // Draw things
    raylib::drawText("Hello, Nova!", 100, 100, 40, (255, 255, 255))
    raylib::drawFPS(10, 10)
}
```

Run the file with `nova my_game.nv`.

---

## Colors

Colors in Nova are **tuples** of three integers `(R, G, B)`, each in the range 0–255.

```nova
let red   = (255, 0, 0)
let green = (0, 255, 0)
let blue  = (0, 0, 255)
let white = (255, 255, 255)
let black = (0, 0, 0)
let gray  = (130, 130, 130)
```

---

## Window & Timing

| Function | Description |
|---|---|
| `raylib::init(title: String, width: Int, height: Int, fps: Int)` | Create a window and set the target FPS. |
| `raylib::rendering() -> Bool` | Process one frame. Returns `false` when the window is closed. Put all drawing between consecutive calls. |
| `raylib::getScreenWidth() -> Int` | Get the current window width. |
| `raylib::getScreenHeight() -> Int` | Get the current window height. |
| `raylib::setTargetFPS(fps: Int)` | Change the target frame rate at runtime. |
| `raylib::getFPS() -> Int` | Get the current (actual) frames per second. |
| `raylib::getTime() -> Float` | Seconds elapsed since `init`. |
| `raylib::getFrameTime() -> Float` | Delta time for the last frame (seconds). |
| `raylib::sleep(ms: Int)` | Pause execution for the given number of milliseconds. |

### Game-loop pattern

```nova
raylib::init("Game", 800, 600, 60)

while raylib::rendering() {
    let dt = raylib::getFrameTime()   // smooth movement
    raylib::clear((0, 0, 0))

    // update & draw ...
}
```

---

## Drawing – Shapes

All draw calls are queued and rendered automatically when `raylib::rendering()` is called.

| Function | Description |
|---|---|
| `raylib::clear(color)` | Fill the background with a color. Call this first every frame. |
| `raylib::drawRectangle(x, y, w, h, color)` | Draw a filled rectangle. |
| `raylib::drawRectangleLines(x, y, w, h, color)` | Draw a rectangle outline. |
| `raylib::drawRoundedRectangle(x, y, w, h, roundness: Float, color)` | Draw a filled rounded rectangle. `roundness` ranges from 0.0 (sharp) to 1.0 (pill-shaped). |
| `raylib::drawCircle(x, y, radius, color)` | Draw a filled circle. |
| `raylib::drawCircleLines(x, y, radius, color)` | Draw a circle outline. |
| `raylib::drawLine(x1, y1, x2, y2, color)` | Draw a 1-pixel line. |
| `raylib::drawLineThick(x1, y1, x2, y2, thickness: Float, color)` | Draw a thick line. |
| `raylib::drawTriangle(x1, y1, x2, y2, x3, y3, color)` | Draw a filled triangle (vertices in counter-clockwise order). |

### Example – shapes

```nova
module shapes_demo

raylib::init("Shapes", 800, 600, 60)

while raylib::rendering() {
    raylib::clear((20, 20, 40))

    // filled
    raylib::drawRectangle(50, 50, 200, 100, (0, 120, 200))
    raylib::drawCircle(400, 200, 60, (200, 50, 50))
    raylib::drawTriangle(600, 100, 550, 250, 650, 250, (50, 200, 50))

    // outlines
    raylib::drawRectangleLines(50, 200, 200, 100, (255, 255, 0))
    raylib::drawCircleLines(400, 400, 80, (255, 128, 0))

    // thick line
    raylib::drawLineThick(50, 450, 750, 450, 4.0, (255, 255, 255))

    // rounded rectangle
    raylib::drawRoundedRectangle(500, 350, 200, 100, 0.5, (128, 0, 255))
}
```

---

## Drawing – Text

| Function | Description |
|---|---|
| `raylib::drawText(text: String, x, y, fontSize, color)` | Draw text at a position. |
| `raylib::drawFPS(x, y)` | Draw the current FPS counter (built-in). |
| `raylib::measureText(text: String, fontSize: Int) -> Int` | Return the pixel width of a string at a given font size. Useful for centering. |

### Centering text

```nova
let msg = "Game Over"
let size = 40
let w = raylib::measureText(msg, size)
let screenW = raylib::getScreenWidth()
raylib::drawText(msg, (screenW - w) / 2, 280, size, (255, 0, 0))
```

---

## Input – Keyboard

| Function | Description |
|---|---|
| `raylib::getKey() -> Option(String)` | Return the name of the key pressed this frame, or `None`. |
| `raylib::isKeyPressed(name: String) -> Bool` | `true` while the key is held down. |
| `raylib::isKeyReleased(name: String) -> Bool` | `true` the frame the key is released. |
| `raylib::isKeyUp(name: String) -> Bool` | `true` when the key is **not** pressed. |

### Key names

Key names are strings like `"KEY_A"` … `"KEY_Z"`, `"KEY_0"` … `"KEY_9"`,
`"KEY_UP"`, `"KEY_DOWN"`, `"KEY_LEFT"`, `"KEY_RIGHT"`,
`"KEY_SPACE"`, `"KEY_ENTER"`, `"KEY_ESCAPE"`,
`"KEY_LEFT_SHIFT"`, `"KEY_LEFT_CONTROL"`, `"KEY_LEFT_ALT"`,
`"KEY_TAB"`, `"KEY_BACKSPACE"`, `"KEY_DELETE"`,
`"KEY_F1"` … `"KEY_F12"`, and more.

### Example – movement

```nova
module move_demo

raylib::init("Movement", 800, 600, 60)

let px = 400
let py = 300
let speed = 5

while raylib::rendering() {
    if raylib::isKeyPressed("KEY_RIGHT") { px += speed }
    if raylib::isKeyPressed("KEY_LEFT")  { px -= speed }
    if raylib::isKeyPressed("KEY_DOWN")  { py += speed }
    if raylib::isKeyPressed("KEY_UP")    { py -= speed }

    raylib::clear((0, 0, 0))
    raylib::drawRectangle(px, py, 40, 40, (0, 200, 100))
}
```

---

## Input – Mouse

| Function | Description |
|---|---|
| `raylib::mousePosition() -> (Int, Int)` | Current mouse position as a tuple. |
| `raylib::isMousePressed(button: String) -> Bool` | `true` while a mouse button is held. |
| `raylib::isMouseReleased(button: String) -> Bool` | `true` the frame a button is released. |
| `raylib::getMouseWheel() -> Float` | Mouse wheel movement this frame (positive = up). |

Button names: `"MOUSE_BUTTON_LEFT"`, `"MOUSE_BUTTON_RIGHT"`, `"MOUSE_BUTTON_MIDDLE"`.

### Example – mouse click

```nova
module mouse_demo

raylib::init("Mouse", 800, 600, 60)

let dots = []: (Int, Int)

while raylib::rendering() {
    if raylib::isMousePressed("MOUSE_BUTTON_LEFT") {
        let pos = raylib::mousePosition()
        dots.push(pos)
    }

    raylib::clear((20, 20, 20))
    for d in dots {
        let (x, y) = d
        raylib::drawCircle(x, y, 8, (255, 100, 50))
    }
}
```

---

## Sprites

Sprites let you display images or procedurally generated pixel art.

| Function | Description |
|---|---|
| `raylib::loadSprite(path: String, height: Int, frameCount: Int) -> Sprite` | Load a sprite from an image file. `height` and `frameCount` control animation frames. |
| `raylib::buildSprite(width: Int, height: Int, frameCount: Int, pixels: [(Int,Int,Int)]) -> Sprite` | Build a sprite from a flat list of RGB pixel data. |
| `raylib::drawSprite(sprite: Sprite, x: Int, y: Int)` | Draw a sprite at a position. |

### Loading an image

```nova
module sprite_demo

raylib::init("Sprite", 800, 600, 60)
let hero = raylib::loadSprite("hero.png", 32, 1)

while raylib::rendering() {
    raylib::clear((0, 0, 0))
    raylib::drawSprite(hero, 100, 100)
}
```

### Procedural sprite

```nova
module proc_sprite

raylib::init("Proc Sprite", 800, 600, 60)

// 4×4 red/blue checkerboard
let pixels = []: (Int, Int, Int)
for y in 0..4 {
    for x in 0..4 {
        if (x + y) % 2 == 0 {
            pixels.push((255, 0, 0))
        } else {
            pixels.push((0, 0, 255))
        }
    }
}

let checker = raylib::buildSprite(4, 4, 1, pixels)

while raylib::rendering() {
    raylib::clear((30, 30, 30))
    raylib::drawSprite(checker, 100, 100)
}
```

---

## Audio

Nova's raylib integration includes full audio support for sound effects and music streams.
Call `raylib::initAudio()` once before loading any sounds or music.

| Function | Description |
|---|---|
| `raylib::initAudio()` | Initialize the audio device. Call once at startup. |
| `raylib::closeAudio()` | Close the audio device and free resources. |
| `raylib::setMasterVolume(vol: Float)` | Set the master volume (0.0 = silent, 1.0 = full). |

### Sound Effects

Sounds are short audio clips loaded entirely into memory. Good for effects like jumps, explosions, and UI clicks.

| Function | Description |
|---|---|
| `raylib::loadSound(path: String) -> Int` | Load a sound file (.wav, .ogg, .mp3). Returns a sound ID. |
| `raylib::playSound(id: Int)` | Play a sound. |
| `raylib::stopSound(id: Int)` | Stop a playing sound. |
| `raylib::pauseSound(id: Int)` | Pause a sound. |
| `raylib::resumeSound(id: Int)` | Resume a paused sound. |
| `raylib::isSoundPlaying(id: Int) -> Bool` | Check if a sound is playing. |
| `raylib::setSoundVolume(id: Int, vol: Float)` | Set per-sound volume (0.0–1.0). |
| `raylib::setSoundPitch(id: Int, pitch: Float)` | Set pitch (1.0 = normal, 2.0 = octave up). |

### Music Streams

Music is streamed from disk in chunks — ideal for background tracks that are too large to keep in memory.
You **must** call `raylib::updateMusic(id)` every frame to keep the buffer filled.

| Function | Description |
|---|---|
| `raylib::loadMusic(path: String) -> Int` | Load a music stream (.ogg, .mp3, .wav). Returns a music ID. |
| `raylib::playMusic(id: Int)` | Start playing a music stream. |
| `raylib::updateMusic(id: Int)` | Refill the stream buffer. **Call every frame.** |
| `raylib::stopMusic(id: Int)` | Stop a music stream. |
| `raylib::pauseMusic(id: Int)` | Pause a music stream. |
| `raylib::resumeMusic(id: Int)` | Resume a paused music stream. |
| `raylib::isMusicPlaying(id: Int) -> Bool` | Check if a music stream is playing. |
| `raylib::setMusicVolume(id: Int, vol: Float)` | Set volume (0.0–1.0). |
| `raylib::setMusicPitch(id: Int, pitch: Float)` | Set pitch (1.0 = normal). |
| `raylib::getMusicLength(id: Int) -> Float` | Total duration in seconds. |
| `raylib::getMusicTimePlayed(id: Int) -> Float` | Elapsed play time in seconds. |
| `raylib::seekMusic(id: Int, pos: Float)` | Seek to a position in seconds. |
| `raylib::setMusicLooping(id: Int, loop: Bool)` | Enable or disable looping. |

### Example – sound effects

```nova
module sound_demo

raylib::init("Sound Demo", 800, 600, 60)
raylib::initAudio()

let jump = raylib::loadSound("jump.wav")

while raylib::rendering() {
    if raylib::isKeyPressed("KEY_SPACE") {
        raylib::playSound(jump)
    }

    raylib::clear((30, 30, 30))
    raylib::drawText("Press SPACE to play sound", 200, 280, 20, (255, 255, 255))
}

raylib::closeAudio()
```

### Example – background music

```nova
module music_demo

raylib::init("Music Demo", 800, 600, 60)
raylib::initAudio()

let bgm = raylib::loadMusic("background.ogg")
raylib::setMusicLooping(bgm, true)
raylib::playMusic(bgm)

while raylib::rendering() {
    raylib::updateMusic(bgm)   // must call every frame!

    let played = raylib::getMusicTimePlayed(bgm)
    let total  = raylib::getMusicLength(bgm)

    raylib::clear((20, 20, 40))
    raylib::drawText(format("Music: {}/{} sec", [Cast::string(Cast::int(played).unwrap()), Cast::string(Cast::int(total).unwrap())]), 100, 280, 20, (200, 200, 200))
}

raylib::closeAudio()
```

---

## Complete Function Reference

### Window & Timing
| Function | Returns |
|---|---|
| `raylib::init(String, Int, Int, Int)` | `Void` |
| `raylib::rendering()` | `Bool` |
| `raylib::getScreenWidth()` | `Int` |
| `raylib::getScreenHeight()` | `Int` |
| `raylib::setTargetFPS(Int)` | `Void` |
| `raylib::getFPS()` | `Int` |
| `raylib::getTime()` | `Float` |
| `raylib::getFrameTime()` | `Float` |
| `raylib::sleep(Int)` | `Void` |

### Drawing
| Function | Returns |
|---|---|
| `raylib::clear((Int,Int,Int))` | `Void` |
| `raylib::drawText(String, Int, Int, Int, (Int,Int,Int))` | `Void` |
| `raylib::drawFPS(Int, Int)` | `Void` |
| `raylib::measureText(String, Int)` | `Int` |
| `raylib::drawRectangle(Int, Int, Int, Int, (Int,Int,Int))` | `Void` |
| `raylib::drawRectangleLines(Int, Int, Int, Int, (Int,Int,Int))` | `Void` |
| `raylib::drawRoundedRectangle(Int, Int, Int, Int, Float, (Int,Int,Int))` | `Void` |
| `raylib::drawCircle(Int, Int, Int, (Int,Int,Int))` | `Void` |
| `raylib::drawCircleLines(Int, Int, Int, (Int,Int,Int))` | `Void` |
| `raylib::drawLine(Int, Int, Int, Int, (Int,Int,Int))` | `Void` |
| `raylib::drawLineThick(Int, Int, Int, Int, Float, (Int,Int,Int))` | `Void` |
| `raylib::drawTriangle(Int, Int, Int, Int, Int, Int, (Int,Int,Int))` | `Void` |

### Sprites
| Function | Returns |
|---|---|
| `raylib::loadSprite(String, Int, Int)` | `Sprite` |
| `raylib::buildSprite(Int, Int, Int, [(Int,Int,Int)])` | `Sprite` |
| `raylib::drawSprite(Sprite, Int, Int)` | `Void` |

### Keyboard
| Function | Returns |
|---|---|
| `raylib::getKey()` | `Option(String)` |
| `raylib::isKeyPressed(String)` | `Bool` |
| `raylib::isKeyReleased(String)` | `Bool` |
| `raylib::isKeyUp(String)` | `Bool` |

### Mouse
| Function | Returns |
|---|---|
| `raylib::mousePosition()` | `(Int, Int)` |
| `raylib::isMousePressed(String)` | `Bool` |
| `raylib::isMouseReleased(String)` | `Bool` |
| `raylib::getMouseWheel()` | `Float` |

### Audio
| Function | Returns |
|---|---|
| `raylib::initAudio()` | `Void` |
| `raylib::closeAudio()` | `Void` |
| `raylib::setMasterVolume(Float)` | `Void` |
| `raylib::loadSound(String)` | `Int` |
| `raylib::playSound(Int)` | `Void` |
| `raylib::stopSound(Int)` | `Void` |
| `raylib::pauseSound(Int)` | `Void` |
| `raylib::resumeSound(Int)` | `Void` |
| `raylib::isSoundPlaying(Int)` | `Bool` |
| `raylib::setSoundVolume(Int, Float)` | `Void` |
| `raylib::setSoundPitch(Int, Float)` | `Void` |
| `raylib::loadMusic(String)` | `Int` |
| `raylib::playMusic(Int)` | `Void` |
| `raylib::updateMusic(Int)` | `Void` |
| `raylib::stopMusic(Int)` | `Void` |
| `raylib::pauseMusic(Int)` | `Void` |
| `raylib::resumeMusic(Int)` | `Void` |
| `raylib::isMusicPlaying(Int)` | `Bool` |
| `raylib::setMusicVolume(Int, Float)` | `Void` |
| `raylib::setMusicPitch(Int, Float)` | `Void` |
| `raylib::getMusicLength(Int)` | `Float` |
| `raylib::getMusicTimePlayed(Int)` | `Float` |
| `raylib::seekMusic(Int, Float)` | `Void` |
| `raylib::setMusicLooping(Int, Bool)` | `Void` |
