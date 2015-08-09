% Peeking

Alright, we made it through `push` and `pop`. I'm not gonna lie, it got a
bit emotional there. Compile-time correctness is a hell of a drug.

Let's cool off by doing something simple: let's just implement `peek_front`.
That was always really easy before. Gotta still be easy, right?

Right?

In fact, I think I can just copy-paste it!

```rust
pub fn peek_front(&self) -> Option<&T> {
    self.head.as_ref().map(|node| {
        &node.elem
    })
}
```

Wait. Not this time.

```rust
pub fn peek_front(&self) -> Option<&T> {
    self.head.as_ref().map(|node| {
        &node.borrow().elem
    })
}
```

HAH.

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:64:14: 64:27 error: borrowed value does not live long enough
src/fourth.rs:64             &node.borrow().elem
                              ^~~~~~~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:63:32: 65:10 note: expansion site
src/fourth.rs:62:44: 66:6 note: reference must be valid for the anonymous lifetime #1 defined on the block at 62:43...
src/fourth.rs:62     pub fn peek_front(&self) -> Option<&T> {
src/fourth.rs:63         self.head.as_ref().map(|node| {
src/fourth.rs:64             &node.borrow().elem
src/fourth.rs:65         })
src/fourth.rs:66     }
src/fourth.rs:63:39: 65:10 note: ...but borrowed value is only valid for the block at 63:38
src/fourth.rs:63         self.head.as_ref().map(|node| {
src/fourth.rs:64             &node.borrow().elem
src/fourth.rs:65         })
error: aborting due to previous error
Could not compile `lists`.
```

Ok I'm just burning my computer.

This is exactly the same logic as our singly linked stack. Why are things
different. WHY.

The answer is really the whole moral of this chapter: RefCells make everything
sadness. Up until now, RefCells have just been a nuisance. Now they're going to
become a nightmare.

So what's going on? To understand that, we need to go back to the definition of
`borrow`:

```rust
fn borrow<'a>(&'a self) -> Ref<'a, T>
fn borrow_mut<'a>(&'a self) -> RefMut<'a, T>
```

In the layout section we said:

> Rather than enforcing this statically, RefCell enforces them at runtime.
> If you break the rules, RefCell will just panic and crash the program.
> Why does it return these Ref and RefMut things? Well, they basically behave
> like `Rc`s but for borrowing. They keep the RefCell borrowed until they go out
> of scope. We'll get to that later.

It's later.

`Ref` and `RefMut` implement `Deref` and `DerefMut` respectively. So for most
intents and purposes they behave *exactly* like `&T` and `&mut T`. However,
because of how those traits work, the reference that's returned is connected
to the lifetime of the Ref, and not actual RefCell. This means that the Ref
has to be sitting around as long as we keep the reference around.

This is in fact necessary for correctness. When a Ref gets dropped, it tells
the RefCell that it's not borrowed anymore. So if *did* manage to hold onto our
reference longer than the Ref existed, we could get a RefMut while a reference
was kicking around and totally break Rust's type system in half.

So where does that leave us? We only want to return a reference, but we need
to keep this Ref thing around. But as soon as we return the reference from
`peek`, the function is over and the `Ref` goes out of scope.

😖

As far as I know, we're actually totally dead in the water here. You can't
totally encapsulate the use of RefCells like that.

But... what if we just give up on totally hiding our implementation details?
What if we returns Refs?

```rust
pub fn peek_front(&self) -> Option<Ref<T>> {
    self.head.as_ref().map(|node| {
        node.borrow()
    })
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:62:40: 62:46 error: use of undeclared type name `Ref` [E0412]
src/fourth.rs:62     pub fn peek_front(&self) -> Option<Ref<T>> {
                                                        ^~~~~~
error: aborting due to previous error
Could not compile `lists`.
```

Blurp. Gotta import some stuff.


```rust
use std::cell::{Ref, RefMut, RefCell};
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:63:9: 65:11 error: mismatched types:
 expected `core::option::Option<core::cell::Ref<'_, T>>`,
    found `core::option::Option<core::cell::Ref<'_, fourth::Node<T>>>`
(expected type parameter,
    found struct `fourth::Node`) [E0308]
src/fourth.rs:63         self.head.as_ref().map(|node| {
src/fourth.rs:64             node.borrow()
src/fourth.rs:65         })
src/fourth.rs:63:9: 65:11 help: run `rustc --explain E0308` to see a detailed explanation
error: aborting due to previous error
Could not compile `lists`.
```

Hmm... that's right. We have a `Ref<Node<T>>`, but we want a `Ref<T>`. We could
abandon all hope of encapsulation and just return that. We could also make
things even more complicated and wrap `Ref<Node<T>>` in a new type to only
expose access to an `&T`.

Both of those options are *kinda* lame.

Instead, we're going to go deeper down the nightly unstable features hole. This
one is actually gratuitous since the newtype solution is actually fine. But
we're already on nightly, and this list already has me deeply depressed. Let's
have some *fun*. Our source of fun is *this beast*:

```rust
map<U, F>(orig: Ref<'b, T>, f: F) -> Ref<'b, U>
    where F: FnOnce(&T) -> &U,
          U: ?Sized
```

> Make a new Ref for a component of the borrowed data.

Yes: just like you can map over an Option, you can map over a Ref.

I'm sure someone somewhere is really excited because *monads* or whatever but
I don't care about any of that. Also I don't think it's a proper monad since
there's no None-like case. But I digress.

It's cool and that's all that matters to me. *I need this*.

```rust
pub fn peek_front(&self) -> Option<Ref<T>> {
    self.head.as_ref().map(|node| {
        Ref::map(node.borrow(), |node| &node.elem)
    })
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:64:13: 64:21 error: use of unstable library feature 'cell_extras': recently added
src/fourth.rs:64             Ref::map(node.borrow(), |node| &node.elem)
                             ^~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:63:32: 65:10 note: expansion site
src/fourth.rs:64:13: 64:21 help: add #![feature(cell_extras)] to the crate attributes to enable
src/fourth.rs:1:22: 1:28 warning: unused import, #[warn(unused_imports)] on by default
src/fourth.rs:1 use std::cell::{Ref, RefMut, RefCell};
                                     ^~~~~~
error: aborting due to previous error
Could not compile `lists`.
```

*Yeah Yeah...*

```rust
// in lib.rs
#![feature(rc_unique, cell_extras)]
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:1:22: 1:28 warning: unused import, #[warn(unused_imports)] on by default
src/fourth.rs:1 use std::cell::{Ref, RefMut, RefCell};
                                     ^~~~~~
```

Awww yissss

Let's make sure this is working by munging up the test from our stack. We need
to do some munging to deal with the fact that Refs don't implement comparisons.

```rust
#[test]
fn peek() {
    let mut list = List::new();
    assert!(list.peek_front().is_none());
    list.push_front(1); list.push_front(2); list.push_front(3);

    assert_eq!(&*list.peek_front().unwrap(), &3);
}
```


```
> cargo test
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:1:22: 1:28 warning: unused import, #[warn(unused_imports)] on by default
src/fourth.rs:1 use std::cell::{Ref, RefMut, RefCell};
                                     ^~~~~~
     Running target/debug/lists-5c71138492ad4b4a

running 10 tests
test first::test::basics ... ok
test fourth::test::basics ... ok
test second::test::basics ... ok
test fourth::test::peek ... ok
test second::test::iter_mut ... ok
test second::test::into_iter ... ok
test third::test::basics ... ok
test second::test::peek ... ok
test second::test::iter ... ok
test third::test::iter ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

Great!
