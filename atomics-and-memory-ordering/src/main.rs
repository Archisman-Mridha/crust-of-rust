#![allow(non_snake_case, unused)]

/*
	Rust does not yet have a defined memory model. Various academics and industry professionals are
	working on various proposals, but for now, this is an under-defined place in the language.
	REFER : https://doc.rust-lang.org/reference/memory-model.html.

	It rather follows the memory model of C++20.

	The memory layout of the AtomicUsize type is same as that of the usize type.
	The only difference between AtomicUsize and usize is the APIs provided (or the CPU instructions
	generated) by AtomicUsize to read / update the underlying value.

	NOTE : We don't need a mutable reference to the AtomicUsize to mutate the underlying value :
				 pub fn store(&self, val: usize, order: Ordering)

	Atomic operation (like fetch_add) : The fetch and add operations will be executed without any
	interruptions from other threads.
	Atomic operations are lock free but not wait free.

	Some platforms provide dedicated hardware supported instructions, cache coherence protocols (like
	MESI) etc. to make some atomic operations overhead free. Like modern Intel x86 processors provide
	direct hardware support for atomic operations through instructions like LOCK CMPXCHG, XCHG, ADD,
	INC etc.

	Ordering tells the compiler, what guarantees we expect for a particular memory access, with
	respect to related things happening in other threads at the same time. The implementation of those
	guarantees depend on the underlying system architecture.

	Atomics are generally shared via Arc rather than Box :

		let x = Arc::new(AtomicUsize::new(5));

		let threadAOwnedX = x.clone();
		tokio::spawn(move || {
			// use threadAOwnedX here.
		})

		let threadBOwnedX = x.clone();
		tokio::spawn(move || {
			// use threadBOwnedX here.
		})
*/

// Implementing our own (bad) Mutex (SpinLock approach).
mod mutex {
	use std::{cell::UnsafeCell, sync::atomic::{AtomicBool, Ordering}, thread::yield_now};

	pub struct Mutex<T> {
		isLocked: AtomicBool,
		data: UnsafeCell<T>
	}

	impl<T> Mutex<T> {
		pub fn new(data: T) -> Self {
			Self {
				isLocked: AtomicBool::new(false),
				data: UnsafeCell::new(data)
			}
		}

