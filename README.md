# simple_cache
Simple rust caching tools

## Usage
Add this to your `Cargo.toml`:

```toml
[dependencies]
simpl_cache = { version = "1.0.0-beta" }
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
#[ttl_cache(30)]
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

The macro will not allow you to apply it to a function that does not return or explicitly 
returns a unit type `()`. For example, the following will not compile:

```rust,ignore
#[ttl_cache(60)]
fn print_hello_world() {
    println!("Hello, world!");
}
```

Additionally, the type returned by the annotated function must implement `Clone`