# Implementing Cursors

Ok so we're only going to bother with std's CursorMut because the immutable version isn't actually interesting. Just like my original design, it has a "ghost" element that contains None to indicate the start/end of the list, and you can "walk over it" to wrap around to the other side of the list. To implement it, we're going to need:

* A pointer to the current node
* A pointer to the list
* The current index

Wait what's the index when we point at the "ghost"? 

*furrows brow* ... *checks std* ... *dislikes std's answer*

Ok so quite reasonably `index` on a Cursor returns an `Option<usize>`. The std implementation does a bunch of junk to avoid storing it as an Option but... we're a linked list, it's fine. Also std has the cursor_front/cursor_back stuff which starts the cursor on the front/back elements, which feels intuitive, but then has to do something weird when the list is empty.

You can implement that stuff if you want, but I'm going to cut down on all the repetitive gunk and corner cases and just make a bare `cursor_mut` method that starts at the ghost, and people can use move_next/move_prev to get the one they want (and then you can wrap that up as cursor_front if you really want).

Let's get cracking:

```rust ,ignore
pub struct CursorMut<'a, T> {
    cur: Link<T>,
    list: &'a mut LinkedList<T>,
    index: Option<usize>,
}
```

Pretty straight-forward, one field for each item of our bulleted list! Now the `cursor_mut` method:

```rust ,ignore
impl<T> LinkedList<T> {
    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        CursorMut { 
            list: self, 
            cur: None, 
            index: None,
        }
    }
}
```

Since we're starting at the ghost, we can just start with everything as None, nice and simple! Next, movement:


```rust ,ignore
impl<'a, T> CursorMut<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn move_next(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                // We're on a real element, go to its next (back)
                self.cur = (*cur.as_ptr()).back;
                if self.cur.is_some() {
                    *self.index.as_mut().unwrap() += 1;
                } else {
                    // We just walked to the ghost, no more index
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            // We're at the ghost, and there is a real front, so move to it!
            self.cur = self.list.front;
            self.index = Some(0)
        } else {
            // We're at the ghost, but that's the only element... do nothing.
        }
    }
}
```

So there's 4 interesting cases:

* The normal case
* The normal case, but we reach the ghost
* The ghost case, where we go to the front of the list
* The ghost case, but the list is empty, so do nothing

move_prev is the exact same logic, but with front/back inverted and the indexing changes inverted:

```rust ,ignore
pub fn move_prev(&mut self) {
    if let Some(cur) = self.cur {
        unsafe {
            // We're on a real element, go to its previous (front)
            self.cur = (*cur.as_ptr()).front;
            if self.cur.is_some() {
                *self.index.as_mut().unwrap() -= 1;
            } else {
                // We just walked to the ghost, no more index
                self.index = None;
            }
        }
    } else if !self.list.is_empty() {
        // We're at the ghost, and there is a real back, so move to it!
        self.cur = self.list.back;
        self.index = Some(self.list.len - 1)
    } else {
        // We're at the ghost, but that's the only element... do nothing.
    }
}
```

Next let's add some methods to look at the elements around the cursor: current, peek_next, and peek_prev. **A Very Important Note:** these methods must borrow our cursor by `&mut self`, and the results must be tied to that borrow. We cannot let the user get multiple copies of a mutable reference, and we cannot let them use any of our insert/remove/split/splice APIs while holding onto such a reference!

Thankfully, this is the default assumption rust makes when you use lifetime elision, so, we will just do the right thing by default!

```rust ,ignore
pub fn current(&mut self) -> Option<&mut T> {
    unsafe {
        self.cur.map(|node| &mut (*node.as_ptr()).elem)
    }
}

pub fn peek_next(&mut self) -> Option<&mut T> {
    unsafe {
        self.cur
            .and_then(|node| (*node.as_ptr()).back)
            .map(|node| &mut (*node.as_ptr()).elem)
    }
}

pub fn peek_prev(&mut self) -> Option<&mut T> {
    unsafe {
        self.cur
            .and_then(|node| (*node.as_ptr()).front)
            .map(|node| &mut (*node.as_ptr()).elem)
    }
}
```

Head empty, Option methods and (omitted) compiler errors do all thinking now. I was skeptical about the `Option<NonNull>` stuff, but, god damn it really just lets me autopilot this code. I've spent way too much time writing array-based collections where you never get to use Option, wow this is nice! (`(*node.as_ptr())` is still miserable but, that's just Rust's raw pointers for you...)

