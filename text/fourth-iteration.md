% Iteration

Let's take a crack at iterating this bad-boy.

## IntoIter

IntoIter, as always, is going to be the easiest. Just wrap the stack and
call `pop`:

```rust
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.pop_front()
    }
}
```

But we have an interesting new development. Where previously there was only
ever one "natural" iteration order for our lists, a Deque is inherently
bi-directional. What's so special about front-to-back? What if someone wants
to iterate in the other direction?

Rust actually has an answer to this: `DoubleEndedIterator`. DoubleEndedIterator
*inherits* from Iterator (meaning all DoubleEndedIterator are Iterators) and
requires one new method: `next_back`. It has the exact same signature as
`next`, but it's supposed to yield elements from the other end. The semantics
of DoubleEndedIterator are super convenient for us: the iterator becomes a
deque. You can consume elements from the front and back until the two ends
converge, at which point the iterator is empty.

Much like Iterator and `next`, it turns out that `next_back` isn't really
something consumers of the DoubleEndedIterator really care about. Rather, the
best part of this interface is that it exposes the `rev` method, which wraps
up the iterator to make a new one that yields the elements in reverse order.
The semantics of this are fairly straight-forward: calls to `next` on the
reversed iterator are just calls to `next_back`.

Anyway, because we're already a deque providing this API is pretty easy:

```rust
impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}
```

And let's test it out:

```rust
#[test]
fn into_iter() {
    let mut list = List::new();
    list.push_front(1); list.push_front(2); list.push_front(3);

    let mut iter = list.into_iter();
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next_back(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);
}
```


```text
cargo test
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
     Running target/debug/lists-5c71138492ad4b4a

running 11 tests
test fourth::test::basics ... ok
test fourth::test::peek ... ok
test fourth::test::into_iter ... ok
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::iter ... ok
test second::test::iter_mut ... ok
test third::test::iter ... ok
test third::test::basics ... ok
test second::test::into_iter ... ok
test second::test::peek ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

Nice.

## Iter

Iter will be a bit less forgiving. We'll have to deal with those awful `Ref`
things again! Because of Refs, we can't store `&Node`s like we did before.
Instead, let's try to store `Ref<Node>`s:

```rust
pub struct Iter<'a, T: 'a>(Option<Ref<'a, Node<T>>>);

impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter(self.head.as_ref().map(|head| head.borrow()))
    }
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

So far so good. Implementing `next` is going to be a bit hairy, but I think
it's the same basic logic as the old stack IterMut but with extra RefCell
madness:

```rust
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = Ref<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().map(|node_ref| {
            self.0 = node_ref.next.as_ref().map(|head| head.borrow());
            Ref::map(node_ref, |node| &node.elem)
        })
    }
}
```

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:154:22: 154:30 error: `node_ref` does not live long enough
src/fourth.rs:154             self.0 = node_ref.next.as_ref().map(|head| head.borrow());
                                       ^~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:153:27: 156:10 note: expansion site
