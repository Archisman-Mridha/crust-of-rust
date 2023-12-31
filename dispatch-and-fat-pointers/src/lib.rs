#![allow(non_snake_case, unused)]

mod monomorphization {
  // Consider this function which takes in a generic argument.
  fn strlen(s: impl AsRef<str>) -> usize {
    s.as_ref( ).len( )}
  //
  fn main( ) {
    // Now we invoke the 'strlen' function :

    strlen("Hello, world!"); // During code compilation, a version of the 'strlen' function gets
                            // generated, where the type of the argument 's' is &'static str.

    strlen(String::from("Hello, world!")); // Similarly in this case, another version of the 'strlen'
                                           // function gets generated, where the type of the argument
                                           // 's' is 'String'.
  }

  // Monomorphization is a compile-time process where generic functions are transformed into
  // specific functions by filling in the concrete types used during compilation. This process
  // generates a specialized version of the generic function for each unique data type used with the
  // function, resulting in more effective optimization and faster output.
  // NOTE - Monomorphization leads to increment of the binary size.
}

trait SayHello {
  fn sayHello(&self);
}

mod staticDispatch {
  use crate::SayHello;

  fn main( ) {
    let entity: &str= "x";

    impl SayHello for &str {
      fn sayHello(&self) { //                                                                      <---
        println!("Hello, {}", self)}                                                               // |
    }                                                                                              // |
                                                                                                   // |
    sayHello(entity); // sayHello is invoked with the 'entity' type being &str. So, during compile    |
                      // time, a version of sayHello will be generated with 'entity: &str'.           |
  }                                                                                                // |
                                                                                                   // |
  fn sayHello(entity: impl SayHello) { // 'impl SayHello' is a syntactic sugar for generics.          |
    entity.sayHello( )  // During compile time, the compiler knows that the sayHello method is      ---
                        // defined here.
  }
}

mod dynamicDisptach {
  use crate::SayHello;

  impl SayHello for String {
    fn sayHello(&self) {
      println!("Hello, {}", self)}
  }

  fn main( ) {
    sayHello(&[&"x", &String::from("y")]);
  }

  fn sayHello(entities: &[&dyn SayHello]) {
    /*                      |
                            |
      This is dynamic dispatching. The items in 'entities' may not be of the same concrete type,
      but all them must implement thr SayHello trait.

      If we did 'entities: &[dyn SayHello]', we would get an error, that size of 'entities' cannot
      be determined at compile time. To get around that, we had to do 'entities: &[&dyn SayHello]'.
      Every element in 'entities' is now a pointer with determinable size.
    */

    for entity in entities {
      // 'entity' is called a trait object.
      entity.sayHello( )

      /*
        At compile time, the compiler doesn't know the location where sayHello function is defined
        for 'entity' (since it's decoupled from its concrete type, so the compiler doesn't know
        where the trait implementation is located). It is determined during runtime.

        Reference to a dynamically sized type (trait objects or slices), also called fat pointers,
        is twice the size of a reference to a sized type. This is because the reference to a trait
        object also stores pointer to a VTable (Virtual Dispatch Table) for the referenced trait.

        The VTable in this case will look like :

        struct SayHelloVtable {
          sayHello: *mut Fn(*mut T)
                    |
                    |- In our case it will be '&<str as SayHello>::sayHello' for &str /
                       '&<String as SayHello>::sayHello' for String.
        }

        A VTable gets constructed (mostly during compile time) for each concrete type turned into a
        trait object.

        During compile time, 'entity.sayHello( )' gets translated to - 'entity.vtable.sayHello(entity.pointer)'
      */
    }

    // NOTE - Monomorphization doesn't apply to this function.
  }

  /*
    Limitations of dynamic dispatching -

    1. MULTIPLE TRAITS -

      We can't do this - fn sayHelloAndBye(entities: &[&dyn (SayHello + SayBye)]).
      Because, then the fat pointer would've to store pointers to 2 VTables (one for SayHello and
      another for SayBye).
      NOTE - It's not impossible to do so.

      A workaround -

        trait SayCombined: SayHello + SayBye { }

        fn sayHelloAndBye(entities: &[&dyn SayCombined]).

    2. ASSOCIATED TYPES -

      trait SayHello {
        type Name;

        ...methods
      }

      fn sayHelloAndBye(entities: &[&dyn SayHello<Name= ( )>]).
                                                    |
                                  We need to mention the associated type here.

    3. STATIC TRAIT METHODS -

      trait SayHello {
        fn staticMethod( ) { }
      }

      We can't use static trait methods with trait objects. Since static trait objects don't refer
      to 'self', a VTable can't be associated with the static trait method.

      trait SayHello {
        fn staticMethod( )
          where Self: Size
        { }
      }
  */
}