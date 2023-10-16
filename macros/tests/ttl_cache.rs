use std::thread::sleep;
use std::time::{Duration, Instant};

use simple_cache_macros::ttl_cache;

#[ttl_cache(duration_s = 100)]
fn expensive_build_string(s: &str) -> String {
    sleep(Duration::from_secs(5));
    String::from(s)
}

#[ttl_cache(duration_s = 120, only_ok = true)]
fn expensive_fallible_build_string(s: &str) -> Result<String, String> {
    Ok(String::from(s))
}

#[ttl_cache(duration_s = 120, only_some = true)]
async fn expensive_optional_build_string(_s: &str) -> Option<String> {
    None
}

#[test]
fn test_ttl_cache_on_function_with_the_same_params() {
    let start_time = Instant::now();
    // inital call, cache miss - will take 5 seconds
    let first = expensive_build_string("string");
    // next call, cache hit - will return cached value
    let second = expensive_build_string("string");
    assert_eq!(first, second);
    assert!(start_time.elapsed().as_secs() <= 5);
}

#[test]
fn test_ttl_cache_on_function_with_changing_params() {
    let start_time = Instant::now();
    let _ = expensive_build_string("foo");
    let _ = expensive_build_string("foo");
    let _ = expensive_build_string("foo");
    assert!(start_time.elapsed().as_secs() <= 5);
    let _ = expensive_build_string("bar");
    let _ = expensive_build_string("bar");
    assert!(start_time.elapsed().as_secs() >= 10);
}

struct Example;

impl Example {
    #[ttl_cache(duration_s = 5)]
    pub fn expensive_function_with_shortlived_cache() -> Option<bool> {
        sleep(Duration::from_secs(5));
        None
    }
}

#[test]
fn test_ttl_cache_expiration() {
    let start_time = Instant::now();
    let _ = Example::expensive_function_with_shortlived_cache();
    let _ = Example::expensive_function_with_shortlived_cache();
    assert!(start_time.elapsed().as_secs() <= 5);
    sleep(Duration::from_secs(6));
    let start_time = Instant::now();
    let _ = Example::expensive_function_with_shortlived_cache();
    assert_eq!(start_time.elapsed().as_secs(), 5);
}
