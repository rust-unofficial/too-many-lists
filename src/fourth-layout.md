# Layout

The key to our design is the `RefCell` type. The heart of
RefCell is a pair of methods:

```rust ,ignore
fn borrow(&self) -> Ref<'_, T>;
fn borrow_mut(&self) -> RefMut<'_, T>;
```

The rules for `borrow` and `borrow_mut` are exactly those of `&` and `&mut`:
you can call `borrow` as many times as you want, but `borrow_mut` requires
exclusivity.

Rather than enforcing this statically, RefCell enforces them at runtime.
If you break the rules, RefCell will just panic and crash the program.
Why does it return these Ref and RefMut things? Well, they basically behave
like `Rc`s but for borrowing. They also keep the RefCell borrowed until they go out
of scope. We'll get to that later.

Now with Rc and RefCell we can become... an incredibly verbose pervasively
mutable garbage collected language that can't collect cycles! Y-yaaaaay...

Alright, we want to be *doubly-linked*. This means each node has a pointer to
the previous and next node. Also, the list itself has a pointer to the
first and last node. This gives us fast insertion and removal on *both*
ends of the list.

So we probably want something like:

```rust ,ignore
use std::rc::Rc;
use std::cell::RefCell;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}
```

```text
> cargo build

warning: field is never used: `head`
 --> src/fourth.rs:5:5
  |
5 |     head: Link<T>,
  |     ^^^^^^^^^^^^^
  |
  = note: #[warn(dead_code)] on by default

warning: field is never used: `tail`
 --> src/fourth.rs:6:5
  |
6 |     tail: Link<T>,
  |     ^^^^^^^^^^^^^

warning: field is never used: `elem`
  --> src/fourth.rs:12:5
   |
12 |     elem: T,
   |     ^^^^^^^

warning: field is never used: `next`
  --> src/fourth.rs:13:5
   |
13 |     next: Link<T>,
   |     ^^^^^^^^^^^^^

warning: field is never used: `prev`
  --> src/fourth.rs:14:5
   |
14 |     prev: Link<T>,
   |     ^^^^^^^^^^^^^
```

Hey, it built! Lots of dead code warnings, but it built! Let's try to use it.
