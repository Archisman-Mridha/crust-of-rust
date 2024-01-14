#![allow(non_snake_case, unused_parens)]

/*
  There are 3 types of procedural macros -

  1. Derive macros - to automatically generate code for trait implementations.

    #[derive(serde::Deserialize)]
    struct Request { }

  2. Attribute macros -  to define custom attributes that can be applied to items in Rust code (such
    as functions, structs, or enums). The macro is then responsible for analyzing the item's
    structure and generating additional code based on its properties.

    #[instrument(name= "Signin", skip(self))]
    fn signin(request: Request) { }

  3. Function like macros (declarative macros)
*/

use proc_macro::TokenStream;
use quote::quote;
use syn::{ parse_macro_input, DeriveInput, Ident, Data, Fields, Type, PathArguments, GenericArgument };

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {

  // Parse the incoming token stream into an Abstract Syntax Tree (AST).
  let ast= parse_macro_input!(input as DeriveInput);

  let structName= ast.ident;
  let builderStructName= format!("{}Builder", structName);
  let builderStructIdent= Ident::new(&builderStructName, structName.span( ));

  let fields= match ast.data {
    Data::Struct(structData) => {

      match structData.fields {
        Fields::Named(namedFields) => namedFields.named,

        _ => unimplemented!( )
      }
    },
    _ => unimplemented!( )
  };

  let optionizedFields= fields.iter( ).map(|field| {
    let fieldIdent= &field.ident;
    let fieldType= &field.ty;

    if isOptionType(fieldType) {
      quote! { #fieldIdent: #fieldType }
    }
    else {
      quote! { #fieldIdent: std::option::Option<#fieldType> }
    }
  });

  let initFields= fields.iter( ).map(|field| {
    let fieldIdent= &field.ident;

    quote! { #fieldIdent: None }
  });

  let builderStructMethods= fields.iter( ).map(|field| {
    let fieldIdent= &field.ident;
    let fieldType= &field.ty;

    if let Some(argType)= unwrapOptionType(fieldType) {
      quote! {
        pub fn #fieldIdent(&mut self, #fieldIdent: #argType) -> &mut Self {
          self.#fieldIdent= Some(#fieldIdent);
          self
        }
      }
    }
    else {
      quote! {
        pub fn #fieldIdent(&mut self, #fieldIdent: #fieldType) -> &mut Self {
          self.#fieldIdent= Some(#fieldIdent);
          self
        }
      }
    }
  });

  let builderStructEachMethods= fields.iter( ).filter_map(|field| {

    for attribute in &field.attrs {
      let segments= &attribute.meta.path( ).segments;

      if (
        segments.len( ) != 1 ||
        segments.first( ).unwrap( ).ident != "builder"

      ) { continue }

      return Some(quote!(/* */))
    }

    None
  });

  let buildMethodFields= fields.iter( ).map(|field| {
    let fieldIdent= &field.ident;
    let fieldType= &field.ty;

    if isOptionType(fieldType) {
      quote! { #fieldIdent: self.#fieldIdent.clone( ) }
    }
    else {
      quote! {
        #fieldIdent: self.#fieldIdent.clone( )
                        .ok_or(concat!(stringify!(#fieldIdent), " is not set"))?
      }
    }
  });

  quote! {

    struct #builderStructIdent {
      #(#optionizedFields,)*
    }

    impl #structName {
      pub fn builder( ) -> #builderStructIdent {
        #builderStructIdent {
          #(#initFields,)*
        }
      }
    }

    impl #builderStructIdent {

      #(#builderStructMethods)*
      #(#builderStructEachMethods)*

      pub fn build(&self) -> Result<#structName, Box<dyn std::error::Error>> {
        Ok(#structName {
          #(#buildMethodFields,)*
        })
      }
    }

  }.into( )
}

fn isOptionType(_type: &Type) -> bool {
  if let Type::Path(ref path)= _type {
    return
      path.path.segments.len( ) == 1 &&
      path.path.segments.iter( ).last( ).unwrap( ).ident == "Option"
  }

  false
}

fn unwrapOptionType(_type: &Type) -> Option<&Type> {
  if let Type::Path(ref path)= _type {
    if (
      path.path.segments.len( ) != 1 ||
      path.path.segments.iter( ).last( ).unwrap( ).ident != "Option"

    ) { return None }

    if let PathArguments::AngleBracketed(ref innerType)= path.path.segments[0].arguments {
      let innerType= innerType.args.first( ).unwrap( );

      if let GenericArgument::Type(ref _innerType)= innerType {
        return Some(_innerType)
      }
    }
  }

  None
}