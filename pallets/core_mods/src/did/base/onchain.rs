use sp_runtime::DispatchError;

use super::super::*;

pub type StoredOnChainDidDetails<T> = WithNonce<T, OnChainDidDetails>;

/// Stores details of an on-chain DID.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct OnChainDidDetails {
    /// Number of keys added for this DID so far.
    pub last_key_id: IncId,
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

impl OnChainDidDetails {
    /// Constructs new on-chain DID details using supplied params.
    ///
    /// - `last_key_id` - last incremental identifier of the key being used for the given DID.
    /// - `active_controller_keys` - amount of currenlty active controller keys for the given DID.
    /// - `active_controllers` - amount of currently active controllers for the given DID.
    pub fn new(
        last_key_id: IncId,
        active_controller_keys: impl Into<u32>,
        active_controllers: impl Into<u32>,
    ) -> Self {
        Self {
            last_key_id,
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

        let mut last_key_id = IncId::new();
        for (key, key_id) in keys_to_insert.into_iter().zip(&mut last_key_id) {
            DidKeys::insert(&did, key_id, key);
        }

        for ctrl in &controllers {
            DidControllers::insert(&did, &ctrl, ());
        }

        let did_details: StoredDidDetails<T> = StoredOnChainDidDetails::new(
            OnChainDidDetails::new(last_key_id, controller_keys_count, controllers.len() as u32),
        )
        .into();

        Dids::<T>::insert(did, did_details);

        deposit_indexed_event!(OnChainDidAdded(did));
        Ok(())
    }

    pub(crate) fn remove_onchain_did_(
        DidRemoval { did, .. }: DidRemoval<T>,
        details: &mut Option<OnChainDidDetails>,
    ) -> Result<Option<()>, DispatchError> {
        details.take();
        DidKeys::remove_prefix(did);
        DidControllers::remove_prefix(did);
        DidServiceEndpoints::remove_prefix(did);
        Dids::<T>::remove(did);

        deposit_indexed_event!(OnChainDidRemoved(did));
        Ok(None)
    }

    /// Executes action over target on-chain DID providing a mutable reference if the given nonce is correct,
    /// i.e. 1 more than the current nonce.
    /// Unlike `try_exec_onchain_did_action`, this action may result in a removal of a DID, if the value under option
    /// will be taken.
    pub(crate) fn try_exec_removable_onchain_did_action<A, F, R, E>(
        action: A,
        f: F,
    ) -> Result<R, DispatchError>
    where
        F: FnOnce(A, &mut Option<OnChainDidDetails>) -> Result<R, E>,
        A: ActionWithNonce<T>,
        A::Target: Into<Did>,
        DispatchError: From<Error<T>> + From<E>,
    {
        Dids::<T>::try_mutate_exists(action.target().into(), |details_opt| {
            let mut details = details_opt
                .take()
                .ok_or(Error::<T>::DidDoesNotExist)?
                .into_onchain()
                .ok_or(Error::<T>::CannotGetDetailForOffChainDid)?;
            details.try_inc_nonce(action.nonce())?;

            let WithNonce { data, nonce } = details;
            let mut data_opt = Some(data);
            let res = f(action, &mut data_opt)?;
            *details_opt = data_opt
                .map(|data| WithNonce { data, nonce })
                .map(Into::into);

            Ok(res)
        })
    }

    /// Executes action over target on-chain DID providing a mutable reference if the given nonce is correct,
    /// i.e. 1 more than the current nonce.
    pub(crate) fn try_exec_onchain_did_action<A, F, R, E>(
        action: A,
        f: F,
    ) -> Result<R, DispatchError>
    where
        F: FnOnce(A, &mut OnChainDidDetails) -> Result<R, E>,
        A: ActionWithNonce<T>,
        A::Target: Into<Did>,
        DispatchError: From<Error<T>> + From<E>,
    {
        Self::try_exec_removable_onchain_did_action(action, |action, details_opt| {
            f(action, details_opt.as_mut().unwrap())
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
            .into_onchain()
            .ok_or(Error::<T>::CannotGetDetailForOffChainDid)
    }
}
