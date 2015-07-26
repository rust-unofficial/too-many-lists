% An Unsafe Singly-Linked Queue

Ok that reference-counted interior mutability stuff got a little out of
control. Surely Rust doesn't really expect you to do that sort of thing
in general? Well, yes and no. A doubly linked list is a pretty degenerate
case of shared
