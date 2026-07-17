#![no_std]
//! `tpt-zero-intrusive`: an intrusive doubly-linked list.
//!
//! Elements opt into the list by embedding a [`Link<T>`] field and implementing
//! [`IntrusiveNode`]. The list stores `NonNull<T>` to the nodes and threads
//! the embedded [`Link<T>`]s, so it adds **zero** extra allocation per node.
//! Insertion/removal are O(1) and never allocate.
//!
//! # Safety invariants
//!
//! - A [`Link<T>`] may be part of **at most one** intrusive list at a time.
//! - A linked node must not be moved or dropped except through the list's
//!   removal API, or [`List`]'s `Drop`.
//! - All `NonNull<T>` pointers remain valid for the list's lifetime.
//!
//! Misuse (e.g. linking the same node into two lists) is **undefined
//! behaviour**.

use core::marker::PhantomData;
use core::ptr::NonNull;

/// A link embedded in a list element of type `T`.
///
/// Holds `NonNull<T>` to the previous/next nodes; recovering the node from a
/// neighbour is just a dereference (no field-offset arithmetic needed).
#[derive(Clone, Copy, Debug)]
pub struct Link<T> {
    prev: Option<NonNull<T>>,
    next: Option<NonNull<T>>,
}

impl<T> Link<T> {
    /// Create a link that is not part of any list.
    pub const fn new() -> Self {
        Link {
            prev: None,
            next: None,
        }
    }

    /// Whether this link is currently part of a list.
    pub fn is_linked(&self) -> bool {
        self.prev.is_some() || self.next.is_some()
    }

    fn unlink(&mut self) {
        self.prev = None;
        self.next = None;
    }
}

impl<T> Default for Link<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps a node `&T`/`&mut T` to its embedded [`Link<T>`].
///
/// # Safety
///
/// `link`/`link_mut` must return a reference to the **same** embedded field
/// for the lifetime of the node.
pub trait IntrusiveNode: Sized {
    /// Reference to the embedded link.
    fn link(&self) -> &Link<Self>;
    /// Mutable reference to the embedded link.
    fn link_mut(&mut self) -> &mut Link<Self>;
}

/// An intrusive doubly-linked list over `T: IntrusiveNode`.
pub struct List<T: IntrusiveNode> {
    head: Option<NonNull<T>>,
    tail: Option<NonNull<T>>,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T: IntrusiveNode> List<T> {
    /// Create an empty list.
    pub const fn new() -> Self {
        List {
            head: None,
            tail: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Number of nodes.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the list is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Push `node` to the front. Its link must not already be in a list.
    ///
    /// # Safety
    ///
    /// `node` must outlive the list entry and `node.link_mut()` must be unlinked.
    pub unsafe fn push_front(&mut self, node: &mut T) {
        let node_ptr = NonNull::from(&mut *node);
        let link = node.link_mut();
        debug_assert!(!link.is_linked(), "node already linked");
        link.prev = None;
        link.next = self.head;
        if let Some(h) = self.head {
            unsafe { (*h.as_ptr()).link_mut().prev = Some(node_ptr) };
        } else {
            self.tail = Some(node_ptr);
        }
        self.head = Some(node_ptr);
        self.len += 1;
    }

    /// Push `node` to the back.
    ///
    /// # Safety
    ///
    /// `node` must outlive the list entry and `node.link_mut()` must be
    /// unlinked. See [`push_front`](Self::push_front) for the full safety
    /// contract.
    pub unsafe fn push_back(&mut self, node: &mut T) {
        let node_ptr = NonNull::from(&mut *node);
        let link = node.link_mut();
        debug_assert!(!link.is_linked(), "node already linked");
        link.prev = self.tail;
        link.next = None;
        if let Some(t) = self.tail {
            unsafe { (*t.as_ptr()).link_mut().next = Some(node_ptr) };
        } else {
            self.head = Some(node_ptr);
        }
        self.tail = Some(node_ptr);
        self.len += 1;
    }

    /// Remove and return the front node, or `None`.
    pub fn pop_front(&mut self) -> Option<&mut T> {
        let head = self.head?;
        let node = unsafe { &mut *head.as_ptr() };
        let link = node.link_mut();
        self.head = link.next;
        if let Some(n) = link.next {
            unsafe { (*n.as_ptr()).link_mut().prev = None };
        } else {
            self.tail = None;
        }
        link.unlink();
        self.len -= 1;
        Some(node)
    }

    /// Remove and return the back node, or `None`.
    pub fn pop_back(&mut self) -> Option<&mut T> {
        let tail = self.tail?;
        let node = unsafe { &mut *tail.as_ptr() };
        let link = node.link_mut();
        self.tail = link.prev;
        if let Some(p) = link.prev {
            unsafe { (*p.as_ptr()).link_mut().next = None };
        } else {
            self.head = None;
        }
        link.unlink();
        self.len -= 1;
        Some(node)
    }

    /// Iterate over node references, front-to-back.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: self.head,
            _marker: PhantomData,
        }
    }
}

impl<T: IntrusiveNode> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: IntrusiveNode> Drop for List<T> {
    fn drop(&mut self) {
        // Detach every link so nodes are no longer "linked". We don't own the
        // nodes, so only reset their link fields.
        let mut cur = self.head;
        while let Some(p) = cur {
            let node = unsafe { &mut *p.as_ptr() };
            let next = node.link().next;
            node.link_mut().unlink();
            cur = next;
        }
        self.head = None;
        self.tail = None;
        self.len = 0;
    }
}

/// Immutable iterator over a [`List`].
pub struct Iter<'a, T: IntrusiveNode> {
    current: Option<NonNull<T>>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: IntrusiveNode> Iterator for Iter<'a, T> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        let p = self.current?;
        let node = unsafe { &*p.as_ptr() };
        self.current = node.link().next;
        Some(node)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;

