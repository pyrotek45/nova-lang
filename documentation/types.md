### Built-in Types

This is how the built-in types are represented under the hood. 


#### `None`
Represents the absence of a value. can be written like `?type`

#### `Int`
Represents an integer type.

#### `Float`
Represents a floating-point number type.

#### `Bool`
Represents a boolean type, which can be either `true` or `false`.

#### `String`
Represents a sequence of characters.

#### `Char`
Represents a single character.

#### `Void`
Represents the absence of a return value.

#### `Custom`
Represents a user-defined type with a name and optional type parameters.
- `name: String` - The name of the custom type.
- `type_params: Vec<TType>` - The type parameters for the custom type.

#### `List`
Represents a list of elements of a specific type.
- `inner: Box<TType>` - The type of elements contained in the list.

#### `Function`
Represents a function type with parameters and a return type.
- `parameters: Vec<TType>` - The types of the function parameters.
- `return_type: Box<TType>` - The return type of the function.

#### `Generic`
Represents a generic type with a name.
- `name: String` - The name of the generic type.

#### `Option`
Represents an optional value that can either be `Some` containing a value or `None`.
- `inner: Box<TType>` - The type of the value contained in the option.

#### `Tuple`
Represents a tuple containing multiple elements of specific types.
- `elements: Vec<TType>` - The types of the elements in the tuple.