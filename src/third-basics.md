# Basics

We already know a lot of the basics of Rust now, so we can do a lot of the
simple stuff again.

For the constructor, we can again just copy-paste:

```rust ,ignore
impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }
}
```

`push` and `pop` don't really make sense anymore. Instead we can provide
`append` and `tail`, which provide approximately the same thing.

Let's start with appending. It takes a list and an element, and returns a
List. Like the mutable list case, we want to make a new node, that has the old
list as its `next` value. The only novel thing is how to *get* that next value,
because we're not allowed to mutate anything.

The answer to our prayers is the Clone trait. Clone is implemented by almost
every type, and provides a generic way to get "another one like this one" that
is logically disjoint given only a shared reference. It's like a copy
constructor in C++, but it's never implicitly invoked.

Rc in particular uses Clone as the way to increment the reference count. So
rather than moving a Box to be in the sublist, we just clone the head of the
old list. We don't even need to match on the head, because Option exposes a
Clone implementation that does exactly the thing we want.

Alright, let's give it a shot:

```rust ,ignore
pub fn append(&self, elem: T) -> List<T> {
    List { head: Some(Rc::new(Node {
        elem: elem,
        next: self.head.clone(),
    }))}
}
```

```text
> cargo build

warning: field is never used: `elem`
  --> src/third.rs:10:5
   |
10 |     elem: T,
   |     ^^^^^^^
   |
   = note: #[warn(dead_code)] on by default

warning: field is never used: `next`
  --> src/third.rs:11:5
   |
11 |     next: Link<T>,
   |     ^^^^^^^^^^^^^
```

Wow, Rust is really hard-nosed about actually using fields. It can tell no
consumer can ever actually observe the use of these fields! Still, we seem good
so far.

`tail` is the logical inverse of this operation. It takes a list and returns the
whole list with the first element removed. All that is is cloning the *second*
element in the list (if it exists). Let's try this:

```rust ,ignore
pub fn tail(&self) -> List<T> {
    List { head: self.head.as_ref().map(|node| node.next.clone()) }
}
```

```text
cargo build

error[E0308]: mismatched types
  --> src/third.rs:27:22
   |
27 |         List { head: self.head.as_ref().map(|node| node.next.clone()) }
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `std::rc::Rc`, found enum `std::option::Option`
   |
   = note: expected type `std::option::Option<std::rc::Rc<_>>`
              found type `std::option::Option<std::option::Option<std::rc::Rc<_>>>`
```

Hrm, we messed up. `map` expects us to return a Y, but here we're returning an
`Option<Y>`. Thankfully, this is another common Option pattern, and we can just
use `and_then` to let us return an Option.

```rust ,ignore
pub fn tail(&self) -> List<T> {
    List { head: self.head.as_ref().and_then(|node| node.next.clone()) }
}
```

```text
> cargo build

```

Great.

Now that we have `tail`, we should probably provide `head`, which returns a
reference to the first element. That's just `peek` from the mutable list:

```rust ,ignore
pub fn head(&self) -> Option<&T> {
    self.head.as_ref().map(|node| &node.elem )
}
```

```text
> cargo build

```

Nice.

That's enough functionality that we can test it:


```rust ,ignore
#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.append(1).append(2).append(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);

    }
}
```

```text
> cargo test

     Running target/debug/lists-5c71138492ad4b4a

running 5 tests
test first::test::basics ... ok
test second::test::into_iter ... ok
test second::test::basics ... ok
test second::test::iter ... ok
test third::test::basics ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured

```

Perfect!

Iter is also identical to how it was for our mutable list:

```rust ,ignore
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { next: self.head.as_ref().map(|node| &**node) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map(|node| &**node);
            &node.elem
        })
    }
}
```

```rust ,ignore
#[test]
fn iter() {
    let list = List::new().append(1).append(2).append(3);

    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&1));
}
```

```text
cargo test

     Running target/debug/lists-5c71138492ad4b4a

running 7 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::iter ... ok
test second::test::into_iter ... ok
test second::test::peek ... ok
test third::test::basics ... ok
test third::test::iter ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured

```

Who ever said dynamic typing was easier?

(chumps did)

Note that we can't implement IntoIter or IterMut for this type. We only have
shared access to elements.
