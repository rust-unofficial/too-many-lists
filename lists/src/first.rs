use std::mem;

pub struct List<T> {
	head: Link<T>,
}

enum Link<T> {
	Empty,
	More(Box<Node<T>>),
}

struct Node<T> {
	elem: T,
	next: Link<T>,
}

impl<T> List<T> {
	pub fn push(&mut self, elem: T) {
		let new_node = Box::new(Node {
			elem: elem,
			next: mem::replace(&mut self.head, Link::Empty),
		});

		self.head = Link::More(new_node);
	}

	pub fn pop(&mut self) -> Option<T> {
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
}
