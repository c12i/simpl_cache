use simple_cache_macros::ttl_cache;

#[ttl_cache(100)]
fn expensive_function(x: bool, y: bool) -> String {
    String::from("foo")
}