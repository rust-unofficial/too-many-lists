# Attempting To Understand Stacked Borrows

In the previous section we tried running our unsafe singly-linked queue under miri, and it said we had broken the rules of *stacked borrows*, and linked us some documentation.

Normally I'd give a guided tour of the docs, but we're not really the target audience of that documentation. It's more designed for compiler developers and academics who are working on the semantics of Rust. 

So I'm going to just give you the high level *idea* of "stacked borrows", and then give you a simple strategy for following the rules.

> **NARRATOR:** Stacked borrows are still "experimental" as a semantic model for Rust, so breaking these rules may not actually mean your program is "wrong". But unless you literally work on the compiler, you should just fix your program when miri complains. Better safe than sorry when it comes to Undefined Behaviour.



# The Motivation: Pointer Aliasing

Before we get into *what* rules we've broken, it will help to understand *why* the rules exist in the first place. There are a few different motivating problems, but I think the most important one is *pointer aliasing*.

We say two pointers *alias* when the pieces of memory they point to overlap. Just as someone who "goes by an alias" can be referred to by two different names, that overlapping piece of memory can be referred to by two different pointers. This can lead to problems.

The compiler uses information about pointer aliasing to optimize accesses to memory, so if the information it has is *wrong* then the program will be miscompiled and do random garbage. 

> **NARRATOR:** Practically speaking, aliasing is more concerned with memory accesses than the pointers themselves, and only really matters when one of the accesses is mutating. Pointers are emphasized because they're a convenient thing to attach rules to.

To understand why pointer aliasing information is important, let's consider *The Parable of the Tiny Angry Man*. 

----

Michiel was looking through their bookshelf one day when they saw a book they didn't remember. They pulled it from the bookcase and looked at the cover. 

"Oh yes, my old copy of *War and Peace*, a book I definitely have read. I loved the part with all the Peace."

Suddenly there was a knock at the door. Michiel returned the book to its shelf and opened the door -- it was their sworn nemesis **Hamslaw**. As Hamslaw prepared a devastating remark about Michiel's clearly inferior codegolfing skills, they sensed an opening: 

"Hey Hamslaw, have you ever read War and Peace?"

"Pfft, no one's *actually* read War and Peace."

"Well I have, look it's right there in my bookcase, which *obviously* means I've read it."

Hamslaw couldn't believe it. Her face shifted from its usual smug demeanor to an iron mask of rage and determination. Hamslaw pushed Michiel aside and power-walked to the book shelf, cleaving the tome from its resting place with the fury of a thousand Valkyries. She turned the ancient text over in her hands, and the instant she saw the cover she began to shake.

Michiel prepared to boast of their clearly unparalleled brilliance, but was interrupted by the sudden laughter of Hamslaw.

"This isn't War and Peace, this is War and *Feet*!"

Tears were rolling down Hamslaw's face. This was clearly the greatest moment of her life.

"N-no! I just looked at it!"

They grabbed the book from Hamslaw and checked the cover. Indeed, the word "Peace" had been scratched out and replaced with "Feet". Michiel was mortified. This was clearly the worst moment of their life.

They fell to their knees and stared blankly at the bookcase. How could this have happened? They had checked the cover only a moment ago!

And then they saw a bit of motion in the bookcase. It was a tiny man. A tiny man with the angriest scowl Michiel had ever seen. The tiny man flipped Michiel off and mouthed the words "no one will believe you" and disappeared back between the books.

Michiel's plan *had* been perfect, but they had failed to account for the possibility of a tiny angry man with a sharpie and the desire for destruction. They thought they knew what the cover of the book said, and they thought that no one could have possibly changed it. But alas, they were wrong.

Hamslaw was already working on a zine commemorating her incredible victory &mdash; Michiel's reputation at the local Internet Cafe would never recover.

----

No one wants to be like Michiel, but no one wants to live in constant fear of the tiny angry man either. We want to know when the tiny angry man could be playing tricks on us. When he is, we will be very careful and paranoid about checking everything before we use it. But when the tiny angry man is gone, we want to be able to remember things.

That's the (very simplified) crux of pointer aliasing: when can the compiler assume it's safe to "remember" (cache) values instead of loading them over and over? To know that, the compiler needs to know whenever there *could* be little angry men mutating the memory behind your back.

> **NARRATOR:** the compiler also uses this information to cache stores, which just means it can avoid committing things to memory if it thinks no one will notice. In this case the problem is still tiny angry men, but they only need to read the memory for it to be a problem.






# Safe Stacked Borrows

Ok so we want the compiler to have good pointer aliasing information, can we do that? Well, seemingly Rust is *designed* for it. Mutable references aren't aliased by definition, and although shared references *can* alias eachother, they can't mutate. Perfect! Ship it!

Except it's more complicated than that. We can "reborrow" mutable pointers like this:

```rust
let mut data = 10;
let ref1 = &mut data;
let ref2 = &mut *ref1;

*ref2 += 2;
*ref1 += 1;

println!("{}", data);
```

The compiles and runs fine. What's the deal? 

Well we can see what's going on by swapping the two uses:

```rust ,ignore
let mut data = 10;
let ref1 = &mut data;
let ref2 = &mut *ref1;

// ORDER SWAPPED!
*ref1 += 1;
*ref2 += 2;

println!("{}", data);
```

```text
error[E0503]: cannot use `*ref1` because it was mutably borrowed
 --> src/main.rs:6:5
  |
4 |     let ref2 = &mut *ref1;
  |                ---------- borrow of `*ref1` occurs here
5 |     
6 |     *ref1 += 1;
  |     ^^^^^^^^^^ use of borrowed `*ref1`
7 |     *ref2 += 2;
  |     ---------- borrow later used here

For more information about this error, try `rustc --explain E0503`.
error: could not compile `playground` due to previous error
```

