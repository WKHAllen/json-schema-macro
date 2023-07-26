use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use serde_json::{Map, Value};
use std::fs;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

/// Parses a JSON schema from a string into a `serde_json::Value`.
fn parse_schema_from_str(schema: &str) -> Result<Value, String> {
    // match JsonSchema::parse(schema) {
    //     Ok(_) => serde_json::from_str::<Value>(schema)
    //         .map_err(|e| format!("error parsing schema as JSON: {}", e)),
    //     Err(e) => Err(format!("error parsing schema: {:?}", e)),
    // }
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
    /// The optional file to output the evaluated schema to.
    pub out: Option<String>,
}

impl Parse for SchemaEvalConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut schema_out = None;

        let schema_value = loop {
            let keyword = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;

            match keyword.to_string().as_str() {
                "out" => {
                    schema_out = Some(input.parse::<LitStr>()?.value());
                }
                "schema" => {
                    let schema_tokens = input.parse::<TokenStream2>()?.to_string();
                    break parse_schema_from_str(&schema_tokens)
                        .map_err(|e| syn::Error::new_spanned(schema_tokens, e));
                }
                "file" => {
                    let schema_file = input.parse::<LitStr>()?.value();
                    break parse_schema_from_file(&schema_file)
                        .map_err(|e| syn::Error::new_spanned(schema_file, e));
                }
                "url" => {
                    let schema_url = input.parse::<LitStr>()?.value();
                    break parse_schema_from_url(&schema_url)
                        .map_err(|e| syn::Error::new_spanned(schema_url, e));
                }
                unknown_keyword => {
                    break Err(syn::Error::new_spanned(
                        keyword,
                        format!("unknown keyword '{}'", unknown_keyword),
                    ));
                }
            }

            input.parse::<Token![,]>()?;
        }?;

        Ok(Self {
            schema: schema_value,
            out: schema_out,
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
