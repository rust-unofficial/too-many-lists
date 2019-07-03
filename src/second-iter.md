# Iter

Alright, let's try to implement Iter. This time we won't be able to rely on
List giving us all the features we want. We'll need to roll our own. The
basic logic we want is to hold a pointer to the current node we want to yield
next. Because that node may not exist (the list is empty or we're otherwise
done iterating), we want that reference to be an Option. When we yield an
element, we want to proceed to the current node's `next` node.

Alright, let's try that:

```rust ,ignore
pub struct Iter<T> {
    next: Option<&Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter { next: self.head.map(|node| &node) }
    }
}

impl<T> Iterator for Iter<T> {
    type Item = &T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &node);
            &node.elem
        })
    }
}
```

```text
> cargo build

error[E0106]: missing lifetime specifier
  --> src/second.rs:72:18
   |
72 |     next: Option<&Node<T>>,
   |                  ^ expected lifetime parameter

error[E0106]: missing lifetime specifier
  --> src/second.rs:82:17
   |
82 |     type Item = &T;
   |                 ^ expected lifetime parameter
```

Oh god. Lifetimes. I've heard of these things. I hear they're a nightmare.

Let's try something new: see that `error[E0106]` thing? That's a compiler error
code. We can ask rustc to explain those with, well, `--explain`:

```text
> rustc --explain E0106
This error indicates that a lifetime is missing from a type. If it is an error
inside a function signature, the problem may be with failing to adhere to the
lifetime elision rules (see below).

Here are some simple examples of where you'll run into this error:

struct Foo { x: &bool }        // error
struct Foo<'a> { x: &'a bool } // correct

enum Bar { A(u8), B(&bool), }        // error
enum Bar<'a> { A(u8), B(&'a bool), } // correct

type MyStr = &str;        // error
type MyStr<'a> = &'a str; //correct
...

```

That uh... that didn't really clarify much (these docs assume we understand
Rust better than we currently do). But it looks like we should add
those `'a` things to our struct? Let's try that.

```
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}
```

```text
> cargo build

error[E0106]: missing lifetime specifier
  --> src/second.rs:83:22
   |
83 | impl<T> Iterator for Iter<T> {
   |                      ^^^^^^^ expected lifetime parameter

error[E0106]: missing lifetime specifier
  --> src/second.rs:84:17
   |
84 |     type Item = &T;
   |                 ^ expected lifetime parameter

error: aborting due to 2 previous errors
```

Alright I'm starting to see a pattern here... let's just add these little guys
to everything we can:

```rust ,ignore
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> List<T> {
    pub fn iter(&'a self) -> Iter<'a, T> {
        Iter { next: self.head.map(|node| &'a node) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&'a mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &'a node);
            &'a node.elem
        })
    }
}
```

```text
> cargo build

error: expected `:`, found `node`
  --> src/second.rs:77:47
   |
77 |         Iter { next: self.head.map(|node| &'a node) }
   |         ---- while parsing this struct        ^^^^ expected `:`

error: expected `:`, found `node`
  --> src/second.rs:85:50
   |
85 |             self.next = node.next.map(|node| &'a node);
   |                                                  ^^^^ expected `:`

error[E0063]: missing field `next` in initializer of `second::Iter<'_, _>`
  --> src/second.rs:77:9
   |
