use std::time::{Duration, Instant};

use simple_cache_macros::ttl_cache;

#[ttl_cache(100)]
fn expensive_build_string(s: &str) -> String {
		println!("call");
		std::thread::sleep(Duration::from_secs(5));
    String::from(s)
}

#[test]
fn test_ttl_cache() {
	let start_time = Instant::now();
	expensive_build_string("string");
	expensive_build_string("string");
	expensive_build_string("stringyy");
	assert!(start_time.elapsed().as_secs() <= 5);
}