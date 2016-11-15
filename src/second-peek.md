# Peek

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

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

That's nice, but we didn't really test to see if we could mutate that `peek_mut` return value, did we?  If a reference is mutable but nobody mutates it, have we really tested the mutability?  Let's try using `map` on this `Option<&mut T>` to put a profound value in:

```rust
#[test]
fn peek() {
    let mut list = List::new();
    assert_eq!(list.peek(), None);
    assert_eq!(list.peek_mut(), None);
    list.push(1); list.push(2); list.push(3);

    assert_eq!(list.peek(), Some(&3));
    assert_eq!(list.peek_mut(), Some(&mut 3));
    list.peek_mut().map(|&mut value| {
        value = 42 });

    assert_eq!(list.peek(), Some(&4));
    assert_eq!(list.pop(), Some(42));
}
```

```text
> cargo test
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/second.rs:158:13: 158:23 error: re-assignment of immutable variable `value` [E0384]
src/second.rs:158             value = 42 });
                              ^~~~~~~~~~
src/second.rs:158:13: 158:23 help: run `rustc --explain E0384` to see a detailed explanation
src/second.rs:157:35: 157:40 note: prior assignment occurs here
src/second.rs:157         list.peek_mut().map(|&mut value| {
                                                    ^~~~~
error: aborting due to previous error
```

The compiler is complaining that `value` is immutable, but we pretty clearly wrote `&mut value`; what gives?  It turns out that writing the argument of the closure that way doesn't specify that `value` is a mutable reference but instead creates a pattern that will be matched against the argument to the closure; `|&mut value|` means "the argument is a mutable reference, but just stick the immutable value into `value`, please."  If we just use `|value|`, the type of `value` will be `&mut i32` and we can actually mutate the head:

```rust
    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1); list.push(2); list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        list.peek_mut().map(|value| {
            *value = 42 });

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }
```

```text
cargo test
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
     Running target/debug/lists-5c71138492ad4b4a

running 3 tests
test first::test::basics ... ok
test second::test::basics ... ok
test second::test::peek ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured

   Doc-tests lists

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

Much better!
