# tpt-zero-intrusive

An intrusive doubly-linked list for `#![no_std]` environments. Elements opt
into the list by embedding a `Link<T>` field and implementing `IntrusiveNode`,
so the list adds **zero** extra allocation per node and insertion/removal are
O(1).

```rust
use tpt_zero_intrusive::{List, Link, IntrusiveNode};

struct Node {
    link: Link<Node>,
    value: i32,
}
impl IntrusiveNode for Node {
    fn link(&self) -> &Link<Self> { &self.link }
    fn link_mut(&mut self) -> &mut Link<Self> { &mut self.link }
}

let mut a = Node { link: Link::new(), value: 1 };
let mut b = Node { link: Link::new(), value: 2 };
let mut list: List<Node> = List::new();
unsafe {
    list.push_back(&mut a);
    list.push_back(&mut b);
}
assert_eq!(list.len(), 2);
assert_eq!(list.pop_front().unwrap().value, 1);
```

## Safety invariants

This crate is built on `unsafe`. Misuse is **undefined behaviour**:

- A `Link<T>` may be part of **at most one** intrusive list at a time.
- A linked node must not be moved or dropped except through the list's
  removal API or `List`'s `Drop`.
- All `NonNull<T>` pointers must remain valid for the list's lifetime.

`push_front` / `push_back` are `unsafe` and `debug_assert!` that the node's
link is not already linked.

## `no_std`

`#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
