use super::*;
use sp_std::borrow::Borrow;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct DidKey {
    /// The public key
    pub public_key: PublicKey,
    /// The different verification relationships the above key has with the DID.
    pub ver_rels: VerRelType,
}

bitflags::bitflags! {
    /// Different verification relation types specified in the DID spec here https://www.w3.org/TR/did-core/#verification-relationships.
    #[derive(Encode, Decode)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(try_from = "u16", into = "u16"))]
    pub struct VerRelType: u16 {
        /// No verification relation set.
        const NONE = 0;
        /// https://www.w3.org/TR/did-core/#authentication
        const AUTHENTICATION = 0b0001;
        /// https://www.w3.org/TR/did-core/#assertion
        const ASSERTION = 0b0010;
        /// A key must have this to control a DID
        /// https://www.w3.org/TR/did-core/#capability-invocation
        const CAPABILITY_INVOCATION = 0b0100;
        /// https://www.w3.org/TR/did-core/#key-agreement
        const KEY_AGREEMENT = 0b1000;
        /// Includes `AUTHENTICATION`, `ASSERTION`, `CAPABILITY_INVOCATION`.
        /// We might add more relationships in future but these 3 are all we care about now.
        const ALL_FOR_SIGNING = 0b0111;
    }
}

impl_bits_conversion! { VerRelType, u16 }

#[derive(Clone, Debug, Default)]
pub struct PublicKeyParams<T> {
    pub sr25519: T,
    pub ed25519: T,
    pub secp256k1: T,
    pub x25519: T,
}

impl<T> PublicKeyParams<T> {
    pub fn over_key<F: FnOnce(T) -> T>(mut self, public_key: &PublicKey, f: F) -> Self {
        match public_key {
            PublicKey::Sr25519(_) => self.sr25519 = f(self.sr25519),
            PublicKey::Ed25519(_) => self.ed25519 = f(self.ed25519),
            PublicKey::Secp256k1(_) => self.secp256k1 = f(self.secp256k1),
            PublicKey::X25519(_) => self.x25519 = f(self.x25519),
        };

        self
    }
}

impl DidKey {
    /// Constructs new `DidKey` using given public key and verification relationships.
    pub fn new(public_key: impl Into<PublicKey>, ver_rels: VerRelType) -> Self {
        DidKey {
            public_key: public_key.into(),
            ver_rels,
        }
    }

    /// Add all possible verification relationships for a given key
    pub fn new_with_all_relationships(public_key: impl Into<PublicKey>) -> Self {
        let public_key = public_key.into();
        let ver_rels = if public_key.can_sign() {
            // We might add more relationships in future but these 3 are all we care about now.
            VerRelType::ALL_FOR_SIGNING
        } else {
            // This is true for the current key type, X25519, used for key agreement but might
            // change in future.
            VerRelType::KEY_AGREEMENT
        };

        DidKey::new(public_key, ver_rels)
    }

    pub fn can_sign(&self) -> bool {
        self.public_key.can_sign()
    }

    /// Checks if the public key has valid verification relationships. Currently, the keys used for
    /// key-agreement cannot (without converting) be used for signing and vice versa
    pub fn is_valid(&self) -> bool {
        !self.can_sign() ^ (self.ver_rels & VerRelType::ALL_FOR_SIGNING == self.ver_rels)
    }

    pub fn can_control(&self) -> bool {
        self.is_valid() && self.ver_rels.intersects(VerRelType::CAPABILITY_INVOCATION)
    }

    pub fn can_authenticate(&self) -> bool {
        self.is_valid() && self.ver_rels.intersects(VerRelType::AUTHENTICATION)
    }

    pub fn for_key_agreement(&self) -> bool {
        self.is_valid() && self.ver_rels.intersects(VerRelType::KEY_AGREEMENT)
    }

    pub fn can_authenticate_or_control(&self) -> bool {
        self.is_valid()
            && self
                .ver_rels
                .intersects(VerRelType::AUTHENTICATION | VerRelType::CAPABILITY_INVOCATION)
    }
}

