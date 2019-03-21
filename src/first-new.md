# New

To associate actual code with a type, we use `impl` blocks:

```rust ,ignore
impl List {
    // TODO, make code happen
}
```

Now we just need to figure out how to actually write code. In Rust we declare
a function like so:

```rust ,ignore
fn foo(arg1: Type1, arg2: Type2) -> ReturnType {
    // body
}
```

The first thing we want is a way to *construct* a list. Since we hide the
implementation details, we need to provide that as a function. The usual way
to do that in Rust is to provide a static method, which is just a
normal function inside an `impl`:

```rust ,ignore
impl List {
    pub fn new() -> Self {
        List { head: Link::Empty }
    }
}
```

A few notes on this:

* Self is an alias for "that type I wrote at the top next to `impl`". Great for
  not repeating yourself!
* We create an instance of a struct in much the same way we declare it, except
  instead of providing the types of its fields, we initialize them with values.
* We refer to variants of an enum using `::`, which is the namespacing operator.
* The last expression of a function is implicitly returned.
  This makes simple functions a little neater. You can still use `return`
  to return early like other C-like languages.























