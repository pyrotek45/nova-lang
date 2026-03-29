# Built-in Functions

These functions are available without any imports.

## Output

| Signature | Description |
|---|---|
| `print(a) -> Void` | Print a value to stdout. |
| `println(a) -> Void` | Print a value to stdout followed by a newline. |

## Type Inspection

| Signature | Description |
|---|---|
| `typeof(a) -> String` | Return the type of a value as a string. |
| `clone(a) -> a` | Create a deep copy of a value. |

## Option Handling

| Signature | Description |
|---|---|
| `Some(a) -> Option(a)` | Wrap a value in an Option. |
| `None(T) -> Option(T)` | Create an empty Option of a given type. |
| `isSome(Option(a)) -> Bool` | Check if an Option contains a value. |
| `unwrap(Option(a)) -> a` | Extract the value from an Option. Panics if None. |

## Type Conversion

| Signature | Description |
|---|---|
| `Cast::string(a) -> String` | Convert any value to a String. |
| `Cast::int(a) -> Option(Int)` | Convert a value to Int. Returns None on failure. |
| `Cast::float(a) -> Option(Float)` | Convert a value to Float. Returns None on failure. |
| `toStr(a) -> String` | Alias for `Cast::string`. |
| `toInt(a) -> Option(Int)` | Alias for `Cast::int`. |
| `chr(Int) -> Char` | Convert an integer (codepoint) to a character. |

## String Functions (`String::`)

| Signature | Description |
|---|---|
| `strlen(String) -> Int` | Return the length of a string. |
| `strToChars(String) -> [Char]` | Convert a string to a list of characters. |
| `charsToStr([Char]) -> String` | Convert a list of characters to a string. |
| `String::contains(String, String) -> Bool` | Check if a string contains a substring. |
| `String::startsWith(String, String) -> Bool` | Check if a string starts with a prefix. |
| `String::endsWith(String, String) -> Bool` | Check if a string ends with a suffix. |
| `String::trim(String) -> String` | Remove leading and trailing whitespace. |
| `String::trimStart(String) -> String` | Remove leading whitespace. |
| `String::trimEnd(String) -> String` | Remove trailing whitespace. |
| `String::toUpper(String) -> String` | Convert to uppercase. |
| `String::toLower(String) -> String` | Convert to lowercase. |
| `String::replace(String, String, String) -> String` | Replace all occurrences of a substring. |
| `String::substring(String, Int, Int) -> String` | Extract a substring by start index and length. |
| `String::indexOf(String, String) -> Option(Int)` | Find the first index of a substring, or None. |
| `String::repeat(String, Int) -> String` | Repeat a string N times. |
| `String::reverse(String) -> String` | Reverse the characters in a string. |
| `String::isEmpty(String) -> Bool` | Check if a string is empty. |
| `String::charAt(String, Int) -> Option(Char)` | Get the character at an index, or None. |
| `String::split(String, String) -> [String]` | Split a string by a delimiter. |
| `join([String], String) -> String` | Join a list of strings with a separator. |
| `String::toInt(String) -> Option(Int)` | Parse a string as an integer. |

## Char Functions (`Char::`)

| Signature | Description |
|---|---|
| `Char::isAlpha(Char) -> Bool` | Check if a character is alphabetic. |
| `Char::isDigit(Char) -> Bool` | Check if a character is a digit. |
| `Char::isWhitespace(Char) -> Bool` | Check if a character is whitespace. |
| `Char::isAlphanumeric(Char) -> Bool` | Check if a character is alphanumeric. |
| `Char::toUpper(Char) -> Char` | Convert a character to uppercase. |
| `Char::toLower(Char) -> Char` | Convert a character to lowercase. |
| `Char::isUpper(Char) -> Bool` | Check if a character is uppercase. |
| `Char::isLower(Char) -> Bool` | Check if a character is lowercase. |
| `Char::toInt(Char) -> Int` | Get the Unicode codepoint of a character. |

