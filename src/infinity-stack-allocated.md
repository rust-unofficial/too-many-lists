# The Stack-Allocated Linked List

This book is largely focused on *heap-allocated* linked lists, because those are the most common and practical, but we don't *have* to use heap allocation. Heap allocation is nice because it makes it easy to dynamically allocate memory. Stack allocation is less friendly in this regard &mdash; things like C's `alloca` are widely regarded as Very Cursed And Problematic.

So let's allocate memory on the stack the easy way: by calling a function and getting a new stack frame with more space! This is a very silly solution to our problem but also genuinely practical and useful. It's done all the time, potentially without actually even thinking about it as a linked list! 

Any time you're doing something recursively, you can just pass a pointer to the current step's state to the next step. If that pointer itself is *part* of the state, then you've created a linked list that's stack-allocated!

Now of course we're in the *silly* part of the book so we're going to do this in a silly way: by making the linked list the star and forcing all the user's code to live in a swamp of callbacks. Everybody loves nested callbacks!

Our List type will just be a Node with a reference to another Node:

```rust
pub struct List<'a, T> {
    pub data: T,
    pub prev: Option<&'a List<'a, T>>,
}
```

And it will have only one operation, `push`, which will take the old list, the state for the current node, and a callback. The new list will be produced in the callback. We will also let callbacks return any value, which `push` will return when it completes:

```rust ,ignore
impl<'a, T> List<'a, T> {
    pub fn push<U>(
        prev: Option<&'a List<'a, T>>, 
        data: T, 
        callback: impl FnOnce(&List<'a, T>) -> U,
    ) -> U {
        let list = List { data, prev };
        callback(&list)
    }
}
```

That's it! We can use it like this:

```rust ,ignore
List::push(None, 3, |list| {
    println!("{}", list.data);
    List::push(Some(list), 5, |list| {
        println!("{}", list.data);
        List::push(Some(list), 13, |list| {
            println!("{}", list.data);
        })
    })
})
```

It's beautiful. 😿

The user can already traverse this list by using while-let to walk over the `prev` values, but just for fun, let's implement an iterator, which is the usual:

```rust ,ignore
impl<'a, T> List<'a, T> {
    pub fn iter(&'a self) -> Iter<'a, T> {
        Iter { next: Some(self) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.prev;
            &node.data
        })
    }
}
```

Let's test it out:

```rust ,ignore
#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn elegance() {
        List::push(None, 3, |list| {
            assert_eq!(list.iter().copied().sum::<i32>(), 3);
            List::push(Some(list), 5, |list| {
                assert_eq!(list.iter().copied().sum::<i32>(), 5 + 3);
                List::push(Some(list), 13, |list| {
                    assert_eq!(list.iter().copied().sum::<i32>(), 13 + 5 + 3);
                })
            })
        })
    }
}
```

```text
> cargo test

running 18 tests
test fifth::test::into_iter ... ok
test fifth::test::iter ... ok
test fifth::test::iter_mut ... ok
test fifth::test::basics ... ok
test fifth::test::miri_food ... ok
test first::test::basics ... ok
test second::test::into_iter ... ok
test fourth::test::peek ... ok
test fourth::test::into_iter ... ok
test second::test::iter_mut ... ok
test fourth::test::basics ... ok
test second::test::basics ... ok
test second::test::iter ... ok
test third::test::basics ... ok
test silly1::test::walk_aboot ... ok
test silly2::test::elegance ... ok
test second::test::peek ... ok
test third::test::iter ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```

Now at this point you might be wonder "hey can I mutate the data stored in a node?". Maybe! Let's try to make the list use mutable references instead of shared ones:


```rust
pub struct List<'a, T> {
    pub data: T,
    pub prev: Option<&'a mut List<'a, T>>,
}

pub struct Iter<'a, T> {
    next: Option<&'a List<'a, T>>,
}

impl<'a, T> List<'a, T> {
    pub fn push<U>(
        prev: Option<&'a mut List<'a, T>>, 
        data: T, 
        callback: impl FnOnce(&mut List<'a, T>) -> U,
    ) -> U {
        let mut list = List { data, prev };
        callback(&mut list)
    }

    pub fn iter(&'a self) -> Iter<'a, T> {
        Iter { next: Some(self) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.prev.as_ref().map(|prev| &**prev);
            &node.data
        })
    }
}

```


```text
> cargo test

error[E0521]: borrowed data escapes outside of closure
  --> src\silly2.rs:47:32
   |
46 |  List::push(Some(list), 13, |list| {
   |                              ----
   |                              |
   |              `list` declared here, outside of the closure body
   |              `list` is a reference that is only valid in the closure body
47 |      assert_eq!(list.iter().copied().sum::<i32>(), 13 + 5 + 3);
   |                 ^^^^^^^^^^^ `list` escapes the closure body here

error[E0521]: borrowed data escapes outside of closure
  --> src\silly2.rs:45:28
   |
44 |  List::push(Some(list), 5, |list| {
   |                             ----
   |                             |
   |              `list` declared here, outside of the closure body
   |              `list` is a reference that is only valid in the closure body
45 |      assert_eq!(list.iter().copied().sum::<i32>(), 5 + 3);
   |                 ^^^^^^^^^^^ `list` escapes the closure body here


<ad infinitum>
```

Whelp. Seems like it doesn't like our iterator. Maybe we messed that up? Let's
simplify the test a bit to check:


```rust ,ignore
#[test]
fn elegance() {
    List::push(None, 3, |list| {
        assert_eq!(list.data, 3);
        List::push(Some(list), 5, |list| {
            assert_eq!(list.data, 5);
            List::push(Some(list), 13, |list| {
                assert_eq!(list.data, 13);
            })
        })
    })
}
```