 		pub fn execute<R>(
			&self,
			f: impl FnOnce(&mut T) -> R
		) -> R {
			// Solution 1 :
			/*
				// Wait until the already acquired lock is released.
				while self.isLocked.load(Ordering::Relaxed) {}

				// There can be a scenario, where both threads A and B exitted the while loop at the same time
				// and they both acquire the lock as well, resulting to an undefined behaviour.

				// NOTE : In John gjengsets machine, assert_eq!( ) wasn't panicking since the final value was
				//				always 10 * 1000, until he did a thread::yield_now( ) here. For me, it panicked even
				//				without the yield_now( ).
				//
				// This calls the underlying OS scheduler's yield primitive, signaling that the calling thread
				// is willing to give up its remaining timeslice so that the OS may schedule other threads on
				// the CPU.
				// This causes a pre-emption between the above load and the below store operations.
				yield_now();

				// Acquire the lock.
				self.isLocked.store(true, Ordering::Relaxed);
			*/

			// NOTE : compare_exchange is stronger than compare_and_swap, since it allows us to specify
			//				different memory orderings for the success and failure cases of the atomic operation.
			// 				In fact, compare_and_swap under the hood uses compare_exchange.

			// Solution 2 :
			/*
				// Removing the race condition by using compare_exchange.
				while self.isLocked.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
								.is_err()
				{
					// compare_exchange is an expensive operation, since all the threads need to coordinate
					// among each other (to understand in detail, have a look at the MESI protocol).
					//
					// So only when the lock is released, we'll execute the next compare_exchange operation.
					while self.isLocked.load(Ordering::Relaxed) {}
				}
			*/

			/*
				// Solution 3 :
				/*
					compare_exchange is only allowed to fail if the actual value doesn't match the desired
					value. But, compare_exchange_weak is allowed to fail even if that holds true.

					On Intel / AMD x86_64, we have the compare and swap (CAS) instruction, because of which the
					compare_exchange opertion has low overhead.
					But on ARM64, we don't have such equivalent instruction. Instead, we have the LDREX and
					STREX instructions. Using LDREX, a CPU core can get exclusive access to a memory location.
					And only if the CPU core still has exclusive access to that memory location, the STREX
					instruction will succeed. So in ARM64, compare_exchange is implemented using a loop
					comprising of the LDREX and STREX instructions.

					This means :

						while self.isLocked.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
										.is_err()

						ends up being a nested loop.

					We will thus, rather use compare_exchange_weak( ). On Intel / AMD x86_64, this is an atomic
					operation, but on ARM64, this is implemented using LDREX and STREX (without any loop).
				*/
				while self.isLocked.compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
								.is_err()
				{
					while self.isLocked.load(Ordering::Relaxed) {}
				}
			*/

			// Solution 4 :

			/*
				The order in which we have written our code is :

					(1) Acquire the lock.

					(2) Execute the closure.

					(3) Release the lock.

				Using Ordering::Relaxed doesn't give us any guarantee about the execution order PERCEIVED by
				other threads.
				Also, the above operations can be reordered by the compiler / CPU for optimization purposes,
				provided the meaning of our program stays the same. From the compiler / CPU's perspective,
				(2) doesn't have any dependency on (1) and (3). So, it might reorder the code to :
				
					(2) -> (1) -> (3) or (1) -> (3) -> (2) etc.

				In order to avoid this, we need to use Ordering::Acquire and Ordering::Release.
			*/

			// Acquire the lock.
			// No reads / writes in the current thread, can be reordered before this load.
			while self.isLocked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
							.is_err()
			{
				while self.isLocked.load(Ordering::Relaxed) {}
			}

			// Execute the closure.
			let closureExecutionResult =
				f(unsafe { &mut *self.data.get() });

			// Release the lock.
			/*
				Ordering::Release (only applicable for store operations) :

				When a store operation is performed with Release ordering, ALL previous memory operations
				BECOME ORDERED before any load of this value with Acquire ordering. No reads / writes can be
				reordered after the store operation.
				All previous writes become visible to all threads that perform an Acquire (or stronger) load
				of this value.

				This always guarantees that : (1) -> (2) -> (3).
			*/
			self.isLocked.store(false, Ordering::Release);

			closureExecutionResult
		}
	}

	unsafe impl<T> Sync for Mutex<T>
		where
			T: Send
	{}
}

use {std::thread::spawn, mutex::*};

fn main() {
	let mutexGuardedValue: &'static Mutex<usize> = Box::leak(Box::new(Mutex::new(0 as usize)));

	let threadHandles: Vec<_> = (0..10).map(|_| {
		spawn(move | | {
			for _ in 0..1000 {
				mutexGuardedValue.execute(|value| {
					*value += 1;
				});
			}
		})
	}).collect();

	for threadHandle in threadHandles {
		threadHandle.join().unwrap();
	}

	assert_eq!(mutexGuardedValue.execute(|value| *value), 10 * 1000);
}

#[cfg(test)]
mod test {
	use std::{sync::atomic::{AtomicUsize, Ordering}, thread::spawn};

	#[test]
	fn demonstrate_order_relaxed_failure( ) {
		let x: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
		let y: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));

		let thread1Handler = spawn(move | | {
			let r1 = y.load(Ordering::Relaxed);
			x.store(r1, Ordering::Relaxed);
			r1
		});

		let thread2Handler = spawn(move | | {
			/*
				Using Ordering::Relaxed doesn't give us any guarantee about the execution order of the
				following code. It might be so that the compiler / CPU reorders the instructions for
				optimization purposes, resulting to :

					y.store(42, Ordering::Relaxed);
					let r2 = x.load(Ordering::Relaxed);
			*/
			let r2 = x.load(Ordering::Relaxed);
			y.store(42, Ordering::Relaxed);
			r2
		});

		let r1 = thread1Handler.join().unwrap();
		let r2 = thread2Handler.join().unwrap();
	}
}

/*
	TODO :

		(1) fetch methods

		(2) Ordering::SeqCst
		
		(3) Atomic fencing

		(4) volatile

I gave up on the stream and started following : https://www.youtube.com/watch?v=99Qzpv325yI&list=PL8AZrEE2-qZkE3Va-PsMepuUFxALaJheW.
*/
