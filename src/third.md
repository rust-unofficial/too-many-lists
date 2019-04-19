# A Persistent Singly-Linked Stack

Alright, we've mastered the art of mutable singly-linked stacks.

Let's move from *single* ownership to *shared* ownership by writing a
*persistent* immutable singly-linked list. This will be exactly the list
that functional programmers have come to know and love. You can get the
head *or* the tail and put someone's head on someone else's tail...
and... that's basically it. Immutability is a hell of a drug.

In the process we'll largely just become familiar with Rc and Arc, but this
will set us up for the next list which will *change the game*.

Let's add a new file called `third.rs`:

```rust ,ignore
// in lib.rs

pub mod first;
pub mod second;
pub mod third;
```

No copy-pasta this time. This is a clean room operation.