    struct Node {
        link: Link<Node>,
        value: i32,
    }
    impl IntrusiveNode for Node {
        fn link(&self) -> &Link<Self> {
            &self.link
        }
        fn link_mut(&mut self) -> &mut Link<Self> {
            &mut self.link
        }
    }

    #[test]
    fn push_pop() {
        let mut a = Node {
            link: Link::new(),
            value: 1,
        };
        let mut b = Node {
            link: Link::new(),
            value: 2,
        };
        let mut c = Node {
            link: Link::new(),
            value: 3,
        };
        let mut list: List<Node> = List::new();
        assert!(list.is_empty());
        unsafe {
            list.push_back(&mut a);
            list.push_back(&mut b);
            list.push_front(&mut c);
        }
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front().unwrap().value, 3);
        assert_eq!(list.pop_back().unwrap().value, 2);
        assert_eq!(list.pop_front().unwrap().value, 1);
        assert!(list.pop_front().is_none());
    }

    #[test]
    fn iter_front_to_back() {
        let mut nodes = alloc::vec![
            Node {
                link: Link::new(),
                value: 10
            },
            Node {
                link: Link::new(),
                value: 20
            },
            Node {
                link: Link::new(),
                value: 30
            },
        ];
        let mut list: List<Node> = List::new();
        unsafe {
            list.push_back(&mut nodes[0]);
            list.push_back(&mut nodes[1]);
            list.push_back(&mut nodes[2]);
        }
        let values: alloc::vec::Vec<i32> = list.iter().map(|n| n.value).collect();
        assert_eq!(values, alloc::vec![10, 20, 30]);
    }

    #[test]
    fn drop_detaches() {
        let mut nodes = alloc::vec![
            Node {
                link: Link::new(),
                value: 1
            },
            Node {
                link: Link::new(),
                value: 2
            },
        ];
        {
            let mut list: List<Node> = List::new();
            unsafe {
                list.push_back(&mut nodes[0]);
                list.push_back(&mut nodes[1]);
            }
            // list dropped here; links must be detached so re-inserting works.
        }
        assert!(!nodes[0].link.is_linked());
        assert!(!nodes[1].link.is_linked());
        let mut list2: List<Node> = List::new();
        unsafe {
            list2.push_back(&mut nodes[0]);
        }
        assert_eq!(list2.len(), 1);
    }
}

#[cfg(test)]
mod proptests {
    extern crate alloc;
    use super::*;
    use proptest::prelude::*;

    #[derive(Debug)]
    struct N {
        link: Link<N>,
        value: u32,
    }
    impl IntrusiveNode for N {
        fn link(&self) -> &Link<Self> {
            &self.link
        }
        fn link_mut(&mut self) -> &mut Link<Self> {
            &mut self.link
        }
    }

    proptest! {
        /// Front-to-back iteration after a sequence of pushes matches the order.
        #[test]
        fn push_sequence_matches(values in proptest::collection::vec(0..1000u32, 1..64)) {
            let expected: alloc::vec::Vec<u32> = values.clone();
            let mut pool: alloc::vec::Vec<N> = expected
                .iter()
                .map(|&v| N { link: Link::new(), value: v })
                .collect();
            let mut list: List<N> = List::new();
            unsafe {
                for n in pool.iter_mut() {
                    list.push_back(n);
                }
            }
            let out: alloc::vec::Vec<u32> = list.iter().map(|n| n.value).collect();
            prop_assert_eq!(out, expected.clone());
            prop_assert_eq!(list.len(), expected.len());
            // The list only borrows the nodes via raw pointers; drop it
            // *before* the pool so its `Drop` (which walks the node links)
            // never touches freed memory.
            drop(list);
            drop(pool);
        }
    }
}
