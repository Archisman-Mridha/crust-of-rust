#![allow(non_snake_case, unused)]

mod onStackDynamicDispatch {
  use std::{fs::File, io};

  fn read(cmdArg: &str) {
    // We don't need to allocate anything on the heap.
    // But a disadvantage : we need to declare multiple variables to bind differently-typed objects.
    let (mut stdinReader, mut fileReader);

    let readable: &mut dyn io::Read=
      if cmdArg == "-" {
        stdinReader= io::stdin( );
        &mut stdinReader
      }
      else {
        fileReader= File::open(cmdArg).unwrap( );
        &mut fileReader
      };
  }
}
