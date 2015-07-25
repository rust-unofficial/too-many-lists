# Learn Rust by writing Entirely Too Many linked lists

Read the pretty version at http://cglab.ca/~abeinges/blah/too-many-lists/book/

Building requires an instance of rustbook be set up on your machine. 
The only way to do this correctly is to build Rust from scratch with 
`./configure --enable-rpath` and steal their copy of rustbook. 

Once you have, you just need to do:

```
rm -rf book/ && rustbook build text/ book/
```
