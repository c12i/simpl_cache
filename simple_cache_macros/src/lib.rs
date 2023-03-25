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
    let function_visibitly = &function.vis;
    let key = function_name.to_string();
    let cached_function = Ident::new(&format!("_{}", &key), Span::call_site());
    let static_var = Ident::new(&key.to_ascii_uppercase(), Span::call_site());
    let function_return_type = match &function.sig.output {
        ReturnType::Type(_, ty) => ty.as_ref(),
        ReturnType::Default => {
            panic!("`ttl_cache` can only be applied to functions that return a value")
        }
    };
    let ttl = parse_macro_input!(attr as LitInt);
    let ttl = ttl.base10_parse::<u64>().expect("Invalid ttl argument");
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
    let function_arg_values = function_args
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(arg_ident) = &*pat_type.pat {
                    Some(quote! { #arg_ident })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    // Generate the key from function name and arg values as token stream
    let key = quote! {
        format!("{}:{:?}", #key, (#(#function_arg_values),*))
    };
    // Generate function
    let output = quote! {
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
