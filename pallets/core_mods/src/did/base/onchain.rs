use super::super::*;
use crate::util::WrappedActionWithNonce;
use crate::ToStateChange;

/// Each on-chain DID is associated with a nonce that is incremented each time the DID does a write (through an extrinsic)
pub type StoredOnChainDidDetails<T> = WithNonce<T, OnChainDidDetails>;

/// Stores details of an on-chain DID.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct OnChainDidDetails {
    /// Number of keys added for this DID so far.
    pub key_counter: IncId,
    /// Number of currently active controller keys.
    pub active_controller_keys: u32,
    /// Number of currently active controllers.
    pub active_controllers: u32,
}

impl<T: Config> From<StoredOnChainDidDetails<T>> for StoredDidDetails<T> {
    fn from(details: StoredOnChainDidDetails<T>) -> Self {
        Self::OnChain(details)
    }
}

impl<T: Config + Debug> TryFrom<StoredDidDetails<T>> for StoredOnChainDidDetails<T> {
    type Error = Error<T>;

    fn try_from(details: StoredDidDetails<T>) -> Result<Self, Self::Error> {
        details
            .into_onchain()
            .ok_or(Error::<T>::CannotGetDetailForOnChainDid)
    }
}

impl OnChainDidDetails {
    /// Constructs new on-chain DID details using supplied params.
    ///
    /// - `key_counter` - last incremental identifier of the key being used for the given DID.
    /// - `active_controller_keys` - amount of currently active controller keys for the given DID.
    /// - `active_controllers` - amount of currently active controllers for the given DID.
    pub fn new(key_counter: IncId, active_controller_keys: u32, active_controllers: u32) -> Self {
        Self {
            key_counter,
            active_controller_keys: active_controller_keys.into(),
            active_controllers: active_controllers.into(),
        }
    }
}

impl<T: Config + Debug> Module<T> {
    pub(crate) fn new_onchain_(
        did: Did,
        keys: Vec<DidKey>,
        mut controllers: BTreeSet<Controller>,
    ) -> Result<(), Error<T>> {
        // DID is not registered already
        ensure!(!Dids::<T>::contains_key(did), Error::<T>::DidAlreadyExists);

        let (keys_to_insert, controller_keys_count) = Self::prepare_keys_to_insert(keys)?;
        // Make self controlled if needed
        if controller_keys_count > 0 {
            controllers.insert(Controller(did));
        }
        ensure!(!controllers.is_empty(), Error::<T>::NoControllerProvided);

        let mut key_counter = IncId::new();
        for (key, key_id) in keys_to_insert.into_iter().zip(&mut key_counter) {
            DidKeys::insert(&did, key_id, key);
        }

        for ctrl in &controllers {
            DidControllers::insert(&did, &ctrl, ());
        }

        let did_details = WithNonce::new(OnChainDidDetails::new(
            key_counter,
            controller_keys_count,
            controllers.len() as u32,
        ));

        Self::insert_did_details(did, did_details);

        deposit_indexed_event!(OnChainDidAdded(did));
        Ok(())
    }

    pub(crate) fn remove_onchain_did_(
        DidRemoval { did, .. }: DidRemoval<T>,
        details: &mut Option<OnChainDidDetails>,
    ) -> Result<(), Error<T>> {
        // This will result in the removal of DID from storage map `Dids`
        details.take();
        DidKeys::remove_prefix(did);
        DidControllers::remove_prefix(did);
        DidServiceEndpoints::remove_prefix(did);

        deposit_indexed_event!(OnChainDidRemoved(did));
        Ok(())
    }

    /// Verifies signature, then executes action over target on-chain DID providing
    /// a mutable reference if the given nonce is correct, i.e. 1 more than the current nonce.
    pub(crate) fn try_exec_signed_action_over_onchain_did<A, F, R, E>(
        action: A,
        signature: DidSignature<Controller>,
        f: F,
    ) -> Result<R, E>
    where
        F: FnOnce(A, &mut OnChainDidDetails) -> Result<R, E>,
        A: ActionWithNonce<T> + ToStateChange<T>,
        A::Target: Into<Did>,
        E: From<Error<T>> + From<NonceError>,
    {
        ensure!(
            Self::verify_sig_from_controller(&action, &signature)?,
            Error::<T>::InvalidSignature
        );

        Self::try_exec_action_over_onchain_did(action, f)
    }

