# simpl_cache
Simple rust caching tools

## Usage
Add this to your `Cargo.toml`:

```toml
[dependencies]
simpl_cache = version = "2.1.0-beta"
```

## `ttl_cache` macro

This proc macro is designed to cache function calls with a time-to-live (TTL) duration. 
It is useful when working with functions that perform expensive computations and have
outputs that don't change frequently.

The macro generates a static variable for the cache that is shared across all calls to 
the function with the same name and input arguments. 

If a cached value is available, it is returned instead of recomputing the result. 
If the cached value has expired or the function is called with different arguments,
the function will be recomputed and the cache will be updated with the new value.


```rust,ignore
use simpl_cache::ttl_cache;

#[ttl_cache(duration_s = 30)]
fn fibonacci(n: u32) -> u32 {
    if n < 2 {
        return n;
    }

    fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    println!("first: {}", fibonacci(10)); // cache miss: return value is cached
    println!("second: {}", fibonacci(10)); // cached hit: cached value is returned
    println!("last: {}", fibonacci(20)); // cache miss: args changed, new result is cached
}
```

You can also cache the `Ok(T)` variant of a function returning a `Result<T, E>`:

```rust,ignore
use simpl_cache::ttl_cache;

// only_ok option ensures that only .is_ok values from the returning Result are cached
#[ttl_cache(duration_s = 30, only_ok = true)] 
fn some_fallible_function(n: u32) -> Result<u32, String> {
    if n == 0 {
        return Err(String::from("zeros are not allowed"))
    }
    Ok(n)
}

fn main() {
     // zero is not cached since function returns an Err since n == 0
    println!("last: {}", some_fallible_function(0));
    // cache miss: 10 is cached since the result is_ok
    println!("last: {}", some_fallible_function(10));
    // cache hit: 10 is retrieved from the cache
    println!("last: {}", some_fallible_function(10));

}
```

Similarly you can also chose to only cache `Some(T)` variants from a function returning an `Option<T>`

```rust,ignore
use simpl_cache::ttl_cache;

// only_some option ensures that only .is_some values from the returning Option are cached
#[ttl_cache(duration_s = 30, only_some = true)] 
fn some_optional_function(n: u32) -> Option<u32> {
    if n == 0 {
        return None;
    }
    Some(n)
}

fn main() {
     // zero is not cached since function returns None since n == 0
    println!("last: {}", some_optional_function(0));
    // cache miss: 10 is cached since the result is_some
    println!("last: {}", some_optional_function(10));
    // cache hit: 10 is retrieved from the cache
    println!("last: {}", some_optional_function(10));

}
```

## Notes
Firstly, this is still a work in progress, so I would not advise using this in a production setting.

The macro is not stable for use with struct and enum methods, specifically those with `self` as an arg.

Note that `only_some` and `only_ok` can only be used when the annotated function returns an
`Option<T>` or `Result<T, E>` respectively. You can also not set both `only_some` and `only_ok`

The macro will also not allow you to apply it to a function that does not return or explicitly 
returns a unit type `()`. For example, the following will not compile:

```rust,ignore
use simpl_cache::ttl_cache;

#[ttl_cache(duration_s = 60)]
fn print_hello_world() {
    println!("Hello, world!");
}
```

Finally, the type returned by the annotated function must implement `Clone`