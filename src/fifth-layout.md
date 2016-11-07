# Layout

So what's a singly-linked queue like? Well, when we had a singly linked stack
we pushed onto one end of the list, and then popped off the same end. The only
difference between a stack and a queue is that a queue pops off the *other*
end. So from our stack implementation we have:

```text
input list:
[Some(ptr)] -> (A, Some(ptr)) -> (B, None)

stack push X:
[Some(ptr)] -> (X, Some(ptr)) -> (A, Some(ptr)) -> (B, None)

stack pop:
[Some(ptr)] -> (A, Some(ptr)) -> (B, None)
```

To make a queue, we just need to decide which operation to move to the
end of the list: push, or pop? Since our list is singly-linked, we can
actually move *either* operation to the end with the same amount of effort.

To move `push` to the end, we just walk all the way to the `None` and set it
to Some with the new element.

```text
input list:
[Some(ptr)] -> (A, Some(ptr)) -> (B, None)

flipped push X:
[Some(ptr)] -> (A, Some(ptr)) -> (B, Some(ptr)) -> (X, None)
```

To move `pop` to the end, we just walk all the way to the node *before* the
None, and `take` it:

```text
input list:
[Some(ptr)] -> (A, Some(ptr)) -> (B, Some(ptr)) -> (X, None)

flipped pop:
[Some(ptr)] -> (A, Some(ptr)) -> (B, None)
```

We could do this today and call it quits, but that would stink! Both of these
operations walk over the *entire* list. Some would argue that such a queue
implementation is indeed a queue because it exposes the right interface. However
I believe that performance guarantees are part of the interface. I don't care
about precise asymptotic bounds, but rather "fast" and "slow". Queues guarantee
that push and pop are fast, and walking over the whole list is definitely *not*
fast.

One key observation is that we're wasting a ton of work doing *the same thing*
over and over. Can we memoize this work? Why, yes! We can store a pointer to
the end of the list, and just jump straight to there!

It turns out that only one inversion of `push` and `pop` works with this.
Because our list is singly-linked, we can't effeciently walk *backwards* in
the list. To invert `pop` we would have to move the "tail" pointer backwards.
But if we instead invert `push` we only have to move the "head" pointer
forwards, which is easy.

Let's try that:

```rust
use std::mem;
# fn main() {}

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>, // NEW!
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_tail = Box::new(Node {
            elem: elem,
            // When you push onto the tail, your next is always None
            next: None,
        });

        // swap the old tail to point to the new tail
        let old_tail = mem::replace(&mut self.tail, Some(new_tail));

        match old_tail {
            Some(mut old_tail) => {
                // If the old tail existed, update it to point to the new tail
                old_tail.next = Some(new_tail);
            }
            None => {
                // Otherwise, update the head to point to it
                self.head = Some(new_tail);
            }
        }
    }
}
```

I'm going a bit faster with the impl details now since we should be pretty
comfortable with this sort of thing. Not that you should necessarily expect
to produce this code on the first try. I'm just skipping over some of the
trial-and-error we've had to deal with before. I actually made a ton of mistakes
writing this code that I'm not showing. You can only see me leave off a `mut` or
`;` so many times before it stops being instructive. Don't worry, we'll see
plenty of *other* error messages!

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fifth.rs:33:38: 33:46 error: use of moved value: `new_tail` [E0382]
src/fifth.rs:33                 old_tail.next = Some(new_tail);
                                                     ^~~~~~~~
src/fifth.rs:28:58: 28:66 note: `new_tail` moved here because it has type `Box<fifth::Node<T>>`, which is non-copyable
src/fifth.rs:28         let old_tail = mem::replace(&mut self.tail, Some(new_tail));
                                                                         ^~~~~~~~
src/fifth.rs:37:34: 37:42 error: use of moved value: `new_tail` [E0382]
src/fifth.rs:37                 self.head = Some(new_tail);
                                                 ^~~~~~~~
