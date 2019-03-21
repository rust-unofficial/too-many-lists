# Using Option

Particularly observant readers may have noticed that we actually reinvented
a really bad version of Option:

```rust ,ignore
enum Link {
    Empty,
    More(Box<Node>),
}
```

Link is just `Option<Box<Node>>`. Now, it's nice not to have to write
`Option<Box<Node>>` everywhere, and unlike `pop`, we're not exposing this
to the outside world, so maybe it's fine. However Option has some *really
nice* methods that we've been manually implementing ourselves. Let's *not*
do that, and replace everything with Options. First, we'll do it naively
by just renaming everything to use Some and None:

```rust ,ignore
use std::mem;

pub struct List {
    head: Link,
}

// yay type aliases!
type Link = Option<Box<Node>>;

struct Node {
    elem: i32,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem: elem,
            next: mem::replace(&mut self.head, None),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        match mem::replace(&mut self.head, None) {
            None => None,
            Some(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, None);
        while let Some(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, None);
        }
    }
}
```

This is marginally better, but the big wins will come from Option's methods.

First, `mem::replace(&mut option, None)` is such an incredibly
common idiom that Option actually just went ahead and made it a method: `take`.

```rust ,ignore
pub struct List {
    head: Link,
}

type Link = Option<Box<Node>>;

struct Node {
    elem: i32,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem: elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        match self.head.take() {
            None => None,
            Some(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}
```

Second, `match option { None => None, Some(x) => Some(y) }` is such an
incredibly common idiom that it was called `map`. `map` takes a function to
execute on the `x` in the `Some(x)` to produce the `y` in `Some(y)`. We could
write a proper `fn` and pass it to `map`, but we'd much rather write what to
do *inline*.

The way to do this is with a *closure*. Closures are anonymous functions with
an extra super-power: they can refer to local variables *outside* the closure!
This makes them super useful for doing all sorts of conditional logic. The
only place we do a `match` is in `pop`, so let's just rewrite that:

```rust ,ignore
pub fn pop(&mut self) -> Option<i32> {
    self.head.take().map(|node| {
        self.head = node.next;
        node.elem
    })
}
```

Ah, much better. Let's make sure we didn't break anything:

```text
> cargo test

     Running target/debug/lists-5c71138492ad4b4a

running 2 tests
test first::test::basics ... ok
test second::test::basics ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured

```

Great! Let's move on to actually improving the code's *behaviour*.