77 |         Iter { next: self.head.map(|node| &'a node) }
   |         ^^^^ missing `next`
```

Oh god. We broke Rust.

Maybe we should actually figure out what the heck this `'a` lifetime stuff
even means.

Lifetimes can scare off a lot of people because
they're a change to something we've known and loved since the dawn of
programming. We've actually managed to dodge lifetimes so far, even though
they've been tangled throughout our programs this whole time.

Lifetimes are unnecessary in garbage collected languages because the garbage
collector ensures that everything magically lives as long as it needs to. Most
data in Rust is *manually* managed, so that data needs another solution. C and
C++ give us a clear example what happens if you just let people take pointers
to random data on the stack: pervasive unmanageable unsafety. This can be
roughly separated into two classes of error:

* Holding a pointer to something that went out of scope
* Holding a pointer to something that got mutated away

Lifetimes solve both of these problems, and 99% of the time, they do this in
a totally transparent way.

So what's a lifetime?

Quite simply, a lifetime is the name of a region (\~block/scope) of code somewhere in a program.
That's it. When a reference is tagged with a lifetime, we're saying that it
has to be valid for that *entire* region. Different things place requirements on
how long a reference must and can be valid for. The entire lifetime system is in
turn just a constraint-solving system that tries to minimize the region of every
reference. If it successfully finds a set of lifetimes that satisfies all the
constraints, your program compiles! Otherwise you get an error back saying that
something didn't live long enough.

Within a function body you generally can't talk about lifetimes, and wouldn't
want to *anyway*. The compiler has full information and can infer all the
contraints to find the minimum lifetimes. However at the type and API-level,
the compiler *doesn't* have all the information. It requires you to tell it
about the relationship between different lifetimes so it can figure out what
you're doing.

In principle, those lifetimes *could* also be left out, but
then checking all the borrows would be a huge whole-program analysis that would
produce mind-bogglingly non-local errors. Rust's system means all borrow
checking can be done in each function body independently, and all your errors
should be fairly local (or your types have incorrect signatures).

But we've written references in function signatures before, and it was fine!
That's because there are certain cases that are so common that Rust will
automatically pick the lifetimes for you. This is *lifetime elision*.

In particular:

```rust ,ignore
// Only one reference in input, so the output must be derived from that input
fn foo(&A) -> &B; // sugar for:
fn foo<'a>(&'a A) -> &'a B;

// Many inputs, assume they're all independent
fn foo(&A, &B, &C); // sugar for:
fn foo<'a, 'b, 'c>(&'a A, &'b B, &'c C);

// Methods, assume all output lifetimes are derived from `self`
fn foo(&self, &B, &C) -> &D; // sugar for:
fn foo<'a, 'b, 'c>(&'a self, &'b B, &'c C) -> &'a D;
```

So what does `fn foo<'a>(&'a A) -> &'a B` *mean*? In practical terms, all it
means is that the input must live at least as long as the output. So if you keep
the output around for a long time, this will expand the region that the input must
be valid for. Once you stop using the output, the compiler will know it's ok for
the input to become invalid too.

With this system set up, Rust can ensure nothing is used after free, and nothing
is mutated while outstanding references exist. It just makes sure the
constraints all work out!

Alright. So. Iter.

Let's roll back to the no lifetimes state:

```rust ,ignore
pub struct Iter<T> {
    next: Option<&Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter { next: self.head.map(|node| &node) }
    }
}

impl<T> Iterator for Iter<T> {
    type Item = &T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &node);
            &node.elem
        })
    }
}
```

We need to add lifetimes only in function and type signatures:

```rust ,ignore
// Iter is generic over *some* lifetime, it doesn't care
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

// No lifetime here, List doesn't have any associated lifetimes
impl<T> List<T> {
    // We declare a fresh lifetime here for the *exact* borrow that
    // creates the iter. Now &self needs to be valid as long as the
    // Iter is around.
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { next: self.head.map(|node| &node) }
    }
}

// We *do* have a lifetime here, because Iter has one that we need to define
impl<'a, T> Iterator for Iter<'a, T> {
    // Need it here too, this is a type declaration
    type Item = &'a T;

    // None of this needs to change, handled by the above.
    // Self continues to be incredibly hype and amazing
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &node);
            &node.elem
        })
    }
}
```

Alright, I think we got it this time y'all.

```text
cargo build

error[E0308]: mismatched types
  --> src/second.rs:77:22
   |
77 |         Iter { next: self.head.map(|node| &node) }
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `second::Node`, found struct `std::boxed::Box`
   |
   = note: expected type `std::option::Option<&second::Node<T>>`
              found type `std::option::Option<&std::boxed::Box<second::Node<T>>>`

error[E0308]: mismatched types
  --> src/second.rs:85:25
   |
85 |             self.next = node.next.map(|node| &node);
   |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `second::Node`, found struct `std::boxed::Box`
   |
   = note: expected type `std::option::Option<&'a second::Node<T>>`
              found type `std::option::Option<&std::boxed::Box<second::Node<T>>>`
```

(╯°□°)╯︵ ┻━┻

OK. SO. We fixed our lifetime errors but now we're getting some new type errors.

We want to be storing `&Node`'s, but we're getting `&Box<Node>`s. Ok, that's easy
enough, we just need to dereference the Box before we take our reference:

```rust ,ignore
impl<T> List<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { next: self.head.map(|node| &*node) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &*node);
            &node.elem
        })
    }
}
```

```text
cargo build
   Compiling lists v0.1.0 (/Users/ABeingessner/dev/temp/lists)
error[E0515]: cannot return reference to local data `*node`
  --> src/second.rs:77:43
   |
