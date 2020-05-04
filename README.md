# Beagle Programming Language
Beagle is a new mid-level programming language whose sole mission is to provide high level programmers the ability to reach to low level at will, without reprocusions or a garbage collector, while maintaining memory safety and efficiency.

## How?
Beagle provides high level features with the option for memory management. It achieves memory safety using a new form of automatic memory management called Memmy. Memmy is your smart **mem**ory **m**anager (added the y at the end to make it sound better). Memmy is responsible for converting high level code into low level memory safe, memory efficient code. This is done by following 3 simple rules:
* Properties are always heap allocated
* Locals are always stack allocated
* Properties are pass-by-reference while locals are pass-by-value.

Memmy pretends there is a borrow checker in place and does the borrow checking for you. There are currently plans put into place to allow the user to take control of memory management at will to any level. There will be `ref`, `mov`, and `dup` keywords (or perhaps just `ref`, `move`, or `copy`; WIP).

## Roadmap (As of 4/15)

* 0.0.1
    * Properties
        * Properties are always heap allocated and can be referenced. They are cleaned up when their declaring scope is gone, and it no longer has living references aka, reference counting. A great point to make, as mentioned before, that this is not a runtime reference counting garbage collector, this is a compile time reference counting.
        * Properties have built in getters and setters.
        * Properties are declared using `val` or `var` for mutable property.
    * Locals
        * Locals are always stack allocated and can only ever be passed by value by moving or copying.
        * Locals do not have built in getters and setters.
        * Locals are declared with `let` or `let mut` for mutable locals.
    * Functions
    * Lambdas (just like in Kotlin)
    * Control flow
        * If-Else
            * Else-If will be replaced with *when*. if-else is only for binary branching.
        * when
        * match
            * This will be used for unwrapping objects, as nullability will be expressed differently than in Kotlin. I will make another post about how type ignorance works but part of it is nullability but the way its represented in memory is an **Option<T\>** object. If you aren't sure what this means please refer to [this Rust document](https://doc.rust-lang.org/std/option/) on the **Result** and **Option** wrappers.
        * loop
             * Infinite loop. Sugar for the `while(true)` loop.
        * for
            * Ranged for or foreach using `for(x in thing)`
        * unless
            * Sugar for `if(!x)`
        * until
            * Sugar for `for(i in 0..n-1)`. Syntax: `until(10)`
    * Modules
        * Modules are single files that act as a top-level domain for a set of code. The module setup will work the same way as in Rust. A root project will have a `src/` where inside of that is either a `lib.bg` or `main.bg`. Apart from that, modules can be single files whose names have been declared by another module. When a module (`mod a`) is declared by another module (`mod b`), a then becomes a submodule of b and can be imported like this: `import b::a`. This syntax will import all the public code declared in that module. This will require a different post to further the information on this.

    * Expressions
        * I don't think its necessary to explain what expressions exist in Beagle. Just know that all the control flow mentioned above can be used as expressions.

* 0.0.2
    * Classes
        * open classes
        * abstract classes
        * member properties and methods/member functions
    * Interfaces
        * Only classes can implement interfaces.
    * Inheritance
        * Inheritance will be represented in memory as a VTable (as they usually are in languages like Rust and C++ and I think C#).
    * Extension functions
    * Encapsulation

* 0.0.3
    * Structs
        * Structs cannot be inherited from nor do they inherit anything. Structs can be abstracted by traits.
    * Traits
        * Traits will be similar to interfaces except they are a compile time only construct that is only found in memory as a bitflag. Structs will be compiled with an extra invisible field that will act as a bitmask, which contains all the flags that represent their traits. Using bitwise AND at runtime, we can find out if an object of a struct based type has the composition of some trait. I will write more about this some other time. This allows you to focus on managing objects that are more concise in memory. It's a complicated subject that requires an entire detailed explanation.

* 0.0.4
    * Broad types
        * Broad types are a way of creating an abstraction that can be implemented by either classes or structs. As mentioned above classes can only implement interfaces and structs can only implement traits. A broad type allows you to create an abstraction that can span across both type systems. A great example of this is `Iterable<T>`. `Iterable<T>` is a type that provides members and composition for types to be used in a foreach loop directly. `for(x in some_iterable_obj)`. Iterables will exist in both type systems. Class based iterables would include *List, Set, Map, Deque, Tree*. Struct based iterables would include *Vector, Scalar, Array, Table, Dictionary*
    * Enums
        * Enums will be a way of tagging data, [similar to in Rust](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html).

* 0.0.5
    * Generics
    * Type Ignorance
        * Type ignorance allows a user to tell the compiler that they do not know everything about an object's type, but provide the information they *do* know. This could be nullability, class only, struct only, struct with trait(s), broad type, or absolute ignorance (similar to an opaque type in C/C++).
