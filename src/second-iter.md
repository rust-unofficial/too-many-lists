# Iter

Alright, let's try to implement Iter. This time we won't be able to rely on
List giving us all the features we want. We'll need to roll our own. The
basic logic we want is to hold a pointer to the current node we want to yield
next. Because that node may not exist (the list is empty or we're otherwise
done iterating), we want that reference to be an Option. When we yield an
element, we want to proceed to the current node's `next` node.

Alright, let's try that:

```rust
pub struct Iter<T> {
    next: Option<&Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter { next: self.head.map(|node| &*node) }
    }
}

impl<T> Iterator for Iter<T> {
    type Item = &T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &*node);
            &node.elem
        })
    }
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/second.rs:62:18: 62:26 error: missing lifetime specifier [E0106]
src/second.rs:62     next: Option<&Node<T>>,
                                  ^~~~~~~~
src/second.rs:62:18: 62:26 help: run `rustc --explain E0106` to see a detailed explanation
src/second.rs:72:17: 72:19 error: missing lifetime specifier [E0106]
src/second.rs:72     type Item = &T;
                                 ^~
src/second.rs:72:17: 72:19 help: run `rustc --explain E0106` to see a detailed explanation
error: aborting due to 2 previous errors
```

Oh god. Lifetimes. I've heard of these things. I hear they're a nightmare. Let's
try that `--explain`:

```
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

That uh... that didn't really clarify much. But it looks like we should add
those `'a` things to our struct? Let's try that.

```
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/second.rs:71:22: 71:29 error: wrong number of lifetime parameters: expected 1, found 0 [E0107]
src/second.rs:71 impl<T> Iterator for Iter<T> {
                                      ^~~~~~~
src/second.rs:71:22: 71:29 help: run `rustc --explain E0107` to see a detailed explanation
src/second.rs:72:17: 72:19 error: missing lifetime specifier [E0106]
src/second.rs:72     type Item = &T;
                                 ^~
src/second.rs:72:17: 72:19 help: run `rustc --explain E0106` to see a detailed explanation
error: aborting due to 2 previous errors

```

Alright I'm starting to see a pattern here... let's just go whole-hog here:

```
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> List<T> {
    pub fn iter(&'a self) -> Iter<'a, T> {
        Iter { next: self.head.map(|node| &'a *node) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&'a mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &'a *node);
            &'a node.elem
        })
    }
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/second.rs:67:34: 67:35 error: expected `:`, found `*`
src/second.rs:67         self.head.map(|node| &'a **node)
                                                  ^
Could not compile `lists`.
```



Oh god. We broke Rust.

Maybe we should actually figure out what the heck this `'a` lifetime stuff
even means.

Lifetimes can scare off a lot of people because
they're a change to something we've known and loved since the dawn of
programming. We've actually managed to dodge lifetimes so far, even though
they've been tangled throughout our programs this whole time.

Lifetimes are unecessary in garbage collected languages because the garbage
collector ensures that everything magically lives as long as it needs to. Most
data in Rust is *manually* managed, so that data needs another solution. C and
C++ give us a clear example what happens if you just let people take pointers
to random data on the stack: pervasive unmanageable unsafety. This can be
roughly seperated into two classes of error:

* Holding a pointer to something that went out of scope
* Holding a pointer to something that got mutated away

Lifetimes solve both of these problems, and 99% of the time, they do this in
a totally transparent way.

So what's a lifetime?

Quite simply, a lifetime is the name of a scope somewhere in a program.
That's it. When a reference is tagged with a lifetime, we're saying that it
has to be valid for that *entire* scope. Different things place requirements on
how long a reference must and can be valid for. The entire lifetime system is in
turn just a constraint-solving system that tries to minimize the scope of every
reference. If it sucessfully finds a set of lifetimes that satisfies all the
constraints, your program compiles! Otherwise you get an error back saying that
something didn't live long enough.

Within a function body you generally can't talk about lifetimes, and wouldn't
want to *anyway*. The compiler has full information and can infer all the
contraints and find the minimum lifetimes. However at the type and API-level,
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

```rust,ignore
// Only one reference in input, so the output must be derived from that input
fn foo(&A) -> &B; // sugar for:
fn foo<'a>(&'a A) -> &'a B;

// Many inputs, assume they're all independent
fn foo(&A, &B, &C); // sugar for:
fn foo<'a, 'b, 'c>(&'a, &'b, &'c);

// Methods, assume all output lifetimes are derived from `self`
fn foo(&self, &B, &C) -> &D; // sugar for:
fn foo<'a, 'b, 'c>(&'a self, &'b B, &'c C) -> &'a D;
```

So what does `fn foo<'a>(&'a A) -> &'a B` *mean*? In practical terms, all it
means is that the input must live at least as long as the output. So if you keep
the output around for a long time, this will *drag* the scope that the `&A` must
be valid for to be larger and larger.

With this system set up, Rust can ensure nothing is used after free, and nothing
is mutated while outstanding references exist. It just makes sure the
constraints all work out!

Alright. So. Iter.

Let's roll back to the no lifetimes state:

```rust
pub struct Iter<T> {
    next: Option<&Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter { next: self.head.map(|node| &*node) }
    }
}

impl<T> Iterator for Iter<T> {
    type Item = &T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &*node);
            &node.elem
        })
    }
}
```

