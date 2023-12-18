#![allow(non_snake_case, unused, non_camel_case_types)]

// NOTE - '&*' is used to cast a raw pointer to a reference pointer (which is safe and bound by the
// ownership rules of Rust).

/*
  Rust memory safety is based on this rule:
  Given an object T, it is only possible to have one of the following:
  |
  |- 1. Having several immutable references (&T) to the object (also known as ALIASING).
  |
  |- 2. Having one mutable reference (&mut T) to the object (also known as MUTABILITY).
  |
  INTERIOR MUTABILITY - in Rust refers to the ability to mutate data even when there are immutable
  references to that data. It is a design pattern that allows you to mutate data through shared
  references (i.e., &T) to achieve mutable fields inside immutable data structures.
  Rust's borrowing rules normally disallow this action, but interior mutability provides a way to
  bend these rules safely. Rust provides several types that enable interior mutability, including
  Cell<T>, RefCell<T>, and OnceCell<T> . These types come with different trade-offs.
*/
mod cell {
  use std::cell::UnsafeCell;

  /*
    Cell<T> implements interior mutability by moving values in and out of the cell. That is, an
    '&mut T' to the inner value can never be obtained, and the value itself cannot be directly
    obtained without replacing it with something else.
    Both of these rules ensure that there is never more than one reference pointing to the inner
    value.
  */
  // NOTE - A thread safe version of Cell doesn't exist, since it's not okay for 2 threads trying
  // to mutate a value at the same time.
  pub struct Cell<T> {

    /*
      All other types that allow internal mutability, such as Cell<T> and RefCell<T>,
      internally use UnsafeCell to wrap their data.
      The UnsafeCell API itself is technically very simple: .get( ) gives you a raw pointer *mut T
      to its contents. It is up to you as the abstraction designer to use that raw pointer correctly.

      NOTE - A raw pointer is a low-level pointer that is not subject to the ownership and borrowing
      rules enforced by the Rust compiler.
    */
    value: UnsafeCell<T>
  }

  // impl<T> !Sync for UnsafeCell<T> { } - means that an UnsafeCell can never be shared between
  // threads. Which implies that we can't do the same with a Cell too.

  impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
      Self {
        value: UnsafeCell::new(value)
      }
    }

    // set drops the older value and stores the new value in the cell.
    pub fn set(&self, value: T) {
      // This is now okay to do, since Cell can't be shared across threads.
      unsafe {
        *self.value.get( )= value;
      }
    }

    // get returns a copy of the value the Cell holds.
    /*
      The reason why we give out copies instead of reference is because we want to avoid this kind
      situation -
      
      let x= Cell::new(vec!{ 42 });
      let first= &x.get( )[0]; ---|
      x.set(vec!{ });             |
      println!("{}", first);     // Reference to 42 is invalid, since now 42 doesn't even exist
                                 // inside the cell.
    */
    pub fn get(&self) -> T where T: Copy {
      unsafe { *self.value.get( )}
    }
  }
}
pub use cell::*;

mod refCell {
  use std::{cell::UnsafeCell, ops::{Deref, DerefMut}};
  use crate::Cell;

  #[derive(PartialEq, Clone, Copy)]
  enum References {
    None,

    // Aliasing - Multiple immutable references of the value exist.
    Shared(usize),

    // A mutable reference of the value exists.
    Exclusive
  }

  /*
    RefCell<T> uses Rust’s lifetimes to implement 'dynamic borrowing', a process whereby one can
    claim temporary, exclusive, mutable access to the inner value. Borrows for RefCell<T>s are
    tracked at runtime.
  */
  // NOTE - The thread safe version of RefCell is RwLock.
  pub struct RefCell<T> {
    // Whenever this value will be borrowed, it will be first verified that Rust's ownership rules
    // are satisfied.
    value: UnsafeCell<T>,
    currentReferences: Cell<References>
  }

  impl<T> RefCell<T> {
    pub fn new(value: T) -> Self {
      Self {
        value: UnsafeCell::new(value), // Thus RefCell can also not be shared across threads.
        currentReferences: Cell::new(References::None)
      }
    }

    pub fn borrow(&self) -> Option<Ref<'_, T>> {
      let currentReferences= self.currentReferences.get( );

      if currentReferences == References::Exclusive {
        return None
      }

