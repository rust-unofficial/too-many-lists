# Miri

*nervously laughs* This unsafe stuff is so easy, I don't know why everyone says otherwise. Our program works perfectly.

> **NARRATOR:** 🙂

...right?

> **NARRATOR:** 🙂

Well, we're writing `unsafe` code now, so the compiler can't help us catch mistakes as well. It could be that the tests *happened* to work, but were actually doing something non-deterministic. Something Undefined Behavioury.

But what can we do? We've pried open the windows and snuck out of rustc's classroom. No one can help us now.

...Wait, who's that sketchy looking person in the alleyway?

*"Hey kid, you wanna interpret some Rust code?"*

Wh- no? Why,

*"It's wild man, it can validate that the actual dynamic execution of your program conforms to the semantics of Rust's memory model. Blows your mind..."*

What?

*"It checks if you Do An Undefined Behaviour."*

I guess I could try interpretters just *once*.

*"You've got rustup installed right?"*

Of course I do, it's *the* tool for having an up to date Rust toolchain!

```text
> rustup +nightly-2022-01-21 component add miri

info: syncing channel updates for 'nightly-2022-01-21-x86_64-pc-windows-msvc'
info: latest update on 2022-01-21, rust version 1.60.0-nightly (777bb86bc 2022-01-20)
info: downloading component 'cargo'
info: downloading component 'clippy'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'rustfmt'
info: installing component 'cargo'
info: installing component 'clippy'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'rustfmt'
info: downloading component 'miri'
info: installing component 'miri'
```

What did you just install on my computer!?

*"The Good Stuff"*

