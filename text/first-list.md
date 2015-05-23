
Alright, so what's a linked list? Well basically, it's a bunch of pieces of data on the heap (shhh Linux Kernel!) that point to each other in sequence. Linked lists are something procedural programmers shouldn't touch with a 10-foot pole, and what functional programmers use for everything. It seems fair, then, that we should ask functional programmers for the definition of a linked list. They will probably give you something like the following definition:

```rust
List a = Empty | Elem a (List a)
```

Which reads approximately as "A List is either Empty or an Element followed by a List". This is a recursive definition expressed as a *sum type*, which is a fancy name for "a type that can have different values which may be different types". Rust in fact has sum types. We call them `enum`s! If you're coming from a C-like language, this is exactly the enum you known and love, but on meth. So let's transcribe this functional definition into Rust!

We'll put our first list in `first.rs`. We need to tell Rust that `first.rs` is something that our lib uses. All the requires is that we put this at the top of `lib.rs` (which Cargo made for us):

```rust
// in lib.rs
pub mod first;
```

Now we can focus on building our list:

```rust
// in first.rs

// pub says we want people outside this module to be able to use List
pub enum List<T> {
    Empty,
    Elem(T, List<T>),
}
```

*phew*, I'm swamped. Let's just go ahead and compile that:

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:1:1: 4:2 error: illegal recursive enum type; wrap the inner value in a box to make it representable [E0072]
src/first.rs:1 pub enum List<T> {
src/first.rs:2    Empty,
src/first.rs:3    Elem(T, List<T>),
src/first.rs:4 }
error: aborting due to previous error
Could not compile `lists`.
```

Noooooooo!!!! Functional programmers tricked us! That made us do something *illegal*! This is entrapment!

...

I'm ok now. Are you ok now? If we actually check out the error message (instead of getting ready to flee the country, as \*ahem\* *some* of us did), we can see that rustc is actually telling us exactly how to solve this problem: 

> illegal recursive enum type; wrap the inner value in a box to make it representable

Alright, `box`. What's that? Let's google `rust box`...

> [std::boxed::Box - Rust](https://doc.rust-lang.org/std/boxed/struct.Box.html) 

Lesse here...

> `pub struct Box<T>(_);`
>
> A pointer type for heap allocation.
> See the [module-level documentation](https://doc.rust-lang.org/std/boxed/) for more.

*clicks link*

> `Box<T>`, casually referred to as a 'box', provides the simplest form of heap allocation in Rust. Boxes provide ownership for this allocation, and drop their contents when they go out of scope.
>
> Examples
>
> Creating a box:
> 
> `let x = Box::new(5);`
>
> Creating a recursive data structure:
>
```
#[derive(Debug)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

fn main() {
    let list: List<i32> = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
    println!("{:?}", list);
}
```
>
> This will print `Cons(1, Box(Cons(2, Box(Nil))))`.
>
> Recursive structures must be boxed, because if the definition of Cons looked like this:
> 
> `Cons(T, List<T>),`
>
> It wouldn't work. This is because the size of a List depends on how many elements are in the list, and so we don't know how much memory to allocate for a Cons. By introducing a Box, which has a defined size, we know how big Cons needs to be.

Wow, uh. That is perhaps the most relevant and helpful documentation I have ever seen. Literally the first thing in the documentation is *exactly what we're trying to write, why it didn't work, and how to fix it*. Dang, yo.

Ok, let's do that:

```rust
pub enum List<T> {
    Empty,
    Elem(T, Box<List<T>>),
}
```

> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)

Hey it built!

...but this is actually a really stupid definition of a List. In particular, we're allocating the `Empty` at the end of every list *on the heap*. This is a strong sign that we're doing something silly. 

So how do we write our List? Well, we could do something like:

```rust
pub enum List<T> {
    Empty,
    ElemThenEmpty(T),
    ElemThenNotEmpty(T, Box<List<T>>),
}
```

but not only is this really complicating things, it's preventing Rust's sweet null pointer optimization! 

In general, if we have an enum like:

```rust
enum Foo {
    D1(T1),
    D2(T2),
    ...
    Dn(Tn),
}
```

it will require `max(sizeof(T1), sizeof(T2), ... sizeof(Tn)) + sizeof_smallest_primitive_to_store(n)` space. The `max` part is of course entirely necessary; we can't use less space than the biggest variant! The extra amount we add on is to store the *tag* of the enum (almost always, this will be a byte), which specifies which variant the the rest of the bits is supposed to represent. However, if we have a special kind of enum:

```rust 
enum Foo {
    A,
    B(ContainsANonNullPtr),
}
```

we get the null pointer optimization, which *eliminates the tag*. If the variant is A, the whole enum is set to all `0`'s. Otherwise, the variant is B. This works because B can never be all `0`'s, because it contains a non-zero pointer! Slick!

So how do we avoid the extra allocation *and* get that sweet null-pointer optimization? We just have to think a little more C-like: structs!

```rust
struct Node<T> {
    elem: T,
    next: List<T>,
}

