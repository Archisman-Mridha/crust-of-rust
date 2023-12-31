#![allow(non_snake_case, unused)]

pub fn strtok<'a>(s: &'a mut &'a str, delimiter: char) -> &'a str {
  unimplemented!( )}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn case_Default( ) {
    let mut x= "hello world"; // When x is declared, its lifetime is 'static'.

    let beforeDelimiter= strtok(&mut x, ' ');
    assert_eq!(beforeDelimiter, "hello");
    // assert_eq!(x, "world");
    //                      |
    //                      |- This will throw error - 'Can't borrow x as mutable since it is already
    //                                                 borrowed immutably.'
    // Before explaining why it throws this error, let's touch some concepts -
    //
    // 1. IMPLICIT LIFETIME SHORTENING :
          fn implicitLifetimeShorteningDemo<'a>( ) {
            let x= "hello world";
            let mut y= &*(String::new( )); // y is declared with a lifetime of 'a'.

            // Now here, we are assigning x (which has lifetime 'static') to y (with lifetime 'a').
            // 'a < 'static.
            // The Rust compiler shortens the lifetime of x from 'static' to 'a'. This is called implicit
            // lifetime shortening.
            y= x;
          }
    //
    // 2. SUBTYPING :
    //    Implicit lifetime shortening is an example of subtyping. 'static' is a subtype of 'a'.
    //    A type T is a subtype of type U, if T is at-least as useful as U.
    //
    // 3. VARIANCE :
    //    There are 3 types of variance -
    //    |
    //    |- Covariance -
    //    |    Consider this function - fn foo(x: &'a str) { }.
    //    |    Here, lifetime of x can be any subtype of 'a'. x is covariant.
    //    |
    //    |- Controvariance -
    //    |    Consider this function - fn foo(x: Fn(y: &'a str) -> ( )) { }.
  }
}