Next we have a choice: we can either jump right to split and splice, the entire point of these APIs, or we can take a baby-step with single element insert/remove. I have a feeling we're just going to want to implement insert/remove in terms of split and splice so... let's just do those first and see where the cards fall (genuinely have no idea as I type this).




# Split

First up, split_before and split_after, which return everything before/after the current element as a LinkedList (stopping at the ghost element, unless you're at the ghost, in which case we just return the whole List and the cursor now points to an empty list):

*squints* ok this one is actually some non-trivial logic so we're going to have to talk it out one step at a time.

I see 4 potentially interesting cases for split_before:

* The normal case
* The normal case, but prev is the ghost
* The ghost case, where we return the whole list and become empty
* The ghost case, but the list is empty, so do nothing and return the empty list

Let's start with the corner cases. The third case I believe is just

```rust
mem::replace(self.list, LinkedList::new())
```

Right? We become empty, we return the whole list, and our fields were already None, so nothing to update. Nice. Oh hey, this also Does The Right Thing on the fourth case too!

So now the normal cases... ok I'm going to need some ASCII diagrams for this. In the most general case, we have something like this:

```text
list.front -> A <-> B <-> C <-> D <- list.back
                          ^
                         cur
```

And we want to produce this:

```text
list.front -> C <-> D <- list.back
              ^
             cur

return.front -> A <-> B <- return.back
```

So we need to break the link between cur and prev, and... god so much needs to change. Ok I just need to break this up into steps so I can convince myself it makes sense. This will be a bit over-verbose but I can at least make sense of it:

```rust ,ignore
pub fn split_before(&mut self) -> LinkedList<T> {
    if let Some(cur) = self.cur {
        // We are pointing at a real element, so the list is non-empty.
        unsafe {
            // Current state
            let old_len = self.list.len;
            let old_idx = self.index.unwrap();
            let prev = (*cur.as_ptr()).front;
            
            // What self will become
            let new_len = old_len - old_idx;
            let new_front = self.cur;
            let new_back = self.list.back;
            let new_idx = Some(0);

            // What the output will become
            let output_len = old_len - new_len;
            let mut output_front = self.list.front;
            let output_back = prev;

            // Break the links between cur and prev
            if let Some(prev) = prev {
                (*cur.as_ptr()).front = None;
                (*prev.as_ptr()).back = None;
            } else {
                // We're at the first node, need to unset the head we "optimistically" set before.
                output_front = None;
            }

            // Produce the result:
            self.list.len = new_len;
            self.list.front = new_front;
            self.list.back = new_back;
            self.index = new_idx;

            LinkedList {
                front: output_front,
                back: output_back,
                len: output_len,
                _boo: PhantomData,
            }
        }
    } else {
        // We're at the ghost, just replace our list with an empty one.
        // No other state needs to be changed.
        std::mem::replace(self.list, LinkedList::new())
    }
}
```

Note that this if-let is handling the "normal case, but prev is the ghost" situation:

```rust ,ignore
if let Some(prev) = prev {
    (*cur.as_ptr()).front = None;
    (*prev.as_ptr()).back = None;
} else {
    output_front = None;
}
```

If *you* want to, you can squash that all together and apply optimizations like:

* fold the two accesses to `(*cur.as_ptr()).front` as just `(*cur.as_ptr()).front.take()` 
* note that new_back is a noop, and just remove both

As far as I can tell, everything else just incidentally Does The Right Thing otherwise. We'll see when we write tests! (copy-paste to make split_after)

I am done Making Mistakes and I am just going to try to write the most foolproof code I can. This is how I *actually* write collections: just break things down into trivial steps and cases until it can fit in my head and seems foolproof. Then write a ton of tests until I'm convinced I didn't manage to mess it up still.

Because most of the collections work I've done is *extremely unsafe* I don't generally get to rely on the compiler catching mistakes, and miri didn't exist back in the day! So I just need to squint at a problem until my head hurts and try my hardest to Never Ever Ever Make A Mistake.

Don't write Unsafe Rust Code! Safe Rust is so much better!!!!




# Splice

Just one more boss to fight, splice_before and splice_after, which I expect to be the corner-casiest one of them all. The two functions *take in* a LinkedList and grafts its contents into outrs. Our list could be empty, their list could be empty, we've got ghosts to deal with... *sigh* let's just take it one step at a time with splice_before.

* If their list is empty, we don't need to do anything. 
* If our list is empty, then our list just becomes their list.
* If we're pointing at the ghost, then this appends to the back (change list.back)
* If we're pointing at the first element (0), then this appends to the front (change list.front)
* In the general case, we do a whole lot of pointer fuckery.

The general case is this:

```text
input.front -> 1 <-> 2 <- input.back

 list.front -> A <-> B <-> C <- list.back
                     ^
                    cur
```

Becoming this:

```text
list.front -> A <-> 1 <-> 2 <-> B <-> C <- list.back
```

Ok? Ok. Let's write that out... *TAKES A HUGE BREATH AND PLUNGES IN*:

```rust ,ignore
    pub fn splice_before(&mut self, mut input: LinkedList<T>) {
        unsafe {
            if input.is_empty() {
                // Input is empty, do nothing.
            } else if let Some(cur) = self.cur {
                if let Some(0) = self.index {
                    // We're appending to the front, see append to back
                    (*cur.as_ptr()).front = input.back.take();
                    (*input.back.unwrap().as_ptr()).back = Some(cur);
                    self.list.front = input.front.take();

                    // Index moves forward by input length
                    *self.index.as_mut().unwrap() += input.len;
                    self.list.len += input.len;
                    input.len = 0;
                } else {
                    // General Case, no boundaries, just internal fixups
                    let prev = (*cur.as_ptr()).front.unwrap();
                    let in_front = input.front.take().unwrap();
                    let in_back = input.back.take().unwrap();

                    (*prev.as_ptr()).back = Some(in_front);
                    (*in_front.as_ptr()).front = Some(prev);
                    (*cur.as_ptr()).front = Some(in_back);
                    (*in_back.as_ptr()).back = Some(cur);

                    // Index moves forward by input length
                    *self.index.as_mut().unwrap() += input.len;
                    self.list.len += input.len;
                    input.len = 0;
                }
            } else if let Some(back) = self.list.back {
                // We're on the ghost but non-empty, append to the back
                // We can either `take` the input's pointers or `mem::forget`
                // it. Using take is more responsible in case we do custom
                // allocators or something that also needs to be cleaned up!
                (*back.as_ptr()).back = input.front.take();
                (*input.front.unwrap().as_ptr()).front = Some(back);
                self.list.back = input.back.take();
                self.list.len += input.len;
                // Not necessary but Polite To Do
                input.len = 0;
            } else {
                // We're empty, become the input, remain on the ghost
                *self.list = input;
            }
        }
    }
```

Ok this one is genuinely horrendous, and really is feeling that `Option<NonNull>` pain now. But there's a lot of cleanups we can do. For one, we can pull this code out to the very end, because we always want to do it. I don't *love*  (although sometimes it's a noop, and setting `input.len` is more a matter of paranoia about future extensions to the code):

```rust ,ignore
self.list.len += input.len;
input.len = 0;
```

> Use of moved value: `input`

Ah, right, in the "we're empty" case we're moving the list. Let's replace that with a swap:

```rust ,ignore
// We're empty, become the input, remain on the ghost
std::mem::swap(self.list, &mut input);
```

In this case the writes will be pointless, but, they still work (we could probably also early-return in this branch to appease the compiler).

This unwrap is just a consequence of me thinking about the cases backwards, and can be fixed by making the if-let ask the right question:

```rust ,ignore
if let Some(0) = self.index {

} else {
    let prev = (*cur.as_ptr()).front.unwrap();
}
```

Adjusting the index is duplicated inside the branches, so can also be hoisted out:

```rust
*self.index.as_mut().unwrap() += input.len;
```

Ok, putting that all together we get this:

```rust
if input.is_empty() {
    // Input is empty, do nothing.
} else if let Some(cur) = self.cur {
    // Both lists are non-empty
    if let Some(prev) = (*cur.as_ptr()).front {
        // General Case, no boundaries, just internal fixups
        let in_front = input.front.take().unwrap();
        let in_back = input.back.take().unwrap();

        (*prev.as_ptr()).back = Some(in_front);
        (*in_front.as_ptr()).front = Some(prev);
        (*cur.as_ptr()).front = Some(in_back);
        (*in_back.as_ptr()).back = Some(cur);
    } else {
        // We're appending to the front, see append to back below
        (*cur.as_ptr()).front = input.back.take();
        (*input.back.unwrap().as_ptr()).back = Some(cur);
        self.list.front = input.front.take();
    }
    // Index moves forward by input length
    *self.index.as_mut().unwrap() += input.len;
} else if let Some(back) = self.list.back {
    // We're on the ghost but non-empty, append to the back
    // We can either `take` the input's pointers or `mem::forget`
    // it. Using take is more responsible in case we do custom
    // allocators or something that also needs to be cleaned up!
    (*back.as_ptr()).back = input.front.take();
    (*input.front.unwrap().as_ptr()).front = Some(back);
    self.list.back = input.back.take();

} else {
    // We're empty, become the input, remain on the ghost
    std::mem::swap(self.list, &mut input);
}

self.list.len += input.len;
// Not necessary but Polite To Do
input.len = 0;

// Input dropped here
```

Alright this still sucks, but mostly because of -- nope ok just spotted a bug:

```rust
    (*back.as_ptr()).back = input.front.take();
    (*input.front.unwrap().as_ptr()).front = Some(back);
```

We `take` input.front and then unwrap it on the next line! *sigh* and we do the same thing in the equivalent mirror case. We would have caught this instantly in tests, but, we're trying to be Perfect now, and I'm just kinda doing this live, and this is the exact moment where I saw it. This is what I get for not being my usual tedious self and doing things in phases. More explicit!

```rust
// We can either `take` the input's pointers or `mem::forget`
// it. Using `take` is more responsible in case we ever do custom
// allocators or something that also needs to be cleaned up!
if input.is_empty() {
    // Input is empty, do nothing.
} else if let Some(cur) = self.cur {
    // Both lists are non-empty
    let in_front = input.front.take().unwrap();
    let in_back = input.back.take().unwrap();

    if let Some(prev) = (*cur.as_ptr()).front {
        // General Case, no boundaries, just internal fixups
        (*prev.as_ptr()).back = Some(in_front);
        (*in_front.as_ptr()).front = Some(prev);
        (*cur.as_ptr()).front = Some(in_back);
        (*in_back.as_ptr()).back = Some(cur);
    } else {
        // No prev, we're appending to the front
        (*cur.as_ptr()).front = Some(in_back);
        (*in_back.as_ptr()).back = Some(cur);
        self.list.front = Some(in_front);
    }
    // Index moves forward by input length
    *self.index.as_mut().unwrap() += input.len;
} else if let Some(back) = self.list.back {
    // We're on the ghost but non-empty, append to the back
    let in_front = input.front.take().unwrap();
    let in_back = input.back.take().unwrap();

    (*back.as_ptr()).back = Some(in_front);
    (*in_front.as_ptr()).front = Some(back);
    self.list.back = Some(in_back);
} else {
    // We're empty, become the input, remain on the ghost
    std::mem::swap(self.list, &mut input);
}

self.list.len += input.len;
// Not necessary but Polite To Do
input.len = 0;

// Input dropped here
```

Alright now this, this I can tolerate. The only complaints I have are that we don't dedupe in_front/in_back (probably we could rejig our conditions but eh whatever). Really this is basically what you would write in C but with `Option<NonNull>` gunk making it tedious. I can live with that. Well no we should just make raw pointers better for this stuff. But, out of scope for this book.

Anyway, I am absolutely exhausted after that, so, `insert` and `remove` and all the other APIs can be left as an excercise to the reader. 

Here's the final code for our Cursor with my attempt at copy-pasting the combinatorics. Did I get it right? I'll only find out when I write the next chapter and test this monstrosity!


```rust ,ignore
pub struct CursorMut<'a, T> {
    list: &'a mut LinkedList<T>,
    cur: Link<T>,
    index: Option<usize>,
}

impl<T> LinkedList<T> {
    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        CursorMut { 
            list: self, 
            cur: None, 
            index: None,
        }
    }
}

impl<'a, T> CursorMut<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn move_next(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                // We're on a real element, go to its next (back)
                self.cur = (*cur.as_ptr()).back;
                if self.cur.is_some() {
                    *self.index.as_mut().unwrap() += 1;
                } else {
                    // We just walked to the ghost, no more index
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            // We're at the ghost, and there is a real front, so move to it!
            self.cur = self.list.front;
            self.index = Some(0)
        } else {
            // We're at the ghost, but that's the only element... do nothing.
        }
    }

    pub fn move_prev(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                // We're on a real element, go to its previous (front)
                self.cur = (*cur.as_ptr()).front;
                if self.cur.is_some() {
                    *self.index.as_mut().unwrap() -= 1;
                } else {
                    // We just walked to the ghost, no more index
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            // We're at the ghost, and there is a real back, so move to it!
            self.cur = self.list.back;
            self.index = Some(self.list.len - 1)
        } else {
            // We're at the ghost, but that's the only element... do nothing.
        }
    }

    pub fn current(&mut self) -> Option<&mut T> {
        unsafe {
            self.cur.map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn peek_next(&mut self) -> Option<&mut T> {
        unsafe {
            self.cur
                .and_then(|node| (*node.as_ptr()).back)
                .map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        unsafe {
            self.cur
                .and_then(|node| (*node.as_ptr()).front)
                .map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn split_before(&mut self) -> LinkedList<T> {
        // We have this:
        //
        //     list.front -> A <-> B <-> C <-> D <- list.back
        //                               ^
        //                              cur
        // 
        //
        // And we want to produce this:
        // 
        //     list.front -> C <-> D <- list.back
        //                   ^
        //                  cur
        //
        // 
        //    return.front -> A <-> B <- return.back
        //
        if let Some(cur) = self.cur {
            // We are pointing at a real element, so the list is non-empty.
            unsafe {
                // Current state
                let old_len = self.list.len;
                let old_idx = self.index.unwrap();
                let prev = (*cur.as_ptr()).front;
                
                // What self will become
                let new_len = old_len - old_idx;
                let new_front = self.cur;
                let new_back = self.list.back;
                let new_idx = Some(0);

                // What the output will become
                let output_len = old_len - new_len;
                let mut output_front = self.list.front;
                let output_back = prev;

                // Break the links between cur and prev
                if let Some(prev) = prev {
                    (*cur.as_ptr()).front = None;
                    (*prev.as_ptr()).back = None;
                } else {
                    // We're at the first node, need to unset the head we "optimistically" set before.
                    output_front = None;
                }

                // Produce the result:
                self.list.len = new_len;
                self.list.front = new_front;
                self.list.back = new_back;
                self.index = new_idx;

                LinkedList {
                    front: output_front,
                    back: output_back,
                    len: output_len,
                    _boo: PhantomData,
                }
            }
        } else {
            // We're at the ghost, just replace our list with an empty one.
            // No other state needs to be changed.
            std::mem::replace(self.list, LinkedList::new())
        }
    }

    pub fn split_after(&mut self) -> LinkedList<T> {
        // We have this:
        //
        //     list.front -> A <-> B <-> C <-> D <- list.back
        //                         ^
        //                        cur
        // 
        //
        // And we want to produce this:
        // 
        //     list.front -> A <-> B <- list.back
        //                         ^
        //                        cur
        //
        // 
        //    return.front -> C <-> D <- return.back
        //
        if let Some(cur) = self.cur {
            // We are pointing at a real element, so the list is non-empty.
            unsafe {
                // Current state
                let old_len = self.list.len;
                let old_idx = self.index.unwrap();
                let next = (*cur.as_ptr()).back;
                
                // What self will become
                let new_len = old_idx + 1;
                let new_back = self.cur;
                let new_front = self.list.front;
                let new_idx = Some(old_idx);

                // What the output will become
                let output_len = old_len - new_len;
                let output_front = next;
                let mut output_back = self.list.back;

                // Break the links between cur and next
                if let Some(next) = next {
                    (*cur.as_ptr()).back = None;
                    (*next.as_ptr()).front = None;
                } else {
                    // We're at the first node, need to unset the tail we "optimistically" set before.
                    output_back = None;
                }

                // Produce the result:
                self.list.len = new_len;
                self.list.front = new_front;
                self.list.back = new_back;
                self.index = new_idx;

                LinkedList {
                    front: output_front,
                    back: output_back,
                    len: output_len,
                    _boo: PhantomData,
                }
            }
        } else {
            // We're at the ghost, just replace our list with an empty one.
            // No other state needs to be changed.
            std::mem::replace(self.list, LinkedList::new())
        }
    }

    pub fn splice_before(&mut self, mut input: LinkedList<T>) {
        // We have this:
        //
        // input.front -> 1 <-> 2 <- input.back
        //
        // list.front -> A <-> B <-> C <- list.back
        //                     ^
        //                    cur
        //
        //
        // Becoming this:
        //
        // list.front -> A <-> 1 <-> 2 <-> B <-> C <- list.back
        //                                 ^
        //                                cur
        //
        unsafe {
            // We can either `take` the input's pointers or `mem::forget`
            // it. Using `take` is more responsible in case we ever do custom
            // allocators or something that also needs to be cleaned up!
            if input.is_empty() {
                // Input is empty, do nothing.
            } else if let Some(cur) = self.cur {
                // Both lists are non-empty
                let in_front = input.front.take().unwrap();
                let in_back = input.back.take().unwrap();

                if let Some(prev) = (*cur.as_ptr()).front {
                    // General Case, no boundaries, just internal fixups
                    (*prev.as_ptr()).back = Some(in_front);
                    (*in_front.as_ptr()).front = Some(prev);
                    (*cur.as_ptr()).front = Some(in_back);
                    (*in_back.as_ptr()).back = Some(cur);
                } else {
                    // No prev, we're appending to the front
                    (*cur.as_ptr()).front = Some(in_back);
                    (*in_back.as_ptr()).back = Some(cur);
                    self.list.front = Some(in_front);
                }
                // Index moves forward by input length
                *self.index.as_mut().unwrap() += input.len;
            } else if let Some(back) = self.list.back {
                // We're on the ghost but non-empty, append to the back
                let in_front = input.front.take().unwrap();
                let in_back = input.back.take().unwrap();

                (*back.as_ptr()).back = Some(in_front);
                (*in_front.as_ptr()).front = Some(back);
                self.list.back = Some(in_back);
            } else {
                // We're empty, become the input, remain on the ghost
                std::mem::swap(self.list, &mut input);
            }

            self.list.len += input.len;
            // Not necessary but Polite To Do
            input.len = 0;
            
            // Input dropped here
        }        
    }

    pub fn splice_after(&mut self, mut input: LinkedList<T>) {
        // We have this:
        //
        // input.front -> 1 <-> 2 <- input.back
        //
        // list.front -> A <-> B <-> C <- list.back
        //                     ^
        //                    cur
        //
        //
        // Becoming this:
        //
        // list.front -> A <-> B <-> 1 <-> 2 <-> C <- list.back
        //                     ^
        //                    cur
        //
        unsafe {
            // We can either `take` the input's pointers or `mem::forget`
            // it. Using `take` is more responsible in case we ever do custom
            // allocators or something that also needs to be cleaned up!
            if input.is_empty() {
                // Input is empty, do nothing.
            } else if let Some(cur) = self.cur {
                // Both lists are non-empty
                let in_front = input.front.take().unwrap();
                let in_back = input.back.take().unwrap();

                if let Some(next) = (*cur.as_ptr()).back {
                    // General Case, no boundaries, just internal fixups
                    (*next.as_ptr()).front = Some(in_back);
                    (*in_back.as_ptr()).back = Some(next);
                    (*cur.as_ptr()).back = Some(in_front);
                    (*in_front.as_ptr()).front = Some(cur);
                } else {
                    // No next, we're appending to the back
                    (*cur.as_ptr()).back = Some(in_front);
                    (*in_front.as_ptr()).front = Some(cur);
                    self.list.back = Some(in_back);
                }
                // Index doesn't change
            } else if let Some(front) = self.list.front {
                // We're on the ghost but non-empty, append to the front
                let in_front = input.front.take().unwrap();
                let in_back = input.back.take().unwrap();

                (*front.as_ptr()).front = Some(in_back);
                (*in_back.as_ptr()).back = Some(front);
                self.list.front = Some(in_front);
            } else {
                // We're empty, become the input, remain on the ghost
                std::mem::swap(self.list, &mut input);
            }

            self.list.len += input.len;
            // Not necessary but Polite To Do
            input.len = 0;
            
            // Input dropped here
        }        
    }
}
```