pub enum List<T> {
    Empty,
    More(Box<Node<T>>),
}
```

Let's check our priorities: 
* tail of a list never allocates: check!
* enum is in delicious null-pointer form: check!

Alright!

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:8:11: 8:18 error: private type in exported type signature
src/first.rs:8    More(Box<Node<T>>),
                           ^~~~~~~
error: aborting due to previous error
Could not compile `lists`.
```

:(

Rust is mad at us again. We marked the `List` as public (because we want people to be able to use it), but not the `Node`. The problem is that the internals of an `enum` are totally public, and we're not allowed to publicly talk about private types. We could make all of `Node` totally public, but generally in Rust we favour keeping implementation details private. Let's make `List` a struct, so that we can hide the implementation details:


```rust
pub struct List<T> {
    head: Link<T>,
}

enum Link<T> {
    Empty,
    More(Box<Node<T>>),
}

struct Node<T> {
    elem: T,
    next: Link<T>,
}
```

Because `List` is a struct with a single field, its size is the same as that field; zero-cost abstraction, yay!

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:2:2: 2:15 warning: struct field is never used: `head`, #[warn(dead_code)] on by default
src/first.rs:2    head: Link<T>,
                  ^~~~~~~~~~~~~
src/first.rs:6:2: 6:7 warning: variant is never used: `Empty`, #[warn(dead_code)] on by default
src/first.rs:6    Empty,
                  ^~~~~
src/first.rs:7:2: 7:20 warning: variant is never used: `More`, #[warn(dead_code)] on by default
src/first.rs:7    More(Box<Node<T>>),
                  ^~~~~~~~~~~~~~~~~~
src/first.rs:11:2: 11:9 warning: struct field is never used: `elem`, #[warn(dead_code)] on by default
src/first.rs:11   elem: T,
                  ^~~~~~~
src/first.rs:12:2: 12:15 warning: struct field is never used: `next`, #[warn(dead_code)] on by default
src/first.rs:12   next: Link<T>,
                  ^~~~~~~~~~~~~
```

Alright, that compiled! Rust is pretty mad, because as far as it can tell, everything we've written is totally useless: we never use `head`, and no one who uses our library can either since it's private. Transitively, that means Link and Node are useless too. So let's solve that! Let's implement some code for our List!

To add methods to a type, we use `impl` blocks. In this case, we want to implement a method for `List<T>`, for *all* choices of `T`, so the `impl` itself must be generic over `T`:

```rust
impl<T> List<T> {
    // TODO, make code happen
}
```

If we wanted to implement something for just, say, `List<u8>` we wouldn't need the extra generic:

```rust
impl List<u8> {
    // TODO, make code happen
}
```

`push` *mutates* the list, so we'll want to take `self` mutably. We also need to take `T` to push:

```rust
impl<T> List<T> {
    pub fn push(&mut self, elem: T) {
        // TODO
    }
}
```

`&mut` is a *mutable reference* or *borrow*. For now, all you need to know is that it's how you mutate stuff without moving it around. 

So first thing's first, we need to make a node to store our element in:

```rust
impl<T> List<T> {
    pub fn push(&mut self, elem: T) {
        let new_node = Node {
            elem: elem,
            next: ?????
        };
    }
}
```

What goes `next`? Well, the entire old list! Can we... just do that?

```rust
impl<T> List<T> {
    pub fn push(&mut self, elem: T) {
        let new_node = Node {
            elem: elem,
            next: self.head,
        };
    }
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:19:10: 19:14 error: cannot move out of borrowed content
src/first.rs:19           next: self.head,
                                ^~~~
error: aborting due to previous error
Could not compile `lists`.
```

Nooooope. Unfortunately, Rust has nothing really helpful to tell us here:

> cannot move out of borrowed content

We're trying to move the `self.head` field out to `next`, but Rust doesn't want us doing that. This would leave `self` only partially initialized when we end the borrow and "give it back" to its rightful owner. That would be super rude, and Rust is a very polite person (it would also be incredibly dangerous, but obviously that isn't why it cares). What if we put something back? Namely, the node that we're creating.


```rust
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            next: self.head,
        });

        self.head = Link::More(new_node);
    }
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:19:10: 19:14 error: cannot move out of borrowed content
src/first.rs:19           next: self.head,
                                ^~~~
