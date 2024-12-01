# nova-lang

![Screenshot](nova-logo.png)

Programming lang WIP

# Getting Started with Cargo and Nova

Nova is built in Rust, which means that you'll need to have Rust installed on your computer in order to run it. If you don't already have Rust installed, you can download it from rust-lang.org.

Once you have Rust installed, you can use Cargo to easily build and run Nova. Cargo is Rust's package manager and build tool, and it comes bundled with Rust.

# Installing Nova

To install Nova using Cargo, follow these steps:

Clone the Nova repository to your local machine by running the following command in your terminal:

    
```bash
git clone https://github.com/pyrotek45/nova-lang
```

Change your working directory to the root of the Nova repository:

```bash

cd nova
```

Build Nova using Cargo:

```bash

cargo build --release
```
This may take a few minutes, especially the first time you build Nova.

Once Cargo has finished building Nova, you can run it using the following command:

```bash

 ./target/release/nova
```

Enjoy this demo!

```swift
/module main

// Type declaration
struct Person {
    name: String,
    age: Int,
};

// Hello world
println("hello world!")


// Creating instance of type
let person : Person = Person {name: "bob", age: 42}

// Optional type annotation
let person2 = Person("joe", 50)

// Creating new variable the easy way
person3 <- Person("bobby", 30)

// Updating a variable 
person3 = Person("jesse", 25)

// Function for type
fn extends display(self: Person) {
    println(self.name)
    println(self.age)
}

// import function
person.display()
Person::display(person2)

// For loop
for i <- 0; i < 10; i += 1 {
    println(i)
}

import super.std.list

// Array
let arr = [1,2,3]
println(arr)

let arr2 = []: Int.fill(10,5)
println(arr2)
 
arr[1] = 4
println(arr)

// Changing struct value
person.name = "bingo"
person.display()

struct Zed {
    test: ()
}

// Creating an instance from a type
let zed = Zed {
    test: fn() {
        print("wow\n")
    }
}

// Now we can access its namespace and call functions directly
zed::test()

// import iterators
import super.std.iter

let myIter = Iter::fromVec([1,2,3,4,5])
    .map(fn(x:Int)->Int{return x * x})
    .collect()

println(myIter)

// Function pointers
struct SomeFunction {
    function: (Int,Int) -> Int
}

let myMul = SomeFunction(fn(x:Int,y:Int)->Int {
    return x * y
})

let myOtherFunc = myMul.function
let simpleSquare = fn(x: Int) -> Int {return x * x};

// Calling function pointer from a struct
println((myMul.function)(4,99))
println(myMul::function(4,99))

// Support for most escape chars
print("hello again!\n")

println(myOtherFunc(4,7))

let myIterTwo = Iter::fromVec([1,2,3,4,5])
    .map(simpleSquare)
    .map(simpleSquare)
    .collect()

println(myIterTwo)

// Creating an empty list
mylist <- []: Int

// function overloading
fn add(x:Int,y:Int) -> Int {
    println("im adding ints")
    return x + y
}

fn add(x:Float,y:Float) -> Float {
    println("im adding floats")
    return x + y
}

println(add(1,3))
println(add(1.0,3.0))

// Passing an overloaded function
myIntAdder <- add@(Int,Int)
println(myIntAdder(1,4))

// Generic functions
fn generic(x: $A) {
    println(x)
}

generic("hello!")
generic(10)
generic(5.5)

// More advance structs
struct Counter {
    value: Int,
    count: (Counter) -> Int,
    reset: (Counter)
}

// Creating a init funciton for Counter
fn CounterInit() -> Counter {
    return Counter {
        value: 0,
        count: fn(self: Counter) -> Int {
            result <- self.value
            self.value += 1
            return result
        },
        reset: fn(self: Counter) {
            self.value = 0
        }
    }
}

// Creating a function for counter outside of the struct
fn extends count(self: Counter) -> Int {
    println("im in a normal function")
    result <- self.value
    self.value += 1
    return result
}

mycounter <- CounterInit()

// The -> takes the function from the struct, and applys it to itself
println(mycounter->count())

// the normal function 'count' will be called here, not from the struct itself
println(mycounter.count())

// Option type ?type lets you represent none
let x: ?Int = Some(20)

// import the isSome() function here
if x.isSome() {
    println(x.unwrap())
}

x = ?Int
if x.isSome() {
    println("i never print")
    println(x.unwrap())
}

fn extends do(x: ?$A, f:($A)) {
    if x.isSome() {
        f(x.unwrap())
    }
}

x.do(fn(x:Int) {println(x)})

// String manipulation
str <- "hello world!"
    .chars()
    .filter(fn(x:Char) -> Bool {return (x != 'l') && (x != 'o') })
    .string()

println(str)

// Currying
fn add(x:Int) -> (Int) -> (Int) -> (Int) -> Int {   
    return fn(y:Int) -> (Int) -> (Int) -> Int {  
        return fn(z:Int) -> (Int) -> Int {            
            return fn(t:Int) -> Int {            
                return x + y + z + t
            }            
        }  
    }
}

inc <- add(1)(2)(3)(4)
println(inc)


fn curry(f:($A,$A) -> $A) -> ($A) -> ($A) -> $A {
    return fn(x: $A) -> ($A) -> $A {
        return fn(y: $A) -> $A {
            return f(x,y)
        }
    }
}

fn mul(x:Int,y:Int) -> Int {
    return x * y
}

curriedmul <- curry(mul@(Int,Int))

println(curriedmul(5)(5))

// using IO struct
import super.std.io

let input = io::prompt("wow")
println(input)
```