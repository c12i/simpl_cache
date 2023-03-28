extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, LitInt, Pat, ReturnType, Type};

/// This proc macro is designed to cache function calls with a
/// time-to-live (TTL) duration.
#[proc_macro_attribute]
pub fn ttl_cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the anotated function signature and extract various properties
    let function = parse_macro_input!(item as ItemFn);
    let function_name = &function.sig.ident;
    let function_args = &function.sig.inputs;
    let function_body = &function.block;
    let function_visibitly = &function.vis;
    let key = function_name.to_string();
    let cached_function = Ident::new(&format!("__{}", &key), Span::call_site());
    let static_var = Ident::new(&key.to_ascii_uppercase(), Span::call_site());
    let function_return_type = get_function_return_type(&function.sig.output);
    let ttl = parse_macro_input!(attr as LitInt)
        .base10_parse::<u64>()
        .expect("Invalid ttl argument");
    // Extract variable names from function arguments
    let (function_args_names, function_arg_values) = get_function_args(function_args);
    // Generate the key from function name and arg values as token stream
    // TODO: This can be improved
    let key = quote! {
        format!("{}:{:?}", #key, (#(#function_arg_values),*))
    };
    // Generate function and ttl cache static variable
    let output = quote! {
        // Each ttl cache annotated function will have its own static variable containing
        // an instance of the TtlCache struct, which holds the cached values
        static #static_var: ::once_cell::sync::Lazy<::simple_cache_core::TtlCache<String, #function_return_type>> = ::once_cell::sync::Lazy::new(
            || ::simple_cache_core::TtlCache::new(std::time::Duration::from_secs(#ttl))
        );
        #function_visibitly fn #function_name(#function_args) -> #function_return_type {
            if let Some(cached_result) = #static_var.get(#key) {
                return cached_result;
            }
            fn #cached_function(#function_args) -> #function_return_type #function_body
            let result = #cached_function(#(#function_args_names),*);
            #static_var.insert(#key, result.clone());
            result
        }
    };
    output.into()
}

fn get_function_return_type(output: &ReturnType) -> &Type {
    match output {
        ReturnType::Type(_, ty) => {
            if let Type::Tuple(ty) = &**ty {
                // prevent #[ttl_cache] from getting applied to functions that explicitly
                // return a unit type: `()`
                if ty.elems.len() == 0 {
                    panic!("`ttl_cache` can only be applied to functions that return a value");
                }
            }
            ty.as_ref()
        }
        ReturnType::Default => {
            panic!("`ttl_cache` can only be applied to functions that return a value")
        }
    }
}

fn get_function_args(
    args: &Punctuated<FnArg, Comma>,
) -> (Vec<Ident>, Vec<proc_macro2::TokenStream>) {
    let names = args
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(arg_name) = &*pat_type.pat {
                    Some(arg_name.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let values = args
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(arg_ident) = &*pat_type.pat {
                    Some(quote! { #arg_ident })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    (names, values)
}
