# Building Up

Alright, we'll start with building the list. That's pretty straight-forward
with this new system. `new` is still trivial, just None out all the fields.
Also because it's getting a bit unwieldy, let's break out a Node constructor
too:

```rust ,ignore
impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem: elem,
            prev: None,
            next: None,
        }))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }
}
```

```text
> cargo build

**A BUNCH OF DEAD CODE WARNINGS BUT IT BUILT**
```

Yay!

Now let's try to write pushing onto the front of the list. Because
doubly-linked lists are significantly more complicated, we're going to need
to do a fair bit more work. Where singly-linked list operations could be
reduced to an easy one-liner, doubly-linked list ops are fairly complicated.

In particular we now need to specially handle some boundary cases around
empty lists. Most operations will only touch the `head` or `tail` pointer.
However when transitioning to or from the empty list, we need to edit
*both* at once.

An easy way for us to validate if our methods make sense is if we maintain
the following invariant: each node should have exactly two pointers to it.
Each node in the middle of the list is pointed at by its predecessor and
successor, while the nodes on the ends are pointed to by the list itself.

Let's take a crack at it:

```rust ,ignore
pub fn push_front(&mut self, elem: T) {
    // new node needs +2 links, everything else should be +0
    let new_head = Node::new(elem);
    match self.head.take() {
        Some(old_head) => {
            // non-empty list, need to connect the old_head
            old_head.prev = Some(new_head.clone()); // +1 new_head
            new_head.next = Some(old_head);         // +1 old_head
            self.head = Some(new_head);             // +1 new_head, -1 old_head
            // total: +2 new_head, +0 old_head -- OK!
        }
        None => {
            // empty list, need to set the tail
            self.tail = Some(new_head.clone());     // +1 new_head
            self.head = Some(new_head);             // +1 new_head
            // total: +2 new_head -- OK!
        }
    }
}
```

```text
cargo build

error[E0609]: no field `prev` on type `std::rc::Rc<std::cell::RefCell<fourth::Node<T>>>`
  --> src/fourth.rs:39:26
   |
39 |                 old_head.prev = Some(new_head.clone()); // +1 new_head
   |                          ^^^^ unknown field

error[E0609]: no field `next` on type `std::rc::Rc<std::cell::RefCell<fourth::Node<T>>>`
  --> src/fourth.rs:40:26
   |
40 |                 new_head.next = Some(old_head);         // +1 old_head
   |                          ^^^^ unknown field
```

Alright. Compiler error. Good start. Good start.

Why can't we access the `prev` and `next` fields on our nodes? It worked before
when we just had an `Rc<Node>`. Seems like the `RefCell` is getting in the way.

We should probably check the docs.

*Google's "rust refcell"*

*[clicks first link](https://doc.rust-lang.org/std/cell/struct.RefCell.html)*

> A mutable memory location with dynamically checked borrow rules
>
> See the [module-level documentation](https://doc.rust-lang.org/std/cell/index.html) for more.

*clicks link*

> Shareable mutable containers.
>
> Values of the `Cell<T>` and `RefCell<T>` types may be mutated through shared references (i.e.
> the common `&T` type), whereas most Rust types can only be mutated through unique (`&mut T`)
> references. We say that `Cell<T>` and `RefCell<T>` provide 'interior mutability', in contrast
> with typical Rust types that exhibit 'inherited mutability'.
>
> Cell types come in two flavors: `Cell<T>` and `RefCell<T>`. `Cell<T>` provides `get` and `set`
> methods that change the interior value with a single method call. `Cell<T>` though is only
> compatible with types that implement `Copy`. For other types, one must use the `RefCell<T>`
> type, acquiring a write lock before mutating.
>
> `RefCell<T>` uses Rust's lifetimes to implement 'dynamic borrowing', a process whereby one can
> claim temporary, exclusive, mutable access to the inner value. Borrows for `RefCell<T>`s are
> tracked 'at runtime', unlike Rust's native reference types which are entirely tracked
> statically, at compile time. Because `RefCell<T>` borrows are dynamic it is possible to attempt
> to borrow a value that is already mutably borrowed; when this happens it results in thread
> panic.
>
> # When to choose interior mutability
>
> The more common inherited mutability, where one must have unique access to mutate a value, is
> one of the key language elements that enables Rust to reason strongly about pointer aliasing,
> statically preventing crash bugs. Because of that, inherited mutability is preferred, and
> interior mutability is something of a last resort. Since cell types enable mutation where it
> would otherwise be disallowed though, there are occasions when interior mutability might be
> appropriate, or even *must* be used, e.g.
>
> * Introducing inherited mutability roots to shared types.
> * Implementation details of logically-immutable methods.
> * Mutating implementations of `Clone`.
>
> ## Introducing inherited mutability roots to shared types
>
> Shared smart pointer types, including `Rc<T>` and `Arc<T>`, provide containers that can be
> cloned and shared between multiple parties. Because the contained values may be
> multiply-aliased, they can only be borrowed as shared references, not mutable references.
> Without cells it would be impossible to mutate data inside of shared boxes at all!
>
> It's very common then to put a `RefCell<T>` inside shared pointer types to reintroduce
> mutability:
>
> ```rust ,ignore
> use std::collections::HashMap;
> use std::cell::RefCell;
> use std::rc::Rc;
>
> fn main() {
>     let shared_map: Rc<RefCell<_>> = Rc::new(RefCell::new(HashMap::new()));
>     shared_map.borrow_mut().insert("africa", 92388);
>     shared_map.borrow_mut().insert("kyoto", 11837);
>     shared_map.borrow_mut().insert("piccadilly", 11826);
>     shared_map.borrow_mut().insert("marbles", 38);
> }
> ```
>
> Note that this example uses `Rc<T>` and not `Arc<T>`. `RefCell<T>`s are for single-threaded
> scenarios. Consider using `Mutex<T>` if you need shared mutability in a multi-threaded
> situation.

Hey, Rust's docs continue to be incredibly awesome.

The meaty bit we care about is this line:

```rust ,ignore
shared_map.borrow_mut().insert("africa", 92388);
```

In particular, the `borrow_mut` thing. Seems we need to explicitly borrow a
RefCell. The `.` operator's not going to do it for us. Weird. Let's try:

```rust ,ignore
pub fn push_front(&mut self, elem: T) {
    let new_head = Node::new(elem);
    match self.head.take() {
        Some(old_head) => {
            old_head.borrow_mut().prev = Some(new_head.clone());
            new_head.borrow_mut().next = Some(old_head);
            self.head = Some(new_head);
        }
        None => {
            self.tail = Some(new_head.clone());
            self.head = Some(new_head);
        }
    }
}
```


```text
> cargo build

warning: field is never used: `elem`
  --> src/fourth.rs:12:5
   |
12 |     elem: T,
   |     ^^^^^^^
   |
   = note: #[warn(dead_code)] on by default
```

Hey, it built! Docs win again.