src/fifth.rs:28:58: 28:66 note: `new_tail` moved here because it has type `Box<fifth::Node<T>>`, which is non-copyable
src/fifth.rs:28         let old_tail = mem::replace(&mut self.tail, Some(new_tail));
                                                                         ^~~~~~~~
error: aborting due to 2 previous errors
Could not compile `lists`.
```

Shoot!

> use of moved value: `new_tail`

Box doesn't implement Copy, so we can't just assign it to two locations. More
importantly, Box *owns* the thing it points to, and will try to free it when
it's dropped. If our `push` implementation compiled, we'd double-free the tail
of our list! Actually, as written, our code would free the old_tail on every
push. Yikes! 🙀

Alright, well we know how to make a non-owning pointer. That's just a reference!

```rust
# fn main() {}
pub struct List<T> {
    head: Link<T>,
    tail: Option<&mut Node<T>>, // NEW!
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_tail = Box::new(Node {
            elem: elem,
            // When you push onto the tail, your next is always None
            next: None,
        });

        // Put the box in the right place, and then grab a reference to its Node
        let new_tail = match self.tail.take() {
            Some(old_tail) => {
                // If the old tail existed, update it to point to the new tail
                old_tail.next = Some(new_tail);
                old_tail.next.as_mut().map(|node| &mut **node)
            }
            None => {
                // Otherwise, update the head to point to it
                self.head = Some(new_tail);
                self.head.as_mut().map(|node| &mut **node)
            }
        };

        self.tail = new_tail;
    }
}
```

Nothing too tricky here. Same basic idea as the previous code, except we're
using some of that implicit return goodness to extract the tail reference from
wherever we stuff the actual Box.

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fifth.rs:3:18: 3:30 error: missing lifetime specifier [E0106]
src/fifth.rs:3     tail: Option<&mut Node<T>>, // NEW!
                                ^~~~~~~~~~~~
src/fifth.rs:3:18: 3:30 help: run `rustc --explain E0106` to see a detailed explanation
error: aborting due to previous error
Could not compile `lists`.
```

Oh right, we need to give references in types lifetimes. Hmm... what's the
lifetime of this reference? Well, this seems like IterMut, right? Let's try
what we did for IterMut, and just add a generic `'a`:

```rust
# fn main() {}
pub struct List<'a, T: 'a> {
    head: Link<T>,
    tail: Option<&'a mut Node<T>>, // NEW!
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<'a, T> List<'a, T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_tail = Box::new(Node {
            elem: elem,
            // When you push onto the tail, your next is always None
            next: None,
        });

        // Put the box in the right place, and then grab a reference to its Node
        let new_tail = match self.tail.take() {
            Some(old_tail) => {
                // If the old tail existed, update it to point to the new tail
                old_tail.next = Some(new_tail);
                old_tail.next.as_mut().map(|node| &mut **node)
            }
            None => {
                // Otherwise, update the head to point to it
                self.head = Some(new_tail);
                self.head.as_mut().map(|node| &mut **node)
            }
        };

        self.tail = new_tail;
    }
}
```

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fifth.rs:35:27: 35:35 error: cannot infer an appropriate lifetime for autoref due to conflicting requirements
src/fifth.rs:35                 self.head.as_mut().map(|node| &mut **node)
                                          ^~~~~~~~