We need to add lifetimes only in function and type signatures:

```
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
        Iter { next: self.head.map(|node| &*node) }
    }
}

// *Do* have a lifetime here, because Iter does have an associated lifetime
impl<'a, T> Iterator for Iter<'a, T> {
    // Need it here too, this is a type declaration
    type Item = &'a T;

    // None of this needs to change, handled by the above.
    // Self continues to be incredibly hype and amazing
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.map(|node| &**node);
            &node.elem
        })
    }
}
```

Alright, I think we got it this time y'all.

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/second.rs:62:1: 64:2 error: the parameter type `T` may not live long enough [E0309]
src/second.rs:62 pub struct Iter<'a, T> {
src/second.rs:63     next: Option<&'a Node<T>>,
src/second.rs:64 }
src/second.rs:62:1: 64:2 help: run `rustc --explain E0309` to see a detailed explanation
src/second.rs:62:1: 64:2 help: consider adding an explicit lifetime bound `T: 'a`...
src/second.rs:62:1: 64:2 note: ...so that the reference type `&'a second::Node<T>` does not outlive the data it points at
src/second.rs:62 pub struct Iter<'a, T> {
src/second.rs:63     next: Option<&'a Node<T>>,
src/second.rs:64 }
error: aborting due to previous error
```

(╯°□°)╯︵ ┻━┻


```text
rustc --explain E0309
Types in type definitions have lifetimes associated with them that represent
how long the data stored within them is guaranteed to be live. This lifetime
must be as long as the data needs to be alive, and missing the constraint that
denotes this will cause this error.

// This won't compile because T is not constrained, meaning the data
// stored in it is not guaranteed to last as long as the reference
struct Foo<'a, T> {
    foo: &'a T
}

// This will compile, because it has the constraint on the type parameter
struct Foo<'a, T: 'a> {
    foo: &'a T
}
```

This is dumb. I think it's dumb. You have to do it.


```rust
pub struct Iter<'a, T: 'a> {
    next: Option<&'a Node<T>>,
}
```

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/second.rs:67:22: 67:31 error: cannot move out of type `second::List<T>`, which defines the `Drop` trait
src/second.rs:67         Iter { next: self.head.map(|node| &*node) }
                                      ^~~~~~~~~
src/second.rs:67:44: 67:49 error: `*node` does not live long enough
src/second.rs:67         Iter { next: self.head.map(|node| &*node) }
                                                            ^~~~~
note: in expansion of closure expansion
src/second.rs:67:36: 67:49 note: expansion site
src/second.rs:66:42: 68:6 note: reference must be valid for the lifetime 'a as defined on the block at 66:41...
src/second.rs:66     pub fn iter<'a>(&'a self) -> Iter<'a, T> {
src/second.rs:67         Iter { next: self.head.map(|node| &*node) }
src/second.rs:68     }
src/second.rs:67:43: 67:49 note: ...but borrowed value is only valid for the scope of parameters for function at 67:42
src/second.rs:67         Iter { next: self.head.map(|node| &*node) }
                                                           ^~~~~~
src/second.rs:76:25: 76:29 error: cannot move out of borrowed content
src/second.rs:76             self.next = node.next.map(|node| &*node);
                                         ^~~~
note: in expansion of closure expansion
src/second.rs:75:23: 78:10 note: expansion site
src/second.rs:76:47: 76:52 error: `*node` does not live long enough
src/second.rs:76             self.next = node.next.map(|node| &*node);
                                                               ^~~~~
note: in expansion of closure expansion
src/second.rs:76:39: 76:52 note: expansion site
note: in expansion of closure expansion
src/second.rs:75:23: 78:10 note: expansion site
src/second.rs:74:46: 79:6 note: reference must be valid for the lifetime 'a as defined on the block at 74:45...
src/second.rs:74     fn next(&mut self) -> Option<Self::Item> {
src/second.rs:75         self.next.map(|node| {
src/second.rs:76             self.next = node.next.map(|node| &*node);
src/second.rs:77             &node.elem
src/second.rs:78         })
src/second.rs:79     }
src/second.rs:76:46: 76:52 note: ...but borrowed value is only valid for the scope of parameters for function at 76:45
src/second.rs:76             self.next = node.next.map(|node| &*node);
                                                              ^~~~~~
error: aborting due to 4 previous errors
```

(ﾉಥ益ಥ）ﾉ﻿ ┻━┻

We forgot `as_ref`:

```rust
pub struct Iter<'a, T:'a> {
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
lists::cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

┬─┬﻿ ノ( ゜-゜ノ)

Let's write a test to be sure we didn't no-op it or anything:

```rust
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
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
     Running target/debug/lists-5c71138492ad4b4a

running 5 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::into_iter ... ok
test second::test::iter ... ok
test second::test::peek ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

Heck yeah.

Finally, it should be noted that we actually apply lifetime elision here:

```rust
impl<T> List<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { next: self.head.as_ref().map(|node| &**node) }
    }
}
```

is equivalent to:

```rust
impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter { next: self.head.as_ref().map(|node| &**node) }
    }
}
```

Yay less lifetimes!
