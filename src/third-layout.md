# Layout

Alright, back to the drawing board on layout.

The most important thing about
a persistent list is that you can manipulate the tails of lists basically
for free:

For instance, this isn't an uncommon workload to see with a persistent list:

```text
list1 = A -> B -> C -> D
list2 = tail(list1) = B -> C -> D
list3 = push(list2, X) = X -> B -> C -> D
```

But at the end we want the memory to look like this:

```text
list1 -> A ---+
              |
              v
list2 ------> B -> C -> D
              ^
              |
list3 -> X ---+
```

This just can't work with Boxes, because ownership of `B` is *shared*. Who
should free it? If I drop list2, does it free B? With boxes we certainly would
expect so!

Functional languages -- and indeed almost every other language -- get away with
this by using *garbage collection*. With the magic of garbage collection, B will
be freed only after everyone stops looking at it. Hooray!

Rust doesn't have anything like the garbage collectors these languages have.
They have *tracing* GC, which will dig through all the memory that's sitting
around at runtime and figure out what's garbage automatically. Instead, all
Rust has today is *reference counting*. Reference counting can be thought of
as a very simple GC. For many workloads, it has significantly less throughput
than a tracing collector, and it completely falls over if you manage to
build cycles. But hey, it's all we've got! Thankfully, for our usecase we'll never run into cycles
(feel free to try to prove this to yourself -- I sure won't).

So how do we do reference-counted garbage collection? `Rc`! Rc is just like
Box, but we can duplicate it, and its memory will *only* be freed when *all*
the Rc's derived from it are dropped. Unfortunately, this flexibility comes at
a serious cost: we can only take a shared reference to its internals. This means
we can't ever really get data out of one of our lists, nor can we mutate them.

So what's our layout gonna look like? Well, previously we had:

```rust ,ignore
pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}
```

Can we just change Box to Rc?

```rust ,ignore
// in third.rs

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}
```

```text
cargo build

error[E0412]: cannot find type `Rc` in this scope
 --> src/third.rs:5:23
  |
5 | type Link<T> = Option<Rc<Node<T>>>;
  |                       ^^ not found in this scope
help: possible candidate is found in another module, you can import it into scope
  |
1 | use std::rc::Rc;
  |
```

Oh dang, sick burn. Unlike everything we used for our mutable lists, Rc is so
lame that it's not even implicitly imported into every single Rust program.
*What a loser*.

```rust ,ignore
use std::rc::Rc;
```

```text
cargo build

warning: field is never used: `head`
 --> src/third.rs:4:5
  |
4 |     head: Link<T>,
  |     ^^^^^^^^^^^^^
  |
  = note: #[warn(dead_code)] on by default

warning: field is never used: `elem`
  --> src/third.rs:10:5
   |
10 |     elem: T,
   |     ^^^^^^^

warning: field is never used: `next`
  --> src/third.rs:11:5
   |
11 |     next: Link<T>,
   |     ^^^^^^^^^^^^^
```

Seems legit. Rust continues to be a *completely* trivial to write. I bet we can just
find-and-replace Box with Rc and call it a day!

...

No. No we can't.
