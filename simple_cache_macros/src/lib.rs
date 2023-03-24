extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitInt, ReturnType};

#[proc_macro_attribute]
pub fn ttl_cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function signature and extract the return type
    let function = parse_macro_input!(item as ItemFn);
    let function_name = &function.sig.ident;
    let function_args = &function.sig.inputs;
    let function_body = &function.block;
    let key = function_name.to_string();
    let cached_function = Ident::new(&format!("_{}", &key), Span::call_site());
    let function_return_type = match &function.sig.output {
        ReturnType::Type(_, ty) => ty.as_ref(),
        ReturnType::Default => {
            panic!("`ttl_cache` can only be applied to functions that return a value")
        }
    };
    let ttl = parse_macro_input!(attr as LitInt);
    let ttl = ttl.base10_parse::<u64>().expect("Invalid ttl argument");
    let ttl_duration = quote! { std::time::Duration::from_secs(#ttl) };

    // Extract variable names from function arguments
    let function_args_names = function_args
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(arg_name) = &*pat_type.pat {
                    Some(arg_name.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Generate the wrapped function
    let output = quote! {
        pub fn #function_name(#function_args) -> #function_return_type {
            let cache = ::simple_cache_core::TtlCache::new(#ttl_duration);
            let key = #key;
            if let Some(cached_result) = cache.get(key) {
                return cached_result;
            }
            fn #cached_function(#function_args) -> #function_return_type #function_body
            let result = #cached_function(#(#function_args_names),*);
            cache.insert(key, result.clone());
            result
        }
    };
    output.into()
}
