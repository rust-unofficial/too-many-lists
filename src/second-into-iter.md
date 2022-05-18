# IntoIter

Collections are iterated in Rust using the *Iterator* trait. It's a bit more
complicated than `Drop`:

```rust ,ignore
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

The new kid on the block here is `type Item`. This is declaring that every
implementation of Iterator has an *associated type* called Item. In this case,
this is the type that it can spit out when you call `next`.

The reason Iterator yields `Option<Self::Item>` is because the interface
coalesces the `has_next` and `get_next` concepts. When you have the next value,
you yield
`Some(value)`, and when you don't you yield `None`. This makes the
API generally more ergonomic and safe to use and implement, while avoiding
redundant checks and logic between `has_next` and `get_next`. Nice!

Sadly, Rust has nothing like a `yield` statement (yet), so we're going to have to
implement the logic ourselves. Also, there's actually 3 different kinds of
iterator each collection should endeavour to implement:

* IntoIter - `T`
* IterMut - `&mut T`
* Iter - `&T`

We actually already have all the tools to implement
IntoIter using List's interface: just call `pop` over and over. As such, we'll
just implement IntoIter as a newtype wrapper around List:


```rust ,ignore
// Tuple structs are an alternative form of struct,
// useful for trivial wrappers around other types.
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // access fields of a tuple struct numerically
        self.0.pop()
    }
}
```

And let's write a test:

```rust ,ignore
#[test]
fn into_iter() {
    let mut list = List::new();
    list.push(1); list.push(2); list.push(3);

    let mut iter = list.into_iter();
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), None);
}
```

```text
> cargo test

     Running target/debug/lists-5c71138492ad4b4a

running 4 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::into_iter ... ok
test second::test::peek ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured

```

Nice!

For the sake of curiosity, let's try to iterate twice over the list:

```rust ,ignore
#[test]
fn into_iter() {
    let mut list = List::new();
    list.push(1); list.push(2); list.push(3);

    let mut iter = list.into_iter();
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), None);

    let mut iter = list.into_iter();
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), None);
}
```

```text
> cargo build
error[E0382]: use of moved value: `list`
   --> src/second.rs:167:24
    |
158 |         let mut list = List::new();
    |             -------- move occurs because `list` has type `second::List<i32>`, which does not implement the `Copy` trait
...
161 |         let mut iter = list.into_iter();
    |                             ----------- `list` moved due to this method call
...
167 |         let mut iter = list.into_iter();
    |                        ^^^^ value used here after move
    |
note: this function takes ownership of the receiver `self`, which moves `list`
   --> src/second.rs:45:22
    |
45  |     pub fn into_iter(self) -> IntoIter<T> {
    |                      ^^^^
```

This is the cost of `into_iter`. In order for `next` to return values and not
references, to give ownership to the result to the caller, our `into_iter` had
to take ownership of the list; which mean the list can't be used anymore. Which
is exactly what we do in the next section.