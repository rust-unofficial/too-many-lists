# Breaking Down

`pop_front` should be the same basic logic as `push_front`, but backwards. Let's
try:

```rust
pub fn pop_front(&mut self) -> Option<T> {
    // need to take the old head, ensuring it's -2
    self.head.take().map(|old_head| {                         // -1 old
        match old_head.borrow_mut().next.take() {
            Some(new_head) => {                               // -1 new
                // not emptying list
                new_head.borrow_mut().prev.take();            // -1 old
                self.head = Some(new_head);                   // +1 new
                // total: -2 old, +0 new
            }
            None => {
                // emptying list
                self.tail.take();                             // -1 old
                // total: -2 old, (no new)
            }
        }
        old_head.elem
    })
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:63:13: 63:26 error: attempted access of field `elem` on type `alloc::rc::Rc<core::cell::RefCell<fourth::Node<T>>>`, but no field with that name was found
src/fourth.rs:63             old_head.elem
                             ^~~~~~~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:49:30: 64:10 note: expansion site
error: aborting due to previous error
Could not compile `lists`.
```

ACK. *RefCells*. Gotta `borrow_mut` again I guess...

```rust
pub fn pop_front(&mut self) -> Option<T> {
    self.head.take().map(|old_head| {
        match old_head.borrow_mut().next.take() {
            Some(new_head) => {
                new_head.borrow_mut().prev.take();
                self.head = Some(new_head);
            }
            None => {
                self.tail.take();
            }
        }
        old_head.borrow_mut().elem
    })
}
```

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:58:13: 58:34 error: cannot move out of borrowed content
src/fourth.rs:58             old_head.borrow_mut().elem
                             ^~~~~~~~~~~~~~~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:48:30: 59:10 note: expansion site
error: aborting due to previous error
Could not compile `lists`.
```

*sigh*

> cannot move out of borrowed content

Hrm... It seems that Box was *really* spoiling us. `borrow_mut` only gets us
an `&mut Node<T>`, but we can't move out of that!

We need something that takes a `RefCell<T>` and gives us a `T`. Let's check
[the docs][refcell] for something like that:

> `fn into_inner(self) -> T`
>
> Consumes the RefCell, returning the wrapped value.

That looks promising!

```rust
old_head.into_inner().elem
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:58:13: 58:21 error: cannot move out of borrowed content
src/fourth.rs:58             old_head.into_inner().elem
                             ^~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:48:30: 59:10 note: expansion site
error: aborting due to previous error
Could not compile `lists`.
```

Uh... what? Nothing changed! Dang compiler!

It's actually telling us the right thing, but it's not giving us quite enough
information to figure it out. So I'm just going to cut the chase: where
previously we were trying to move a `T` out of the `&mut Node<T>`, now we're
trying to move a `RefCell<T>` out of an `&RefCell<T>`. We've just pushed the
problem up the ownership chain.

As we saw in the previous chapter, `Rc<T>` only lets us get shared references
into it's internals. That makes sense, because that's *the whole point* of
reference counted pointers: they're shared!

This was a problem for us when we wanted to implement Drop for our reference
counted list, because we needed to gain ownership of the nodes to drop them
one at a time. 

Now we can use `Rc::try_unwrap`, which moves out the contents of an Rc out
if its refcount is 1.

```rust
Rc::try_unwrap(old_head).unwrap().into_inner().elem
```

`Rc::try_unwrap` returns a `Result<T, Rc<T>>`. Results are basically a
generalized `Option`, where the `None` case has data associated with it. In
this case, the `Rc` you tried to unwrap. Since we don't care about the case
where it fails (if we wrote our program correctly, it *has* to succeed), we
just call `unwrap` on it.

`unwrap` is a function on Results and Options that basically says "get the
the value out, or panic the program". Panics are A Whole Thing, so I'll simplify
things by just saying a panics crash your program in a controlled manner,
effectively making every function instantly return. Notably, it makes sure to
call all the destructors that happen to be laying around on the way out. So your
memory will get freed (meh? OS does that anyway, right?) and your Connections
and Files will get properly saved/closed (yay!).

This is a bit of a lie, there's cases where you can stop a panic. In particular
at a thread boundary if you're into that whole "parallelism" thing. That whole
freeing your memory thing is less lame there, *I guess*.

Anyway, let's see what compiler error we get next (let's face it, there's going
to be one).

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:58:38: 58:46 error: no method named `unwrap` found for type `core::result::Result<core::cell::RefCell<fourth::Node<T>>, alloc::rc::Rc<core::cell::RefCell<fourth::Node<T>>>>` in the current scope
src/fourth.rs:58             Rc::try_unwrap(old_head).unwrap().into_inner().elem
                                                      ^~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:48:30: 59:10 note: expansion site
src/fourth.rs:58:38: 58:46 note: the method `unwrap` exists but the following trait bounds were not satisfied: `alloc::rc::Rc<core::cell::RefCell<fourth::Node<T>>> : core::fmt::Debug`
error: aborting due to previous error
Could not compile `lists`.
```

UGH. This is dumb. Rust is dumb. `unwrap` on Result requires that you can
debug-print the error case. `RefCell<T>` only implements `Debug` if `T` does.
`Node` doesn't implement Debug. We could just implement Debug for Node but I'm
not in the mood. Let's just work around this by turning the `Result` into an
`Option` with the `ok` method:

```rust
Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
```

PLEASE.

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

YES.

*phew*

We did it.

We implemented `push` and `pop`.

Let's test by stealing the old `stack` basic test (because that's all that
we've implemented so far):

```
#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }
}
```

```text
cargo test
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
     Running target/debug/lists-5c71138492ad4b4a

running 9 tests
test first::test::basics ... ok
test fourth::test::basics ... ok
test second::test::iter_mut ... ok
test second::test::basics ... ok
test fifth::test::iter_mut ... ok
test third::test::basics ... ok
test second::test::iter ... ok
test third::test::iter ... ok
test second::test::into_iter ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

*Nailed it*.

Now that we can properly remove things from the list, we can implement Drop.
Drop is a little more conceptually interesting this time around. Where
previously we bothered to implement Drop for our stacks just to avoid unbounded
recursion, now we need to implement Drop to get *anything* to happen at all.

`Rc` can't deal with cycles. If there's a cycle, everything will keep everything
else alive. A doubly linked list, as it turns out, is just a big chain of tiny
cycles! So when we drop our list, the two end nodes will have their refcounts
decremented down to 1... and then nothing else will happen. Well, if our list
contains exactly one node we're good to go. But ideally a list should work right
if it contains multiple elements. Maybe that's just me.

As we saw, removing elements was a bit painful. So the easiest thing for us to
do is just `pop` until we get None:

```rust
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}
```

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

(We actually could have done this with our mutable stacks, but shortcuts are for
people who understand things!)

We could look at implementing the `_back` versions of `push` and `pop`, but
they're just copy-paste jobs which we'll defer to later in the chapter. For now
let's look at more interesting things!


[refcell]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
[multirust]: https://github.com/brson/multirust
[downloads]: https://www.rust-lang.org/install.html