77 |         Iter { next: self.head.map(|node| &*node) }
   |                                           ^^^^^^ returns a reference to data owned by the current function

error[E0507]: cannot move out of borrowed content
  --> src/second.rs:77:22
   |
77 |         Iter { next: self.head.map(|node| &*node) }
   |                      ^^^^^^^^^ cannot move out of borrowed content

error[E0515]: cannot return reference to local data `*node`
  --> src/second.rs:85:46
   |
85 |             self.next = node.next.map(|node| &*node);
   |                                              ^^^^^^ returns a reference to data owned by the current function

error[E0507]: cannot move out of borrowed content
  --> src/second.rs:85:25
   |
85 |             self.next = node.next.map(|node| &*node);
   |                         ^^^^^^^^^ cannot move out of borrowed content
```

(ﾉಥ益ಥ）ﾉ﻿ ┻━┻

We forgot `as_ref`, so we're moving the box into `map`, which means it would
be dropped, which means our references would be dangling:

```rust ,ignore
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { next: self.head.as_ref().map(|node| &*node) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map(|node| &*node);
            &node.elem
        })
    }
}
```

```text
cargo build
   Compiling lists v0.1.0 (/Users/ABeingessner/dev/temp/lists)
error[E0308]: mismatched types
  --> src/second.rs:77:22
   |
77 |         Iter { next: self.head.as_ref().map(|node| &*node) }
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `second::Node`, found struct `std::boxed::Box`
   |
   = note: expected type `std::option::Option<&second::Node<T>>`
              found type `std::option::Option<&std::boxed::Box<second::Node<T>>>`

error[E0308]: mismatched types
  --> src/second.rs:85:25
   |
85 |             self.next = node.next.as_ref().map(|node| &*node);
   |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `second::Node`, found struct `std::boxed::Box`
   |
   = note: expected type `std::option::Option<&'a second::Node<T>>`
              found type `std::option::Option<&std::boxed::Box<second::Node<T>>>`

```

😭

`as_ref` added another layer of indirection we need to remove:


```rust ,ignore
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { next: self.head.as_ref().map(|node| &**node) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map(|node| &**node);
            &node.elem
        })
    }
}
```

```text
cargo build

```

🎉 🎉 🎉

You may be thinking "wow that `&**` thing is really janky", and you're not wrong.
Normally Rust is very good at doing this kind of conversion implicitly, through
a process called *deref coercion*, where basically it can insert \*'s
throughout your code to make it type-check. It can do this because we have the
borrow checker to ensure we never mess up pointers!

But in this case the closure in conjunction with the fact that we
have an `Option<&T>` instead of `&T` is a bit too complicated for it to work
out, so we need to do this for it. Thankfully this is pretty rare, in my experience.

Just for completeness' sake, we *could* give it a *different* hint with the *turbofish*:

```rust ,ignore
    self.next = node.next.as_ref().map::<&Node<T>, _>(|node| &node);
```

See, map is a generic function:

```rust ,ignore
pub fn map<U, F>(self, f: F) -> Option<U>
```

The turbofish, `::<>`, lets us tell the compiler what we think the types of those
generics should be. In this case `::<&Node<T>, _>` says "it should return a
`&Node<T>`, and I don't know/care about that other type".

This in turn lets the compiler know that `&node` should have deref coercion
applied to it, so we don't need to manually apply all those \*'s!

But in this case I don't think it's really an improvement, this was just a
thinly veiled excuse to show off deref coercion and the sometimes-useful turbofish. 😅

Let's write a test to be sure we didn't no-op it or anything:

```rust ,ignore
#[test]
fn iter() {
    let mut list = List::new();
    list.push(1); list.push(2); list.push(3);

    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&1));
}
```

```text
> cargo test

     Running target/debug/lists-5c71138492ad4b4a

running 5 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::into_iter ... ok
test second::test::iter ... ok
test second::test::peek ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured

```

Heck yeah.

Finally, it should be noted that we *can* actually apply lifetime elision here:

```rust ,ignore
impl<T> List<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { next: self.head.as_ref().map(|node| &**node) }
    }
}
```

is equivalent to:

```rust ,ignore
impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter { next: self.head.as_ref().map(|node| &**node) }
    }
}
```

Yay fewer lifetimes!

Or, if you're not comfortable "hiding" that a struct contains a lifetime,
you can use the Rust 2018 "explicitly elided lifetime" syntax,  `'_`:

```rust ,ignore
impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { next: self.head.as_ref().map(|node| &**node) }
    }
}
```