    /// Verifies signature, then executes action over target on-chain DID providing a mutable reference
    /// if the given nonce is correct, i.e. 1 more than the current nonce.
    /// Unlike `try_exec_signed_action_over_onchain_did`, this action may result in a removal of a DID,
    /// if the value under option will be taken.
    pub(crate) fn try_exec_signed_removable_action_over_onchain_did<A, F, R, E>(
        action: A,
        signature: DidSignature<Controller>,
        f: F,
    ) -> Result<R, E>
    where
        F: FnOnce(A, &mut Option<OnChainDidDetails>) -> Result<R, E>,
        A: ActionWithNonce<T> + ToStateChange<T>,
        A::Target: Into<Did>,
        E: From<Error<T>> + From<NonceError>,
    {
        ensure!(
            Self::verify_sig_from_controller(&action, &signature)?,
            Error::<T>::InvalidSignature
        );

        Self::try_exec_removable_action_over_onchain_did(action, f)
    }

    /// Try executing an action by a DID. Each action of a DID is supposed to have a nonce which should
    /// be one more than the current one. This function will check that payload has correct nonce and
    /// will then execute the given function `f` on te action and if `f` executes successfully, it will increment
    /// the DID's nonce by 1.
    pub(crate) fn try_exec_signed_action_from_onchain_did<A, F, S, R, E>(
        action: A,
        signature: DidSignature<S>,
        f: F,
    ) -> Result<R, E>
    where
        F: FnOnce(A, S) -> Result<R, E>,
        A: ActionWithNonce<T, Target = ()> + ToStateChange<T>,
        S: Into<Did> + Copy,
        E: From<Error<T>> + From<NonceError>,
    {
        ensure!(
            Self::verify_sig_from_auth_or_control_key(&action, &signature)?,
            Error::<T>::InvalidSignature
        );

        Self::try_exec_action_over_onchain_did(
            WrappedActionWithNonce::new(action.nonce(), signature.did, action),
            |WrappedActionWithNonce { action, target, .. }, _| f(action, target),
        )
    }

    /// Executes action over target on-chain DID providing a mutable reference if the given
    /// nonce is correct, i.e. 1 more than the current nonce.
    pub(crate) fn try_exec_action_over_onchain_did<A, F, R, E>(action: A, f: F) -> Result<R, E>
    where
        F: FnOnce(A, &mut OnChainDidDetails) -> Result<R, E>,
        A: ActionWithNonce<T>,
        A::Target: Into<Did>,
        E: From<Error<T>> + From<NonceError>,
    {
        Self::try_exec_removable_action_over_onchain_did(action, |action, details_opt| {
            f(action, details_opt.as_mut().unwrap())
        })
    }

    /// Executes action over target on-chain DID providing a mutable reference if the given
    /// nonce is correct, i.e. 1 more than the current nonce.
    /// Unlike `try_exec_action_over_onchain_did`, this action may result in a removal of a DID,
    /// if the value under option will be taken.
    pub(crate) fn try_exec_removable_action_over_onchain_did<A, F, R, E>(
        action: A,
        f: F,
    ) -> Result<R, E>
    where
        F: FnOnce(A, &mut Option<OnChainDidDetails>) -> Result<R, E>,
        A: ActionWithNonce<T>,
        A::Target: Into<Did>,
        E: From<Error<T>> + From<NonceError>,
    {
        ensure!(!action.is_empty(), Error::<T>::EmptyPayload);

        Dids::<T>::try_mutate_exists(action.target().into(), |details_opt| {
            WithNonce::try_update_opt_with(details_opt, action.nonce(), |data_opt| {
                f(action, data_opt)
            })
            .ok_or(Error::<T>::DidDoesNotExist)?
        })
    }

    pub fn is_onchain_did(did: &Did) -> Result<bool, Error<T>> {
        Self::did(did)
            .as_ref()
            .map(StoredDidDetails::is_onchain)
            .ok_or(Error::<T>::DidDoesNotExist)
    }

    /// Get DID detail of an on-chain DID. Throws error if DID does not exist or is off-chain.
    pub fn onchain_did_details(did: &Did) -> Result<StoredOnChainDidDetails<T>, Error<T>> {
        Self::did(did)
            .ok_or(Error::<T>::DidDoesNotExist)?
            .try_into()
    }
}
