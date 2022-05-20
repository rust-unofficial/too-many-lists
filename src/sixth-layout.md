# Layout

Let us begin by first studying the structure of our enemy. A Doubly-Linked List is conceptually simple, but that's how it decieves and manipulates you. It's the same kind of linked list we've looked at over and over, but the links go both ways. Double the links, double the evil.

So rather than this (gonna drop the Some/None stuff to keep it cleaner):

```text
... -> (A, ptr) -> (B, ptr) -> ...
```

We have this:

```text
... <-> (ptr, A, ptr) <-> (ptr, B, ptr) <-> ...
```

This lets you traverse the list from either direction, or seek back and forth with a [cursor](https://doc.rust-lang.org/std/collections/struct.LinkedList.html#method.cursor_back_mut).

In exchange for this flexibility, every node has to store twice as many pointers, and every operation has to fix up way more pointers. It's a significant enough complication that it's a lot easier to make a mistake, so we're going to be doing a lot of testing.

You might have also noticed that I intentionally haven't drawn the *ends* of the list. This is because this is one of the places where there are genuinely defensible options for our implementation. We *definitely* need our implementation to have two pointers: one to the start of the list, and one to the end of the list.

There are two notable ways to do this in my mind: "traditional" and "dummy node".

The traditional approach is the simple extension of how we did a Stack &mdash; just store the head and tail pointers on the stack:

```text
[ptr, ptr] <-> (ptr, A, ptr) <-> (ptr, B, ptr)
  ^                                        ^
  +----------------------------------------+
```

This is fine, but it has one downside: corner cases. There are now two edges to our list, which means twice as many corner cases. It's easy to forget one and have a serious bug.

The dummy node approach attempts to smooth out these corner cases by adding an extra node to our list which contains no data but links the two ends together into a ring:

```text
[ptr] -> (ptr, ?DUMMY?, ptr) <-> (ptr, A, ptr) <-> (ptr, B, ptr)
           ^                                                 ^
           +-------------------------------------------------+ 
```

By doing this, every node *always* has actual pointers to a previous and next node in the list. Even when you remove the last element from the list, you just end up stitching the dummy node to point at itself:

```text
[ptr] -> (ptr, ?DUMMY?, ptr) 
           ^             ^
           +-------------+
```

There is a part of me that finds this *very* satisfying and elegant. Unfortunately, it has a couple practical problems:

Problem 1: An extra indirection and allocation, especially for the empty list, which must include the dummy node. Potential solutions include:

* Don't allocate the dummy node until something is inserted: simple and effective, but it adds back some of the corner cases we were trying to avoid by using dummy pointers!

* Use a static copy-on-write empty singleton dummy node, with some really clever scheme that lets the Copy-On-Write checks piggy-back on normal checks: look I'm really tempted, I really do love that shit, but we can't go down that dark path in this book. Read [ThinVec's sourcecode](https://docs.rs/thin-vec/0.2.4/src/thin_vec/lib.rs.html#319-325) if you want to see that kind of perverted stuff.

* Store the dummy node on the stack - not practical in a language without C++-style move-constructors. I'm sure there's something weird thing we could do here with [pinning](https://doc.rust-lang.org/std/pin/index.html) but we're not gonna.

Problem 2: What *value* is stored in the dummy node? Sure if it's an integer it's fine, but what if we're storing a list full of `Box`? It may be impossible for us to initialize this value! Potential solutions include:

* Make every node store `Option<T>`: simple and effective, but also bloated and annoying.

* Make every node store [`MaybeUninit<T>`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html). Horrifying and annoying.

* *Really* careful and clever inheritance-style type punning so the dummy node doesn't include the data field. This is also tempting but it's extremely dangerous and annoying. Read [BTreeMap's source](https://doc.rust-lang.org/1.55.0/src/alloc/collections/btree/node.rs.html#49-104) if you want to see that kind of perverted stuff.

The problems really outweigh the convenience for a language like Rust, so we're going to stick to the traditional layout. We'll be using the same basic design as we did for the unsafe queue in the previous chapter:

```rust
pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
}

type Link<T> = *mut Node<T>;

struct Node<T> {
    front: Link<T>,
    back: Link<T>,
    elem: T, 
}
```

(Now that we have reached the doubly-linked-deque, we have finally earned the right to call ourselves LinkedList, for this is the True Linked List.)

This isn't quite a *true* production-quality layout yet. It's *fine* but there's magic tricks we can do to tell Rust what we're doing a bit better. To do that we're going to need to go... deeper.