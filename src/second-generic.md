# Making it all Generic

We've already touched a bit on generics with Option and Box. However so
far we've managed to avoid declaring any new type that is actually generic
over arbitrary elements.

It turns out that's actually really easy. Let's make all of our types generic
right now:

```rust ,ignore
pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}
```

You just make everything a little more pointy, and suddenly your code is
generic. Of course, we can't *just* do this, or else the compiler's going
to be Super Mad.


```text
> cargo test

error[E0107]: wrong number of type arguments: expected 1, found 0
  --> src/second.rs:14:6
   |
14 | impl List {
   |      ^^^^ expected 1 type argument

error[E0107]: wrong number of type arguments: expected 1, found 0
  --> src/second.rs:36:15
   |
36 | impl Drop for List {
   |               ^^^^ expected 1 type argument

```

The problem is pretty clear: we're talking about this `List` thing but that's not
real anymore. Like Option and Box, we now always have to talk about
`List<Something>`.

But what's the Something we use in all these impls? Just like List, we want our
implementations to work with *all* the T's. So, just like List, let's make our
`impl`s pointy:


```rust ,ignore
impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}
```

...and that's it!


```
> cargo test

     Running target/debug/lists-5c71138492ad4b4a

running 2 tests
test first::test::basics ... ok
test second::test::basics ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured

```

All of our code is now completely generic over arbitrary values of T. Dang,
Rust is *easy*. I'd like to make a particular shout-out to `new` which didn't
even change:

```rust ,ignore
pub fn new() -> Self {
    List { head: None }
}
```

Bask in the Glory that is Self, guardian of refactoring and copy-pasta coding.
Also of interest, we don't write `List<T>` when we construct an instance of
list. That part's inferred for us based on the fact that we're returning it
from a function that expects a `List<T>`.

Alright, let's move on to totally new *behaviour*!
