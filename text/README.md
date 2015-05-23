% Learning Rust With Entirely Too Many Linked Lists

Let me preface this entire post with the following: I hate linked lists. With a passion.

With that said, in this series of articles I will teach you basic and advanced Rust programming entirely by having you implement linked lists 7 times.

* The following pointer types: `&`, `&mut`, `Box`, `Rc`, `Arc`, `*const`, `*mut` 
* Ownership, borrowing, inherited mutability, interior mutability
* All the Keywords: struct, enum, fn, pub, impl, use, ...
* pattern matching, generics
* Unsafe Rust

Yes, linked lists are so truly awful that you deal with all of these concepts in making them real.

Just so we're on the same page about stuff, I'll be including all the commands and results. We'll be developing all our lists in a little cargo project (Rust's package manager):

```text
> cargo new lists
> cd lists
```

We'll put each list in a separate file so that we don't lose any of our work.