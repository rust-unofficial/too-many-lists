% Drop

Like the mutable lists, we have a the recursive destructor problem.
Admittedly, this isn't as bad of a problem for the immutable list: if we ever
hit another node that's the head of another list *somewhere*, we won't
recursively drop it. However it's still a thing we should care about, and
how to deal with isn't as clear. Here's how we solved it before:

```rust
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}
```

The problem is the body of the loop:

```rust
cur_link = boxed_node.next.take();
```

This is mutating the Node inside the Box, but we can't do that with Rc; it only
gives us shared access. There's two ways to handle this.

The first way is that we can keep grabbing the tail of the list and dropping the
previous one to decrement its count. This will prevent the old list from
recursively dropping the rest of the list because we hold an outstanding
reference to it. This has the unfortunate problem that we traverse the *entire*
list whenever we drop it. In particular this means building a list of length
n in place takes O(n<sup>2</sup>) as we traverse a lists of length `n-1`,
`n-2`, .., `1` to guard against overflow (this is really really really
really bad).

The second way is if we could identify that we're the last list that knows
about this node, we could in *principle* actually move the Node out of the Rc.
Then we could also know when to stop: whenver we *can't* hoist out the Node.
For reference, the unstable function is called `try_unwrap`.

Rc actually lets you do this... but only in nightly Rust. Honestly, I'd rather
risk blowing the stack sometimes than iterate every list whenever it gets
dropped. Still if you'd rather not blow the stack, here's the first
(O(n)) solution:

```rust
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // Steal the list's head
        let mut cur_list = self.head.take();
        while let Some(node) = cur_list {
            // Clone the current node's next node.
            cur_list = node.next.clone();
            // Node dropped here. If the old node had
            // refcount 1, then it will be dropped and freed, but it won't
            // be able to fully recurse and drop its child, because we
            // hold another Rc to it.
        }
    }
}
```

and here's the second (amortized O(1)) solution (only works on nightly):

```rust
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}
```

