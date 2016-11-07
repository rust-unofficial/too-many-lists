# Basic Data Layout

Alright, so what's a linked list? Well basically, it's a bunch of pieces of data
on the heap (shhh Linux Kernel!) that point to each other in sequence. Linked
lists are something procedural programmers shouldn't touch with a 10-foot pole,
and what functional programmers use for everything. It seems fair, then, that we
should ask functional programmers for the definition of a linked list. They will
probably give you something like the following definition:

```haskell
List a = Empty | Elem a (List a)
```

Which reads approximately as "A List is either Empty or an Element followed by a
List". This is a recursive definition expressed as a *sum type*, which is a
fancy name for "a type that can have different values which may be different
types". Rust calls sum types `enum`s! If you're coming from a C-like language,
this is exactly the enum you know and love, but on meth. So let's transcribe
this functional definition into Rust!

For now we'll avoid generics to keep things simple. We'll only support
storing signed 32-bit integers:

```rust,ignore
// in first.rs

// pub says we want people outside this module to be able to use List
pub enum List {
    Empty,
    Elem(i32, List),
}
```

*phew*, I'm swamped. Let's just go ahead and compile that:

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:1:1: 4:2 error: recursive type `first::List` has infinite size [E0072]
src/first.rs:1 pub enum List {
              ^
src/first.rs:1:1: 4:2 help: run `rustc --explain E0072` to see a detailed explanation
src/first.rs:1:1: 4:2 help: insert indirection (e.g., a `Box`, `Rc`, or `&`) at some point to make `first::List` representable
error: aborting due to previous error
error: Could not compile `list`.

To learn more, run the command again with --verbose.
```

Noooooooo!!!! Functional programmers tricked us! That made us do something
*illegal*! This is entrapment!

...

I'm ok now. Are you ok now? If we actually check out the error message (instead
of getting ready to flee the country, as \*ahem\* *some* of us did), we can see
that rustc is actually telling us exactly how to solve this problem:

> insert indirection (e.g., a `Box`, `Rc`, or `&`) at some point to make `first::List` representable

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
```
>
```
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

Wow, uh. That is perhaps the most relevant and helpful documentation I have ever seen. Literally the first thing in the documentation is *exactly what we're trying to write, why it didn't work, and how to fix it*. Dang, yo. Docs.

Ok, let's do that:

```rust
pub enum List {
    Empty,
    Elem(i32, Box<List>),
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
```

Hey it built!

...but this is actually a really stupid definition of a List, for a few reasons.

Consider a list with two elements:

```text
[] = Stack
() = Heap

[Elem A, ptr] -> (Elem B, ptr) -> (Empty *junk*)
```

There are two key issues:

* We're allocating a node that just says "I'm not actually a Node"
* One of our nodes isn't allocated at all.

On the surface, these two seem to cancel each-other out. We allocate an
extra node, but one of our nodes doesn't need to be allocated at all.
However, consider the following potential layout for our list:

```text
[ptr] -> (Elem A, ptr) -> (Elem B, *null*)
```

In this layout we now unconditionally heap allocate our nodes. The
key difference is the absence of the *junk* from our first layout. What is
this junk? To understand that, we'll need to look at how an enum is laid out
in memory.

In general, if we have an enum like:

```rust
enum Foo {
    D1(T1),
    D2(T2),
    ...
    Dn(Tn),
}
```

A Foo will need to store some integer to indicate which *variant* of the enum it
represents (`D1`, `D2`, .. `Dn`). This is the *tag* of the enum. It will also
need enough space to store the *largest* of `T1`, `T2`, .. `Tn` (plus some extra
space to satisfy alignment requirements).

The big takeaway here is that even though `Empty` is a single bit of
information, it necessarily consumes enough space for a pointer and an element,
because it has to be ready to become an `Elem` at any time. Therefore the first
layout heap allocates an extra element that's just full of junk, consuming a
bit more space than the second layout.

One of our nodes not being allocated at all is also, perhaps surprisingly,
*worse* than always allocating it. This is because it gives us a *non-uniform*
node layout. This doesn't have much of an appreciable effect on pushing and
popping nodes, but it does have an effect on splitting and merging lists.

Consider splitting a list in both layouts:

```text
layout 1:

[Elem A, ptr] -> (Elem B, ptr) -> (Elem C, ptr) -> (Empty *junk*)

split off C:

[Elem A, ptr] -> (Elem B, ptr) -> (Empty *junk*)
[Elem C, ptr] -> (Empty *junk*)
```

```text
layout 2:

[ptr] -> (Elem A, ptr) -> (Elem B, ptr) -> (Elem C, *null*)

split off C:

[ptr] -> (Elem A, ptr) -> (Elem B, *null*)
[ptr] -> (Elem C, *null*)
```

Layout 2's split involves just copying B's pointer to the stack and nulling
the old value out. Layout 1 ultimately does the same thing, but also has to
copy C from the heap to the stack. Merging is the same process in reverse.

One of the few nice things about a linked list is that you can construct the
element in the node itself, and then freely shuffle it around lists without
ever moving it. You just fiddle with pointers and stuff gets "moved". Layout 1
trashes this property.

Alright, I'm reasonably convinced Layout 1 is bad. How do we rewrite our List?
Well, we could do something like:

```rust
pub enum List {
    Empty,
    ElemThenEmpty(i32),
    ElemThenNotEmpty(i32, Box<List>),
}
```

Hopefully this seems like an even worse idea to you. For one, this really
complicates our logic. In particular, there is now a completely invalid state:
`ElemThenNotEmpty(0, Box(Empty))`. It also *still* suffers from non-uniformly
allocating our elements.

However it does have *one* interesting property: it totally avoids allocating
the Empty case, reducing the total number of heap allocations by 1. Unfortunately,
in doing so it manages to waste *even more space*! This is because the previous
layout took advantage of the *null pointer optimization*.

We previously saw that every enum has to store a *tag* to specify which variant
of the enum its bits represent. However, if we have a special kind of enum:

```rust
enum Foo {
    A,
    B(ContainsANonNullPtr),
}
```

the null pointer optimization kicks in, which *eliminates the space needed for
the tag*. If the variant is A, the whole enum is set to all `0`'s. Otherwise,
the variant is B. This works because B can never be all `0`'s, since it contains
a non-zero pointer. Slick!

Can you think of other enums and types that could do this kind of optimization?
There's actually a lot! This is why Rust leaves enum layout totally unspecified.
Sadly the null pointer optimization is the only one implemented today -- though
it's pretty important! It means `&`, `&mut`, `Box`, `Rc`, `Arc`, `Vec`, and
several other important types in Rust have no overhead when put in an `Option`!
(We'll get to most of these in due time).

So how do we avoid the extra junk, uniformly allocate, *and* get that sweet
null-pointer optimization? We need to better separate out the idea of having an
element from allocating another list. To do this, we have to think a little more
C-like: structs!

While enums let us declare a type that can contain *one* of several values,
structs let us declare a type that contains *many* values at once. Let's break
our List into two types: A List, and a Node.

As before, a List is either Empty or has an element followed by another List.
However by representing the "has an element followed by another List" case by an
entirely separate type, we can hoist the Box to be in a more optimal position:

```rust
struct Node {
    elem: i32,
    next: List,
}

pub enum List {
    Empty,
    More(Box<Node>),
}
```

Let's check our priorities:

* Tail of a list never allocates extra junk: check!
* `enum` is in delicious null-pointer-optimized form: check!
* All elements are uniformly allocated: check!

Alright! We actually just constructed exactly the layout that we used to
demonstrate that our first layout (as suggested by the official Rust
documentation) was problematic.

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:8:11: 8:18 error: private type in exported type signature
src/first.rs:8    More(Box<Node>),
                           ^~~~~~~
error: aborting due to previous error
Could not compile `lists`.
```

:(

Rust is mad at us again. We marked the `List` as public (because we want people
to be able to use it), but not the `Node`. The problem is that the internals of
an `enum` are totally public, and we're not allowed to publicly talk about
private types. We could make all of `Node` totally public, but generally in Rust
we favour keeping implementation details private. Let's make `List` a struct, so
that we can hide the implementation details:


```rust
pub struct List {
    head: Link,
}

enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    next: Link,
}
```

Because `List` is a struct with a single field, its size is the same as that
field. Yay zero-cost abstractions!

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:2:2: 2:15 warning: struct field is never used: `head`, #[warn(dead_code)] on by default
src/first.rs:2    head: Link,
                  ^~~~~~~~~~~~~
src/first.rs:6:2: 6:7 warning: variant is never used: `Empty`, #[warn(dead_code)] on by default
src/first.rs:6    Empty,
                  ^~~~~
src/first.rs:7:2: 7:20 warning: variant is never used: `More`, #[warn(dead_code)] on by default
src/first.rs:7    More(Box<Node>),
                  ^~~~~~~~~~~~~~~~~~
src/first.rs:11:2: 11:9 warning: struct field is never used: `elem`, #[warn(dead_code)] on by default
src/first.rs:11   elem: i32,
                  ^~~~~~~
src/first.rs:12:2: 12:15 warning: struct field is never used: `next`, #[warn(dead_code)] on by default
src/first.rs:12   next: Link,
                  ^~~~~~~~~~~~~
```

Alright, that compiled! Rust is pretty mad, because as far as it can tell,
everything we've written is totally useless: we never use `head`, and no one who
uses our library can either since it's private. Transitively, that means Link
and Node are useless too. So let's solve that! Let's implement some code for our
List!
