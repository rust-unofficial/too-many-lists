% Learning Rust With Entirely Too Many Linked Lists

Got any issues or want to check out all the final code at once?
[Everything's on Github!][github]

I fairly frequently get asked how to implement a linked list in Rust. The
answer honestly depends on what your requirements are, and it's obviously not
super easy to answer the question on the spot. As such I've decided to write
this series of articles to comprehensively answer the question once and for all.

In this series I will teach you basic and advanced Rust programming
entirely by having you implement linked lists 7 times. In doing so, you should
learn:

* The following pointer types: `&`, `&mut`, `Box`, `Rc`, `Arc`, `*const`, `*mut`
* Ownership, borrowing, inherited mutability, interior mutability, Copy
* All The Keywords: struct, enum, fn, pub, impl, use, ...
* Pattern matching, generics, destructors
* Testing
* Unsafe Rust

Yes, linked lists are so truly awful that you deal with all of these concepts in
making them real.





# An Obligatory Public Service Announcement

Just so we're totally 100% clear: I hate linked lists. With
a passion. Linked lists are terrible data structures. Now of course there's
several great use cases for a linked list:

* You want to do *a lot* of splitting or merging of big lists. *A lot*.
* You're doing some awesome lock-free concurrent thing.
* You're writing a kernel and want to use an intrusive list.
* You're using a pure functional language and the limited semantics and absence
  of mutation makes linked lists easier to work with.

But all of these cases are *super rare* for anyone writing a Rust program. 99%
of the time you should just use a Vec (array stack), and 99% of the other 1%
of the time you should be using a VecDeque (array deque). These are blatantly
superior data structures for most workloads due to less frequent allocation,
lower memory overhead, true random access, and cache locality.

Linked lists are as *niche* and *vague* of a data structure as a trie. Few would
balk at me claiming a trie is a niche structure that your average programmer
could happily never learn in an entire productive career -- and yet linked lists
have some bizarre celebrity status. We teach every undergrad how to write a
linked list. It's the only niche collection
[I couldn't kill from std::collections][rust-std-list]. It's
[*the* list in C++][cpp-std-list]!

We should all as a community say *no* to linked lists as a "standard" data
structure.





# Take a Breath

Ok. That's out of the way. Let's write a bajillion linked lists.

Just so we're all the same page, I'll be writing out all the commands that I
feed into my terminal. I'll be using Rust's standard package manager, Cargo,
to develop the project. Cargo isn't necessary to write a Rust program, but it's
*so much* better.

```text
> cargo new lists
> cd lists
```

We'll put each list in a separate file so that we don't lose any of our work.



[rust-std-list]: https://doc.rust-lang.org/std/collections/struct.LinkedList.html
[cpp-std-list]: http://en.cppreference.com/w/cpp/container/list
[github]: https://github.com/Gankro/too-many-lists
