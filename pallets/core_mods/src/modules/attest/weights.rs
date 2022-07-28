//! Autogenerated weights for attest
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-07-07, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("mainnet"), DB CACHE: 128

// Executed Command:
// ./target/release/dock-node
// benchmark
// --execution
// wasm
// --chain
// mainnet
// --wasm-execution
// compiled
// --pallet
// attest
// --extra
// --extrinsic
// *
// --repeat
// 20
// --steps
// 50
// --output
// ./pallets/core_mods/src/modules/attest/weights.rs
// --template
// node/module-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for attest.
pub trait WeightInfo {
    fn set_claim_sr25519(l: u32) -> Weight;
    fn set_claim_ed25519(l: u32) -> Weight;
    fn set_claim_secp256k1(l: u32) -> Weight;
}

/// Weights for attest using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn set_claim_sr25519(l: u32) -> Weight {
        (80_837_000 as Weight)
            // Standard Error: 0
            .saturating_add((5_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn set_claim_ed25519(l: u32) -> Weight {
        (77_113_000 as Weight)
            // Standard Error: 0
            .saturating_add((4_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn set_claim_secp256k1(l: u32) -> Weight {
        (363_435_000 as Weight)
            // Standard Error: 0
            .saturating_add((6_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn set_claim_sr25519(l: u32) -> Weight {
        (80_837_000 as Weight)
            // Standard Error: 0
            .saturating_add((5_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn set_claim_ed25519(l: u32) -> Weight {
        (77_113_000 as Weight)
            // Standard Error: 0
            .saturating_add((4_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn set_claim_secp256k1(l: u32) -> Weight {
        (363_435_000 as Weight)
            // Standard Error: 0
            .saturating_add((6_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
}