src/fifth.rs:18:5: 40:6 help: consider using an explicit lifetime parameter as shown: fn push(&'a mut self, elem: T)
src/fifth.rs:18     pub fn push(&mut self, elem: T) {
src/fifth.rs:19         let new_tail = Box::new(Node {
src/fifth.rs:20             elem: elem,
src/fifth.rs:21             // When you push onto the tail, your next is always None
src/fifth.rs:22             next: None,
src/fifth.rs:23         });
                ...
error: aborting due to previous error
```

Oh lord. When the compiler starts telling us to just start adding lifetimes in
random places, it's a red flag that the compiler is deeply confused. But uh...
ok let's try that I guess?

```rust,ignore
    pub fn push(&'a mut self, elem: T) {
```

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fifth.rs:9:5: 9:12 warning: struct field is never used: `elem`, #[warn(dead_code)] on by default
src/fifth.rs:9     elem: T,
                   ^~~~~~~
```

Oh, hey, that worked! Great!

Let's just do `pop` too:

```rust
pub fn pop(&'a mut self) -> Option<T> {
    // Grab the list's current head
    self.head.take().map(|head| {
        let head = *head;
        self.head = head.next;

        // If we're out of `head`, make sure to set the tail to `None`.
        if self.head.is_none() {
            self.tail = None;
        }

        head.elem
    })
}
```

Let's try to just write a quick test for that:

```
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
    }
}
```

```text
cargo test
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/fifth.rs:68:9: 68:13 error: cannot borrow `list` as mutable more than once at a time
src/fifth.rs:68         list.push(2);
                        ^~~~
src/fifth.rs:66:9: 66:13 note: previous borrow of `list` occurs here; the mutable borrow prevents subsequent moves, borrows, or modification of `list` until the borrow ends
src/fifth.rs:66         list.push(1);
                        ^~~~
src/fifth.rs:70:6: 70:6 note: previous borrow ends here
src/fifth.rs:59     fn basics() {
...
src/fifth.rs:70     }
                    ^


**NOT SHOWN: LITERALLY A THOUSAND LINES OF BORROW CHECK ERRORS**


src/fifth.rs:84:20: 84:24 error: cannot borrow `list` as mutable more than once at a time
src/fifth.rs:84         assert_eq!(list.pop(), None);
                                   ^~~~
<std macros>:1:1: 9:39 note: in expansion of assert_eq!
src/fifth.rs:84:9: 84:38 note: expansion site
src/fifth.rs:83:20: 83:24 note: previous borrow of `list` occurs here; the mutable borrow prevents subsequent moves, borrows, or modification of `list` until the borrow ends
src/fifth.rs:83         assert_eq!(list.pop(), Some(1));
                                   ^~~~
<std macros>:1:1: 9:39 note: in expansion of assert_eq!
src/fifth.rs:83:9: 83:41 note: expansion site
src/fifth.rs:85:6: 85:6 note: previous borrow ends here
src/fifth.rs:59     fn basics() {
...
src/fifth.rs:85     }
                    ^
error: aborting due to 66 previous errors
```

🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀

OH MY GEEZ WHAT.

66 borrow check errors.

Oh my goodness.

[I'm pretty sure we just hit this bug in the compiler](https://github.com/rust-lang/rust/issues/27485).

But the compiler's not wrong for vomiting all over us. We just committed a
cardinal Rust sin: we stored a reference to ourselves *inside ourselves*.
Somehow, we managed to convince Rust that this totally made sense in our
`push` and `pop` implementations (I was legitimately shocked we did). I believe
the reason is that Rust can't yet tell that the reference is into ourselves
from just `push` and `pop` -- or rather, Rust doesn't really have that notion
at all. Reference-into-yourself falls over as an emergent behaviour.

However as soon as we tried to *use* our list, everything quickly fell apart.
When we call `push` or `pop`, we promptly store a reference to ourselves in
ourselves and become *trapped*. We are literally borrowing ourselves.

Our `pop` implementation hints at why this could be really dangerous:

```rust,ignore
// ...
if self.head.is_none() {
    self.tail = None;
}
```

What if we forgot to do this? Then our tail would point to some node *that
had been removed from the list*. Such a node would be instantly freed, and we'd
have a dangling pointer which Rust was supposed to protect us from!

And indeed Rust is protecting us from that kind of danger. Just in a very...
**roundabout** way.

So what can we do? Go back to `Rc<RefCell>>` hell?

Please. No.

No instead we're going to go off the rails and use *raw pointers*.
Our layout is going to look like this:

```rust
# fn main() {}
pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>, // DANGER DANGER
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}
```

And that's that. None of this wimpy reference-counted-dynamic-borrow-checking
nonsense! Real. Hard. Unchecked. Pointers.

Let's be C everyone. Let's be C all day.

I'm home. I'm ready.

Hello `unsafe`.

