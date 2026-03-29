# Built-in Types

This document describes Nova's built-in type representations.

## Primitive Types

| Type | Description |
|---|---|
| `Int` | 64-bit signed integer. |
| `Float` | 64-bit IEEE 754 floating-point number. |
| `Bool` | Boolean value: `true` or `false`. |
| `String` | UTF-8 encoded text. |
| `Char` | Single Unicode character. |
| `Void` | Absence of a return value. |

## Composite Types

### `Option(T)`

An optional value -- either `Some(value)` or `None`.

- `inner: T` -- the type of the contained value.

### `List`

A dynamically-sized sequence of elements of a single type, written `[T]`.

- `inner: T` -- the element type.

### `Tuple`

A fixed-size, heterogeneous collection, written `(A, B, ...)`.

- `elements: [Type]` -- the type of each position.

### `Function`

A callable value with typed parameters and a return type, written `fn(A, B) -> R`.

- `parameters: [Type]` -- the parameter types.
- `return_type: Type` -- the return type.

## User-defined Types

### `Custom`

A user-defined struct or enum type.

- `name: String` -- the type name (e.g. `Person`, `Shape`).
- `type_params: [Type]` -- type parameters for generic types (e.g. `Pair(Int, String)`).

### `Generic`

A type variable used in generic definitions, written `$T`.

- `name: String` -- the name of the type variable (e.g. `T`, `A`, `B`).

## Special Types

### `Dyn`

A structural constraint type for duck-typed dispatch. Written `Dyn(T = field: Type + ...)`.
Any struct with matching fields satisfies the constraint.

### `None`

The absence of a value inside an `Option`. Can be written as `None(T)` to specify the
expected inner type.
