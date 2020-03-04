use codec::{Encode, Decode};
use sp_std::prelude::Vec;
use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult, traits::Get};

/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    //type DIDByteSize: Get<u8>;
}

pub const DID_BYTE_SIZE: usize = 32;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub enum PublicKeyType {
    Sr25519,
    Ed25519,
    Secp256k1
}

impl Default for PublicKeyType {
    fn default() -> Self {
        PublicKeyType::Sr25519
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct KeyDetail {
    controller: [u8; DID_BYTE_SIZE],
    //controller: [u8; 32],
    public_key_type: PublicKeyType,
    public_key: Vec<u8>,
}

// XXX: Map requires having a default value for DIDDetail
impl Default for KeyDetail {
    fn default() -> Self {
        KeyDetail {
            //controller: [0; DID_BYTE_SIZE],
            controller: [0; 32],
            public_key_type: PublicKeyType::default(),
            public_key: Vec::new()
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId
    {
        DIDAdded(Vec<u8>),
        DIDAlreadyExists(Vec<u8>),
        DummyEvent(AccountId),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as DidModule {
        Dids get(did): map [u8; DID_BYTE_SIZE] => (KeyDetail, T::BlockNumber);
        //Dids: map [u8; 32] => (KeyDetail, T::BlockNumber);
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        //fn new(_origin, did: [u8; DID_BYTE_SIZE], detail: KeyDetail) -> DispatchResult {
        fn new(_origin, did: [u8; 32], detail: KeyDetail) -> DispatchResult {
            if Dids::<T>::exists(did) {
                Self::deposit_event(RawEvent::DIDAlreadyExists(did.to_vec()));
            } else {
                let current_block_no = <system::Module<T>>::block_number();
                Dids::<T>::insert(did, (detail, current_block_no));
                Self::deposit_event(RawEvent::DIDAdded(did.to_vec()));
            }
            Ok(())
        }

    }
}