# An Production Unsafe Doubly-Linked Deque

We finally made it. My greatests nemesis: **[std::collections::LinkedList][linked-list], the Doubly-Linked Deque**. 

The one that I tried and failed to destroy.

Our story begins as 2014 was coming to a close and we were rapidly approaching the release of Rust 1.0, Rust's first stable release. I had found myself in the role of caring for `std::collections`, or as we affectionately called it in those times, libcollections.

libcollections had spent years as a dumping ground for everyone's Cute Ideas and Vaguely Useful Things. This was all well and good when Rust was a fledgling experimental language, but if my children were going to escape the nest and be stabilized, they would have to prove their worth.

Until then I had encouraged and nurtured them all, but it was now time for them to face judgement for their failings.

I sunk my claws into the bedrock and carved tombstones for my most foolish children. A grisly monument that I placed in the town square for all to see:

**[Kill TreeMap, TreeSet, TrieMap, TrieSet, LruCache and EnumSet](https://github.com/rust-lang/rust/pull/19955)**

Their fates were sealed, for my word was absolute. The other collections were horrified by my brutality, but they were not yet safe from their mother's wrath. I soon returned with two more tombstones:

**[Deprecate BitSet and BitVec](https://github.com/rust-lang/rust/pull/26034)**

The Bit twins were more cunning than their fallen comrades, but they lacked the strength to escape me. Most thought my work done, but I soon took one more: 

**[Deprecate VecMap](https://github.com/rust-lang/rust/pull/26734)**

VecMap had tried to survive through stealth &mdash; it was so small and inoffensive! But that wasn't enough for the libcollections I saw in my vision of the future.

I surveyed the land and saw what remained:

* Vec and VecDeque - hearty and simple, the heart of computing.
* HashMap and HashSet - powerful and wise, the brain of computing.
* BTreeMap and BTreeSet - awkward but necessary, the liver of computing.
* BinaryHeap - crafty and dextrous, the ankle of computing.

I nodded in contentment. Simple and effective. My work was don&mdash;

No, [DList](https://github.com/rust-lang/rust/blob/0a84308ebaaafb8fd89b2fd7c235198e3ec21384/src/libcollections/dlist.rs), it can't be! I thought you died in that tragic garbage collection incident! The one which was definitely an accident and not intentional at all!

They had faked their death and taken on a new name, but it was still them: LinkedList, the shadowy and untrustworthy schemer of computing. 

I spread word of their misdeeds to all that would hear me, but hearts were unmoved. LinkedList was a silver-tongued devil who had convinced everyone around me that it was some sort of fundamental and natural datastructure of computing. It had even convinced C++ that it was [*the* list](https://en.cppreference.com/w/cpp/container/list)!

"How could you have a standard library without a *LinkedList*?"

Easily! Trivially!

"It's non-trivial unsafe code, so it makes sense to have it in the standard library!"

So are GPU drivers and video codecs, libcollections is minimalist!

But alas, LinkedList had gathered too many allies and grown too strong while I was distracted with its kin.

I fled to my laboratory and tried to devise some sort of [evil clone](https://github.com/contain-rs/linked-list) or [enhanced cyborg replicant](https://github.com/contain-rs/blist) that could rival and destroy it, but my grant funding ran out because my research was "too murderously evil" or somesuch nonsense.

LinkedList had won. I was defeated and forced into exile.

But you're here now. You've come this far. Surely now you can understand the depths of LinkedList's debauchery! Come, I will you show you everything you need to know to help me destroy it once and for all &mdash; everything you need to know to implement an unsafe production-quality Doubly-Linked Deque.




[linked-list]: https://github.com/rust-lang/rust/blob/master/library/alloc/src/collections/linked_list.rs
