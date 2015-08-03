# Learn Rust by writing Entirely Too Many linked lists

Read the pretty version at http://cglab.ca/~abeinges/blah/too-many-lists/book/

# Building

Building requires an instance of rustbook be set up on your machine. 
A mirror of the rustbook code can be found [here](https://github.com/steveklabnik/rustbook).
This requires a nightly version of the Rust compiler, as well as Cargo:

```sh
cd <rustbook-dir>
cargo build --release
```

Once built, the binary can be found at `<rustbook-dir>/target/bin/rustbook`.
Now just copy or link rustbook to be somewhere on your path.

Once you have, you just need to do:

```sh
cd <too-many-lists-dir>
rm -rf book/ && rustbook build text/ book/
```
