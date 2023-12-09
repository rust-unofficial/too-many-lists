# Send, Sync, and Compile Tests

Ok actually we do have one more pair of traits to think about, but they're special. We have to deal with Rust's Holy Roman Empire: The Unsafe Opt-In Built-In Traits (OIBITs): [Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html), which are in fact opt-out and built-out (1 out of 3 is pretty good!).

Like Copy, these traits have absolutely no code associated with them, and are just markers that your type has a particular property. Send says that your type is safe to send to another thread. Sync says your type is safe to share between threads (&Self: Send).

The same argument for LinkedList being covariant applies here: generally normal collections which don't use fancy interior mutability tricks are safe to make Send and Sync.

But I said they're *opt out*. So actually, are we already? How would we know?

Let's add some new magic to our code: random private garbage that won't compile unless our types have the properties we expect:  

```rust ,ignore
#[allow(dead_code)]
fn assert_properties() {
    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    is_send::<LinkedList<i32>>();
    is_sync::<LinkedList<i32>>();

    is_send::<IntoIter<i32>>();
    is_sync::<IntoIter<i32>>();

    is_send::<Iter<i32>>();
    is_sync::<Iter<i32>>();

    is_send::<IterMut<i32>>();
    is_sync::<IterMut<i32>>();

    fn linked_list_covariant<'a, T>(x: LinkedList<&'static T>) -> LinkedList<&'a T> { x }
    fn iter_covariant<'i, 'a, T>(x: Iter<'i, &'static T>) -> Iter<'i, &'a T> { x }
    fn into_iter_covariant<'a, T>(x: IntoIter<&'static T>) -> IntoIter<&'a T> { x }
}
```

```text
cargo build
   Compiling linked-list v0.0.3 
error[E0277]: `NonNull<Node<i32>>` cannot be sent between threads safely
   --> src\lib.rs:433:5
    |
433 |     is_send::<LinkedList<i32>>();
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ `NonNull<Node<i32>>` cannot be sent between threads safely
    |
    = help: within `LinkedList<i32>`, the trait `Send` is not implemented for `NonNull<Node<i32>>`
    = note: required because it appears within the type `Option<NonNull<Node<i32>>>`
note: required because it appears within the type `LinkedList<i32>`
   --> src\lib.rs:8:12
    |
8   | pub struct LinkedList<T> {
    |            ^^^^^^^^^^
note: required by a bound in `is_send`
   --> src\lib.rs:430:19
    |
430 |     fn is_send<T: Send>() {}
    |                   ^^^^ required by this bound in `is_send`

<a million more errors>
```

Oh geez, what gives! I had that great Holy Roman Empire joke!

Well, I lied to you when I said raw pointers have only one safety guard: this is the other. `*const` AND `*mut` explicitly opt out of Send and Sync to be safe, so we do *actually* have to opt back in:

```rust ,ignore
unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

unsafe impl<'a, T: Send> Send for Iter<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}
```

Note that we have to write *unsafe impl* here: these are *unsafe traits*! Unsafe code (like concurrency libraries) gets to rely on us only implementing these traits correctly! Since there's no actual code, the guarantee we're making is just that, yes, we are indeed safe to Send or Share between threads!

Don't just slap these on lightly, but I am a Certified Professional here to say: yep there's are totally fine. Note how we don't need to implement Send and Sync for IntoIter: it just contains LinkedList, so it auto-derives Send and Sync &mdash; I told you they were actually opt out! (You opt out with the hillarious syntax of `impl !Send for MyType {}`.)

```text
cargo build
   Compiling linked-list v0.0.3
    Finished dev [unoptimized + debuginfo] target(s) in 0.18s
```

Ok nice!

...Wait, actually it would be really dangerous if stuff that *shouldn't* be these things wasn't. In particular, IterMut *definitely* shouldn't be covariant, because it's "like" `&mut T`. But how can we check that?

With Magic! Well, actually, with rustdoc! Ok well we don't have to use rustdoc for this, but it's the funniest way to do it. See, if you write a doccomment and include a code block, then rustdoc will try to compile and run it, so we can use that to make fresh anonymous "programs" that don't affect the main one:


```rust ,ignore
    /// ```
    /// use linked_list::IterMut;
    /// 
    /// fn iter_mut_covariant<'i, 'a, T>(x: IterMut<'i, &'static T>) -> IterMut<'i, &'a T> { x }
    /// ```
    fn iter_mut_invariant() {}
