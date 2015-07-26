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
previous one. This will prevent the old list from recursively dropping the rest
of the list. This has the unfortunate problem that we traverse the *entire*
list whenever we drop it.

The second way is if we could identify that we're the last list that knows
about this node, we could in *principle* actually move the Node out of the Rc.
Then we could also know when to stop: whenver we *can't* hoist out the Node.
For reference, the unstable function is called `make_unique`.

Rc actually lets you do this... but only in nightly Rust. Honestly, I'd rather
risk blowing the stack sometimes than iterate every list whenever it gets
dropped. In particular this implies building a list is an O(n<sup>2</sup>)
operation. Still if you'd rather not blow the stack, here's the code:

```
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}
```
