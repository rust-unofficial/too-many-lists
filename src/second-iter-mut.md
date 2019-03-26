# IterMut

I'm gonna be honest, IterMut is WILD. Which in itself seems like a wild
thing to say; surely it's identical to Iter!

Semantically, yes, but the nature of shared and mutable references means
that Iter is "trivial" while IterMut is Legit Wizard Magic.

The key insight comes from our implementation of Iterator for Iter:

```rust ,ignore
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> { /* stuff */ }
}
```

Which can be desugarred to:

```rust ,ignore
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next<'b>(&'b mut self) -> Option<&'a T> { /* stuff */ }
}
```

The signature of `next` establishes *no* constraint between the lifetime
of the input and the output! Why do we care? It means we can call `next`
over and over unconditionally!


```rust ,ignore
let mut list = List::new();
list.push(1); list.push(2); list.push(3);

let mut iter = list.iter();
let x = iter.next().unwrap();
let y = iter.next().unwrap();
let z = iter.next().unwrap();
```

Cool!

This is *definitely fine* for shared references because the whole point is that
you can have tons of them at once. However mutable references *can't* coexist.
The whole point is that they're exclusive.

The end result is that it's notably harder to write IterMut using safe
code (and we haven't gotten into what that even means yet...). Surprisingly,
IterMut can actually be implemented for many structures completely safely!

We'll start by just taking the Iter code and changing everything to be mutable:

```rust ,ignore
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&self) -> IterMut<'_, T> {
        IterMut { next: self.head.as_mut().map(|node| &mut **node) }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_mut().map(|node| &mut **node);
            &mut node.elem
        })
    }
}
```

```text
> cargo build
error[E0596]: cannot borrow `self.head` as mutable, as it is behind a `&` reference
  --> src/second.rs:95:25
   |
94 |     pub fn iter_mut(&self) -> IterMut<'_, T> {
   |                     ----- help: consider changing this to be a mutable reference: `&mut self`
95 |         IterMut { next: self.head.as_mut().map(|node| &mut **node) }
   |                         ^^^^^^^^^ `self` is a `&` reference, so the data it refers to cannot be borrowed as mutable

error[E0507]: cannot move out of borrowed content
   --> src/second.rs:103:9
    |
103 |         self.next.map(|node| {
    |         ^^^^^^^^^ cannot move out of borrowed content
```

Ok looks like we've got two different errors here. The first one looks really clear
though, it even tells us how to fix it! You can't upgrade a shared reference to a mutable
one, so `iter_mut` needs to take `&mut self`. Just a silly copy-paste error.

```rust ,ignore
pub fn iter_mut(&mut self) -> IterMut<'_, T> {
    IterMut { next: self.head.as_mut().map(|node| &mut **node) }
}
```

What about the other one?

Oops! I actually accidentally made an error when writing the `iter` impl in
the previous section, and we were just getting lucky that it worked!

We have just had our first run in with the magic of Copy. When we introduced [ownership][ownership] we
said that when you move stuff, you can't use it anymore. For some types, this
makes perfect sense. Our good friend Box manages an allocation on the heap for
us, and we certainly don't want two pieces of code to think that they need to
free its memory.

However for other types this is *garbage*. Integers have no
ownership semantics; they're just meaningless numbers! This is why integers are
marked as Copy. Copy types are known to be perfectly copyable by a bitwise copy.
As such, they have a super power: when moved, the old value *is* still usable.
As a consequence, you can even move a Copy type out of a reference without
replacement!

All numeric primitives in rust (i32, u64, bool, f32, char, etc...) are Copy.
You can also declare any user-defined type to be Copy as well, as long as
all its components are Copy.

Critically to why this code was working, shared references are also Copy!
Because `&` is copy, `Option<&>` is *also* Copy. So when we did `self.next.map` it
was fine because the Option was just copied. Now we can't do that, because
`&mut` isn't Copy (if you copied an &mut, you'd have two &mut's to the same
location in memory, which is forbidden). Instead, we should properly `take`
the Option to get it.


```rust ,ignore
fn next(&mut self) -> Option<Self::Item> {
    self.next.take().map(|node| {
        self.next = node.next.as_mut().map(|node| &mut **node);
        &mut node.elem
    })
}
```

```text
> cargo build

```

Uh... wow. Holy shit! IterMut Just Works!

Let's test this:


```rust ,ignore
#[test]
fn iter_mut() {
    let mut list = List::new();
    list.push(1); list.push(2); list.push(3);

    let mut iter = list.iter_mut();
    assert_eq!(iter.next(), Some(&mut 3));
    assert_eq!(iter.next(), Some(&mut 2));
    assert_eq!(iter.next(), Some(&mut 1));
}
```

```text
> cargo test

     Running target/debug/lists-5c71138492ad4b4a

running 6 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::iter_mut ... ok
test second::test::into_iter ... ok
test second::test::iter ... ok
test second::test::peek ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured

```

Yep. It works.

Holy shit.

What.

Ok I mean it actually *is* supposed to work, but there's usually something
stupid that gets in the way! Let's be clear here:

We have just implemented a piece of code that takes a singly-linked list, and
returns a mutable reference to every single element in the list at most once.
And it's statically verified to do that. And it's totally safe. And we didn't
have to do anything wild.

That's kind of a big deal, if you ask me. There are a couple reasons why
this works:

* We `take` the `Option<&mut>` so we have exclusive access to the mutable
  reference. No need to worry about someone looking at it again.
* Rust understands that it's ok to shard a mutable reference into the subfields
  of the pointed-to struct, because there's no way to "go back up", and they're
  definitely disjoint.

It turns out that you can apply this basic logic to get a safe IterMut for an
array or a tree as well! You can even make the iterator DoubleEnded, so that
you can consume the iterator from the front *and* the back at once! Woah!

[ownership]: first-ownership.md
