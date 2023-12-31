#![allow(non_snake_case, unused)]

// NOTE - In a HashMap, bins are the individual linked lists that store entries. Each bin is
// identified by a hash code generated from the key of the entry.
//
// When a new entry is added to the HashMap, its hash code is used to determine which bin it should
// be placed in. If there is already an entry in that bin with the same hash code, the equals( )
// method is used to check if the keys are equal.
// |
// |- If the keys are equal, the value of the existing entry is overwritten with the new value.
// |
// |- Otherwise the new entry is added to the end of the linked list in that bin.
/*
  CONCURRENT HASHMAP :

  A hash table which with full concurrency for retrievals (read operations will never be blocked)
  and high expected concurrency for updates (write operations maybe be blocked sometimes).
  Retrievals reflect the results of the most recently completed update operations holding upon their
  onset.

  Iterators/Enumerators return elements reflecting the state of the hash table at some point at or
  since the creation of the iterator/enumeration. However, iterators are designed to be used by only
  one thread at a time.

  The table is dynamically expanded when there are too many collisions (size of bins are not small),
  with the expected average effect of maintaining roughly two bins per mapping. Resizing this or any
  other kind of hash table may be a relatively slow operation.
*/

mod node {
  use std::{cell::UnsafeCell, sync::atomic::Ordering};
  use crossbeam::epoch::{Atomic, Guard, Shared};

  pub(crate) enum BinItem<K, V> {
    Node(Node<K, V>)
  }

  impl<K, V> BinItem<K, V>
    where K: Eq
  {
    pub(crate) fn find<'g>(&'g self, hash: u64, key: &K, guard: &'g Guard) -> Option<Shared<'g, Node<K, V>>> {
      match *self {
        BinItem::Node(ref startingNode) => {
          if startingNode.hash == hash && startingNode.key == *key {
            return Some(Shared::from(startingNode as *const _))}

          todo!( )
        }
      }
    }
  }

  pub(crate) struct Node<K, V> {
    pub(crate) hash: u64,

    pub(crate) key: K,
    pub(crate) value: UnsafeCell<V>,  // The value is stored in UnsafeCell, since it can be updated in the
                                      // future.
    pub(crate) next: Atomic<Node<K, V>>
  }
}