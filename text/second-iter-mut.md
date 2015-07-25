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
point is that they're exclusive. The end result is that *we can't write
IterMut*. Well, we can. We just haven't seen how yet. And we won't for a long
time, either. IterMut *requires unsafe code* to write, which isn't something
we'll get to for a couple entire lists.

So for now... no IterMut!
