use procedural_macros::Builder;

#[derive(Builder)]
pub struct Command {
  executable: String,

  #[builder(each = "arg")]
  args: Vec<String>,

  #[builder(each = "env")]
  env: Vec<String>,

  current_dir: Option<String>,
}

fn main( ) {
  Command::builder( )
    .executable("cargo".to_owned( ))
    .args(vec!{ "build".to_owned( ), "--release".to_owned( ) })
    .env(vec!{ })
    .build( )
    .unwrap( );
}