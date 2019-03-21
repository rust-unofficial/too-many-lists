# Push

So let's write pushing a value onto a list. `push` *mutates* the list,
so we'll want to take `&mut self`. We also need to take an i32 to push:

```rust ,ignore
impl List {
    pub fn push(&mut self, elem: i32) {
        // TODO
    }
}
```

First thing's first, we need to make a node to store our element in:

```rust ,ignore
    pub fn push(&mut self, elem: i32) {
        let new_node = Node {
            elem: elem,
            next: ?????
        };
    }
```

What goes `next`? Well, the entire old list! Can we... just do that?

```rust ,ignore
impl List {
    pub fn push(&mut self, elem: i32) {
        let new_node = Node {
            elem: elem,
            next: self.head,
        };
    }
}
```

```text
> cargo build
error[E0507]: cannot move out of borrowed content
  --> src/first.rs:19:19
   |
19 |             next: self.head,
   |                   ^^^^^^^^^ cannot move out of borrowed content
```

Nooooope. Rust is telling us the right thing, but it's certainly not obvious
what exactly it means, or what to do about it:

> cannot move out of borrowed content

We're trying to move the `self.head` field out to `next`, but Rust doesn't want
us doing that. This would leave `self` only partially initialized when we end
the borrow and "give it back" to its rightful owner. As we said before, that's
the *one* thing you can't do with an `&mut`: It would be super rude,
and Rust is very polite (it would also be incredibly dangerous, but surely
*that* isn't why it cares).

What if we put something back? Namely, the node that we're creating:


```rust ,ignore
pub fn push(&mut self, elem: i32) {
    let new_node = Box::new(Node {
        elem: elem,
        next: self.head,
    });

    self.head = Link::More(new_node);
}
```

```text
> cargo build
error[E0507]: cannot move out of borrowed content
  --> src/first.rs:19:19
   |
19 |             next: self.head,
   |                   ^^^^^^^^^ cannot move out of borrowed content
```

No dice. In principle, this is something Rust could actually accept, but it
won't (for various reasons -- the most serious being [exception safety][]). We need
some way to get the head without Rust noticing that it's gone. For advice, we
turn to infamous Rust Hacker Indiana Jones:

![Indy Prepares to mem::replace](img/indy.gif)

Ah yes, Indy suggests the `mem::replace` maneuver. This incredibly useful
function lets us steal a value out of a borrow by *replacing* it with another
value. Let's just pull in `std::mem` at the top of the file, so that `mem` is in
local scope:

```rust ,ignore
use std::mem;
```

and use it appropriately:

```rust ,ignore
pub fn push(&mut self, elem: i32) {
    let new_node = Box::new(Node {
        elem: elem,
        next: mem::replace(&mut self.head, Link::Empty),
    });

    self.head = Link::More(new_node);
}
```

Here we `replace` self.head temporarily with Link::Empty before replacing it
with the new head of the list. I'm not gonna lie: this is a pretty unfortunate
thing to have to do. Sadly, we must (for now).

But hey, that's `push` all done! Probably. We should probably test it, honestly.
Right now the easiest way to do that is probably to write `pop`, and make sure
that it produces the right results.





[exception safety]: https://doc.rust-lang.org/nightly/nomicon/exception-safety.html