```text
> cargo test

error[E0521]: borrowed data escapes outside of closure
  --> src\silly2.rs:46:17
   |
44 |   List::push(Some(list), 5, |list| {
   |                              ----
   |                              |
   |              `list` declared here, outside of the closure body
   |              `list` is a reference that is only valid in the closure body
45 |       assert_eq!(list.data, 5);
46 | /     List::push(Some(list), 13, |list| {
47 | |         assert_eq!(list.data, 13);
48 | |     })
   | |______^ `list` escapes the closure body here

error[E0521]: borrowed data escapes outside of closure
  --> src\silly2.rs:44:13
   |
42 |   List::push(None, 3, |list| {
   |                        ----
   |                        |
   |              `list` declared here, outside of the closure body
   |              `list` is a reference that is only valid in the closure body
43 |       assert_eq!(list.data, 3);
44 | /     List::push(Some(list), 5, |list| {
45 | |         assert_eq!(list.data, 5);
46 | |         List::push(Some(list), 13, |list| {
47 | |             assert_eq!(list.data, 13);
48 | |         })
49 | |     })
   | |______________^ `list` escapes the closure body here
```

Hmm no that's still some hot garbage.

The problem is that our list is accidentally(😉) relying on *variance*. [Variance is a complicated subject](https://doc.rust-lang.org/nomicon/subtyping.html) but let's look at it in simplified terms here:

Each list contains a reference to a List with *the exact same type as itself*. From the perspective of the inner-most list, that means all lists are using the same lifetime as itself, but this is *objectively* false: each node in the list lives strictly longer than the next one, because they are literally in nested scopes!

So... why did the code compile when we were using shared references? Because in many cases, the compiler knows it's safe to have something that lives "too long"! When we stuff a reference to a list into the next one, the compiler is quietly "shrinking" down the lifetimes to make them fit what the new list expects. This lifetime shrinking is *variance*. 

It's the exact same trick in languages with inheritance that let's you pass a Cat where an Animal (a supertype of a Cat) is expected. Intuitively we know it's fine to pass a Cat when an Animal is expected because a Cat is just an Animal *and more*. It's *fine* to forget the "and more" part for a while, right?

Similarly, a larger lifetime is just a smaller lifetime *and more*. So it's fine to forget the "and more" here too!

But of course you are now wondering: then why doesn't the mutable reference version work!?

Well, variance *isn't* always safe. If our code *did* compile, we could have written a use-after-free like this:

```rust ,ignore
List::push(None, 3, |list| {
    List::push(Some(list), 5, |list| {
        List::push(Some(list), 13, |list| {
            // HAHAHA all the lifetimes are the same, so the compiler will
            // let me rewrite my parent to hold a mutable reference to myself!
            // I will create all the use-after-frees!!
            *list.prev.as_mut().unwrap().prev = Some(list);
        })
    })
})
```

The problem with forgetting details is that *somewhere else might remember
those details and expect them to remain true*. That is a very big problem
once you introduce *mutation*. If you're not careful, the code that doesn't
remember the "and more" that we threw away might think it's fine to write
things to places that "remember" and *expect* the "and more" to still be there.

Put in terms of inheritance: this code has to be illegal:

```rust ,ignore
let mut my_kitty = Cat;                  // Make a Cat (long lifetime)
let animal: &mut Animal = &mut my_kitty; // Forget it's a Cat (shorten lifetime)
*animal = Dog;                           // Write a Dog (short lifetime)
my_kitty.meow();                         // Meowing Dog! (Use After Free)
```

So while you *can* shorten the lifetime of a mutable reference, once you start
*nesting* them things become "invariant" and you're not allowed to shorten
lifetimes anymore. 

Specifically `&mut &'big mut T` cannot be converted to `&mut &'small mut T`, 
where `'big` is bigger than `'small`. Or more formally, `&'a mut T` is covariant
over `'a` but invariant over `T`.

Fun fact: Java actually specifically *lets* you do this kind of thing, but it
[does runtime checks to prevent meowing dogs](https://docs.oracle.com/javase/7/docs/api/java/lang/ArrayStoreException.html).

----

So what can we do to mutate the data? Use interior mutability! This lets us
tell the compiler that we just want to be able to mutate the *data* but won't
touch the references.

We can just revert back to the previous version of our code with shared references,
and use `Cell` in a new test:

```rust ,ignore
#[test]
fn cell() {
    use std::cell::Cell;

    List::push(None, Cell::new(3), |list| {
        List::push(Some(list), Cell::new(5), |list| {
            List::push(Some(list), Cell::new(13), |list| {
                // Multiply every value in the list by 10
                for val in list.iter() {
                    val.set(val.get() * 10)
                }

                let mut vals = list.iter();
                assert_eq!(vals.next().unwrap().get(), 130);
                assert_eq!(vals.next().unwrap().get(), 50);
                assert_eq!(vals.next().unwrap().get(), 30);
                assert_eq!(vals.next(), None);
                assert_eq!(vals.next(), None);
            })
        })
    })
}
```

```text
> cargo test

running 19 tests
test fifth::test::into_iter ... ok
test fifth::test::basics ... ok
test fifth::test::iter_mut ... ok
test fifth::test::iter ... ok
test fourth::test::basics ... ok
test fourth::test::into_iter ... ok
test second::test::into_iter ... ok
test first::test::basics ... ok
test fourth::test::peek ... ok
test second::test::basics ... ok
test fifth::test::miri_food ... ok
test silly2::test::cell ... ok
test third::test::iter ... ok
test second::test::iter_mut ... ok
test second::test::peek ... ok
test silly1::test::walk_aboot ... ok
test silly2::test::elegance ... ok
test third::test::basics ... ok
test second::test::iter ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```

Easy as recursive pie! ✨