```

```text
cargo test

...

   Doc-tests linked-list

running 1 test
test src\lib.rs - assert_properties::iter_mut_invariant (line 458) ... FAILED

failures:

---- src\lib.rs - assert_properties::iter_mut_invariant (line 458) stdout ----
error[E0308]: mismatched types
 --> src\lib.rs:461:86
  |
6 | fn iter_mut_covariant<'i, 'a, T>(x: IterMut<'i, &'static T>) -> IterMut<'i, &'a T> { x }
  |                                                                                      ^ lifetime mismatch
  |
  = note: expected struct `linked_list::IterMut<'_, &'a T>`
             found struct `linked_list::IterMut<'_, &'static T>`
```

Ok cool, we've proved it's invariant, but uh, now our tests fail. No worries, rustdoc lets you say that's expected by annotating the fence with compile_fail!

(Actually we only proved it's "not covariant" but honestly if you manage to make a type "accidentaly and incorrectly contravariant" then, congrats?)

```rust ,ignore
    /// ```compile_fail
    /// use linked_list::IterMut;
    /// 
    /// fn iter_mut_covariant<'i, 'a, T>(x: IterMut<'i, &'static T>) -> IterMut<'i, &'a T> { x }
    /// ```
    fn iter_mut_invariant() {}
```

```text
cargo test
   Compiling linked-list v0.0.3
    Finished test [unoptimized + debuginfo] target(s) in 0.49s
     Running unittests src\lib.rs

...

   Doc-tests linked-list

running 1 test
test src\lib.rs - assert_properties::iter_mut_invariant (line 458) - compile fail ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

Yay! I recommend always making the test without compile_fail so that you can confirm that it fails to compile *for the right reason*. For instance, that test will also fail (and therefore pass) if you forget the `use`, which, is not what we want! While it's conceptually appealing to be able to "require" a specific error from the compiler, this would be an absolute nightmare that would effectively make it a breaking change *for the compiler to produce better errors*. We want the compiler to get better, so, no you don't get to have that.

(Oh wait, we can actually just specify the error code we want next to the compile_fail **but this only works on nightly and is a bad idea to rely on for the reasons state above. It will be silently ignored on not-nightly.**)

```rust ,ignore
    /// ```compile_fail,E0308
    /// use linked_list::IterMut;
    /// 
    /// fn iter_mut_covariant<'i, 'a, T>(x: IterMut<'i, &'static T>) -> IterMut<'i, &'a T> { x }
    /// ```
    fn iter_mut_invariant() {}
```

...also, did you notice the part where we actually made IterMut invariant? It was easy to miss, since I "just" copy-pasted Iter and dumped it at the end. It's the last line here:

```rust ,ignore
pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: PhantomData<&'a mut T>,
}
```

Let's try removing that PhantomData:

```text
 cargo build
   Compiling linked-list v0.0.3 (C:\Users\ninte\dev\contain\linked-list)
error[E0392]: parameter `'a` is never used
  --> src\lib.rs:30:20
   |
30 | pub struct IterMut<'a, T> {
   |                    ^^ unused parameter
   |
   = help: consider removing `'a`, referring to it in a field, or using a marker such as `PhantomData`
```

Ha! The compiler has our back and won't just let us *not* use the lifetime. Let's try using the *wrong* example instead:

```rust ,ignore
    _boo: PhantomData<&'a T>,
```

```text
cargo build
   Compiling linked-list v0.0.3 (C:\Users\ninte\dev\contain\linked-list)
    Finished dev [unoptimized + debuginfo] target(s) in 0.17s
```

It builds! Do our tests catch a problem now?

```text
cargo test

...

   Doc-tests linked-list

running 1 test
test src\lib.rs - assert_properties::iter_mut_invariant (line 458) - compile fail ... FAILED

failures:

---- src\lib.rs - assert_properties::iter_mut_invariant (line 458) stdout ----
Test compiled successfully, but it's marked `compile_fail`.

failures:
    src\lib.rs - assert_properties::iter_mut_invariant (line 458)

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

Eyyy!!! The system works! I love having tests that actually do their job, so that I don't have to be quite so horrified of looming mistakes!

