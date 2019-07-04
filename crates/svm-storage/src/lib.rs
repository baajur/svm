#![deny(missing_docs)]
#![deny(unused)]

//! `svm-storage` crate is responsible on the contract storage part of the `svm`
//! Each smart contract has its own storage

mod cacheable_pages;
mod cacheable_pages_slices;
mod default_page_hasher;
mod mem_kv_store;
mod mem_pages_storage;
mod pages_storage_impl;
mod traits;

use default_page_hasher::DefaultPageHasher;
use mem_kv_store::MemKVStore;
pub use mem_pages_storage::MemPagesStorage;
use pages_storage_impl::PagesStorageImpl;

pub use cacheable_pages::CacheablePages;
// pub use cacheable_pages_slices::CacheablePagesSlices;
