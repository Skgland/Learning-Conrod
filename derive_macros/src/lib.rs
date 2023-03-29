#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;

mod derive_macros;

#[allow(clippy::let_and_return)]
#[proc_macro_derive(Bounded, attributes(skip))]
pub fn bounded_trait_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    let result = derive_macros::bounded::impl_bounded_trait_derive(&ast);
    //panic!(result.to_string());
    result
}

#[allow(clippy::let_and_return)]
#[proc_macro_derive(Enumerable, attributes(skip))]
pub fn enumerable_trait_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    let result = derive_macros::enumerable::impl_enumerable_trait_derive(&ast);
    //panic!(result.to_string());
    result
}
