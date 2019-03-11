# Unsafe Rust

This is a serious, big, complicated, and dangerous topic.
It's so serious that I wrote [an entire other book][nom] on it.

The long and the short of it is that *every* language is actually unsafe as soon
as you allow calling into other languages, because you can just have C do
arbitrarily bad things. Yes: Java, Python, Ruby, Haskell... everyone is wildly
unsafe in the face of Foreign Function Interfaces (FFI).

Rust embraces this truth by splitting itself into two languages: Safe Rust, and
Unsafe Rust. So far we've only worked with Safe Rust. It's completely 100%
safe... except that it can FFI into Unsafe Rust.

Unsafe Rust is a *superset* of Safe Rust. It's completely the same as Safe Rust in all its
semantics and rules, you're just allowed to do a few *extra* things that are
wildly unsafe and can cause the dreaded Undefined Behaviour that haunts C.

Again, this is a really huge topic that has a lot of interesting corner cases.
I *really* don't want to go really deep into it (well, I do. I did. [Read that
book][nom]). That's ok, because with linked lists we can actually ignore almost
all of it.

The main Unsafe tool we'll be using are *raw pointers*. Raw pointers are
basically C's pointers. They have no inherent aliasing rules. They have no
lifetimes. They can be null. They can be dangling. They can point to
uninitialized memory. They can be cast to and from integers. They can be cast
to point to a different type. Mutability? Cast it. Pretty much everything goes,
and that means pretty much anything can go wrong.

This is some bad stuff and honestly you'll live a happier life never having
to touch these. Unfortunately, we want to write linked lists, and linked lists
are awful. That means we're going to have to use unsafe pointers.

There are two kinds of raw pointer: `*const T` and `*mut T`. These are meant to
be `const T*` and `T*` from C, but we really don't care about what C thinks they
mean that much. You can only dereference a `*const T` to an `&T`, but much like
the mutability of a variable, this is just a lint against incorrect usage. At
most it just means you have to cast the `*const` to a `*mut` first. Although if
you don't actually have permission to mutate the referrent of the pointer,
you're gonna have a bad time.

Anyway, we'll get a better feel for this as we write some code. For now,
`*mut T == &unchecked mut T`!

[nom]: https://doc.rust-lang.org/nightly/nomicon/