## Float Functions (`Float::`)

| Signature | Description |
|---|---|
| `Float::floor(Float) -> Float` | Round down to the nearest integer. |
| `Float::ceil(Float) -> Float` | Round up to the nearest integer. |
| `Float::round(Float) -> Float` | Round to the nearest integer. |
| `Float::abs(Float) -> Float` | Absolute value. |
| `Float::sqrt(Float) -> Float` | Square root. |
| `Float::sin(Float) -> Float` | Sine (radians). |
| `Float::cos(Float) -> Float` | Cosine (radians). |
| `Float::tan(Float) -> Float` | Tangent (radians). |
| `Float::atan2(Float, Float) -> Float` | Two-argument arctangent. |
| `Float::log(Float) -> Float` | Natural logarithm. |
| `Float::log10(Float) -> Float` | Base-10 logarithm. |
| `Float::pow(Float, Float) -> Float` | Raise to a power. |
| `Float::exp(Float) -> Float` | e raised to a power. |
| `Float::min(Float, Float) -> Float` | Minimum of two floats. |
| `Float::max(Float, Float) -> Float` | Maximum of two floats. |
| `Float::clamp(Float, Float, Float) -> Float` | Clamp a value between a min and max. |
| `Float::isNan(Float) -> Bool` | Check if a float is NaN. |
| `Float::isInfinite(Float) -> Bool` | Check if a float is infinite. |
| `Float::pi() -> Float` | The constant π. |
| `Float::e() -> Float` | The constant e. |

## List Functions

| Signature | Description |
|---|---|
| `len([a]) -> Int` | Return the length of a list. |
| `push([a], a) -> Void` | Append an element to a list. |
| `pop([a]) -> Option(a)` | Remove and return the last element. |
| `remove([a], Int) -> Void` | Remove an element at an index. |
| `insert([a], Int, a) -> Void` | Insert an element at an index. |
| `swap([a], Int, Int) -> Void` | Swap two elements by index. |
| `clear([a]) -> Void` | Remove all elements from a list. |
| `set([a], Int, a) -> Void` | Set the element at an index. |

## I/O

| Signature | Description |
|---|---|
| `readln() -> String` | Read a line of input from stdin. |
| `readFile(String) -> String` | Read the contents of a file. |
| `writeFile(String, String) -> Void` | Write content to a file (creates or overwrites). |
| `appendFile(String, String) -> Void` | Append content to a file. |
| `fileExists(String) -> Bool` | Check if a file exists. |
| `printf(String, [String]) -> Void` | Print a formatted string (`{}` placeholders). |
| `format(String, [String]) -> String` | Format a string without printing it. |

## Random

| Signature | Description |
|---|---|
| `random(Int, Int) -> Int` | Generate a random integer in a range (inclusive). |
| `randomFloat(Float, Float) -> Float` | Generate a random float in a range. |
| `randomBool() -> Bool` | Generate a random boolean. |

## Time

| Signature | Description |
|---|---|
| `sleep(Int) -> Void` | Pause execution for the given number of milliseconds. |
| `now() -> Int` | Current time in milliseconds since the Unix epoch. |
| `nowSec() -> Int` | Current time in seconds since the Unix epoch. |

## Terminal

| Signature | Description |
|---|---|
| `terminal::clearScreen() -> Void` | Clear the terminal screen. |
| `terminal::hideCursor() -> Void` | Hide the terminal cursor. |
| `terminal::showCursor() -> Void` | Show the terminal cursor. |
| `terminal::rawmode(Bool) -> Void` | Enable or disable terminal raw mode. |
| `terminal::getch() -> Option(Char)` | Read a single character without waiting for newline. |
| `terminal::rawread(Int) -> Option(Char)` | Read a character in raw mode with a timeout (ms). |

## Regex