> **NARRATOR:** Some weird stuff going on with toolchain versions:
>
> The tool we're installing, `miri`, works closely with rustc's internals, 
> so it's only available for nightly toolchains.
>
> `+nightly-2022-01-21` tells `rustup` we want to install miri with the rust 
> nightly toolchain for that date. I'm giving a specific date because sometimes
> miri falls behind and can't be built for a few nightlies. rustup will
> automatically download whatever toolchain we specify with `+` if we don't
> have it installed yet.
>
> 2022-01-21 is just a nightly I know has miri support, which you can check 
> [on this status page](https://rust-lang.github.io/rustup-components-history/).
> You can just use `+nightly` if you're feeling lucky.
> 
> Whenever we invoke miri via `cargo miri` we will also use this `+` syntax to
> specify the toolchain we installed miri on. If you don't want to have to
> specify it every time, you can use [`rustup override set`](https://rust-lang.github.io/rustup/overrides.html).

```text
> cargo +nightly-2022-01-21 miri test

I will run `"cargo.exe" "install" "xargo"` to install
a recent enough xargo. Proceed? [Y/n]
```

UH WHAT ON EARTH IS XARGO?

*"It's fine, don't worry about it."*

```text
> y

    Updating crates.io index
  Installing xargo v0.3.24
...
    Finished release [optimized] target(s) in 10.65s
  Installing C:\Users\ninte\.cargo\bin\xargo-check.exe
  Installing C:\Users\ninte\.cargo\bin\xargo.exe
   Installed package `xargo v0.3.24` (executables `xargo-check.exe`, `xargo.exe`)

I will run `"rustup" "component" "add" "rust-src"` to install 
the `rust-src` component for the selected toolchain. Proceed? [Y/n]
```

UH???

*"Who doesn't love having a copy of Rust's source code?"*

```text
> y

info: downloading component 'rust-src'
info: installing component 'rust-src'
```

*"Aw yeah it's ready, here's the good part."*

```text
   Compiling lists v0.1.0 (C:\Users\ninte\dev\tmp\lists)
    Finished test [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests (lists-5cc11d9ee5c3e924.exe)

error: Undefined Behavior: trying to reborrow for Unique at alloc84055, 
       but parent tag <209678> does not have an appropriate item in 
       the borrow stack

   --> \lib\rustlib\src\rust\library\core\src\option.rs:846:18
    |
846 |             Some(x) => Some(f(x)),
    |                  ^ trying to reborrow for Unique at alloc84055, 
    |                    but parent tag <209678> does not have an 
    |                    appropriate item in the borrow stack
    |
    = help: this indicates a potential bug in the program: 
      it performed an invalid operation, but the rules it 
      violated are still experimental
    = help: see https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md 
      for further information

    = note: inside `std::option::Option::<std::boxed::Box<fifth::Node<i32>>>::map::<i32, [closure@src\fifth.rs:31:30: 40:10]>` at \lib\rustlib\src\rust\library\core\src\option.rs:846:18

note: inside `fifth::List::<i32>::pop` at src\fifth.rs:31:9
   --> src\fifth.rs:31:9
    |
31  | /         self.head.take().map(|head| {
32  | |             let head = *head;
33  | |             self.head = head.next;
34  | |
...   |
39  | |             head.elem
40  | |         })
    | |__________^
note: inside `fifth::test::basics` at src\fifth.rs:74:20
   --> src\fifth.rs:74:20
    |
74  |         assert_eq!(list.pop(), Some(1));
    |                    ^^^^^^^^^^
note: inside closure at src\fifth.rs:62:5
   --> src\fifth.rs:62:5
    |
61  |       #[test]
    |       ------- in this procedural macro expansion
62  | /     fn basics() {
63  | |         let mut list = List::new();
64  | |
65  | |         // Check empty list behaves right
...   |
96  | |         assert_eq!(list.pop(), None);
97  | |     }
    | |_____^
 ...
error: aborting due to previous error
```

Woah. That's one heck of an error.

*"Yeah, look at that shit. You love to see it."*

Thank you?

*"Here take the bottle of estradiol too, you're gonna need it later."*

Wait why?

*"You're about to think about memory models, trust me."*

> **NARRATOR:** The mysterious person then proceeded to transform into a fox and scampered through a hole in the wall. The author then stared into the middle distance for several minutes while they tried to process everything that just happened.


-------

The mysterious fox in the alleyway was right about more than just my gender: miri really is The Good Shit.

Ok so what *is* [miri](https://github.com/rust-lang/miri)?

> An experimental interpreter for Rust's mid-level intermediate representation (MIR). It can run binaries and test suites of cargo projects and detect certain classes of undefined behavior, for example:
>
> * Out-of-bounds memory accesses and use-after-free
> * Invalid use of uninitialized data
> * Violation of intrinsic preconditions (an unreachable_unchecked being reached, calling copy_nonoverlapping with overlapping ranges, ...)
> * Not sufficiently aligned memory accesses and references
> * Violation of some basic type invariants (a bool that is not 0 or 1, for example, or an invalid enum discriminant)
> * Experimental: Violations of the Stacked Borrows rules governing aliasing for reference types
> * Experimental: Data races (but no weak memory effects)
>
> On top of that, Miri will also tell you about memory leaks: when there is memory still allocated at the end of the execution, and that memory is not reachable from a global static, Miri will raise an error.
>
> ...
>
> However, be aware that Miri will not catch all cases of undefined behavior in your program, and cannot run all programs

TL;DR: it interprets your program and notices if you break the rules *at runtime* and Do An Undefined Behaviour. This is necessary because Undefined Behaviour is *generally* a thing that happens at runtime. If the issue could be found at compile time, the compiler would just make it an error!

If you're familiar with tools like ubsan and tsan: it's basically that but all together and more extreme.

-------

Miri is now hanging outside the classroom window with a knife. A learning knife.

If we ever want miri to check our work, we can ask them to interpret our test suite with

```text
> cargo +nightly-2022-01-21 miri test
```

Now let's take a closer look at what they carved into our desk:

```text
error: Undefined Behavior: trying to reborrow for Unique at alloc84055, but parent tag <209678> does not have an appropriate item in the borrow stack

   --> \lib\rustlib\src\rust\library\core\src\option.rs:846:18
    |
846 |             Some(x) => Some(f(x)),
    |                  ^ trying to reborrow for Unique at alloc84055, 
    |                    but parent tag <209678> does not have an 
    |                    appropriate item in the borrow stack
    |

    = help: this indicates a potential bug in the program: it 
      performed an invalid operation, but the rules it 
      violated are still experimental
    
    = help: see 
      https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md 
      for further information
```

Well I can see we made an error, but that's a confusing error message. What's the "borrow stack"?

We'll try to figure that out in the next section.