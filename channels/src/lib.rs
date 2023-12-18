#![allow(non_snake_case, unused)]

use std::{sync::{Arc, Mutex, Condvar}, collections::VecDeque, mem::swap};

/*
  NOTE :

  Mutex - allows multiple threads to access a shared resource while ensuring that only one thread
  can access it at a time.

  Arc<Mutex<T>> - Shared ownership of a resource across threads.
*/

pub struct Sender<T> {
  channel: Arc<Channel<T>>
}

impl<T> Sender<T> {
  pub fn send(&mut self, t: T) {
    let mut queueRefWithMutexLock= self.channel.queue.lock( ).unwrap( );
    queueRefWithMutexLock._queue.push_back(t);

    drop(queueRefWithMutexLock); // Dropping the mutex lock first, so that when the receiver wakes
                                 // up, it can immediately take the mutex lock.
    self.channel.condVar.notify_one( );
  }
}

impl<T> Clone for Sender<T> {
  fn clone(&self) -> Self {
    let mut queueRefWithMutexLock= self.channel.queue.lock( ).unwrap( );
    queueRefWithMutexLock.senderCount += 1;

    drop(queueRefWithMutexLock);

    Self {
      channel: Arc::clone(&self.channel)
    }
  }
}

impl<T> Drop for Sender<T> {
  fn drop(&mut self) {
    let mut queueRefWithMutexLock= self.channel.queue.lock( ).unwrap( );
    queueRefWithMutexLock.senderCount -= 1;

    let noSenders= (queueRefWithMutexLock.senderCount == 0);

    drop(queueRefWithMutexLock);

    if noSenders {
      self.channel.condVar.notify_one( )}
  }
}

pub struct Receiver<T> {
  channel: Arc<Channel<T>>,

  // Assuming that we only have 1 receiver, we do a little optimization 😉.
  cache: VecDeque<T>
}

impl<T> Receiver<T> {
  pub fn receive(&mut self) -> Option<T> {
    if let Some(t)= self.cache.pop_front( ) {
      return Some(t)}

    let mut queueRefWithMutexLock= self.channel.queue.lock( ).unwrap( );
    loop {
      match queueRefWithMutexLock._queue.pop_front( ) {
        Some(t) => {
          // Cache remaining items in the queue, so everytime we don't need to acquire mutex locks
          // everytime for them.
          if !queueRefWithMutexLock._queue.is_empty( ) {
            // swap swaps the values at two mutable locations without deinitializing either one.
            swap(&mut self.cache, &mut queueRefWithMutexLock._queue);}

          return Some(t)
        },

        // If there are 0 senders.
        None if queueRefWithMutexLock.senderCount == 0 => return None,
  
        // If there are no items available in the queue, then the OS makes the receiver thread go
        // back to sleep. It wakes up only when there are items in the queue to consume. The item
        // from the queue is then returned to the receiver in the 2nd iteration of the loop.
        // Before going to sleep, the receiver thread needs to give up the mutex lock, so that it
        // can be acquired by the sender. When the receiver wakes up, it takes back the mutex lock.
        None => queueRefWithMutexLock= self.channel.condVar.wait(queueRefWithMutexLock).unwrap( )
      }
    }
  }
}

struct Queue<T> {
  _queue: VecDeque<T>,
  senderCount: usize
}

struct Channel<T> {
  queue: Mutex<Queue<T>>,

  // Condition variables represent the ability to block a thread such that it consumes no CPU time
  // while waiting for an event to occur.
  condVar: Condvar
}

impl<T> Channel<T> {
  pub fn new( ) -> (Sender<T>, Receiver<T>) {
    let channel= Arc::new(Channel {

      queue: Mutex::new(Queue {
        _queue: VecDeque::new( ),
        senderCount: 1
      }),

      condVar: Condvar::new( )
    });

    (
      Sender { channel: channel.clone( ) },
      Receiver { channel, cache: VecDeque::new( ) }
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn case_SingleSender( ) {
    let (mut sender, mut receiver)= Channel::new( );

    sender.send(42);
    assert_eq!(Some(42), receiver.receive( ));
  }

  #[test]
  fn case_NoSenders( ) {
    let (mut sender, mut receiver)= Channel::<( )>::new( );

    drop(sender);
    assert_eq!(None, receiver.receive( ));
  }
}

/*
  Channle flavours -
  |
  |- 1. Asynchronous (Unbounded) channels
  |
  |- 2. Synchronous (bounded) channels - The queue in this case is of constant size. When the queue
  |     fills up, all senders remain blocked until there is again some space available in the queue.
  |
  |- 3. Rendezvous channels
  |
  |- 4. Oneshot channels.
*/