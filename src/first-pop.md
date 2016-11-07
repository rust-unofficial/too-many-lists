# Pop

Like `push`, `pop` wants to mutate the list. However, unlike `push` we actually
want to return something. However `pop` also has to deal with a tricky corner
case: what if the list is empty? To represent this case, we use the trusty
`Option` type:

```rust
pub fn pop(&mut self) -> Option<i32> {
    //TODO
}
```

`Option<T>` is an enum that represents a value that may exist. It can either be
`Some(T)` or `None`. We could make our own enum for this like we did for
Link, but we want our users to be able to understand what the heck our return
type is, and Option is so ubiquitous that *everyone* knows it. In fact, it's so
fundamental that it's implicitly imported into scope in every file, as well
as its variants `Some` and `None` (so we don't have to say `Option::None`).

The pointy bits on `Option<T>` indicate that Option is actually *generic* over
T. That means that you can make an Option for *any* type!

So uh, we have this `Link` thing, how do we figure out if it's Empty or has
More? Pattern matching with `match`!

```rust
pub fn pop(&mut self) -> Option<i32> {
    match self.head {
        Link::Empty => {
            // TODO
        }
        Link::More(node) => {
            // TODO
        }
    };
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:27:2: 36:3 error: not all control paths return a value [E0269]
src/first.rs:27   pub fn pop(&mut self) -> Option<i32> {
src/first.rs:28       match self.head {
src/first.rs:29           Link::Empty => {
src/first.rs:30               // TODO
src/first.rs:31           }
src/first.rs:32           Link::More(node) => {
              ...
error: aborting due to previous error
Could not compile `lists`.
```

Whoops, `pop` has to return a value, and we're not doing that yet. We *could*
return `None`, but in this case it's probably a better idea to return
`unimplemented!()`, to indicate that we aren't done implementing the function.
`unimplemented!()` is a macro (`!` indicates a macro) that panics (basically
just crashes in a controlled manner) the program when we get to it.

```rust
pub fn pop(&mut self) -> Option<i32> {
    match self.head {
        Link::Empty => {
            // TODO
        }
        Link::More(node) => {
            // TODO
        }
    };
    unimplemented!()
}
```

Unconditional panics are an example of a [diverging function][diverging].
Diverging functions never return to the caller, so they may be used in places
where a value of any type is expected. Here, `unimplemented!()` is being
used in place of a value of type `Option<T>`.

Note also that we don't need to write `return` in our program. The last
expression (basically line) in a function is implicitly its return value. This
lets us express really simple things a bit more concisely. You can always
explicitly return early with `return` like any other C-like language.

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:28:9: 28:13 error: cannot move out of borrowed content
src/first.rs:28       match self.head {
                            ^~~~
src/first.rs:32:15: 32:19 note: attempting to move value to here
src/first.rs:32           Link::More(node) => {
                                     ^~~~
src/first.rs:32:15: 32:19 help: to prevent the move, use `ref node` or `ref mut node` to capture value by reference
error: aborting due to previous error
Could not compile `lists`.
```

Come on Rust, get off our back! As always, Rust is hella mad at us. Thankfully,
this time it's also giving us the full scoop!

```text
src/first.rs:17:9: 17:13 error: cannot move out of borrowed content
src/first.rs:17       match self.head {
                            ^~~~
src/first.rs:21:15: 21:19 note: attempting to move value to here
src/first.rs:21           Link::More(node) => {
                                     ^~~~
```

Pattern matches by default move the value they match on, so that's why Rust's
mad.

```text
help: to prevent the move, use `ref node` or `ref mut node` to capture value by reference
```

to avoid that, we use the `ref` keyword to indicate that we want to bind the
`node` subpattern by reference instead. Let's do that:

```rust
pub fn pop(&mut self) -> Option<i32> {
    match self.head {
        Link::Empty => {
            // TODO
        }
        Link::More(ref node) => {
            // TODO
        }
    };
    unimplemented!()
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/lists)
src/first.rs:13:2: 13:9 warning: struct field is never used: `elem`, #[warn(dead_code)] on by default
src/first.rs:13   elem: i32,
                  ^~~~~~~
src/first.rs:14:2: 14:15 warning: struct field is never used: `next`, #[warn(dead_code)] on by default
src/first.rs:14   next: Link<T>,
                  ^~~~~~~~~~~~~
src/first.rs:32:15: 32:23 warning: unused variable: `node`, #[warn(unused_variables)] on by default
src/first.rs:32           Link::More(ref node) => {
                                     ^~~~~~~~
```

Hooray, compiling again! Now let's figure out that logic. We want to make an
Option, so let's make a variable for that. In the Empty case we need to return
None. In the More case we need to return `Some(i32)`, and change the head of
the list. So, let's try to do basically that?

```rust
pub fn pop(&mut self) -> Option<i32> {
    let result;
    match self.head {
        Link::Empty => {
            result = None;
        }
        Link::More(ref node) => {
            result = Some(node.elem);
            self.head = node.next;
        }
    };
    result
}
```

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/first.rs:39:29: 39:33 error: cannot move out of borrowed content
src/first.rs:39                 self.head = node.next;
                                            ^~~~
src/first.rs:39:17: 39:38 error: cannot assign to `self.head` because it is borrowed
src/first.rs:39                 self.head = node.next;
                                ^~~~~~~~~~~~~~~~~~~~~
src/first.rs:37:24: 37:32 note: borrow of `self.head` occurs here
src/first.rs:37             Link::More(ref node) => {
                                       ^~~~~~~~
error: aborting due to 2 previous errors
Could not compile `lists`.
```

*head*

*desk*

Now we have two *different* errors. First, we're trying to move out of `node`
when all we have is a shared reference to it. Second, we're trying to mutate
`self.head` while we've already borrowed it to get the reference to `node`!

This is a tangled mess.

We should probably step back and think about what we're trying to do. We want
to:

* Check if the list is empty.
* If it's empty, just return None
* It it's *not* empty
    * remove the head of the list
    * remove its `elem`
    * replace the lists head with its `next`
    * return `Some(elem)`

The key insight is we want to *remove* things, which means we want to get the
head of the list *by value*. We certainly can't do that through the shared
reference we get through `ref node`. We also "only" have a mutable reference,
so the only way we can move stuff is to *replace it*. Looks like we're doing
the Empty dance again! Let's try that:


```rust
pub fn pop(&mut self) -> Option<i32> {
    let result;
    match mem::replace(&mut self.head, Link::Empty) {
        Link::Empty => {
            result = None;
        }
        Link::More(node) => {
            result = Some(node.elem);
            self.head = node.next;
        }
    };
    result
}
```

```text
cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

O M G

It compiled without *any* warnings!!!!!

Actually I'm going to apply my own personal lint here: we made this `result`
value to return, but actually we didn't need to do that at all! Just as a
function evaluated to its last expression, every block also evaluates to
its last expression. Normally we supress this behaviour with semi-colons,
which instead makes the block evaluate to the empty tuple, `()`. This is
actually the value that functions which don't declare a return value -- like
`push` -- return.

So instead, we can write `pop` as:

```rust
pub fn pop(&mut self) -> Option<i32> {
    match mem::replace(&mut self.head, Link::Empty) {
        Link::Empty => None,
        Link::More(node) => {
            self.head = node.next;
            Some(node.elem)
        }
    }
}
```

Which is a bit more concise and idiomatic. Note that the Link::Empty branch
completely lost its braces, because we only have one expression to
evaluate. Just a nice shorthand for simple cases.

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
src/first.rs:36:22: 36:31 error: use of moved value: `node` [E0382]
src/first.rs:36                 Some(node.elem)
                                     ^~~~~~~~~
src/first.rs:35:29: 35:38 note: `node` moved here (through moving `node.next`) because it has type `first::Link`, which is non-copyable
src/first.rs:35                 self.head = node.next;
                                            ^~~~~~~~~
error: aborting due to previous error
```

WHAT. COME ON.

Why the heck did our code stop working?!

It turns out we accidentally got lucky with the previous code. We have just had
our first run in with the magic of Copy. When we introduced [ownership][] we
said that when you move stuff, you can't use it anymore. For some types, this
makes perfect sense. Our good friend Box manages an allocation on the heap for
us, and we certainly don't want two pieces of code to think that they need to
free its memory.

However for other types this is *butts*. Integers have no
ownership semantics; they're just meaningless numbers! This is why integers are
marked as Copy. Copy types are known to be perfectly copyable by a bitwise copy.
As such, they have a super power: when moved, the old value *is* still usable.
As a consequence, you can even move a Copy type out of a reference without
replacement!

All numeric primitives in rust (i32, u64, bool, f32, char, etc...) are Copy.
Also, shared references are Copy, which is super useful! You can also declare
any user-defined type to be Copy as well, as long as all its components are
Copy.

Anyway, back to the code: what went wrong? In our first iteration, we were
actually *copying* the i32 `elem` when we assigned to result, so the node was
left unscathed for the next operation. Now we're *moving* the `next` value
(which isn't Copy), and that consumes the whole Box before we can get to `elem`.

Now, we could just rearrange again to get `elem` first, but we're only using
i32 as a placeholder for *some* data. Later we'll want to work with non-Copy
data, so we should figure out how to handle this now.

The *right* answer is to pull the *whole* node out of the Box, so that we can
tear it apart in peace. We do that by explicitly dereferencing it:

```rust
pub fn pop(&mut self) -> Option<i32> {
    match mem::replace(&mut self.head, Link::Empty) {
        Link::Empty => None,
        Link::More(boxed_node) => {
            let node = *boxed_node;
            self.head = node.next;
            Some(node.elem)
        }
    }
}
```

After that, Rust understands an on-the-stack value well enough to let you take
it apart piece-by-piece.

```text
> cargo build
   Compiling lists v0.1.0 (file:///Users/ABeingessner/dev/too-many-lists/lists)
```

Nice.

Box is actually really special in Rust, because it's sufficiently built into the
language that the compiler lets you do some stuff that nothing else can do. We
actually have been doing one such thing this whole time: `DerefMove`. Whenever
you have a pointer type you can derefence it with `*` or `.` to get at its
contents. Usually you can get a `Deref` or maybe even a `DerefMut`,
corresponding to a shared or mutable reference respectively.

However because Box totally owns its contents, you can actually *move out of*
a dereference. This is total magic, because there's no way for any other type
to opt into this. There's tons of other cool tricks the compiler knows how to do
with Box because it *just is* Box, but they were all feature-gated at 1.0
pending further design. Ideally Box will be totally user definable in the
future.



[ownership]: first-ownership.html
[diverging]: http://doc.rust-lang.org/nightly/book/functions.html#diverging-functions
