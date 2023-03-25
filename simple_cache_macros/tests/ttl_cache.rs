use std::time::{Duration, Instant};
use std::thread::sleep;

use simple_cache_macros::ttl_cache;

#[ttl_cache(100)]
fn expensive_build_string(s: &str) -> String {
    sleep(Duration::from_secs(5));
    String::from(s)
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
    assert!(start_time.elapsed().as_secs() <= 5);
    let _ = expensive_build_string("bar");
    let _ = expensive_build_string("bar");
    assert!(start_time.elapsed().as_secs() >= 10);
}

#[ttl_cache(5)]
fn expensive_function_with_shortlived_cache() -> Option<bool> {
    sleep(Duration::from_secs(5));
    None
}

#[test]
fn test_ttl_cache_expiration() {
    let start_time = Instant::now();
    let _ = expensive_function_with_shortlived_cache();
    let _ = expensive_function_with_shortlived_cache();
    assert!(start_time.elapsed().as_secs() <= 5);
    sleep(Duration::from_secs(6));
    let start_time = Instant::now();
    let _ = expensive_function_with_shortlived_cache();
    assert_eq!(start_time.elapsed().as_secs(), 5);
}
