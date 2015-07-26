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
* You're writing a kernel/embedded thing and want to use an intrusive list.
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
structure. It's a fine data structure with several great usecases, but those
use cases are *exceptional*, not common.

Several people apparently read the first paragraph of this PSA and then stop
reading. Like, literally they'll try rebut my argument by listing one of the
things in my list of *great use cases*. The thing right after the first
paragraph!

Just so I can link directly to a detailed argument, here are several attempts
at counter-arguments I have seen, and my response to them




## Performance doesn't always matter

Yes! Maybe your application is I/O-bound or the code in question is in some
cold case that just doesn't matter. But this isn't even an argument for using
a linked list. This is an argument for using *whatever at all*. Why settle for
a linked list? Use a linked hash map!

If performance doesn't matter, then it's *surely* fine to apply the natural
default of an array.





## They have O(1) split-append-insert-remove if you have a pointer there

Yep! Although as [Bjarne Stroustrup notes][bjarne] *this doesn't actually
matter* if the time it takes to get that pointer completely dwarfs the
time it would take to just copy over all the elements in an array (which is
really quite fast).

Unless you have a workload that is heavily dominated by splitting and merging
costs, the penalty *every other* operation takes due to caching effects and code
complexity will eliminate any theoretical gains.

*But yes, if you're profiling your application to spend a lot of time in
splitting and merging, you may have gains in a linked list*.





## I can't afford amortization

You've already entered a pretty niche space, most can afford amortization.
Still, arrays are amortized *in the worst case*. Just because you're using an
array, doesn't mean you have amortized costs. If you can predict how many
elements you're going to store (or even have an upper-bound), you can
pre-reserve all the space you need. In my experience it's *very* common to be
able to predict how many elements you'll need. In Rust in particular, all
iterators provide a `size_hint` for exactly this case.

Then `push` and `pop` will be truly O(1) operations. And they're going to be
*considerably* faster than `push` and `pop` on linked list. You do a pointer
offset, write the bytes, and increment an integer. No need to go to any kind of
allocator.

How's that for low latency?

*But yes, if you can't predict your load, there are latency savings to be had!*





## Linked lists waste less space

Well, this is complicated. A "standard" array resizing strategy is to grow
or shrink so that at most half the array is empty. This is indeed a lot of
wasted space. Especially in Rust, we don't automatically shrink collections
(it's a waste if you're just going to fill it back up again), so the wastage
can approach infinity!

But this is a worst-case scenario. In the best-case, an array stack only has
three pointers of overhead for the entire array. Basically no overhead.

Linked lists on the other hand unconditionally waste space per element.
A singly-linked lists wastes one pointer while a doubly-linked list wastes
two. Unlike an array, the relative wasteage is proportional to the size of
the element. If you have *huge* elements this approaches 0 waste. If you have
tiny elements (say, bytes), then this can be as much as 16x memory overhead
(8x on 32-bit)!

Actually, it's more like 23x (11x on 32-bit) because padding will be added
to the byte to align the whole node's size to a pointer.

This is also assuming the best-case for your allocator: that allocating and
deallocating nodes is being done densely and you're not losing memory to
fragmentation.

*But yes, if you have huge elements, can't predict your load, and have a
decent allocator, there are memory savings to be had!*





## I use linked lists all the time in &lt;functional language&gt;

Great! Linked lists are super elegant to use in functional languages
because you can manipulate them without any mutation, can describe them
recursively, and also work with infinite lists do to laziness.

However it should be noted that Rust can pattern match on arrays and talk
about sub-arrays [using slices][slices]! It's actually even more expressive
than a functional list because you can talk about the last element or even
"the array without the first and last two elements" or whatever you want.

It is true that you can't *build* a list using slices. You can only tear
them apart.

For laziness we instead have [iterators][]. These can be infinite and you
can map, filter, reverse, and concatenate them just like a functional list,
and it will all be done just as lazily. Slices can be coerced to an iterator.

*But yes, if you're limited to immutable semantics, linked lists can be very
nice*.





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

It should be noted that the *authentic* Rust learning experience involves
writing code, having the compiler scream at you, and trying to figure out
what the heck that means. I will be carefully ensuring that this occurs as
frequently as possible. Learning to read and understand Rust's generally
excellent compiler errors is *incredibly* important to being a productive
Rust programmer.



[rust-std-list]: https://doc.rust-lang.org/std/collections/struct.LinkedList.html
[cpp-std-list]: http://en.cppreference.com/w/cpp/container/list
[github]: https://github.com/Gankro/too-many-lists
[bjarne]: https://www.youtube.com/watch?v=YQs6IC-vgmo
[slices]: https://doc.rust-lang.org/book/slice-patterns.html
[iterators]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
