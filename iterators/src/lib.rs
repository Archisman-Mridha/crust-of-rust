#![allow(unused, non_snake_case)]

fn main( ) {
  for x in vec![ 2, 3 ] { }
  //
  // Rust for loops are just syntactic sugar. Under the hood they get translated to iterators.
  // Kind of like this (not exactly like this though) :
  //
  let mut iter= vec![ 2, 3 ].into_iter( );
  while let Some(x)= iter.next( ) { }

  /*
    for x in vec![ 1, 2 ] {
      // We own x.
    }

    for x in vec![ 1, 2 ].iters( ) {
    // Alternative syntax - for x in &vec![ 1, 2 ]

      // We get a shared reference to x.
    }
  */
}

//---

pub fn flatten<I>(iterator: I) -> Flatten<I>
  where I: IntoIterator,
        I::Item: IntoIterator
{
  Flatten::new(iterator)
}

pub struct Flatten<O>
  where O: IntoIterator,
        O::Item: IntoIterator
{
  outerIterator: O,

  frontInnerIterator: Option<<O::Item as IntoIterator>::IntoIter>,
  backInnerIterator: Option<<O::Item as IntoIterator>::IntoIter>
}

impl<O> Flatten<O>
  where O: IntoIterator,
        O::Item: IntoIterator
{
  pub fn new(iterator: O) -> Self {
    Self {
      outerIterator: iterator,
      frontInnerIterator: None,
      backInnerIterator: None
    }
  }
}

impl<O> Iterator for Flatten<O>
  where O: Iterator,
        O::Item: IntoIterator
{
  type Item = <O::Item as IntoIterator>::Item;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(ref mut innerIterator)= self.frontInnerIterator {
        let nextItem= innerIterator.next( );
  
        if nextItem.is_some( ) {
          return nextItem;
        }
  
        // Inner iterator is exhausted.
        self.frontInnerIterator= None;
      }

      // Since the inner iterator is exhausted, we poll the next item from the outer iterator,
      // convert it to an iterator and then set it as the new inner iterator.
      if let Some(nextFrontIterator)= self.outerIterator.next( ) {
        self.frontInnerIterator= Some(nextFrontIterator.into_iter( ));
      }
      // TODO: Explain.
      else {
        return self.backInnerIterator.as_mut( )?.next( );
      }
    }
  }
}

impl<O> DoubleEndedIterator for Flatten<O>
  where O: DoubleEndedIterator,
        O::Item: IntoIterator,
        <O::Item as IntoIterator>::IntoIter: DoubleEndedIterator
{
  fn next_back(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(ref mut innerIterator)= self.backInnerIterator {
        let nextItem= innerIterator.next_back( );
  
        if nextItem.is_some( ) {
          return nextItem;
        }

        self.backInnerIterator= None;
      }

      if let Some(nextBackIterator)= self.outerIterator.next_back( ) {
        self.backInnerIterator= Some(nextBackIterator.into_iter( ));
      }
      // TODO: Explain.
      else {
        return self.frontInnerIterator.as_mut( )?.next_back( );
      }
    }
  }
}