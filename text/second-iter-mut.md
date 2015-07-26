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

This is *fine* for shared references because the whole point is that you can
have tons of them at once. However mutable references *can't* coexist. The whole
point is that they're exclusive.

The end result is that it's *incredibly* hard to write an IterMut using safe
code (and we haven't gotten into what that even means yet...). Surprisingly,
IterMut can actually be implemented for List completely safely! This is the
only collection I've ever seen that can actually do this!

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
src/second.rs:104:9: 104:13 error: cannot move out of borrowed content
src/second.rs:104         self.next.map(|node| {
                          ^~~~
error: aborting due to previous error
```

Uh... That's not where I was expecting an error.

Oh. I'm an idiot. I actually accidentally made an error when writing the
the `iter` impl, but Copy saved the day. `&` is Copy, as we saw before. But
that also means `Option<&>` is *also* Copy. So when we did `self.next.map` it
was fine because the Option was just copied. Now we can't do that, because
`&mut` isn't Copy. Instead, we should properly `take` the Option to get it.


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
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

Uh... ok. I'm legitimately surprised that just compiled!

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

running 7 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::iter_mut ... ok
test second::test::into_iter ... ok
test second::test::iter ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

Yep. It works.

Holy shit.

This isn't supposed to work.

What.

Ok I mean it actually *is* supposed to work, but there's always something
stupid that gets in the way! IterMut properly works because Rust is actually
a little bit smart about mutable references. It understands that you can
"shard out" a mutable reference to disjoint subfields, since they can't alias.
It then concludes that it's totally fine to store one and return the other,
because you can't ever "go back up" with a mutable reference and see the
same element again!

Crazy!
