#![allow(unused, non_snake_case)]

use std::ops::{Deref, DerefMut};

pub struct Boks<T> {
  p: *mut T
}

impl<T> Boks<T> {
  pub fn new(p: T) -> Self {
    Self { p: Box::into_raw(Box::new(p)) }
  }
}

// Since lifetime of raw pointers are not automatically managed by Rust, we need to manually
// deallocate the memory when the Boks instance is dropped. 
impl<T> Drop for Boks<T> {
  fn drop(&mut self) {
    unsafe { Box::from_raw(self.p) };
  }
}

impl<T> Deref for Boks<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.p }
  }
}

impl<T> DerefMut for Boks<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.p }
  }
}

fn main( ) {
  let mut x= 42;
  let boks= Boks::new(&mut x);
  // println!("{:?}", x);
  // |
  // This will throw error that x is already mutably borrowed by 'boks'. Notice that 'boks' doesn't
  // actually access 'x'. But since it has a custom 'Drop' implementation, the Rust compiler thinks,
  // maybe inside that Drop implementation, we can do something like -
  //
  // let _: u8= unsafe { std::ptr::read(self.p as *const u8) };
  //
  // That's y it throws that error. We can prompt the Rust compiler that we are never accessing 'x'
  // even in the custom Drop implementation in this way -
  //
  // #![feature(dropck_eyepatch)]
  //
  // unsafe impl<#[may_dangle] T> Drop for Boks<T> { ... }
}