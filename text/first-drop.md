% Testing

Alright, so we've got `push` and `pop` written, now we can actually test out
our stack! Rust and cargo support testing as a first-class feature, so this
will be super easy. All we have to do is write function, and annotate it with
`#[test]`.

Generally, we try to keep our tests next to the code that its testing in the
Rust community. However we usually make a new namespace for the tests, to
avoid conflicting with the "real" code. Just as we used `mod` to specify that
`first.rs` should be included in `lib.rs`, we can use `mod` to basically
create a whole new file *inline*:


```
// in first.rs

mod test {
    #[test]
    basics() {
        // TODO
    }
}
