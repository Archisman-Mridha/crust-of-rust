#![allow(non_snake_case, unused)]

// NOTE - Identifiers defined inside a macro and identifiers defined outside the macro live
// completely in separate worlds.

#[macro_export]
macro_rules! _vec {

  ($($element: expr),* $(,)?) => {{
    const COUNT: usize= $crate::_count!(@COUNT $($element),*);

    let mut xs= Vec::with_capacity(COUNT);
    $(xs.push($element);)*

    xs
  }};

  // CASE - _vec!(1; 4)
  ($element: expr; $count: expr) => {{
    let x= $element;
    let count= $count;

    let mut xs= Vec::with_capacity(count);

    for _ in 1..count {
      /*
        x can be an expression which must be firt evaluated and the evaluation may be
        computationally expensive. So we do the computation once and store it in x.

        Reason for doing x.clone( ) -

        let mut y= Some(42);
        let xs= _vec![ y.take( ).unwrap( ); 2 ]
      */
      xs.push(x.clone( ));
    }

    // Optimized way (no bound checks will be done for each push) -
    // xs.extend(std::iter::repeat($element).take($count));
    // or
    // xs.resize($count, $element);

    xs
  }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _count {

  (@COUNT $($element: expr),*) => {
    // Invoke the implementation of [( )].len( ) for &[$($crate::_vec!(@SUBSTITUTE $element)),*].
    <[( )]>::len(
      &[$($crate::_count!(@SUBSTITUTE $element)),*]
    )
  };

  (@SUBSTITUTE $_element: expr) => {( )}
}

fn test( ) {
  let xs: Vec<u32> = _vec!( );

  let xs= _vec!(2);

  let xs= _vec!(2, 3);
  let xs= _vec!(2, 3,);

  let xs= _vec!(1; 4);
}