It's suddenly a compiler error!

When we reborrow a mutable pointer, the original pointer can't be used anymore until the borrower is done with it (no more uses). 

In the code that works, there's a nice little nesting of the uses. We reborrow the pointer, use the new pointer for a while, and then stop using it before using the older pointer again. In the code that *doesn't* work, that doesn't happen. We just interleave the uses arbitrarily.

This is how we can have reborrows and still have aliasing information: all of our reborrows clearly nest, so we can consider only one of them "live" at any given time.

Hey, you know what's a great way to represent cleanly nested things? A stack. A stack of borrows.

Oh hey it's *Stacked Borrows*!

Whatever's at the top of the borrow stack is "live" and knows it's effectively unaliased. When you reborrow a pointer, the new pointer is pushed onto the stack, becoming *the* live pointer. When you use an older pointer it's brought back to life by popping everything on the borrow stack above it. At this point the pointer "knows" it was reborrowed and that the memory might have been modified, but that it once more has exclusive access -- no need to worry about little angry men.

So it's actually *always* ok to access a reborrowed pointer, because we can always pop everything above it. The real trouble is accessing a pointer that has already been popped off of the borrow stack -- then you've messed up.

Thankfully the design of the borrowchecker ensures that safe Rust programs follow these rules, as we saw in the above example, but the compiler generally views this problem "backwards" from the stacked borrows perspective. Instead of saying using `ref1` invalidates `ref2`, it insists that `ref2` *must* be valid for all its uses, and that `ref1` is the one messing things up by going out of turn.

Hence "cannot use `*ref1` because it was mutably borrowed". It's the same result (especially with non-lexical lifetimes), but framed in a way that's probably more intuitive.

But the borrowchecker can't help us when we start using unsafe pointers!





# Unsafe Stacked Borrows

So we want to somehow have a way for unsafe pointers to participate in this stacked borrows system, even though the compiler can't track them properly. And we also want the system to be fairly permissive so that it's not *too* easy to mess it up and cause UB.

That's a hard problem, and I don't know how to solve it, but the folks who worked on Stacked Borrows came up with something plausible, and miri tries to implement it.

The very high-level concept is that when you convert a reference (or any other safe pointer) into an raw pointer it's *basically* like taking a reborrow. So now the raw pointer is allowed to do whatever it wants with that memory, and when the reborrow expires it's just like when that happens with normal reborrows.

But the question is, when does that reborrow expire? Well, probably a good time to expire it is when you start using the original reference again. Otherwise things aren't a nice nested stack.

But wait, you can turn a raw pointer *into* a reference! And you can copy raw pointers! What if you go `&mut -> *mut -> &mut -> *mut` and then access the first `*mut`? How the heck do the stacked borrows work then?

I genuinely don't know! That's why things are complicated. In fact they're *extra* complicated because stacked borrows are *trying* to be more permissive and let more unsafe code work the way you'd expect it to. This is why I run things under miri to try to help me catch mistakes.

In fact, this messiness is why there is an extra-experimental extra-strict mode of miri: `-Zmiri-tag-raw-pointers`.

To enable it, we need to pass it via a MIRIFLAGS environment variable like this:

```text
MIRIFLAGS="-Zmiri-tag-raw-pointers" cargo +nightly-2022-01-21 miri test
```

Or like this on Windows, where you need to just set the variable globally:

```text
$env:MIRIFLAGS="-Zmiri-tag-raw-pointers"
cargo +nightly-2022-01-21 miri test
```

We'll generally be trying to conform to this extra-strict mode just to be *extra* confident in our work. It's also in some sense "simpler", so it's actually better for messing around and getting an intuition for stacked borrows.




# Managing Stacked Borrows

So when using raw pointers we're going to try to stick to a heuristic that's simple and blunt and will hopefully have a large margin of error: 

**Once you start using raw pointers, try to ONLY use raw pointers.**

This makes it as unlikely as possible to accidentally lose the raw pointer's "permission" to access the memory.

> **NARRATOR:** this is oversimplified in two regards:
>
> 1. Safe pointers often assert more properties than just aliasing: the memory is allocated, it's aligned, it's large enough to fit the type of the pointee, the pointee is properly initialized, etc. So it's even more dangerous to wildly throw them around when things are in a dubious state.
>
> 2. Even if you stay in raw pointer land, you can't just wildly alias any memory. Pointers are conceptually tied to specific "allocations" (which can be as granular as a local variable on the stack), and you're not supposed to take a pointer from one allocation, offset it, and then access memory in a different allocation. If this was allowed, there would *always* be the threat of tiny angry men *everywhere*. This is part of the reason "pointers are just integers" is a *problematic* viewpoint.

Now, we still want safe references in our *interface*, because we want to build a nice *safe abstraction* so the user of our list doesn't have to know or worry about. 

So what we're going to do is:

1. At the start of a method, use the input references to get our raw pointers
2. Do our best to only use unsafe pointers from this point on
3. Convert our raw pointers back to safe pointers at the end if needed

But the fields of our types are private so we're going to keep those *entirely* as raw pointers.

In fact, part of the big mistake we made was continuing to use Box! Box has a special annotation in it that tells the compiler "hey this is a lot like `&mut`, because it uniquely owns that pointer". Which is true!

But the raw pointer we were keeping to the end of the list was pointing into a Box, so whenever we access the Box normally we're probably invalidating that raw pointer's "reborrow"! ☠

In the next section we'll return to our true form and hit our heads against a bunch of examples.


