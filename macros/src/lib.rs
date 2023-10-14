use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, Pat, ReturnType, Type};

#[derive(Debug, FromMeta)]
struct TtlCacheAttributes {
    duration_s: u64,
    only_some: Option<bool>,
    only_ok: Option<bool>,
}

#[derive(Debug)]
struct FunctionReturnType<'a> {
    ty: &'a Type,
    is_option: bool,
    is_result: bool,
}

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
    let internal_function = Ident::new(&format!("__{}", &key), Span::call_site());

    let static_var = Ident::new(&key.to_ascii_uppercase(), internal_function.span());
    // let ttl = parse_macro_input!(attr as LitInt);
    let macro_attributes = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let TtlCacheAttributes {
        duration_s,
        only_some,
        only_ok,
    } = match TtlCacheAttributes::from_list(&macro_attributes) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    let FunctionReturnType {
        ty,
        is_option,
        is_result,
    } = get_function_return_type(&function.sig.output);
    let only_ok = only_ok.is_some();
    let only_some = only_some.is_some();
    let function_return_type = ty;
    if only_ok || only_some {
        assert_ne!(
            only_some, only_ok,
            "`only_some` and `only_ok` cannot both be set"
        );
    }
    if only_ok {
        assert_eq!(
            only_ok, is_result,
            "`only_ok` can only be applied if the function's return type is a `Result`"
        );
    }
    if only_some {
        assert_eq!(
            only_some, is_option,
            "`only_some` can only be applied if the function's return type is an `Option`"
        );
    }
    // Extract variable names from function arguments
    let (function_args_names, function_arg_values) = get_function_args(function_args);
    // Generate the key from function name and arg values as token stream
    // TODO: This can be improved
    let key = quote! {
        format!("{}:{:?}", #key, (#(#function_arg_values),*))
    };
    let mut cache_insert = quote! {
        cache.insert(#key, result.clone());
    };
    if only_some && is_option {
        cache_insert = quote! {
            if result.is_some() {
                cache.insert(#key, result.clone());
            }
        };
    }
    if only_ok && is_result {
        cache_insert = quote! {
            if result.is_ok() {
                cache.insert(#key, result.clone());
            }
        };
    }
    // Generate function and ttl cache static variable
    let output = quote! {
        #function_visibitly fn #function_name(#function_args) -> #function_return_type {
            // Each ttl cache annotated function will have its own static variable containing
            // an instance of the TtlCache struct, which holds the cached values
            thread_local! {
                static #static_var: ::std::cell::RefCell<::simple_cache_core::TtlCache<String, #function_return_type>> = ::std::cell::RefCell::new(
                    ::simple_cache_core::TtlCache::new(::std::time::Duration::from_secs(#duration_s))
                );
            }
            #static_var.with(|var| {
                let cache = var.borrow_mut();
                if let Some(cached_result) = cache.get(#key) {
                    return cached_result;
                } else {
                    fn #internal_function(#function_args) -> #function_return_type {
                        #function_body
                    }
                    let result = #internal_function(#(#function_args_names),*);
                    #cache_insert
                    return result;
                }
            })
        }
    };
    output.into()
}

fn get_function_return_type<'a>(output: &'a ReturnType) -> FunctionReturnType<'a> {
    match output {
        ReturnType::Type(_, ty) => {
            let mut is_option = false;
            let mut is_result = false;
            if let Type::Tuple(ty) = &**ty {
                // prevent #[ttl_cache] from getting applied to functions that explicitly
                // return a unit type: `()`
                if ty.elems.is_empty() {
                    panic!("`ttl_cache` can only be applied to functions that return a value");
                }
            }
            if let Type::Path(path) = &**ty {
                let type_str = path
                    .path
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string());
                if let Some(s) = type_str {
                    if s == "Result" {
                        is_result = true;
                    } else if s == "Option" {
                        is_option = true
                    }
                }
            }
            FunctionReturnType {
                ty,
                is_option,
                is_result,
            }
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
