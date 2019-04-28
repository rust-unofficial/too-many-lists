# Arc

One reason to use an immutable linked list is to share data across threads.
After all, shared mutable state is the root of all evil, and one way to solve
that is to kill the *mutable* part forever.

Except our list isn't thread-safe at all. In order to be thread-safe, we need
to fiddle with reference counts *atomically*. Otherwise, two threads could
try to increment the reference count, *and only one would happen*. Then the
list could get freed too soon!

In order to get thread safety, we have to use *Arc*. Arc is completely identical
to Rc except for the fact that reference counts are modified atomically. This
has a bit of overhead if you don't need it, so Rust exposes both.
All we need to do to make our list is replace every reference to Rc with
`std::sync::Arc`. That's it. We're thread safe. Done!

But this raises an interesting question: how do we *know* if a type is
thread-safe or not? Can we accidentally mess up?

No! You can't mess up thread-safety in Rust!

The reason this is the case is because Rust models thread-safety in a
first-class way with two traits: `Send` and `Sync`.

A type is *Send* if it's safe to *move* to another thread. A type is *Sync* if
it's safe to *share* between multiple threads. That is, if `T` is Sync, `&T` is
Send. Safe in this case means it's impossible to cause *data races*, (not to
be mistaken with the more general issue of *race conditions*).

These are marker traits, which is a fancy way of saying they're traits that
provide absolutely no interface. You either *are* Send, or you aren't. It's just
a property *other* APIs can require. If you aren't appropriately Send,
then it's statically impossible to be sent to a different thread! Sweet!

Send and Sync are also automatically derived traits based on whether you are
totally composed of Send and Sync types. It's similar to how you can only
implement Copy if you're only made of Copy types, but then we just go ahead
and implement it automatically if you are.

Almost every type is Send and Sync. Most types are Send because they totally
own their data. Most types are Sync because the only way to share data across
threads is to put them behind a shared reference, which makes them immutable!

However there are special types that violate these properties: those that have
*interior mutability*. So far we've only really interacted with *inherited
mutability* (AKA external mutability): the mutability of a value is inherited
from the mutability of its container. That is, you can't just randomly mutate
some field of a non-mutable value because you feel like it.

Interior mutability types violate this: they let you mutate through a shared
reference. There are two major classes of interior mutability: cells, which
only work in a single-threaded context; and locks, which work in a
multi-threaded context. For obvious reasons, cells are cheaper when you can
use them. There's also atomics, which are primitives that act like a lock.

So what does all of this have to do with Rc and Arc? Well, they both use
interior mutability for their *reference count*. Worse, this reference count
is shared between every instance! Rc just uses a cell, which means it's not
thread safe. Arc uses an atomic, which means it *is* thread safe. Of course,
you can't magically make a type thread safe by putting it in Arc. Arc can only
derive thread-safety like any other type.

I really really really don't want to get into the finer details of atomic
memory models or non-derived Send implementations. Needless to say, as you get
deeper into Rust's thread-safety story, stuff gets more complicated. As a
high-level consumer, it all *just works* and you don't really need to think
about it.
