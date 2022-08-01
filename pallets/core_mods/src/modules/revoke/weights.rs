//! Autogenerated weights for revoke
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-08-01, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Native), WASM-EXECUTION: Compiled, CHAIN: Some("mainnet"), DB CACHE: 128

// Executed Command:
// ./target/production/dock-node
// benchmark
// --execution
// native
// --chain
// mainnet
// --wasm-execution
// compiled
// --pallet
// revoke
// --extra
// --extrinsic=*
// --repeat=20
// --steps=50
// --template=node/module-weight-template.hbs
// --output=./pallets/core_mods/src/modules/revoke/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for revoke.
pub trait WeightInfo {
    fn revoke_sr25519(r: u32) -> Weight;
    fn revoke_ed25519(r: u32) -> Weight;
    fn revoke_secp256k1(r: u32) -> Weight;
    fn unrevoke_sr25519(r: u32) -> Weight;
    fn unrevoke_ed25519(r: u32) -> Weight;
    fn unrevoke_secp256k1(r: u32) -> Weight;
    fn remove_registry_sr25519() -> Weight;
    fn remove_registry_ed25519() -> Weight;
    fn remove_registry_secp256k1() -> Weight;
    fn new_registry(c: u32) -> Weight;
}

/// Weights for revoke using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn revoke_sr25519(r: u32) -> Weight {
        (59_678_000 as Weight)
            // Standard Error: 0
            .saturating_add((899_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn revoke_ed25519(r: u32) -> Weight {
        (54_011_000 as Weight)
            // Standard Error: 0
            .saturating_add((849_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn revoke_secp256k1(r: u32) -> Weight {
        (152_679_000 as Weight)
            // Standard Error: 0
            .saturating_add((889_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn unrevoke_sr25519(r: u32) -> Weight {
        (55_584_000 as Weight)
            // Standard Error: 0
            .saturating_add((909_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn unrevoke_ed25519(r: u32) -> Weight {
        (52_775_000 as Weight)
            // Standard Error: 0
            .saturating_add((851_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn unrevoke_secp256k1(r: u32) -> Weight {
        (149_068_000 as Weight)
            // Standard Error: 0
            .saturating_add((897_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn remove_registry_sr25519() -> Weight {
        (143_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(103 as Weight))
    }
    fn remove_registry_ed25519() -> Weight {
        (141_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(103 as Weight))
    }
    fn remove_registry_secp256k1() -> Weight {
        (237_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(103 as Weight))
    }
    fn new_registry(_c: u32) -> Weight {
        (13_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn revoke_sr25519(r: u32) -> Weight {
        (59_678_000 as Weight)
            // Standard Error: 0
            .saturating_add((899_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn revoke_ed25519(r: u32) -> Weight {
        (54_011_000 as Weight)
            // Standard Error: 0
            .saturating_add((849_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn revoke_secp256k1(r: u32) -> Weight {
        (152_679_000 as Weight)
            // Standard Error: 0
            .saturating_add((889_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn unrevoke_sr25519(r: u32) -> Weight {
        (55_584_000 as Weight)
            // Standard Error: 0
            .saturating_add((909_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn unrevoke_ed25519(r: u32) -> Weight {
        (52_775_000 as Weight)
            // Standard Error: 0
            .saturating_add((851_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn unrevoke_secp256k1(r: u32) -> Weight {
        (149_068_000 as Weight)
            // Standard Error: 0
            .saturating_add((897_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    fn remove_registry_sr25519() -> Weight {
        (143_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(103 as Weight))
    }
    fn remove_registry_ed25519() -> Weight {
        (141_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(103 as Weight))
    }
    fn remove_registry_secp256k1() -> Weight {
        (237_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(103 as Weight))
    }
    fn new_registry(_c: u32) -> Weight {
        (13_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
}
