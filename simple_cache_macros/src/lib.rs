extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn ttl_cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function signature and extract the return type
    let function = parse_macro_input!(item as ItemFn);
    let function_name = &function.sig.ident;
    let function_args = &function.sig.inputs;
    let function_body = &function.block;
    let function_return_type = match &function.sig.output {
        ReturnType::Type(_, ty) => ty.as_ref(),
        ReturnType::Default => {
            panic!("`ttl_cache` can only be applied to functions that return a value")
        }
    };
    let ttl = attr
        .to_string()
        .parse::<u64>()
        .expect("Invalid TTL argument");
    let ttl_duration = quote! { std::time::Duration::from_secs(#ttl) };

    // Generate the wrapped function
    let output = quote! {
        fn #function_name(#function_args) -> #function_return_type {
            let cache = ::simple_cache_core::TtlCache::new(#ttl_duration);
            let key = stringify!(#function_name);
            if let Some(cached_result) = cache.get(&key) {
                return cached_result;
            }
            fn cached_function(#function_args) -> #function_return_type {
                #function_body
            }
            let result = cached_function(#function_args);
            cache.insert(key, result.clone());
            result
        }
    };
    output.into()
}
