#![forbid(unsafe_code)]

mod eval;
mod parse;

use crate::parse::*;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn schema_macro(_attr: TokenStream, input: TokenStream) -> TokenStream {
    parse_schema_macro(input)
}

#[proc_macro]
pub fn eval_schema(input: TokenStream) -> TokenStream {
    parse_eval_schema(input)
}
