# Basics

Alright, back to basics. How do we construct our list?

Before we just did:

```rust ,ignore
impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }
}
```

But we're not using Option for the `tail` anymore:

```text
> cargo build

error[E0308]: mismatched types
  --> src/fifth.rs:15:34
   |
15 |         List { head: None, tail: None }
   |                                  ^^^^ expected *-ptr, found enum `std::option::Option`
   |
   = note: expected type `*mut fifth::Node<T>`
              found type `std::option::Option<_>`
```

We *could* use an Option, but unlike Box, `*mut` *is* nullable. This means it
can't benefit from the null pointer optimization. Instead, we'll be using `null`
to represent None.

So how do we get a null pointer? There's a few ways, but I prefer to use
`std::ptr::null_mut()`. If you want, you can also use `0 as *mut _`, but that
just seems so *messy*.

```rust ,ignore
use std::ptr;

// defns...

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: ptr::null_mut() }
    }
}
```

```text
cargo build

warning: field is never used: `head`
 --> src/fifth.rs:4:5
  |
4 |     head: Link<T>,
  |     ^^^^^^^^^^^^^
  |
  = note: #[warn(dead_code)] on by default

warning: field is never used: `tail`
 --> src/fifth.rs:5:5
  |
5 |     tail: *mut Node<T>,
  |     ^^^^^^^^^^^^^^^^^^

warning: field is never used: `elem`
  --> src/fifth.rs:11:5
   |
11 |     elem: T,
   |     ^^^^^^^

warning: field is never used: `head`
  --> src/fifth.rs:12:5
   |
12 |     head: Link<T>,
   |     ^^^^^^^^^^^^^
```

*shush* compiler, we will use them soon.

Alright, let's move on to writing `push` again. This time, instead of grabbing
an `Option<&mut Node<T>>` after we insert, we're just going to grab a
`*mut Node<T>` to the insides of the Box right away. We know we can soundly do
this because the contents of a Box has a stable address, even if we move the
Box around. Of course, this isn't *safe*, because if we just drop the Box we'll
have a pointer to freed memory.

How do we make a raw pointer from a normal pointer? Coercions! If a variable
is declared to be a raw pointer, a normal reference will coerce into it:

```rust ,ignore
let raw_tail: *mut _ = &mut *new_tail;
```

We have all the info we need. We can translate our code into, approximately,
the previous reference version:

```rust ,ignore
pub fn push(&mut self, elem: T) {
    let mut new_tail = Box::new(Node {
        elem: elem,
        next: None,
    });

    let raw_tail: *mut _ = &mut *new_tail;

    // .is_null checks for null, equivalent to checking for None
    if !self.tail.is_null() {
        // If the old tail existed, update it to point to the new tail
        self.tail.next = Some(new_tail);
    } else {
        // Otherwise, update the head to point to it
        self.head = Some(new_tail);
    }

    self.tail = raw_tail;
}
```

```text
> cargo build

error[E0609]: no field `next` on type `*mut fifth::Node<T>`
  --> src/fifth.rs:31:23
   |
31 |             self.tail.next = Some(new_tail);
   |             ----------^^^^
   |             |
   |             help: `self.tail` is a raw pointer; try dereferencing it: `(*self.tail).next`
```

Huh? We have a pointer to a Node, why can't we get the `next` field?

Rust is kinda a jerk when you use raw pointers. To access the contents of a
raw pointer, it insists that we manually deref them, because it's such an unsafe
operation. So let's do that:

```rust ,ignore
*self.tail.next = Some(new_tail);
```

```text
> cargo build

error[E0609]: no field `next` on type `*mut fifth::Node<T>`
  --> src/fifth.rs:31:23
   |
31 |             *self.tail.next = Some(new_tail);
   |             -----------^^^^
   |             |
   |             help: `self.tail` is a raw pointer; try dereferencing it: `(*self.tail).next`
```

Uuuugh operator precedence.

```rust ,ignore
(*self.tail).next = Some(new_tail);
```

```text
> cargo build

error[E0133]: dereference of raw pointer is unsafe and requires unsafe function or block
  --> src/fifth.rs:31:13
   |
31 |             (*self.tail).next = Some(new_tail);
   |             ^^^^^^^^^^^^^^^^^ dereference of raw pointer
   |
   = note: raw pointers may be NULL, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
```

