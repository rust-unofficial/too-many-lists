% Peek

One thing we didn't even bother to implement last time was peeking. Let's go
ahead and do that. All we need to do is return a reference to the element in
the head of the list, if it exists. Sounds easy, let's try:

```
pub fn peek(&self) -> Option<&T> {
    self.head.map(|node| {
        &node.elem
    })
}
```


```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/second.rs:45:9: 45:18 error: cannot move out of type `second::List<T>`, which defines the `Drop` trait
src/second.rs:45         self.head.map(|node| {
                         ^~~~~~~~~
src/second.rs:46:14: 46:23 error: `node.elem` does not live long enough
src/second.rs:46             &node.elem
                              ^~~~~~~~~
note: in expansion of closure expansion
src/second.rs:45:23: 47:10 note: expansion site
src/second.rs:44:38: 48:6 note: reference must be valid for the anonymous lifetime #1 defined on the block at 44:37...
src/second.rs:44     pub fn peek(&self) -> Option<&T> {
src/second.rs:45         self.head.map(|node| {
src/second.rs:46             &node.elem
src/second.rs:47         })
src/second.rs:48     }
src/second.rs:45:30: 47:10 note: ...but borrowed value is only valid for the scope of parameters for function at 45:29
src/second.rs:45         self.head.map(|node| {
src/second.rs:46             &node.elem
src/second.rs:47         })
error: aborting due to 2 previous errors
```

*Sigh*. What now, Rust?

Map takes `self` by value, which would move the Option out of the thing it's in.
Previously this was fine because we had just `take`n it out, but now we actually
want to leave it where it was. The *correct* way to handle this is with the
`as_ref` method on Option, which has the following definition:

```rust
impl<T> Option<T> {
    pub fn as_ref(&self) -> Option<&T>;
}
```

It demotes the Option<T> to an Option to a reference to its internals. We could
do this ourselves with an explicit match but *ugh no*. It does mean that we
need to do an extra derefence to cut through the extra indirection, but
thankfully the `.` operator handles that for us.


```rust
pub fn peek(&self) -> Option<&T> {
    self.head.as_ref().map(|node| {
        &node.elem
    })
}
```

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

Nailed it.

We can also make a *mutable* version of this method using `as_mut`:

```rust
pub fn peek_mut(&mut self) -> Option<&mut T> {
    self.head.as_mut().map(|node| {
        &mut node.elem
    })
}
```

```text
lists::cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

EZ

Don't forget to test it:

```rust
#[test]
fn peek() {
    let mut list = List::new();
    assert_eq!(list.peek(), None);
    assert_eq!(list.peek_mut(), None);
    list.push(1); list.push(2); list.push(3);

    assert_eq!(list.peek(), Some(&3));
    assert_eq!(list.peek_mut(), Some(&mut 3));
}
```


```
cargo test
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
     Running target/debug/lists-5c71138492ad4b4a

running 3 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::peek ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```
