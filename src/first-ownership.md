# Ownership 101

Now that we can construct a list, it'd be nice to be able to *do* something
with it. We do that with "normal" (non-static) methods. Methods are a special
case of function in Rust because of  the `self` argument, which doesn't have
a declared type:

```rust ,ignore
fn foo(self, arg2: Type2) -> ReturnType {
    // body
}
```

There are 3 primary forms that self can take: `self`, `&mut self`, and `&self`.
These 3 forms represent the three primary forms of ownership in Rust:

* `self` - Value
* `&mut self` - mutable reference
* `&self` - shared reference

A value represents *true* ownership. You can do whatever you want with a value:
move it, destroy it, mutate it, or loan it out via a reference. When you pass
something by value, it's *moved* to the new location. The new location now
owns the value, and the old location can no longer access it. For this reason
most methods don't want `self` -- it would be pretty lame if trying to work with
a list made it go away!

A mutable reference represents temporary *exclusive access* to a value that you
don't own. You're allowed to do absolutely anything you want to a value you
have a mutable reference to as long you leave it in a valid state when you're
done (it would be rude to the owner otherwise!). This means you can actually completely
overwrite the value. A really useful special case of this is *swapping* a value
out for another, which we'll be using a lot. The only thing you can't do with an
`&mut` is move the value out with no replacement. `&mut self` is great for
methods that want to mutate `self`.

A shared reference represents temporary *shared access* to a value that you
don't own. Because you have shared access, you're generally not allowed to
mutate anything. Think of `&` as putting the value out on display in a museum.
`&` is great for methods that only want to observe `self`.

Later we'll see that the rule about mutation can be bypassed in certain cases.
This is why shared references aren't called *immutable* references. Really,
mutable references could be called *unique* references, but we've found that
relating ownership to mutability gives the right intuition 99% of the time.