error: aborting due to previous error
Could not compile `lists`.
```

No dice. In principle, this is something Rust could actually accept, but it's not (for various reasons). We need some way to get the head without Rust noticing that it's gone. For advice, we turn to infamous Rust Hacker Indiana Jones:

![Indy Prepares to mem::swap](indy.gif)

Ah yes, Indy suggests the `mem::replace` maneuver. This incredibly useful function lets us steal a value out of a borrow by *replacing* it with another value. Let's just pull in `std::mem` at the top of the file, so that `mem` is in local scope:

```rust
use std::mem;
```

and use it appropriately:

```rust
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(new_node);
    }
```

Here we `replace` self.head temporarily with Link::Empty before replacing it with the new head of the list. I'm not gonna lie: this is a pretty unfortunate thing to have to do. Sadly, we must (for now).

But hey, that's `push` all done! Probably. We should probably test it, honestly. Right now the easiest way to do that is probably to write `pop`, and make sure that it produces the right results.

```rust
    pub fn pop(&mut self) -> Option<T> {
        //TODO
    }
```

So that's going to be our signature: we take ourselves mutably, and (maybe) return the T at the front of the list. That's what the `Option<T>` type represents. It's either `Some(T)` or `None`. Option is so important that it's implicitly imported into scope in every file, as well as its variants `Some` and `None` (so we don't have to say `Option::None`).

So uh, we have this `Link` thing, how do we figure out if it's Empty or has More? Pattern matching with `match`!

```rust
    pub fn pop(&mut self) -> Option<T> {
        match self.head {
            Link::Empty => {
                // TODO
            }
            Link::More(Node) => {
                // TODO
            }
        };
    }
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:27:2: 36:3 error: not all control paths return a value [E0269]
src/first.rs:27   pub fn pop(&mut self) -> Option<T> {
src/first.rs:28       match self.head {
src/first.rs:29           Link::Empty => {
src/first.rs:30               // TODO
src/first.rs:31           }
src/first.rs:32           Link::More(node) => {
              ...
error: aborting due to previous error
Could not compile `lists`.
```

Whoops, `pop` has to return a value, and we're not doing that yet. We *could* return `None`, but in this case it's probably a better idea to return `unreachable!()`, to indicate that we aren't done implementing the function. `unreachable!()` is a macro (`!` indicates a macro) that panics (basically just crashes) the program when we get to it. Unconditional panics have a special type that unifies with any other type, so we can "return" it where an Option is expected.

```
    pub fn pop(&mut self) -> Option<T> {
        match self.head {
            Link::Empty => {
                // TODO
            }
            Link::More(node) => {
                // TODO
            }
        };
        unreachable!()
    }
```

Note also that we don't need to write `return` in our program. The last statement in a function is implicitly its return value.

```
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:28:9: 28:13 error: cannot move out of borrowed content
src/first.rs:28       match self.head {
                            ^~~~
src/first.rs:32:15: 32:19 note: attempting to move value to here
src/first.rs:32           Link::More(node) => {
                                     ^~~~
src/first.rs:32:15: 32:19 help: to prevent the move, use `ref node` or `ref mut node` to capture value by reference
error: aborting due to previous error
Could not compile `lists`.
```

Come on Rust, get off our back! As always, Rust is hella mad at us. Thankfully, it is also giving us the full scoop!

```
src/first.rs:17:9: 17:13 error: cannot move out of borrowed content
src/first.rs:17       match self.head {
                            ^~~~
src/first.rs:21:15: 21:19 note: attempting to move value to here
src/first.rs:21           Link::More(node) => {
                                     ^~~~
```

Pattern matches by default move the value they capture.

```
help: to prevent the move, use `ref node` or `ref mut node` to capture value by reference
```

to avoid that, we use the `ref` keyword to indicate that we want to bind the `node` subpattern by reference instead. Let's do that:

```rust
    pub fn pop(&mut self) -> Option<T> {
        match self.head {
            Link::Empty => {
                // TODO
            }
            Link::More(ref node) => {
                // TODO
            }
        };
        unreachable!()
    }
```

```
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:13:2: 13:9 warning: struct field is never used: `elem`, #[warn(dead_code)] on by default
src/first.rs:13   elem: T,
                  ^~~~~~~
src/first.rs:14:2: 14:15 warning: struct field is never used: `next`, #[warn(dead_code)] on by default
src/first.rs:14   next: Link<T>,
                  ^~~~~~~~~~~~~
src/first.rs:32:15: 32:23 warning: unused variable: `node`, #[warn(unused_variables)] on by default
src/first.rs:32           Link::More(ref node) => {
                                     ^~~~~~~~
```

Hooray, compiling again! Now let's figure out that logic. We want to make an Option, so let's make a variable for that. In the Empty case we need to return None. In the More case we need to return Some(T).

```rust
    pub fn pop(&mut self) -> Option<T> {
        let result;
        match self.head {
            Link::Empty => {
                result = None
            }
            Link::More(ref node) => {
                result = Some(node.elem)
            }
        };
        result
    }
```
