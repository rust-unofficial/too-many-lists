# Learn Rust by writing Entirely Too Many linked lists [![Build Status](https://travis-ci.org/rust-unofficial/too-many-lists.svg?branch=master)](https://travis-ci.org/rust-unofficial/too-many-lists)

Read the pretty version at http://cglab.ca/~abeinges/blah/too-many-lists/book/

# Building

Building requires an instance of rustbook be set up on your machine.

A mirror of the rustbook code can be found [here](https://github.com/steveklabnik/rustbook).
This requires a nightly version of the Rust compiler, as well as Cargo:

```sh
cd rustbook/
cargo build --release
```

Once built, the binary can be found at `rustbook/target/release/rustbook`.

---

If that doesn't work (#13), rustbook can also be built as part of rustc.
Here's instructions for
[building Rust from source](https://github.com/rust-lang/rust/#building-from-source).

However it needs to be a slight deviation from the normal process:

* You should do `./configure --enable-rpath` instead of `./configure`
* You don't need to `install` (I don't think rustbook will use that -- although
  maybe I'm wrong and make install will install rustbook too -- happy to be wrong!)

Once built, rustbook will be somewhere deep in the build target
directories. This is a bit platform-specific, to be honest. On my
machine it's at `x86_64-apple-darwin/stage2/bin/rustbook`. The
`x86_64-apple-darwin` bit is the *really* platform specific part,
where I hope you can guess what your platform will sort of look
like. On windows you may need to look in stage3.

Now just copy or link rustbook to be somewhere on your path.

---

Once you have the rustbook binary, you just need to do:

```sh
cd too-many-lists/
rm -rf book/ && rustbook build text/ book/
```

---

If you'd prefer, this project can also be built with
[GitBook](https://github.com/GitbookIO/gitbook), although GitBook
is not officially supported and compatibility is therefore
uncertain and incidental.