impl<T: Config + Debug> Module<T> {
    pub(crate) fn add_keys_(
        AddKeys { did, keys, .. }: AddKeys<T>,
        OnChainDidDetails {
            active_controllers,
            active_controller_keys,
            last_key_id,
            ..
        }: &mut OnChainDidDetails,
    ) -> Result<(), Error<T>> {
        // If DID was not self controlled first, check if it can become by looking
        let (keys_to_insert, controller_keys_count) = Self::prepare_keys_to_insert(keys)?;
        *active_controller_keys += controller_keys_count;

        // Make self controlled if needed
        let add_self_controlled = controller_keys_count > 0 && !Self::is_self_controlled(&did);
        if add_self_controlled {
            DidControllers::insert(&did, &Controller(did), ());
            *active_controllers += 1;
        }

        for (key, key_id) in keys_to_insert.iter().zip(last_key_id) {
            DidKeys::insert(did, key_id, key);
        }

        deposit_indexed_event!(DidKeysAdded(did));
        Ok(())
    }

    pub(crate) fn remove_keys_(
        RemoveKeys { did, keys, .. }: RemoveKeys<T>,
        OnChainDidDetails {
            active_controllers,
            active_controller_keys,
            ..
        }: &mut OnChainDidDetails,
    ) -> Result<(), Error<T>> {
        for key_id in &keys {
            let key = DidKeys::get(&did, key_id).ok_or(Error::<T>::NoKeyForDid)?;

            if key.can_control() {
                *active_controller_keys -= 1;
            }
        }

        for key in &keys {
            DidKeys::remove(did, key);
        }

        // If no self-control keys exist for the given DID, remove self-control
        let remove_self_controlled = *active_controller_keys == 0 && Self::is_self_controlled(&did);
        if remove_self_controlled {
            DidControllers::remove(&did, &Controller(did));
            *active_controllers -= 1;
        }

        deposit_indexed_event!(DidKeysRemoved(did));
        Ok(())
    }

    /// Return `did`'s key with id `key_id` only if has control capability, otherwise returns an error.
    pub fn control_key(did: &Controller, key_id: IncId) -> Result<PublicKey, Error<T>> {
        let did_key = DidKeys::get(did.0, key_id).ok_or(Error::<T>::NoKeyForDid)?;

        if did_key.can_control() {
            Ok(did_key.public_key)
        } else {
            Err(Error::<T>::InsufficientVerificationRelationship)
        }
    }

    /// Return `did`'s key with id `key_id` only if it can authenticate or control otherwise returns an error
    pub fn auth_or_control_key(did: &Did, key_id: IncId) -> Result<PublicKey, Error<T>> {
        let did_key = DidKeys::get(did, key_id).ok_or(Error::<T>::NoKeyForDid)?;

        if did_key.can_authenticate_or_control() {
            Ok(did_key.public_key)
        } else {
            Err(Error::<T>::InsufficientVerificationRelationship)
        }
    }

    /// Prepare `DidKey`s to insert. The DID is assumed to be self controlled as well if there is any key
    /// that is capable of invoking a capability. Returns the keys along with the
    /// amount of controller keys being met. The following logic is contentious.
    pub(crate) fn prepare_keys_to_insert(
        keys: Vec<DidKey>,
    ) -> Result<(Vec<DidKey>, u32), Error<T>> {
        let mut controller_keys_count = 0;
        let mut keys_to_insert = Vec::with_capacity(keys.len());
        for key in keys {
            let key = if key.ver_rels.is_empty() {
                DidKey::new_with_all_relationships(key.public_key)
            } else {
                if !key.is_valid() {
                    fail!(Error::<T>::IncompatibleVerificationRelation)
                }

                key
            };
            if key.can_control() {
                controller_keys_count += 1;
            }

            keys_to_insert.push(key);
        }

        Ok((keys_to_insert, controller_keys_count))
    }

    #[allow(unused)]
    pub(super) fn key_counts<I>(keys: I) -> PublicKeyParams<u32>
    where
        I: IntoIterator,
        I::Item: Borrow<DidKey>,
    {
        keys.into_iter().fold(Default::default(), |params, key| {
            params.over_key(&key.borrow().public_key, |val| val + 1)
        })
    }
}
