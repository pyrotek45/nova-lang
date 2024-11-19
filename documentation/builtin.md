### Built-in Functions

#### `fn exit() -> Void`
Terminates the program.

#### `fn typeof(a) -> String`
Returns the type of the given value as a string.

#### `fn isSome(?a) -> Bool`
Checks if the given option contains a value.

#### `fn unwrap(?a) -> a`
Extracts the value from an option and panicking if it is `None`.

#### `fn Some(a) -> ?a`
Wraps a value in an option

#### `fn print(a) -> Void`
Prints the given value to the standard output.

#### `fn println(a) -> Void`
Prints the given value to the standard output, followed by a newline.

#### `fn clone(a) -> a`
Creates a deep copy of the given value.

#### `fn cliArgs() -> [String]`
Retrieves the command line arguments.

#### `fn hidecursor() -> Void`
Hides the cursor in the terminal.

#### `fn showcursor() -> Void`
Shows the cursor in the terminal.

#### `fn toInt(a) -> ?Int`
Converts a generic value to an integer, if possible.

#### `fn toStr(a) -> String`
Converts a generic value to a string.

#### `fn len([a]) -> Int`
Returns the length of a list.

#### `fn sleep(Int) -> Void`
Pauses the program for a specified number of milliseconds.

#### `fn rawmode(Bool) -> Void`
Enables or disables raw mode in the terminal.

#### `fn getch() -> ?Char`
Reads a single character from the terminal without waiting for a newline.

#### `fn rawread(Int) -> ?Char`
Reads a specified number of characters from the terminal in raw mode.

#### `fn readln() -> String`
Reads a line of input from the terminal.

#### `fn clearscreen() -> Void`
Clears the terminal screen.

#### `fn push([a], a) -> Void`
Adds an element to the end of a list.

#### `fn pop([a]) -> ?a`
Removes and returns the last element of a list.

#### `fn randomInt(Int, Int) -> Int`
Generates a random integer within a specified range.

#### `fn strlen(String) -> Int`
Returns the length of a string.

#### `fn strToChars(String) -> [Char]`
Converts a string to a list of characters.

#### `fn charsToStr([Char]) -> String`
Converts a list of characters to a string.

#### `fn toStr(a) -> String`
Converts a generic value to a string.

#### `fn toChar(Int) -> Char`
Converts an integer to a character.

#### `fn readFile(String) -> String`
Reads the contents of a file and returns it as a string.