# A Bad but Safe Doubly-Linked Deque

Now that we've seen Rc and heard about interior mutability, this gives an
interesting thought... maybe we *can* mutate through an Rc. And if *that's*
the case, maybe we can implement a *doubly-linked* list totally safely!

In the process we'll become familiar with *interior mutability*, and probably
learn the hard way that safe doesn't mean *correct*. Doubly-linked lists are
hard, and I always make a mistake somewhere.

Let's add a new file called `fourth.rs`:

```rust ,ignore
// in lib.rs

pub mod first;
pub mod second;
pub mod third;
pub mod fourth;
```

This will be another clean-room operation, though as usual we'll probably find
some logic that applies verbatim again.

Disclaimer: this chapter is basically a demonstration that this is a very bad idea.
