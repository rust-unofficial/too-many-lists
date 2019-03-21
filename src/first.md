# A Bad Singly-Linked Stack

This one's gonna be *by far* the longest, as we need to introduce basically
all of Rust, and are gonna build up some things "the hard way" to better
understand the language.

We'll put our first list in `src/first.rs`. We need to tell Rust that `first.rs` is
something that our lib uses. All that requires is that we put this at the top of
`src/lib.rs` (which Cargo made for us):

```rust ,ignore
// in lib.rs
pub mod first;
```

