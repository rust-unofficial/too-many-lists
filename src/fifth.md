# An Unsafe Singly-Linked Queue

Ok that reference-counted interior mutability stuff got a little out of
control. Surely Rust doesn't really expect you to do that sort of thing
in general? Well, yes and no. Rc and Refcell can be great for handling
simple cases, but they can get unwieldy. Especially if you
want to hide that it's happening. There's gotta be a better way!

In this chapter we're going to roll back to singly-linked lists and
implement a singly-linked queue to dip our toes into *raw pointers*
and *Unsafe Rust*.

Let's add a new file called `fifth.rs`:

```rust ,ignore
// in lib.rs

pub mod first;
pub mod second;
pub mod third;
pub mod fourth;
pub mod fifth;
```

Our code is largely going to be derived from second.rs, since a queue is
mostly an augmentation of a stack in the world of linked lists. Still, we're
going to go from scratch because there's some fundamental issues we want to
address with layout and what-not.
