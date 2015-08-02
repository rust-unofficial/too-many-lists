# Learn Rust by writing Entirely Too Many linked lists

Read the pretty version at http://cglab.ca/~abeinges/blah/too-many-lists/book/

# Building

Building requires an instance of rustbook be set up on your machine. 
The only way to do this correctly is to [build Rust from source](https://github.com/rust-lang/rust/#building-from-source) 

However it needs to be a slight deviation from the normal process:

* You should do `./configure --enable-rpath` instead of `./configure` 
* You don't need to `install` (I don't think rustbook will use that)

Once built, rustbook will be somewhere deep in the build target
directories. This is a bit platform-specific, to be honest. On my
machine it's at `x86_64-apple-darwin/stage2/bin/rustbook`. The
`x86_64-apple-darwin` bit is the *really* platform specific part,
where I hope you can guess what your platform will sort of look
like. On windows you may need to look in stage3.

Now just copy or link rustbook to be somewhere on your path.

Once you have, you just need to do:

```
rm -rf book/ && rustbook build text/ book/
```
