#![doc = include_str!("../README.md")]

pub use futures::executor::block_on;
pub use simple_cache_core::TtlCache;
pub use simple_cache_macros::ttl_cache;
