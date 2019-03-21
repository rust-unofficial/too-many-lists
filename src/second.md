# An Ok Singly-Linked Stack

In the previous chapter we wrote up a minimum viable singly-linked
stack. However there's a few design decisions that make it kind of sucky.
Let's make it less sucky. In doing so, we will:

* Deinvent the wheel
* Make our list able to handle any element type
* Add peeking
* Make our list iterable

And in the process we'll learn about

* Advanced Option use
* Generics
* Lifetimes
* Iterators

Let's add a new file called `second.rs`:

```rust ,ignore
// in lib.rs

pub mod first;
pub mod second;
```

And copy everything from `first.rs` into it.