      self.currentReferences.set(
        match currentReferences {
          References::None => References::Shared(0),
          References::Shared(previousImmutableReferenceCount) => References::Shared(previousImmutableReferenceCount + 1),
  
          _ => unreachable!( )
        }
      );
      Some(Ref { refCell: self })
    }

    // borrowMut returns a mutable reference to the value. If immutable references of the value
    // already exists, then 'None' is returned.
    pub fn borrowMut(&self) -> Option<RefMut<'_, T>> {
      let currentReferences= self.currentReferences.get( );

      if currentReferences != References::None {
        return None
      }

      self.currentReferences.set(References::Exclusive);
      Some(RefMut { refCell: self })
    }
  }

  // Ref is a smart pointer wrapping an immutable reference to the value stored in the RefCell.
  pub struct Ref<'refCell, T> {
    refCell: &'refCell RefCell<T>
  }
  impl<T> Deref for Ref<'_, T> {
    type Target= T;

    fn deref(&self) -> &Self::Target {
      unsafe { &*self.refCell.value.get( )}
    }
  }
  // To update RefCell.currentReferences when the reference is dropped.
  impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
      self.refCell.currentReferences.set(
        match self.refCell.currentReferences.get( ) {
          References::Shared(1) => References::None,
          References::Shared(previousImmutableReferenceCount) => References::Shared(previousImmutableReferenceCount - 1),

          _ => unreachable!( )
        }
      );
    }
  }

  // RefMut is a smart pointer wrapping a mutable reference to the value stored in the RefCell.
  pub struct RefMut<'refCell, T> {
    refCell: &'refCell RefCell<T>
  }
  impl<T> Deref for RefMut<'_, T> {
    type Target= T;

    fn deref(&self) -> &Self::Target {
      unsafe { &*self.refCell.value.get( )}
    }
  }
  impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe { &mut *self.refCell.value.get( )}
    }
  }
  // To update RefCell.currentReferences when the reference is dropped.
  impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
      self.refCell.currentReferences.set(References::None)
    }
  }
}
pub use refCell::*;

mod rc {
  use std::{ops::Deref, ptr::NonNull, marker::PhantomData};
  use crate::Cell;

  struct _Rc<T> {
    value: T,
    refCount: Cell<usize>
  }

  /*
    Rc is a single-threaded reference-counting pointer that provides shared ownership of a value
    allocated in the heap. Rc uses non-atomic reference counting so the overhead is very low.

    NOTE - If you need mutability, put a Cell or RefCell inside the Rc.
  */
  // NOTE - The thread safe version of Rc is Arc.
  pub struct Rc<T> {
    _rc: NonNull<_Rc<T>>, // NonNull gives us a non-zero and covariant '*mut T'.

    // Relates to the 'drop check' concept in Rust.
    _marker: PhantomData<Rc<T>>
  }

  impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
      let _rc= Box::new(_Rc {
        value,
        refCount: Cell::new(0)
      });

      Rc {
        _rc: unsafe { NonNull::new_unchecked(Box::into_raw(_rc)) },
        _marker: PhantomData
      }
    }
  }

  impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
      let _rc= unsafe { self._rc.as_ref( ) };

      let previousRefCount= _rc.refCount.get( );
      _rc.refCount.set(previousRefCount + 1);

      Rc {
        _rc: self._rc,
        _marker: PhantomData
      }
    }
  }

  impl<T> Deref for Rc<T> {
    type Target= T;

    fn deref(&self) -> &Self::Target {
      &unsafe { self._rc.as_ref( ) }.value
    }
  }

  // To update _Rc.refCount when the reference is dropped.
  impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
      let _rc= unsafe { self._rc.as_ref( ) };

      match _rc.refCount.get( ) {
        // drop _Rc along with this Rc (which holds the last reference to _Rc).
        1 => {
          let _= _rc;
          let _= unsafe { Box::from_raw(self._rc.as_ptr( ))};
        },

        previousRefCount => _rc.refCount.set(previousRefCount - 1)
      }
    }
  }
}
pub use rc::*;

// Cow - The enum Cow is a smart pointer providing clone-on-write functionality: it can enclose and
// provide immutable access to borrowed data, and clone the data lazily when mutation or ownership
// is required.
// It is used when most of the times we want to read the data but rarely want to mutate it.