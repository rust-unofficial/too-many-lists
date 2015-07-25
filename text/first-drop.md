% Drop

We can make a stack, push on to, pop off it, and we've even tested that it all
works right!

Do we need to worry about cleaning up our list? Technically, no, not at all!
Like C++, Rust uses destructors to automatically clean up resources when they're
done with. A type has a destructor if it implements a *trait* called Drop.
Traits are Rust's fancy term for interfaces. The Drop trait has the following
interface:

```
pub trait Drop {
    fn drop(&mut self);
}
```

Basically, "when you go out of scope, I'll give you a second to clean up your
affairs".

You don't actually need to implement Drop if you contain types that implement
Drop, and all you'd want to do is call *their* destructors. In the case of
List, all it would want to do is drop its head, which in turn would *maybe*
try to drop a `Box<Node>`. All that's handled for us automatically... with one
hitch.

The automatic handling is going to be bad.

Let's consider a simple list:


```text
list -> A -> B -> C
```

When `list` gets dropped, it will try to drop A, which will try to drop B,
which will try to drop C. Some of you might rightly be getting nervous. This is
recursive code, and recursive code can blow the stack!

Some of you might be thinking "this is clearly tail recursive, and any decent
language would ensure that such code wouldn't blow the stack". This is, in fact,
incorrect! To see why, let's try to write what the compiler has to do, by
manually implementing Drop for our List as the compiler would:


```rust
impl Drop for List {
    fn drop(&mut self) {
        // NOTE: you can't actually explicitly call `drop` in real Rust code;
        // we're pretending to be the compiler!
        list.head.drop(); // tail recursive - good!
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        match list.head {
            Link::Empty => {} // Done!
            Link::More(ref mut boxed_node) => {
                boxed_node.drop(); // tail recursive - good!
            }
        }
    }
}

impl Drop for Box<Node> {
    fn drop(&mut self) {
        self.ptr.drop(); // uh oh, not tail recursive!
        deallocate(self.ptr);
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.next.drop();
    }
}
```

We *can't* drop the contents of the Box *after* deallocating, so there's no
way to drop in a tail-recursive manner! Instead we're going to have to manually
write an iterative drop for `List` that hoists nodes out of their boxes.


```rust
impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        // `while let` == "do this thing until this pattern doesn't match"
        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
            // boxed_node goes out of scope and gets dropped here;
            // but its Node's `next` field has been set to Link::Empty
            // so no unbounded recursion occurs.
        }
    }
}
```

```text
> cargo test
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
     Running target/debug/lists-5c71138492ad4b4a

running 1 test
test first::test::basics ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

Great!
