use dock_testnet_runtime::{
    did::{self, Did, KeyDetail},
    master::Membership,
    AuraConfig, BalancesConfig, DIDModuleConfig, GenesisConfig, GrandpaConfig, MasterConfig,
    SystemConfig, WASM_BINARY,
};
use hex_literal::hex;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::Ss58Codec;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature,
};

type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <MultiSignature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key for Aura
pub fn get_authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Create a public key from an SS58 address
fn pubkey_from_ss58<T: Public>(ss58: &str) -> T {
    Ss58Codec::from_string(ss58).unwrap()
}

/// Create an account id from a SS58 address
fn account_id_from_ss58<T: Public>(ss58: &str) -> AccountId
where
    AccountPublic: From<T>,
{
    AccountPublic::from(pubkey_from_ss58::<T>(ss58)).into_account()
}

/// Create a non-secure development did with specified secret key
fn did_from_seed(did: &[u8; 32], seed: &[u8; 32]) -> (Did, KeyDetail) {
    let pk = sr25519::Pair::from_seed(seed).public().0;
    (
        *did,
        KeyDetail::new(*did, did::PublicKey::Sr25519(did::Bytes32 { value: pk })),
    )
}

pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        || {
            GenesisBuilder {
                initial_authorities: vec![get_authority_keys_from_seed("Alice")],
                endowed_accounts: ["Alice", "Bob", "Alice//stash", "Bob//stash"]
                    .iter()
                    .cloned()
                    .map(get_account_id_from_seed::<sr25519::Public>)
                    .collect(),
                master: Membership {
                    members: [
                        b"Alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Charlie\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ]
                    .iter()
                    .cloned()
                    .cloned()
                    .collect(),
                    vote_requirement: 2,
                },
                dids: [
                    (
                        b"Alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Alicesk\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    (
                        b"Bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Bobsk\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    (
                        b"Charlie\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Charliesk\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                ]
                .iter()
                .map(|(name, sk)| did_from_seed(name, sk))
                .collect(),
            }
            .build()
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        || {
            GenesisBuilder {
                initial_authorities: vec![
                    get_authority_keys_from_seed("Alice"),
                    get_authority_keys_from_seed("Bob"),
                ],
                endowed_accounts: [
                    "Alice",
                    "Bob",
                    "Charlie",
                    "Dave",
                    "Eve",
                    "Ferdie",
                    "Alice//stash",
                    "Bob//stash",
                    "Charlie//stash",
                    "Dave//stash",
                    "Eve//stash",
                    "Ferdie//stash",
                ]
                .iter()
                .cloned()
                .map(get_account_id_from_seed::<sr25519::Public>)
                .collect(),
                master: Membership {
                    members: [
                        b"Alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Charlie\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ]
                    .iter()
                    .cloned()
                    .cloned()
                    .collect(),
                    vote_requirement: 2,
                },
                dids: [
                    (
                        b"Alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Alicesk\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    (
                        b"Bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Bobsk\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    (
                        b"Charlie\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"Charliesk\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                ]
                .iter()
                .map(|(name, sk)| did_from_seed(name, sk))
                .collect(),
            }
            .build()
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

pub fn remote_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "RemoteDevelopment",
        "remdev",
        ChainType::Live,
        || {
            GenesisBuilder {
                initial_authorities: vec![
                    (
                        pubkey_from_ss58::<AuraId>(
                            "5FkKCjCwd36ztkEKatp3cAbuUWjUECi4y5rQnpkoEeagTimD",
                        ),
                        pubkey_from_ss58::<GrandpaId>(
                            "5CemoFcouqEdmBgMYjQwkFjBFPzLRc5jcXyjD8dKvqBWwhfh",
                        ),
                    ),
                    (
                        pubkey_from_ss58::<AuraId>(
                            "5DfRTtDzNyLuoCV77im5D6UyUx62HxmNYYvtkepaGaeMmoKu",
                        ),
                        pubkey_from_ss58::<GrandpaId>(
                            "5FJir6hEEWvVCt4PHJ95ygtw5MvgD2xoET9xqskTu4MZBC98",
                        ),
                    ),
                ],
                endowed_accounts: [
                    "5CUrmmBsA7oPP2uJ58yPTjZn7dUpFzD1MtRuwLdoPQyBnyWM",
                    "5DS9inxHmk3qLvTu1ZDWF9GrvkJRCR2xeWdCfa1k7dwwL1e2",
                ]
                .iter()
                .cloned()
                .map(account_id_from_ss58::<sr25519::Public>)
                .collect(),
                master: Membership {
                    members: [
                        b"nm\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"nl\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        b"ec\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ]
                    .iter()
                    .cloned()
                    .cloned()
                    .collect(),
                    vote_requirement: 2,
                },
                dids: [
                    (
                        b"nm\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        hex!("2a6f70c3dc8cd003075bbf14567c4251b512c5514dff069c293c14679f91913d"),
                    ),
                    (
                        b"nl\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        hex!("848001ef27f057719a31e0e457d4edd946c5792d03a8cb203bc025bdda825301"),
                    ),
                    (
                        b"ec\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                        hex!("c85c62af598cb718ce4bd1b0b739605fa7a4252db508ceb23dbd3eb4ca523062"),
                    ),
                ]
                .iter()
                .cloned()
                .map(|(did, pk)| {
                    (
                        *did,
                        KeyDetail::new(*did, did::PublicKey::Sr25519(did::Bytes32 { value: pk })),
                    )
                })
                .collect(),
            }
            .build()
        },
        vec![
            "/dns4/testnet-bootstrap1.dock.io/tcp/30333/p2p/\
             QmaWVer8pXKR8AM6u2B8r9gXivTW9vTitb6gjLM6FYQcXS"
                .parse()
                .unwrap(),
            "/dns4/testnet-bootstrap2.dock.io/tcp/30333/p2p/\
             QmPSP1yGiECdm5wVXVDF9stGfvVPSY8QUT4PhYB4Gnk77Q"
                .parse()
                .unwrap(),
        ],
        None,
        None,
        None,
        None,
    )
}

struct GenesisBuilder {
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    endowed_accounts: Vec<AccountId>,
    master: Membership,
    dids: Vec<(Did, KeyDetail)>,
}

impl GenesisBuilder {
    fn build(self) -> GenesisConfig {
        self.validate().unwrap();
        GenesisConfig {
            system: Some(SystemConfig {
                code: WASM_BINARY.to_vec(),
                changes_trie_config: Default::default(),
            }),
            balances: Some(BalancesConfig {
                balances: self
                    .endowed_accounts
                    .iter()
                    .cloned()
                    .map(|k| (k, 1 << 60))
                    .collect(),
            }),
            aura: Some(AuraConfig {
                authorities: self
                    .initial_authorities
                    .iter()
                    .map(|x| (x.0.clone()))
                    .collect(),
            }),
            grandpa: Some(GrandpaConfig {
                authorities: self
                    .initial_authorities
                    .iter()
                    .map(|x| (x.1.clone(), 1))
                    .collect(),
            }),
            master: Some(MasterConfig {
                members: self.master,
            }),
            did: Some(DIDModuleConfig { dids: self.dids }),
        }
    }

    fn validate(&self) -> Result<(), String> {
        // Every DID in master must be pre-declared
        for did in self.master.members.iter() {
            if !self.dids.iter().any(|(k, _v)| k == did) {
                return Err(format!(
                    "Master contains DID {:x?}.. that is not pre-declared",
                    did[0],
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn expected_did_from_seed() {
        let did = [1u8; 32];
        let pk = hex!("c02bab578b07e7e41997fcb03de683f4780e3ad383e573d817d2462f4a27c701");
        let sk = hex!("d2a75ce109331dde9b6f7782d56b0668ad2f7ad53d32aec4c1618292c2127e87");
        assert_eq!(
            did_from_seed(&did, &sk),
            (
                did,
                KeyDetail::new(did, did::PublicKey::Sr25519(did::Bytes32 { value: pk })),
            )
        );
    }
}
