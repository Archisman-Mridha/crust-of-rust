#![allow(non_snake_case, unused)]

/*
  Marker traits - traits that don't require any methods to be implemented. Instead, they are used to
  add metadata / indicate certain properties about the types that implement them.

  1. Send - used to indicate that it is safe to transfer ownership of the implementing type across
     thread boundaries.

  2. Sync - used to indicate that it is safe for multiple threads to have references to the
     implementing type concurrently.

  The 'Send' and 'Sync' traits are marker traits. Also, they are 'auto' traits. The compiler will
  automatically implement them for a type, if all members of that type already implement them.

  NOTE - Most but not all marker traits are auto traits.
*/

// Rc and Mutex guards (that's because, in some Operating systems, it is a requirement that only the
// thread that acquired a mutex guard can unlock the mutex guard.) are not Send.
// Any type which requires access to thread locals, cannot be Send.

// A type T is Sync if and only if &T is Send. There are some values of T, for which &T is Send but
// T is not Sync. For example - Cell and RefCell.
//
// NOTE - Since Rc gives out references to its inner value, it is not Send. But Cell / RefCell
// doesn't give out reference to their inner values, thats'y they are Send.