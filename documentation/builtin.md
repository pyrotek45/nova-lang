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
| `toChar(Int) -> Char` | Convert an integer to a character. |

## String and Char

| Signature | Description |
|---|---|
| `strlen(String) -> Int` | Return the length of a string. |
| `strToChars(String) -> [Char]` | Convert a string to a list of characters. |
| `charsToStr([Char]) -> String` | Convert a list of characters to a string. |

## Lists

| Signature | Description |
|---|---|
| `len([a]) -> Int` | Return the length of a list. |
| `push([a], a) -> Void` | Append an element to a list. |
| `pop([a]) -> Option(a)` | Remove and return the last element. |

## I/O

| Signature | Description |
|---|---|
| `readln() -> String` | Read a line of input from stdin. |
| `readFile(String) -> String` | Read the contents of a file. |

## Random

| Signature | Description |
|---|---|
| `randomInt(Int, Int) -> Int` | Generate a random integer in a range (inclusive). |

## Terminal

| Signature | Description |
|---|---|
| `clearscreen() -> Void` | Clear the terminal screen. |
| `hidecursor() -> Void` | Hide the terminal cursor. |
| `showcursor() -> Void` | Show the terminal cursor. |
| `rawmode(Bool) -> Void` | Enable or disable terminal raw mode. |
| `getch() -> Option(Char)` | Read a single character without waiting for newline. |
| `rawread(Int) -> Option(Char)` | Read a character in raw mode with a timeout (ms). |
| `sleep(Int) -> Void` | Pause execution for a number of milliseconds. |

## Program

| Signature | Description |
|---|---|
| `exit() -> Void` | Terminate the program. |
| `cliArgs() -> [String]` | Retrieve command-line arguments. |
