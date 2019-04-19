# Layout

So what's a singly-linked queue like? Well, when we had a singly-linked stack
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
about precise asymptotic bounds, just "fast" vs "slow". Queues guarantee
that push and pop are fast, and walking over the whole list is definitely *not*
fast.

One key observation is that we're wasting a ton of work doing *the same thing*
over and over. Can we memoize this work? Why, yes! We can store a pointer to
the end of the list, and just jump straight to there!

It turns out that only one inversion of `push` and `pop` works with this.
To invert `pop` we would have to move the "tail" pointer backwards, but
because our list is singly-linked, we can't do that efficiently.
If we instead invert `push` we only have to move the "head" pointer
forwards, which is easy.

Let's try that:

```rust ,ignore
use std::mem;

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

error[E0382]: use of moved value: `new_tail`
  --> src/fifth.rs:38:38
   |
26 |         let new_tail = Box::new(Node {
   |             -------- move occurs because `new_tail` has type `std::boxed::Box<fifth::Node<T>>`, which does not implement the `Copy` trait
...
33 |         let old_tail = mem::replace(&mut self.tail, Some(new_tail));
   |                                                          -------- value moved here
...
38 |                 old_tail.next = Some(new_tail);
   |                                      ^^^^^^^^ value used here after move
```

Shoot!

> use of moved value: `new_tail`

Box doesn't implement Copy, so we can't just assign it to two locations. More
importantly, Box *owns* the thing it points to, and will try to free it when
it's dropped. If our `push` implementation compiled, we'd double-free the tail
of our list! Actually, as written, our code would free the old_tail on every
push. Yikes! 🙀

Alright, well we know how to make a non-owning pointer. That's just a reference!

```rust ,ignore
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

error[E0106]: missing lifetime specifier
 --> src/fifth.rs:3:18
  |
3 |     tail: Option<&mut Node<T>>, // NEW!
  |                  ^ expected lifetime parameter
```

Oh right, we need to give references in types lifetimes. Hmm... what's the
lifetime of this reference? Well, this seems like IterMut, right? Let's try
what we did for IterMut, and just add a generic `'a`:

```rust ,ignore
pub struct List<'a, T> {
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

error[E0495]: cannot infer an appropriate lifetime for autoref due to conflicting requirements
  --> src/fifth.rs:35:27
   |
35 |                 self.head.as_mut().map(|node| &mut **node)
   |                           ^^^^^^
   |
note: first, the lifetime cannot outlive the anonymous lifetime #1 defined on the method body at 18:5...
  --> src/fifth.rs:18:5
   |
18 | /     pub fn push(&mut self, elem: T) {
19 | |         let new_tail = Box::new(Node {
20 | |             elem: elem,
21 | |             // When you push onto the tail, your next is always None
...  |
39 | |         self.tail = new_tail;
40 | |     }
   | |_____^
note: ...so that reference does not outlive borrowed content
  --> src/fifth.rs:35:17
   |
35 |                 self.head.as_mut().map(|node| &mut **node)
   |                 ^^^^^^^^^
note: but, the lifetime must be valid for the lifetime 'a as defined on the impl at 13:6...
  --> src/fifth.rs:13:6
   |
13 | impl<'a, T> List<'a, T> {
   |      ^^
   = note: ...so that the expression is assignable:
           expected std::option::Option<&'a mut fifth::Node<T>>
              found std::option::Option<&mut fifth::Node<T>>


```

Woah, that's a really detailed error message. That's a bit concerning, because it
suggests we're doing something really messed up. Here's an interesting part:

> the lifetime must be valid for the lifetime `'a` as defined on the impl

We're borrowing from `self`, but the compiler wants us to last as long as `'a`,
what if we tell it `self` *does* last that long..?

```rust ,ignore
    pub fn push(&'a mut self, elem: T) {
```

```text
cargo build

warning: field is never used: `elem`
 --> src/fifth.rs:9:5
  |
9 |     elem: T,
  |     ^^^^^^^
  |
  = note: #[warn(dead_code)] on by default
```

Oh, hey, that worked! Great!

Let's just do `pop` too:

```rust ,ignore
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

And write a quick test for that:

```rust ,ignore
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

error[E0499]: cannot borrow `list` as mutable more than once at a time
  --> src/fifth.rs:68:9
   |
65 |         assert_eq!(list.pop(), None);
   |                    ---- first mutable borrow occurs here
...
68 |         list.push(1);
   |         ^^^^
   |         |
   |         second mutable borrow occurs here
   |         first borrow later used here

error[E0499]: cannot borrow `list` as mutable more than once at a time
  --> src/fifth.rs:69:9
   |
65 |         assert_eq!(list.pop(), None);
   |                    ---- first mutable borrow occurs here
...
69 |         list.push(2);
   |         ^^^^
   |         |
   |         second mutable borrow occurs here
   |         first borrow later used here

error[E0499]: cannot borrow `list` as mutable more than once at a time
  --> src/fifth.rs:70:9
   |
65 |         assert_eq!(list.pop(), None);
   |                    ---- first mutable borrow occurs here
...
70 |         list.push(3);
   |         ^^^^
   |         |
   |         second mutable borrow occurs here
   |         first borrow later used here


....

** WAY MORE LINES OF ERRORS **

....

error: aborting due to 11 previous errors
```

🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀🙀

Oh my goodness.

The compiler's not wrong for vomiting all over us. We just committed a
cardinal Rust sin: we stored a reference to ourselves *inside ourselves*.
Somehow, we managed to convince Rust that this totally made sense in our
`push` and `pop` implementations (I was legitimately shocked we did). I believe
the reason is that Rust can't yet tell that the reference is into ourselves
from just `push` and `pop` -- or rather, Rust doesn't really have that notion
at all. Reference-into-yourself failing to work is just an emergent behaviour.

As soon as we tried to *use* our list, everything quickly fell apart.
When we call `push` or `pop`, we promptly store a reference to ourselves in
ourselves and become *trapped*. We are literally borrowing ourselves.

Our `pop` implementation hints at why this could be really dangerous:

```rust ,ignore
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

```rust ,ignore
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

