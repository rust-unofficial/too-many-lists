# An Introduction To Cursors

OK!!! We now have a LinkedList that's on par with std's 1.0 implementation! Which of course means that our LinkedList is *still completely useless*. We've taken the enormous performance penalty of implementing a Deque as a linked list, **and we don't have any of the APIs that make it actually useful**. 

Here's how we do against the "killer apps" of linked lists:

* 🚫 Getting to do [weird intrusive stuff](https://docs.rs/linked-hash-map/latest/linked_hash_map/)
* 🚫 Getting to do [weird lockfree stuff](https://doc.rust-lang.org/std/sync/mpsc/)
* 🚫 Getting to store [Dynamically Sized Types](https://doc.rust-lang.org/nomicon/exotic-sizes.html#dynamically-sized-types-dsts)
* 🌟 O(1) push/pop without [amortization](https://en.wikipedia.org/wiki/Amortized_analysis) (if you are willing to believe that malloc is O(1))
* 🚫 O(1) list splitting
* 🚫 O(1) list splicing

Well... 1 out of 6 is... better than nothing! Do you see why I wanted to rip this thing out of std?

We're not going to make our list support "weird" stuff, because that's all adhoc and domain-specific. But the splitting and splicing thing, now that's something we can do!

But here's the problem: actually *reaching* the k<sup>th</sup> element in a LinkedList takes O(k) time, so how can we *possibly* do arbitrary splits and merges in O(1)? Well, the trick is that you don't have an API like `split_at(index)` -- you make a system where the user can statefully iterate to a position in the list and make O(1) modifications at that point!

Hey, we already have iterators! Can we use them for this? Kind of... but one of their super-powers gets in the way. You may recall that the way that we write out the lifetimes for by-ref iterators means that the references they return *aren't* tied to the iterator. This lets us repeatedly call `next` and hold onto the elements:

```rust ,ignore
let mut list = ...;
let iter = list.iter_mut();
let elem1 = list.next();
let elem2 = list.next();

if elem1 == elem2 { ... }
```

If the returned references borrowed the iterator, then this code wouldn't work at all. The compiler would just complain about the second call to `next`! This flexibility is great, but it puts some implicit constraints on us:

* By-Mutable-Ref Iterators can never go backwards and yield an element again, because the user would be able to get two `&mut`'s to the same element, breaking fundamental rules of the language.

* By-Ref Iterators can't have extra methods which could possibly modify the underlying collection in a way that would invalidate any reference that has already been yielded.

Unfortunately, both of these things are *exactly* what we want our LinkedList API to do! So we can't just use iterators, we need something new: *Cursors*.

Cursors are exactly like the little blinking `|` you get when you're editing some text on a computer. It's a position in a sequence (the text) that you can move around (with the arrow keys), and whenever you type the edits happen at that point.

See if I just

press

enter

the whole

text

gets broken in half.

Sorry you're standing behind me and watching me type this right? So that totally makes sense, right? Right.

Now if you've ever had the misfortune of having a keyboard with an "insert" key and actually pressed it, you know that there's actually technically two interpretations of cursors: they can either lie between elements (characters) or *on* elements. I'm pretty sure no one has ever pressed "insert" on purpose in their life, and that it exists purely as a Suffering Button, so it's pretty obvious which one is Better and Right: cursors go between elements!

Pretty rock-solid logic right there, I don't think anyone can disagree with me.

Sorry what? There was an [RFC in 2018 to add Cursors to Rust's LinkedList](https://github.com/rust-lang/rfcs/blob/master/text/2570-linked-list-cursors.md)?

> With a Cursor one can seek back and forth through a list and get the current element. With a CursorMut One can seek back and forth and get mutable references to elements, and it can insert and delete elements before and behind the current element (along with performing several list operations such as splitting and splicing).

*Current element*? This cursor is *on* elements, not between them! I can't believe they didn't accept my totally rock-solid argument! So yeah you can just go use the Cursor in std... wait, it's [2023, and Rust 1.71 still has Cursor marked as unstable](https://doc.rust-lang.org/1.71.0/std/collections/linked_list/struct.CursorMut.html)?

Hey wait:

> Cursors always rest between two elements in the list, and index in a logically circular way. To accommodate this, there is a "ghost" non-element that yields None between the head and tail of the list.

HEY WAIT. This is the opposite of what the RFC says??? But wait all the docs on the methods still refer to "current" elements... wait hold on, where have I seen this ghost stuff before. Oh wait, didn't I do that in [my old linked-list fork](https://docs.rs/linked-list/0.0.3/linked_list/struct.Cursor.html) where I prototyped?

> Cursors always rest between two elements in the list, and index in a logically circular way. To accomadate this, there is a "ghost" non-element that yields None between the head and tail of the List.

Hold up what the fuck. This isn't a gag, I am actually trying to Read The Docs right now. Did std actually RFC a different design from the one I proposed in 2015, but then copy-paste the docs from my prototype??? Is std meta-shitposting me for writing a book about how much I hate LinkedList????? Like yeah I built that prototype to demonstrate the concept so that people would let me add it to std and make LinkedList not useless but, qu'est-ce que le fuck??????????????

Ok you know what, clearly std is blessing my design as the objectively superior one, so we're going to do my design. Also that's nice because this entire chapter is me actually literally rewriting that library from scratch, so not changing the API sounds Good To Me!

Here's the full top-level docs I wrote:

> A Cursor is like an iterator, except that it can freely seek back-and-forth, and can safely mutate the list during iteration. This is because the lifetime of its yielded references are tied to its own lifetime, instead of just the underlying list. This means cursors cannot yield multiple elements at once.
>
> Cursors always rest between two elements in the list, and index in a logically circular way. To accomadate this, there is a "ghost" non-element that yields None between the head and tail of the List.
>
> When created, cursors start between the ghost and the front of the list. That is, next will yield the front of the list, and prev will yield None. Calling prev again will yield the tail.

Cute, even though we concluded that the whole "sentinel-node" thing was more trouble than it's worth, we're still going to end up with semantics that "pretend" there's a sentinel node so that the cursor can wrap around to the other side of the list.

*Skims over my old APIs some more*

```rust ,ignore
fn splice(&mut self, other: &mut LinkedList<T>)
```

> Inserts the entire list's contents right after the cursor.

Oh yeah, this is coming back to me. I wrote this when I was really mad about combinatoric explosion, and was trying to come up with a way for there to only be one copy of each operation. Unfortunately this is... semantically problematic. See, when the user wants to splice one list into another, they might want the cursor to end up *before* the splice or *after it*. The inserted list can be arbitrarily large, so it's a genuine issue for us to only allow for one and expect the user to walk over the entire inserted list!

We're gonna have to rework this design from the ground up after all. What does our Cursor type need? Well it needs to:

* point "between" two elements
* as a nice little feature, keep track of what "index" is next
* update the list itself to modify front/back/len. 

How do you point between two elements? Well, you don't. You just point at the "next" element. So, yeah even though we're exposing "cursor goes in-between" semantics, we're really implementing it as "cursor is on", and just pretending everything happens before or after that point.

But there's a reason! The splice use-case wants to let the user choose whether they end up before or after the list, but this is... *horribly* complicated to express with the std API! They have splice_after and splice_before, but neither changes the cursor's position, so really you'd need splice_after_before and splice_after_after...

Wait no I'm being silly. In the std API you can just choose the node you want to end up on, and then use splice_after/before as appropriate.

*squints*

Wait is the std API actually good.

*skims through the code*

Ok the std API is actually good.

Alright screw it, we're going to [implement the RFC](https://github.com/rust-lang/rfcs/blob/master/text/2570-linked-list-cursors.md). Or at least the interesting parts of it.

I have my quibbles with some of the terminology std uses, but cursors are always going to be a bit brain-melty: `iter().next_back()`  gets you `back()`, so that's good, but then each subsequent `next_back()` is actually bringing you *closer to the front* and indeed, every pointer we follow is a "front" pointer! If I think about this seeming-paradox too much it hurts my brain, so, I can certainly respect going for different terminology to avoid this.

The std API talks about operations before "before" (towards the front) and "after" (towards the back), and instead of `next` and `next_back`, it... calls things `move_next` and `move_prev`. HRM. Ok so they're getting into a bit of the iterator terminology, but at least `next` doesn't evoke front/back, and helps you orient how things behave compared to the iterators.

We can work with this.