| Signature | Description |
|---|---|
| `Regex::matches(String, String) -> Bool` | Test whether a string matches a regex pattern. |
| `Regex::first(String, String) -> Option((Int, Int, String))` | Find the first match: (start, end, text). |
| `Regex::captures(String, String) -> [String]` | Return all capture groups from a match. |

## Program

| Signature | Description |
|---|---|
| `exit() -> Void` | Terminate the program. |
| `cliArgs() -> [String]` | Retrieve command-line arguments. |

## Raylib

See the [Raylib Guide](raylib.md) for detailed usage, examples, and key-name reference.

### Window & Timing
| Signature | Description |
|---|---|
| `raylib::init(String, Int, Int, Int) -> Void` | Create a window (title, width, height, fps). |
| `raylib::rendering() -> Bool` | Process a frame. Returns false when the window closes. |
| `raylib::getScreenWidth() -> Int` | Get the window width. |
| `raylib::getScreenHeight() -> Int` | Get the window height. |
| `raylib::setTargetFPS(Int) -> Void` | Change the target FPS. |
| `raylib::getFPS() -> Int` | Get the current FPS. |
| `raylib::getTime() -> Float` | Seconds since init. |
| `raylib::getFrameTime() -> Float` | Delta time for the last frame. |
| `raylib::sleep(Int) -> Void` | Pause execution (milliseconds). |

### Drawing
| Signature | Description |
|---|---|
| `raylib::clear((Int,Int,Int)) -> Void` | Clear the background. |
| `raylib::drawText(String, Int, Int, Int, (Int,Int,Int)) -> Void` | Draw text. |
| `raylib::drawFPS(Int, Int) -> Void` | Draw the FPS counter. |
| `raylib::measureText(String, Int) -> Int` | Pixel width of text at a font size. |
| `raylib::drawRectangle(Int, Int, Int, Int, (Int,Int,Int)) -> Void` | Filled rectangle. |
| `raylib::drawRectangleLines(Int, Int, Int, Int, (Int,Int,Int)) -> Void` | Rectangle outline. |
| `raylib::drawRoundedRectangle(Int, Int, Int, Int, Float, (Int,Int,Int)) -> Void` | Rounded rectangle. |
| `raylib::drawCircle(Int, Int, Int, (Int,Int,Int)) -> Void` | Filled circle. |
| `raylib::drawCircleLines(Int, Int, Int, (Int,Int,Int)) -> Void` | Circle outline. |
| `raylib::drawLine(Int, Int, Int, Int, (Int,Int,Int)) -> Void` | 1-pixel line. |
| `raylib::drawLineThick(Int, Int, Int, Int, Float, (Int,Int,Int)) -> Void` | Thick line. |
| `raylib::drawTriangle(Int, Int, Int, Int, Int, Int, (Int,Int,Int)) -> Void` | Filled triangle. |

### Sprites
| Signature | Description |
|---|---|
| `raylib::loadSprite(String, Int, Int) -> Sprite` | Load a sprite from a file. |
| `raylib::buildSprite(Int, Int, Int, [(Int,Int,Int)]) -> Sprite` | Build a sprite from pixel data. |
| `raylib::drawSprite(Sprite, Int, Int) -> Void` | Draw a sprite. |

### Keyboard
| Signature | Description |
|---|---|
| `raylib::getKey() -> Option(String)` | Key pressed this frame. |
| `raylib::isKeyPressed(String) -> Bool` | Is the key held down? |
| `raylib::isKeyReleased(String) -> Bool` | Was the key released this frame? |
| `raylib::isKeyUp(String) -> Bool` | Is the key not pressed? |

### Mouse
| Signature | Description |
|---|---|
| `raylib::mousePosition() -> (Int, Int)` | Current mouse position. |
| `raylib::isMousePressed(String) -> Bool` | Is the button held? |
| `raylib::isMouseReleased(String) -> Bool` | Was the button released this frame? |
| `raylib::getMouseWheel() -> Float` | Mouse wheel movement. |
