//! Autogenerated weights for bbs_plus
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-08-01, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Native), WASM-EXECUTION: Interpreted, CHAIN: Some("mainnet"), DB CACHE: 128

// Executed Command:
// ./target/production/dock-node
// benchmark
// --execution=native
// --chain=mainnet
// --pallet=bbs_plus
// --extra
// --extrinsic=*
// --repeat=20
// --steps=50
// --template=node/module-weight-template.hbs
// --output=./pallets/core_mods/src/modules/bbs_plus/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for bbs_plus.
pub trait WeightInfo {
    fn add_params_sr25519(b: u32, l: u32) -> Weight;
    fn add_params_ed25519(b: u32, l: u32) -> Weight;
    fn add_params_secp256k1(b: u32, l: u32) -> Weight;
    fn remove_params_sr25519() -> Weight;
    fn remove_params_ed25519() -> Weight;
    fn remove_params_secp256k1() -> Weight;
    fn add_public_sr25519(b: u32) -> Weight;
    fn add_public_ed25519(b: u32) -> Weight;
    fn add_public_secp256k1(b: u32) -> Weight;
    fn remove_public_sr25519() -> Weight;
    fn remove_public_ed25519() -> Weight;
    fn remove_public_secp256k1() -> Weight;
}

/// Weights for bbs_plus using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn add_params_sr25519(b: u32, l: u32) -> Weight {
        (52_181_000 as Weight)
            // Standard Error: 0
            .saturating_add((7_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 0
            .saturating_add((9_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn add_params_ed25519(b: u32, l: u32) -> Weight {
        (52_658_000 as Weight)
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn add_params_secp256k1(b: u32, l: u32) -> Weight {
        (154_268_000 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 1_000
            .saturating_add((2_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn remove_params_sr25519() -> Weight {
        (56_041_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn remove_params_ed25519() -> Weight {
        (52_544_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn remove_params_secp256k1() -> Weight {
        (155_224_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn add_public_sr25519(b: u32) -> Weight {
        (59_312_000 as Weight)
            // Standard Error: 0
            .saturating_add((12_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn add_public_ed25519(b: u32) -> Weight {
        (58_693_000 as Weight)
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn add_public_secp256k1(_b: u32) -> Weight {
        (162_846_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn remove_public_sr25519() -> Weight {
        (59_284_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn remove_public_ed25519() -> Weight {
        (57_625_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn remove_public_secp256k1() -> Weight {
        (161_804_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn add_params_sr25519(b: u32, l: u32) -> Weight {
        (52_181_000 as Weight)
            // Standard Error: 0
            .saturating_add((7_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 0
            .saturating_add((9_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn add_params_ed25519(b: u32, l: u32) -> Weight {
        (52_658_000 as Weight)
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn add_params_secp256k1(b: u32, l: u32) -> Weight {
        (154_268_000 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 1_000
            .saturating_add((2_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn remove_params_sr25519() -> Weight {
        (56_041_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn remove_params_ed25519() -> Weight {
        (52_544_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn remove_params_secp256k1() -> Weight {
        (155_224_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn add_public_sr25519(b: u32) -> Weight {
        (59_312_000 as Weight)
            // Standard Error: 0
            .saturating_add((12_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn add_public_ed25519(b: u32) -> Weight {
        (58_693_000 as Weight)
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn add_public_secp256k1(_b: u32) -> Weight {
        (162_846_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn remove_public_sr25519() -> Weight {
        (59_284_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn remove_public_ed25519() -> Weight {
        (57_625_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn remove_public_secp256k1() -> Weight {
        (161_804_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
}
