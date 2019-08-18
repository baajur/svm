#![allow(missing_docs)]
#![allow(unused)]

//! `svm-storage` crate is responsible on the contract storage part of the `svm`
//! Each smart contract has its own storage

mod default_page_cache;
mod default_page_hasher;
mod default_page_index_hasher;
mod default_pages_storage;
mod default_state_hasher;
mod merkle_pages_storage;
mod page_slice_cache;

/// Contains definitions of `Page` related structures. For example: `Page` / `PageIndex` / `SliceIndex`
pub mod page;

/// Contains definitions `State`-related.
pub mod state;

pub use crate::page_slice_cache::PageSliceCache;

/// Contains `svm storage` related default implementations for traits defined under the `traits` module.
pub mod default {
    pub use crate::default_page_cache::DefaultPageCache;
    pub use crate::default_page_hasher::DefaultPageHasher;
    pub use crate::default_page_index_hasher::DefaultPageIndexHasher;
    pub use crate::default_pages_storage::DefaultPagesStorage;
    pub use crate::default_state_hasher::DefaultStateHasher;
}

/// Do-nothing implementation for various storage related abstractions.
/// Very usable for code requiring a storage dependencies it doesn't care about
pub mod null_storage;

/// Storage related traits
#[macro_use]
pub mod traits;

/// Common storage macros
#[macro_use]
pub mod macros;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "svm_memory")] {
        mod mem_kv_store;
        mod mem_pages;
        mod mem_page_cache;
        mod mem_merkle_pages;

        /// Implements `svm storage` related in-memory data-structures.
        pub mod memory {
            pub use crate::mem_kv_store::MemKVStore;
            pub use crate::mem_pages::MemPages;
            pub use crate::mem_page_cache::{MemPageCache, MemPageCache32};
            pub use crate::mem_merkle_pages::MemMerklePages;
        }
    }
}

cfg_if! {
    if #[cfg(feature = "svm_leveldb")]  {
        // mod level_key;
        // mod leveldb_kv;
        //
        // pub use level_key::*;
        // pub use leveldb_kv::LevelDB;
    }
}
