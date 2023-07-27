use crate::eval::*;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use serde_json::{Map, Value};
use std::fs;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, FnArg, ItemFn};
use syn::{Ident, LitStr, Token};

/// Parses a JSON schema from a string into a `serde_json::Value`.
fn parse_schema_from_str(schema: &str) -> Result<Value, String> {
    serde_json::from_str::<Value>(schema)
        .map_err(|e| format!("error parsing schema as JSON: {}", e))
}

/// Parses a JSON schema that exists in a file.
fn parse_schema_from_file(file: &str) -> Result<Value, String> {
    match fs::read_to_string(file) {
        Ok(value) => parse_schema_from_str(&value),
        Err(e) => Err(e.to_string()),
    }
}

/// Parses a JSON schema that exists at a URL.
fn parse_schema_from_url(url: &str) -> Result<Value, String> {
    match reqwest::blocking::get(url) {
        Ok(res) => match res.text() {
            Ok(value) => parse_schema_from_str(&value),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

/// Configuration options for the schema evaluation.
pub struct SchemaEvalConfig {
    /// The schema itself.
    pub schema: Value,
}

impl Parse for SchemaEvalConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let schema_value = {
            let keyword = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;

            match keyword.to_string().as_str() {
                "schema" => {
                    let schema_tokens = input.parse::<TokenStream2>()?.to_string();
                    parse_schema_from_str(&schema_tokens)
                        .map_err(|e| syn::Error::new_spanned(schema_tokens, e))
                }
                "file" => {
                    let schema_file = input.parse::<LitStr>()?.value();
                    parse_schema_from_file(&schema_file)
                        .map_err(|e| syn::Error::new_spanned(schema_file, e))
                }
                "url" => {
                    let schema_url = input.parse::<LitStr>()?.value();
                    parse_schema_from_url(&schema_url)
                        .map_err(|e| syn::Error::new_spanned(schema_url, e))
                }
                unknown_keyword => Err(syn::Error::new_spanned(
                    keyword,
                    format!("unknown keyword '{}'", unknown_keyword),
                )),
            }
        }?;

        Ok(Self {
            schema: schema_value,
        })
    }
}

/// A call to a JSON schema macro.
pub struct MacroCall {
    /// The macro's name.
    pub name: String,
    /// The corresponding function's identifier.
    pub ident: Ident,
    /// The JSON value passed to the function.
    pub args: Value,
}

pub fn parse_macro_call(invocation: &Map<String, Value>) -> MacroCall {
    let keys = invocation.keys().collect::<Vec<_>>();
    let invocation_key = keys.first().unwrap();
    let name = &invocation_key[2..invocation_key.len() - 2];
    let value = invocation.get(*invocation_key).unwrap();

    MacroCall {
        name: name.to_owned(),
        ident: format_ident!("{}", name),
        args: value.to_owned(),
    }
}

macro_rules! throw_err {
    ( $err:expr, $tokens:expr ) => {{
        return ::syn::Error::new_spanned(::proc_macro2::TokenStream::from($tokens), $err)
            .to_compile_error()
            .into();
    }};
}

pub fn parse_schema_macro(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let item = parse_macro_input!(input as ItemFn);

    let fn_ident = &item.sig.ident;
    let param_ty = match item.sig.inputs.first() {
        Some(param) => match param {
            FnArg::Typed(param) => &*param.ty,
            FnArg::Receiver(_) => throw_err!("expected typed parameter, not receiver", tokens),
        },
        None => throw_err!(
            "schema macro implementations must take exactly one parameter",
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

pub fn parse_eval_schema(input: TokenStream) -> TokenStream {
    let schema_config = parse_macro_input!(input as SchemaEvalConfig);
    let schema_str = schema_config.schema.to_string();
    let internal_path = match crate_name("json-schema-macro") {
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{}", name);
            quote!(::#ident::__internal)
        }
        _ => quote!(::json_schema_macro::__internal),
    };
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
                let param_value = match #internal_path::serde_json::from_str::<#alias_ident>(#args_str) {
                    Ok(v) => v,
                    Err(e) => {
                        break 'ret Err(e.to_string());
                    }
                };
                let macro_pointer = schema_value.pointer_mut(#macro_pointer).unwrap();
                *macro_pointer = match #ident(param_value) {
                    Ok(v) => #internal_path::serde_json::to_value(v).unwrap(),
                    Err(e) => {
                        break 'ret Err(e);
                    }
                };
            }
        })
        .collect::<Vec<_>>();

    quote! {{
        let mut schema_value = #internal_path::serde_json::from_str::<#internal_path::serde_json::Value>(#schema_str).unwrap();

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