src/fourth.rs:152:46: 157:6 note: reference must be valid for the lifetime 'a as defined on the block at 152:45...
src/fourth.rs:152     fn next(&mut self) -> Option<Self::Item> {
src/fourth.rs:153         self.0.take().map(|node_ref| {
src/fourth.rs:154             self.0 = node_ref.next.as_ref().map(|head| head.borrow());
src/fourth.rs:155             Ref::map(node_ref, |node| &node.elem)
src/fourth.rs:156         })
src/fourth.rs:157     }
src/fourth.rs:153:38: 156:10 note: ...but borrowed value is only valid for the scope of parameters for function at 153:37
src/fourth.rs:153         self.0.take().map(|node_ref| {
src/fourth.rs:154             self.0 = node_ref.next.as_ref().map(|head| head.borrow());
src/fourth.rs:155             Ref::map(node_ref, |node| &node.elem)
src/fourth.rs:156         })
error: aborting due to previous error
Could not compile `lists`.
```

Shoot.

`node_ref` doesn't live long enough. Unlike normal references, Rust doesn't let
us just split Refs up like that. The Ref we get out of `head.borrow()` is only
allowed to live as long as `node_ref`, but we end up trashing that in our
`Ref::map` call.

I don't think there's anything we can do here. It's a dead end. Let's try
getting out of the RefCells. What if we store an `&RefCell<Node>`?


```rust
pub struct Iter<'a, T: 'a>(Option<&'a RefCell<Node<T>>>);

impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter(self.head.as_ref().map(|head| &** head))
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = Ref<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().map(|node| {
            self.0 = node.borrow().next.as_ref().map(|head| &**head);
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fourth.rs:154:22: 154:35 error: borrowed value does not live long enough
src/fourth.rs:154             self.0 = node.borrow().next.as_ref().map(|head| &**head);
                                       ^~~~~~~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:153:27: 156:10 note: expansion site
src/fourth.rs:152:46: 157:6 note: reference must be valid for the lifetime 'a as defined on the block at 152:45...
src/fourth.rs:152     fn next(&mut self) -> Option<Self::Item> {
src/fourth.rs:153         self.0.take().map(|node| {
src/fourth.rs:154             self.0 = node.borrow().next.as_ref().map(|head| &**head);
src/fourth.rs:155             Ref::map(node.borrow(), |node| &node.elem)
src/fourth.rs:156         })
src/fourth.rs:157     }
src/fourth.rs:154:13: 154:70 note: ...but borrowed value is only valid for the statement at 154:12
src/fourth.rs:154             self.0 = node.borrow().next.as_ref().map(|head| &**head);
                              ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:153:27: 156:10 note: expansion site
src/fourth.rs:154:13: 154:70 help: consider using a `let` binding to increase its lifetime
src/fourth.rs:154             self.0 = node.borrow().next.as_ref().map(|head| &**head);
                              ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
note: in expansion of closure expansion
src/fourth.rs:153:27: 156:10 note: expansion site
error: aborting due to previous error
Could not compile `lists`.
```

Hmm... no this is the same problem again. The `borrow` makes a Ref that we can't
keep alive long enough.

Hang on, though. We've got `Rc`s. Who said we even needed to store references?
Why can't we just Clone the whole Rc to get a nice owning handle into the middle
of the list?

```rust
pub struct Iter<T>(Option<Rc<Node<T>>>);

impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter(self.head.as_ref().map(|head| head.clone()))
    }
}

impl<T> Iterator for Iter<T> {
    type Item =
```

Uh... Wait what do we return now? `&T`? `Ref<T>`?

No, none of those work... our Iter doesn't have a lifetime anymore! Both `&T`
and `Ref<T>` require us to declare some lifetime up front before we get into
`next`. But anything we manage to get out of our Rc would be borrowing the
Iterator... brain... hurt... aaaaaahhhhhh

Maybe we can... map... the Rc... to get an `Rc<T>`? Is that a thing? Rc's docs
don't seem to have anything like that. Actually someone made [a crate][own-ref]
that lets you do that.

But wait, even if we do *that* then we've got an even bigger problem: the
dreaded spectre of iterator invalidation. Previously we've been totally immune
to iterator invalidation, because the Iter borrowed the list, leaving it totally
immutable. However now that our Iter is full of Rcs, it doesn't borrow the list
at all! That means people can start calling `push` and `pop` on the list while
they have an Iter!

Oh lord, what will that do?!

Well, pushing is actually fine. We've got a view into some sub-range of the
list, and the list will just grow beyond our sights. No biggie.

However `pop` is another story. If they're popping elements outside outside of
our range, it should *still* be fine. We can't see those nodes so nothing will
happen. However if they try to pop off the node we're pointing at... everything
will blow up! In particular when they go to `unwrap` the result of the
`try_unwrap`, it will actually fail, and the whole program will panic.

That's actually pretty cool. We can get tons of interior owning pointers into
the list and mutate it at the same time *and it will just work* until they
try to remove the nodes that we're pointing at. And even then we don't get
dangling pointers or anything, the program will deterministically panic!

But having to deal with iterator invalidation on top of mapping Rcs just
seems... lame. `Rc<RefCell>` has really truly finally failed us. Interestingly,
we've experienced an inversion of the persistent stack case. Where the
persistent stack struggled to ever reclaim ownership of the data but could get
references all day every day, our list had no problem gaining ownership, but
really struggled to loan our references.

Although to be fair, most of our struggles revolved around wanting to hide the
implementation details and have a decent API. We *could* do everything fine
if we wanted to just pass around Nodes all over the place.

Heck, we could make multiple concurrent IterMuts that were runtime checked to
not be mutable accessing the same element!

Really, this design is more appropriate for an internal data structure that
never makes it out to consumers of the API. Interior mutability is great for
writing safe *applications*. Not so much safe *libraries*.

Anyway, that's me giving up on Iter and IterMut. We could do them, but *ugh*.

[own-ref]:
