#![doc = include_str!("../README.md")]

extern crate futures;
extern crate simple_cache_core;
extern crate simple_cache_macros;

pub use futures::executor::block_on;
pub use simple_cache_core::TtlCache;
pub use simple_cache_macros::ttl_cache;
