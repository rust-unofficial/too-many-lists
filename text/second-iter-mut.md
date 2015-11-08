% IterMut

I'm gonna be honest, IterMut is crazy. Which in itself seems like a crazy
thing to say; surely it's identical to Iter!

Semantically, yes. However the nature of shared and mutable references means
that Iter is "trivial" while IterMut is Legit Wizard Magic.

The key insight comes from our implementation of Iterator for Iter:

```rust
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> { /* stuff */ }
}
```

Which can be desugarred to:

```rust
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next<'b>(&'b mut self) -> Option<&'a T> { /* stuff */ }
}
```

The signature of `next` establishes *no* constraint between the lifetime
of the input and the output! Why do we care? It means we can call `next`
over and over unconditionally!


```rust
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

The end result is that it's notably harder to write an IterMut using safe
code (and we haven't gotten into what that even means yet...). Surprisingly,
IterMut can actually be implemented for many structures completely safely!
Borrow checking magic!

We'll start by just taking the Iter code and changing everything to be mutable:

```rust
pub struct IterMut<'a, T: 'a> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&self) -> IterMut<T> {
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
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/second.rs:96:25: 96:34 error: cannot borrow immutable field `self.head` as mutable
src/second.rs:96         IterMut { next: self.head.as_mut().map(|node| &mut **node) }
                                         ^~~~~~~~~
src/second.rs:104:9: 104:13 error: cannot move out of borrowed content
src/second.rs:104         self.next.map(|node| {
                          ^~~~
error: aborting due to previous error
```

Oops! I actually accidentally made an error when writing the
`iter` impl, but Copy saved the day. `&` is Copy, as we saw before. But
that also means `Option<&>` is *also* Copy. So when we did `self.next.map` it
was fine because the Option was just copied. Now we can't do that, because
`&mut` isn't Copy (if you copied an &mut, you'd have two &mut's to the same
location in memory, which is verboten. Instead, we should properly `take`
the Option to get it.


```rust
fn next(&mut self) -> Option<Self::Item> {
    self.next.take().map(|node| {
        self.next = node.next.as_mut().map(|node| &mut **node);
        &mut node.elem
    })
}
```

```text
> cargo build
src/second.rs:65:25: 65:34 error: cannot borrow immutable field `self.head` as mutable
src/second.rs:65         IterMut { next: self.head.as_mut().map(|node| &mut **node) }
                                         ^~~~~~~~~
error: aborting due to previous error
```

Uh... what? Looks we messed up mutability somewhere in `iter_mut`:

```rust
pub fn iter_mut(&self) -> IterMut<T> {
    IterMut { next: self.head.as_mut().map(|node| &mut **node) }
}
```

Classic copy-paste error. `self` is a shared reference! We can't
get mutable references out of that!

```rust
pub fn iter_mut(&mut self) -> IterMut<T> {
    IterMut { next: self.head.as_mut().map(|node| &mut **node) }
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

Uh... wow. Holy shit! IterMut Just Works!

Let's test this:


```rust
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
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
     Running target/debug/lists-5c71138492ad4b4a

running 6 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::iter_mut ... ok
test second::test::into_iter ... ok
test second::test::iter ... ok
test second::test::peek ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

Yep. It works.

Holy shit.

What.

Ok I mean it actually *is* supposed to work, but there's usually something
stupid that gets in the way! Let's be clear here:

We have just implemented a piece of code that takes a singly-linked list, and
returns a mutable reference to every single element in the list at most once.
And it's statically verified to do that. And it's totally safe. And we didn't
have to do anything crazy.

That's kind of a big deal, if you ask me. There are a couple reasons why
this works:

* We `take` the `Option<&mut>` so we have exclusive access to the mutable
  reference. No need to worry about someone looking at it again.
* Rust understands that it's ok to shard a mutable reference into the subfields
  of the pointed-to struct, because there's no way to "go back up", and they're
  definitely disjoint.

It turns out that you can apply this basic logic to get a safe IterMut for an
array or a tree as well! You can even make the iterator DoubleEnded, so that
you can consume the iterator from the front *and* the back at once! Wild!

