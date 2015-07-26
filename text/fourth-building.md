% Building Up

Alright, we'll start with building the list. That's pretty straight-forward
with this new system. `new` is still trivial, just None out all the fields.
Also because it's getting a bit unwieldy, let's break out a Node constructor
too:

```rust
impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem: elem,
            prev: None,
            next: None,
        }))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }
}
```

```text
> cargo build
A BUNCH OF WARNINGS BUT IT BUILT
```

Yay!

Now let's try to write pushing onto the front of the list. Because
doubly-linked lists are signficantly more complicated, we're going to need
to do a fair bit more work. Where singly-linked list operations could be
reduced to an easy one-liner, doubly-linked list ops are fairly complicated.

An easy way for us to validate if our methods make sense is if we maintain
the following invariant: each node should have exactly two pointers to it.
Each node in the middle of the list is pointed at by its predecessor and
successor, while the nodes on the ends are pointed to by the list itself.



```
pub fn push_front(&mut self, elem: T) {
    let new_head = Node::new(elem);
    match self.head.take() {
        Some(old_head) => {
            // non-empty list, need to connect the old_head
            old_head.borrow_mut().prev = Some(new_head.clone());
            new_head.borrow_mut().next = Some(old_head);
            self.head = Some(new_head);
        }
        None => {
            // empty list, need to set the tail
            self.tail = Some(new_head.clone());
            self.head = Some(new_head);
        }
    }
}
```