THIS SHOULDN'T BE THIS HARD.

Remember how I said Unsafe Rust is like an FFI language for Safe Rust? Well, the
compiler wants us to explicitly delimit where we're doing this FFI-ing. We have
two options. First, we can mark our *entire* function as unsafe, in which case
it becomes an Unsafe Rust function and can only be called in an `unsafe`
context. This isn't great, because we want our list to be safe to use. Second,
we can write an `unsafe` block inside our function, to delimit the FFI boundary.
This declares the overall function to be safe. Let's do that one:


```rust ,ignore
pub fn push(&mut self, elem: T) {
    let mut new_tail = Box::new(Node {
        elem: elem,
        next: None,
    });

    let raw_tail: *mut _ = &mut *new_tail;

    // Put the box in the right place, and then grab a reference to its Node
    if !self.tail.is_null() {
        // If the old tail existed, update it to point to the new tail
        unsafe {
            (*self.tail).next = Some(new_tail);
        }
    } else {
        // Otherwise, update the head to point to it
        self.head = Some(new_tail);
    }

    self.tail = raw_tail;
}
```

```text
> cargo build
warning: field is never used: `elem`
  --> src/fifth.rs:11:5
   |
11 |     elem: T,
   |     ^^^^^^^
   |
   = note: #[warn(dead_code)] on by default
```

Yay!

It's kind've interesting that that's the *only* place we've had to write an
unsafe block so far. We do raw pointer stuff all over the place, what's up with
that?

It turns out that Rust is a massive rules-lawyer pedant when it comes to
`unsafe`. We quite reasonably want to maximize the set of Safe Rust programs,
because those are programs we can be much more confident in. To accomplish this,
Rust carefully carves out a minimal surface area for unsafety. Note that all
the other places we've worked with raw pointers has been *assigning* them, or
just observing whether they're null or not.

If you never actually dereference a raw pointer *those are totally safe things
to do*. You're just reading and writing an integer! The only time you can
actually get into trouble with a raw pointer is if you actually dereference it.
So Rust says *only* that operation is unsafe, and everything else is totally
safe.

Super. Pedantic. But technically correct.

However this raises an interesting problem: although we're supposed to delimit
the scope of the unsafety with the `unsafe` block, it actually depends on
state that was established outside of the block. Outside of the function, even!

This is what I call unsafe *taint*. As soon as you use `unsafe` in a module,
that whole module is tainted with unsafety. Everything has to be correctly
written in order to make sure that invariants are upheld for the unsafe code.

This taint is manageable because of *privacy*. Outside of our module, all of our
struct fields are totally private, so no one else can mess with our state in
arbitrary ways. As long as no combination of the APIs we expose causes bad stuff
to happen, as far as an outside observer is concerned, all of our code is safe!
And really, this is no different from the FFI case. No one needs to care
if some python math library shells out to C as long as it exposes a safe
interface.

Anyway, let's move on to `pop`, which is pretty much verbatim the reference
version:

```rust ,ignore
pub fn pop(&mut self) -> Option<T> {
    self.head.take().map(|head| {
        let head = *head;
        self.head = head.next;

        if self.head.is_none() {
            self.tail = ptr::null_mut();
        }

        head.elem
    })
}
```

Again we see another case where safety is stateful. If we fail to null out the
tail pointer in *this* function, we'll see no problems at all. However
subsequent calls to `push` will start writing to the dangling tail!

Let's test it out:

```rust ,ignore
#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
```

This is just the stack test, but with the expected `pop` results flipped around.
I also added some extra steps at the end to make sure that tail-pointer
corruption case in `pop` doesn't occur.

```text
cargo test

     Running target/debug/lists-5c71138492ad4b4a

running 12 tests
test fifth::test::basics ... ok
test first::test::basics ... ok
test fourth::test::basics ... ok
test fourth::test::peek ... ok
test second::test::basics ... ok
test fourth::test::into_iter ... ok
test second::test::into_iter ... ok
test second::test::iter ... ok
test second::test::iter_mut ... ok
test second::test::peek ... ok
test third::test::basics ... ok
test third::test::iter ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured
```

Gold Star!


