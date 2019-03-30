# Testing

Alright, so we've got `push` and `pop` written, now we can actually test out
our stack! Rust and cargo support testing as a first-class feature, so this
will be super easy. All we have to do is write a function, and annotate it with
`#[test]`.

Generally, we try to keep our tests next to the code that it's testing in the
Rust community. However we usually make a new namespace for the tests, to
avoid conflicting with the "real" code. Just as we used `mod` to specify that
`first.rs` should be included in `lib.rs`, we can use `mod` to basically
create a whole new file *inline*:


```rust ,ignore
// in first.rs

mod test {
    #[test]
    fn basics() {
        // TODO
    }
}
```

And we invoke it with `cargo test`.

```text
> cargo test
   Compiling lists v0.1.0 (/Users/ABeingessner/dev/temp/lists)
    Finished dev [unoptimized + debuginfo] target(s) in 1.00s
     Running /Users/ABeingessner/dev/lists/target/debug/deps/lists-86544f1d97438f1f

running 1 test
test first::test::basics ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
; 0 filtered out
```

Yay our do-nothing test passed! Let's make it not-do-nothing. We'll do that
with the `assert_eq!` macro. This isn't some special testing magic. All it
does is compare the two things you give it, and panic the program if they don't
match. Yep, you indicate failure to the test harness by freaking out!

```rust ,ignore
mod test {
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }
}
```

```text
> cargo test

error[E0433]: failed to resolve: use of undeclared type or module `List`
  --> src/first.rs:43:24
   |
43 |         let mut list = List::new();
   |                        ^^^^ use of undeclared type or module `List`


```

Oops! Because we made a new module, we need to pull in List explicitly to use
it.

```rust ,ignore
mod test {
    use super::List;
    // everything else the same
}
```

```text
> cargo test

warning: unused import: `super::List`
  --> src/first.rs:45:9
   |
45 |     use super::List;
   |         ^^^^^^^^^^^
   |
   = note: #[warn(unused_imports)] on by default

    Finished dev [unoptimized + debuginfo] target(s) in 0.43s
     Running /Users/ABeingessner/dev/lists/target/debug/deps/lists-86544f1d97438f1f

running 1 test
test first::test::basics ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
; 0 filtered out
```

Yay!

What's up with that warning though...? We clearly use List in our test!

...but only when testing! To appease the compiler (and to be friendly to our
consumers), we should indicate that the whole `test` module should only be
compiled if we're running tests.


```rust ,ignore
#[cfg(test)]
mod test {
    use super::List;
    // everything else the same
}
```

And that's everything for testing!
