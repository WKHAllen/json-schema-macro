#![forbid(unsafe_code)]

mod eval;
mod parse;

use crate::eval::*;
use crate::parse::*;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn};

macro_rules! throw_err {
    ( $err:expr, $tokens:expr ) => {{
        return ::syn::Error::new_spanned(::proc_macro2::TokenStream::from($tokens), $err)
            .to_compile_error()
            .into();
    }};
}

#[proc_macro_attribute]
pub fn schema_macro(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let item = parse_macro_input!(input as ItemFn);

    let fn_ident = &item.sig.ident;
    let param_ty = match item.sig.inputs.first() {
        Some(param) => match param {
            FnArg::Typed(param) => &*param.ty,
            FnArg::Receiver(_) => throw_err!("expected typed parameter, not receiver", tokens),
        },
        None => throw_err!(
            "schema macro implementations can only take one parameter",
            tokens
        ),
    };
    let alias_ident = format_ident!("__{}_macro_param_ty", fn_ident);

    quote! {
        #item

        #[allow(non_camel_case_types)]
        type #alias_ident = #param_ty;
    }
    .into()
}

#[proc_macro]
pub fn eval_schema(input: TokenStream) -> TokenStream {
    let schema_config = parse_macro_input!(input as SchemaEvalConfig);
    let schema_str = schema_config.schema.to_string();
    let macro_pointers = find_macro_invocations(&schema_config.schema);
    let macro_expansions = macro_pointers
        .iter()
        .map(|macro_pointer| {
            let macro_invocation = schema_config
                .schema
                .pointer(macro_pointer)
                .unwrap()
                .as_object()
                .unwrap();
            let macro_call = parse_macro_call(macro_invocation);
            let MacroCall {
                name: _,
                ident,
                args,
            } = macro_call;
            let alias_ident = format_ident!("__{}_macro_param_ty", ident);
            let args_str = args.to_string();

            quote! {
                let param_value = match ::serde_json::from_str::<#alias_ident>(#args_str) {
                    Ok(v) => v,
                    Err(e) => {
                        break 'ret Err(e.to_string());
                    }
                };
                let macro_pointer = schema_value.pointer_mut(#macro_pointer).unwrap();
                *macro_pointer = match #ident(param_value) {
                    Ok(v) => ::serde_json::to_value(v).unwrap(),
                    Err(e) => {
                        break 'ret Err(e);
                    }
                };
            }
        })
        .collect::<Vec<_>>();

    quote! {{
        let mut schema_value = ::serde_json::from_str::<::serde_json::Value>(#schema_str).unwrap();

        let ret = 'ret: {
            #({
                #macro_expansions
            })*

            Ok(())
        };

        match ret {
            Ok(_) => Ok(schema_value),
            Err(e) => Err(e)
        }
    }}
    .into()
}
