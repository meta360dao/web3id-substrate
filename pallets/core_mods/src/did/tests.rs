use super::*;

use crate::did::service_endpoints::ServiceEndpointType;
use crate::keys_and_sigs::{get_secp256k1_keypair, SigValue};
use crate::test_common::*;
use crate::util::{Bytes64, Bytes65};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::{ed25519, sr25519, Pair};

fn not_key_agreement(key: &DidKey) {
    assert!(key.can_sign());
    assert!(key.can_authenticate());
    assert!(key.can_control());
    assert!(key.can_authenticate_or_control());
    assert!(!key.for_key_agreement());
}

fn only_key_agreement(key: &DidKey) {
    assert!(!key.can_sign());
    assert!(!key.can_authenticate());
    assert!(!key.can_control());
    assert!(!key.can_authenticate_or_control());
    assert!(key.for_key_agreement());
}

fn check_did_detail(
    did: &Did,
    last_key_id: u32,
    active_controller_keys: u32,
    active_controllers: u32,
    nonce: <Test as frame_system::Config>::BlockNumber,
) {
    let did_detail = DIDModule::onchain_did_details(did).unwrap();
    assert_eq!(did_detail.last_key_id, last_key_id.into());
    assert_eq!(did_detail.active_controller_keys, active_controller_keys);
    assert_eq!(did_detail.active_controllers, active_controllers);
    assert_eq!(
        did_detail.nonce,
        <Test as system::Config>::BlockNumber::from(nonce)
    );
}

/// Ensure that all keys in storage corresponding to the DID are deleted. This check should be
/// performed when a DID is removed.
fn ensure_onchain_did_gone(did: &Did) {
    assert!(DIDModule::did(did).is_none());
    let mut i = 0;
    for (_, _) in DidKeys::iter_prefix(did) {
        i += 1;
    }
    assert_eq!(i, 0);
    for (_, _) in DidControllers::iter_prefix(did) {
        i += 1;
    }
    assert_eq!(i, 0);
    for (_, _) in DidServiceEndpoints::iter_prefix(did) {
        i += 1;
    }
    assert_eq!(i, 0);
}

#[test]
fn offchain_did() {
    // Creating an off-chain DID
    ext().execute_with(|| {
        let alice = 1u64;
        let did: Did = [5; Did::BYTE_SIZE].into();
        let doc_ref = OffChainDidDocRef::Custom(vec![129; 60].into());
        let too_big_doc_ref = OffChainDidDocRef::Custom(vec![129; 300].into());

        assert_noop!(
            DIDModule::new_offchain(Origin::signed(alice), did.clone(), too_big_doc_ref.clone()),
            Error::<Test>::DidDocUriTooBig
        );

        // Add a DID
        assert_ok!(DIDModule::new_offchain(
            Origin::signed(alice),
            did.clone(),
            doc_ref.clone()
        ));

        // Try to add the same DID and same uri again and fail
        assert_noop!(
            DIDModule::new_offchain(Origin::signed(alice), did.clone(), doc_ref.clone()),
            Error::<Test>::DidAlreadyExists
        );

        // Try to add the same DID and different uri and fail
        let doc_ref_1 = OffChainDidDocRef::URL(vec![205; 99].into());
        assert_noop!(
            DIDModule::new_offchain(Origin::signed(alice), did, doc_ref_1),
            Error::<Test>::DidAlreadyExists
        );

        assert!(DIDModule::is_offchain_did(&did).unwrap());
        assert!(!DIDModule::is_onchain_did(&did).unwrap());

        assert_noop!(
            DIDModule::onchain_did_details(&did),
            Error::<Test>::CannotGetDetailForOffChainDid
        );

        let did_detail_storage = Dids::<Test>::get(&did).unwrap();
        let OffChainDidDetails {
            account_id: owner,
            doc_ref: fetched_ref,
        } = did_detail_storage.into_offchain().unwrap();
        assert_eq!(owner, alice);
        assert_eq!(fetched_ref, doc_ref);

        let bob = 2u64;
        let new_ref = OffChainDidDocRef::CID(vec![235; 99].into());
        assert_noop!(
            DIDModule::set_offchain_did_uri(Origin::signed(bob), did, new_ref.clone()),
            Error::<Test>::DidNotOwnedByAccount
        );

        assert_noop!(
            DIDModule::set_offchain_did_uri(Origin::signed(alice), did.clone(), too_big_doc_ref),
            Error::<Test>::DidDocUriTooBig
        );

        assert_ok!(DIDModule::set_offchain_did_uri(
            Origin::signed(alice),
            did.clone(),
            new_ref.clone()
        ));
        let did_detail_storage = Dids::<Test>::get(&did).unwrap();
        let fetched_ref = did_detail_storage.into_offchain().unwrap().doc_ref;
        assert_eq!(fetched_ref, new_ref);

        assert_noop!(
            DIDModule::remove_offchain_did(Origin::signed(bob), did),
            Error::<Test>::DidNotOwnedByAccount
        );

        assert_ok!(DIDModule::remove_offchain_did(Origin::signed(alice), did));
        assert!(Dids::<Test>::get(&did).is_none());
    });
}

#[test]
fn onchain_keyless_did_creation() {
    // Creating an on-chain DID with no keys but only controllers, i.e. DID is controlled by other DIDs
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [5; Did::BYTE_SIZE].into();
        let did_2: Did = [3; Did::BYTE_SIZE].into();
        let controller_1 = Controller([7; Did::BYTE_SIZE].into());
        let controller_2 = Controller([20; Did::BYTE_SIZE].into());

        assert_noop!(
            DIDModule::new_onchain(
                Origin::signed(alice),
                did_1.clone(),
                vec![],
                vec![].into_iter().collect()
            ),
            Error::<Test>::NoControllerProvided
        );

        run_to_block(20);
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![].into_iter().collect(),
            vec![controller_1].into_iter().collect()
        ));

        assert!(!DIDModule::is_offchain_did(&did_1).unwrap());
        assert!(DIDModule::is_onchain_did(&did_1).unwrap());

        assert!(!DIDModule::is_self_controlled(&did_1));
        assert!(!DIDModule::is_controller(&did_1, &controller_2));
        assert!(DIDModule::is_controller(&did_1, &controller_1));

        check_did_detail(&did_1, 0, 0, 1, 20);

        assert_noop!(
            DIDModule::new_onchain(
                Origin::signed(alice),
                did_1.clone(),
                vec![].into_iter().collect(),
                vec![controller_1].into_iter().collect()
            ),
            Error::<Test>::DidAlreadyExists
        );

        run_to_block(55);
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![].into_iter().collect(),
            vec![Controller(did_1), controller_1, controller_2]
                .into_iter()
                .collect()
        ));

        assert!(!DIDModule::is_offchain_did(&did_2).unwrap());
        assert!(DIDModule::is_onchain_did(&did_2).unwrap());

        assert!(!DIDModule::is_self_controlled(&did_2));
        assert!(DIDModule::is_controller(&did_2, &Controller(did_1)));
        assert!(DIDModule::is_controller(&did_2, &controller_1));
        assert!(DIDModule::is_controller(&did_2, &controller_2));

        check_did_detail(&did_2, 0, 0, 3, 55);
    });
}

#[test]
fn onchain_keyed_did_creation_with_self_control() {
    // Creating an on-chain DID with keys but no other controllers
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [5; Did::BYTE_SIZE].into();
        let did_2: Did = [4; Did::BYTE_SIZE].into();
        let did_3: Did = [3; Did::BYTE_SIZE].into();
        let did_4: Did = [2; Did::BYTE_SIZE].into();
        let did_5: Did = [11; Did::BYTE_SIZE].into();
        let did_6: Did = [111; Did::BYTE_SIZE].into();
        let did_7: Did = [71; Did::BYTE_SIZE].into();
        let did_8: Did = [82; Did::BYTE_SIZE].into();
        let did_9: Did = [83; Did::BYTE_SIZE].into();
        let did_10: Did = [84; Did::BYTE_SIZE].into();
        let did_11: Did = [85; Did::BYTE_SIZE].into();

        let (pair_sr, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr = pair_sr.public().0;
        let (pair_ed, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed = pair_ed.public().0;
        let (_, pk_secp) = get_secp256k1_keypair(&[21; 32]);

        run_to_block(5);

        // DID controls itself when adding keys capable of signing without specifying any verificatiion relationship
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr),
                ver_rels: VerRelType::NONE.into()
            }],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_1, 1, 1, 1, 5);

        let key_1 = DidKeys::get(&did_1, IncId::from(1u32)).unwrap();
        not_key_agreement(&key_1);

        run_to_block(6);

        // DID controls itself and specifies another controller as well
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![DidKey {
                public_key: PublicKey::ed25519(pk_ed),
                ver_rels: VerRelType::NONE.into()
            }],
            vec![did_1].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 1, 1, 2, 6);

        let key_2 = DidKeys::get(&did_2, IncId::from(1u32)).unwrap();
        not_key_agreement(&key_2);

        run_to_block(7);

        // DID controls itself and specifies multiple another controllers as well
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_3.clone(),
            vec![DidKey {
                public_key: pk_secp.clone(),
                ver_rels: VerRelType::NONE.into()
            }],
            vec![did_1, did_2].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_3));
        check_did_detail(&did_3, 1, 1, 3, 7);

        let key_3 = DidKeys::get(&did_3, IncId::from(1u32)).unwrap();
        not_key_agreement(&key_3);

        run_to_block(8);

        // Adding x25519 key does not make the DID self controlled
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_4.clone(),
            vec![DidKey {
                public_key: PublicKey::x25519(pk_ed),
                ver_rels: VerRelType::NONE.into()
            }],
            vec![Controller(did_3.clone())].into_iter().collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_4));
        check_did_detail(&did_4, 1, 0, 1, 8);

        let key_4 = DidKeys::get(&did_4, IncId::from(1u32)).unwrap();
        only_key_agreement(&key_4);

        // x25519 key cannot be added for incompatible relationship types
        for vr in vec![
            VerRelType::AUTHENTICATION,
            VerRelType::ASSERTION,
            VerRelType::CAPABILITY_INVOCATION,
        ] {
            assert_noop!(
                DIDModule::new_onchain(
                    Origin::signed(alice),
                    did_5.clone(),
                    vec![DidKey {
                        public_key: PublicKey::x25519(pk_ed),
                        ver_rels: vr.into()
                    }],
                    vec![].into_iter().collect()
                ),
                Error::<Test>::IncompatibleVerificationRelation
            );
        }

        for pk in vec![
            PublicKey::sr25519(pk_sr),
            PublicKey::ed25519(pk_ed),
            pk_secp.clone(),
        ] {
            assert_noop!(
                DIDModule::new_onchain(
                    Origin::signed(alice),
                    did_5.clone(),
                    vec![DidKey {
                        public_key: pk,
                        ver_rels: VerRelType::KEY_AGREEMENT.into()
                    }],
                    vec![].into_iter().collect()
                ),
                Error::<Test>::IncompatibleVerificationRelation
            );
        }

        run_to_block(10);

        // Add single key and specify relationship as `capabilityInvocation`
        for (did, pk) in vec![
            (did_5, PublicKey::sr25519(pk_sr)),
            (did_6, PublicKey::ed25519(pk_ed)),
            (did_7, pk_secp.clone()),
        ] {
            assert_ok!(DIDModule::new_onchain(
                Origin::signed(alice),
                did.clone(),
                vec![DidKey {
                    public_key: pk,
                    ver_rels: VerRelType::CAPABILITY_INVOCATION.into()
                }],
                vec![].into_iter().collect()
            ));
            assert!(DIDModule::is_self_controlled(&did));
            let key = DidKeys::get(&did, IncId::from(1u32)).unwrap();
            assert!(key.can_sign());
            assert!(!key.can_authenticate());
            assert!(key.can_control());
            assert!(key.can_authenticate_or_control());
            assert!(!key.for_key_agreement());
            check_did_detail(&did, 1, 1, 1, 10);
        }

        run_to_block(13);

        // Add single key with single relationship and but do not specify relationship as `capabilityInvocation`
        for (did, pk, vr) in vec![
            (
                [72; Did::BYTE_SIZE],
                PublicKey::sr25519(pk_sr),
                VerRelType::ASSERTION,
            ),
            (
                [73; Did::BYTE_SIZE],
                PublicKey::ed25519(pk_ed),
                VerRelType::ASSERTION,
            ),
            ([74; Did::BYTE_SIZE], pk_secp.clone(), VerRelType::ASSERTION),
            (
                [75; Did::BYTE_SIZE],
                PublicKey::sr25519(pk_sr),
                VerRelType::AUTHENTICATION,
            ),
            (
                [76; Did::BYTE_SIZE],
                PublicKey::ed25519(pk_ed),
                VerRelType::AUTHENTICATION,
            ),
            (
                [77; Did::BYTE_SIZE],
                pk_secp.clone(),
                VerRelType::AUTHENTICATION,
            ),
        ] {
            let did: Did = did.into();
            assert_ok!(DIDModule::new_onchain(
                Origin::signed(alice),
                did.clone(),
                vec![DidKey {
                    public_key: pk,
                    ver_rels: vr.into()
                }],
                vec![Controller(did_1.clone())].into_iter().collect()
            ));
            assert!(!DIDModule::is_self_controlled(&did));
            let key = DidKeys::get(&did, IncId::from(1u32)).unwrap();
            assert!(key.can_sign());
            assert!(!key.can_control());
            if vr == VerRelType::AUTHENTICATION {
                assert!(key.can_authenticate());
                assert!(key.can_authenticate_or_control());
            }
            assert!(!key.for_key_agreement());
            check_did_detail(&did, 1, 0, 1, 13);
        }

        run_to_block(19);

        // Add single key, specify multiple relationships and but do not specify relationship as `capabilityInvocation`
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_8.clone(),
            vec![DidKey {
                public_key: PublicKey::ed25519(pk_ed),
                ver_rels: (VerRelType::AUTHENTICATION | VerRelType::ASSERTION).into()
            }],
            vec![Controller(did_9)].into_iter().collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_8));
        let key_8 = DidKeys::get(&did_8, IncId::from(1u32)).unwrap();
        assert!(key_8.can_sign());
        assert!(key_8.can_authenticate());
        assert!(!key_8.can_control());
        check_did_detail(&did_8, 1, 0, 1, 19);

        run_to_block(20);

        // Add multiple keys and specify multiple relationships
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_9.clone(),
            vec![
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed),
                    ver_rels: VerRelType::AUTHENTICATION.into()
                },
                DidKey {
                    public_key: PublicKey::sr25519(pk_sr),
                    ver_rels: VerRelType::ASSERTION.into()
                },
                DidKey {
                    public_key: pk_secp.clone(),
                    ver_rels: (VerRelType::ASSERTION | VerRelType::AUTHENTICATION).into()
                },
            ],
            vec![Controller(did_8.clone())].into_iter().collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_9));
        let key_9_1 = DidKeys::get(&did_9, IncId::from(1u32)).unwrap();
        assert!(key_9_1.can_sign());
        assert!(key_9_1.can_authenticate());
        assert!(!key_9_1.can_control());
        let key_9_2 = DidKeys::get(&did_9, IncId::from(2u32)).unwrap();
        assert!(key_9_2.can_sign());
        assert!(!key_9_2.can_authenticate());
        assert!(!key_9_2.can_control());
        let key_9_3 = DidKeys::get(&did_9, IncId::from(3u32)).unwrap();
        assert!(key_9_3.can_sign());
        assert!(key_9_3.can_authenticate());
        assert!(!key_9_3.can_control());
        check_did_detail(&did_9, 3, 0, 1, 20);

        run_to_block(22);

        // Add multiple keys and specify multiple relationships
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_10.clone(),
            vec![
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed),
                    ver_rels: (VerRelType::AUTHENTICATION | VerRelType::ASSERTION).into()
                },
                DidKey {
                    public_key: PublicKey::sr25519(pk_sr),
                    ver_rels: VerRelType::ASSERTION.into()
                },
                DidKey {
                    public_key: pk_secp,
                    ver_rels: VerRelType::CAPABILITY_INVOCATION.into()
                },
            ],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_10));
        let key_10_1 = DidKeys::get(&did_10, IncId::from(1u32)).unwrap();
        assert!(key_10_1.can_sign());
        assert!(key_10_1.can_authenticate());
        assert!(!key_10_1.can_control());
        let key_10_2 = DidKeys::get(&did_10, IncId::from(2u32)).unwrap();
        assert!(key_10_2.can_sign());
        assert!(!key_10_2.can_authenticate());
        assert!(!key_10_2.can_control());
        let key_10_3 = DidKeys::get(&did_10, IncId::from(3u32)).unwrap();
        assert!(key_10_3.can_sign());
        assert!(!key_10_3.can_authenticate());
        assert!(key_10_3.can_control());
        check_did_detail(&did_10, 3, 1, 1, 22);

        run_to_block(23);

        // Add multiple keys, specify multiple relationships and other controllers as well
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_11.clone(),
            vec![
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed),
                    ver_rels: (VerRelType::AUTHENTICATION | VerRelType::ASSERTION).into()
                },
                DidKey {
                    public_key: PublicKey::sr25519(pk_sr),
                    ver_rels: VerRelType::CAPABILITY_INVOCATION.into()
                },
            ],
            vec![did_1, did_2].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_11));
        let key_11_1 = DidKeys::get(&did_11, IncId::from(1u32)).unwrap();
        assert!(key_11_1.can_sign());
        assert!(key_11_1.can_authenticate());
        assert!(!key_11_1.can_control());
        let key_11_2 = DidKeys::get(&did_11, IncId::from(2u32)).unwrap();
        assert!(key_11_2.can_sign());
        assert!(!key_11_2.can_authenticate());
        assert!(key_11_2.can_control());
        check_did_detail(&did_11, 2, 1, 3, 23);
    });
}

#[test]
fn onchain_keyed_did_creation_with_and_without_self_control() {
    // Creating an on-chain DID with keys and other controllers
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();
        let did_2: Did = [52; Did::BYTE_SIZE].into();
        let did_3: Did = [54; Did::BYTE_SIZE].into();
        let did_4: Did = [55; Did::BYTE_SIZE].into();
        let did_5: Did = [56; Did::BYTE_SIZE].into();
        let did_6: Did = [57; Did::BYTE_SIZE].into();

        let controller_1 = Controller([61; Did::BYTE_SIZE].into());
        let controller_2 = Controller([62; Did::BYTE_SIZE].into());
        let controller_3 = Controller([63; Did::BYTE_SIZE].into());
        let controller_4 = Controller([64; Did::BYTE_SIZE].into());

        let (pair_sr, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr = pair_sr.public().0;
        let (pair_ed, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed = pair_ed.public().0;
        let (_, pk_secp) = get_secp256k1_keypair(&[21; 32]);

        run_to_block(10);

        // DID does not control itself, some other DID does
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr),
                ver_rels: VerRelType::AUTHENTICATION.into()
            }],
            vec![controller_1].into_iter().collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_1));
        assert!(DIDModule::is_controller(&did_1, &controller_1));
        check_did_detail(&did_1, 1, 0, 1, 10);

        run_to_block(11);

        // DID does not control itself, some other DID does
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![DidKey {
                public_key: PublicKey::ed25519(pk_ed),
                ver_rels: VerRelType::ASSERTION.into()
            }],
            vec![controller_2].into_iter().collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        assert!(DIDModule::is_controller(&did_2, &controller_2));
        check_did_detail(&did_2, 1, 0, 1, 11);

        run_to_block(12);

        // DID does not control itself, some other DID does
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_3.clone(),
            vec![DidKey {
                public_key: PublicKey::x25519(pk_ed),
                ver_rels: VerRelType::KEY_AGREEMENT.into()
            }],
            vec![controller_3].into_iter().collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_3));
        assert!(DIDModule::is_controller(&did_3, &controller_3));
        check_did_detail(&did_3, 1, 0, 1, 12);

        run_to_block(13);

        // DID does not control itself, some other DID does
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_4.clone(),
            vec![
                DidKey {
                    public_key: PublicKey::sr25519(pk_sr),
                    ver_rels: VerRelType::AUTHENTICATION.into()
                },
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed),
                    ver_rels: VerRelType::ASSERTION.into()
                }
            ],
            vec![controller_4].into_iter().collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_4));
        assert!(DIDModule::is_controller(&did_4, &controller_4));
        check_did_detail(&did_4, 2, 0, 1, 13);

        run_to_block(14);

        // DID is controlled by itself and another DID as well
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_5.clone(),
            vec![
                DidKey {
                    public_key: pk_secp.clone(),
                    ver_rels: (VerRelType::AUTHENTICATION | VerRelType::CAPABILITY_INVOCATION)
                        .into()
                },
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed),
                    ver_rels: VerRelType::ASSERTION.into()
                }
            ],
            vec![controller_1].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_5));
        assert!(DIDModule::is_controller(&did_5, &controller_1));
        check_did_detail(&did_5, 2, 1, 2, 14);

        run_to_block(15);

        // DID has 2 keys to control itself and another DID
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_6.clone(),
            vec![
                DidKey {
                    public_key: pk_secp,
                    ver_rels: (VerRelType::AUTHENTICATION | VerRelType::CAPABILITY_INVOCATION)
                        .into()
                },
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed),
                    ver_rels: (VerRelType::ASSERTION | VerRelType::CAPABILITY_INVOCATION).into()
                }
            ],
            vec![controller_1].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_6));
        assert!(DIDModule::is_controller(&did_6, &controller_1));
        check_did_detail(&did_6, 2, 2, 2, 15);
    });
}

#[test]
fn add_keys_to_did() {
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();
        let did_2: Did = [52; Did::BYTE_SIZE].into();

        // Add keys to a DID that has not been registered yet should fail
        let (pair_sr_1, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr_1 = pair_sr_1.public().0;
        let (pair_sr_2, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr_2 = pair_sr_2.public().0;
        let (pair_ed_1, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed_1 = pair_ed_1.public().0;
        let (pair_ed_2, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed_2 = pair_ed_2.public().0;
        let (_, pk_secp_1) = get_secp256k1_keypair(&[21; 32]);
        let (_, pk_secp_2) = get_secp256k1_keypair(&[22; 32]);

        run_to_block(5);

        // At least one key must be provided
        let add_keys = AddKeys {
            did: did_1.clone(),
            keys: vec![],
            nonce: 5,
        };
        let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr_1);
        assert_noop!(
            DIDModule::add_keys(
                Origin::signed(alice),
                add_keys,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::NoKeyProvided
        );

        let add_keys = AddKeys {
            did: did_1.clone(),
            keys: vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr_1),
                ver_rels: VerRelType::NONE.into(),
            }],
            nonce: 5,
        };
        let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr_1);
        assert_noop!(
            DIDModule::add_keys(
                Origin::signed(alice),
                add_keys,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::OnlyControllerCanUpdate
        );

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![
                DidKey {
                    public_key: PublicKey::sr25519(pk_sr_1),
                    ver_rels: VerRelType::NONE.into()
                },
                DidKey {
                    public_key: PublicKey::sr25519(pk_sr_2),
                    ver_rels: VerRelType::NONE.into()
                },
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed_2),
                    ver_rels: VerRelType::AUTHENTICATION.into()
                },
            ],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_1, 3, 2, 1, 5);

        run_to_block(7);

        // This DID does not control itself
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![DidKey {
                public_key: PublicKey::ed25519(pk_ed_1),
                ver_rels: VerRelType::AUTHENTICATION.into()
            }],
            vec![did_1].into_iter().map(Controller).collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 1, 0, 1, 7);

        run_to_block(10);

        // Since did_2 does not control itself, it cannot add keys to itself
        let add_keys = AddKeys {
            did: did_2.clone(),
            keys: vec![DidKey {
                public_key: pk_secp_1.clone(),
                ver_rels: VerRelType::NONE.into(),
            }],
            nonce: 7 + 1,
        };
        let sig = SigValue::ed25519(&add_keys.to_state_change().encode(), &pair_ed_1);
        assert_noop!(
            DIDModule::add_keys(
                Origin::signed(alice),
                add_keys,
                DidSignature {
                    did: Controller(did_2.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::OnlyControllerCanUpdate
        );

        // Nonce should be 1 greater than existing 7, i.e. 8
        for nonce in vec![6, 7, 9, 10, 100, 10245] {
            let add_keys = AddKeys {
                did: did_2.clone(),
                keys: vec![DidKey {
                    public_key: pk_secp_1.clone(),
                    ver_rels: VerRelType::NONE.into(),
                }],
                nonce,
            };
            let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr_1);
            assert_noop!(
                DIDModule::add_keys(
                    Origin::signed(alice),
                    add_keys,
                    DidSignature {
                        did: Controller(did_1.clone()),
                        key_id: 1u32.into(),
                        sig
                    }
                ),
                Error::<Test>::IncorrectNonce
            );
        }

        // Invalid signature should fail
        let add_keys = AddKeys {
            did: did_2.clone(),
            keys: vec![DidKey {
                public_key: pk_secp_1.clone(),
                ver_rels: VerRelType::NONE.into(),
            }],
            nonce: 7 + 1,
        };
        // Using some arbitrary bytes as signature
        let sig = SigValue::Sr25519(Bytes64 { value: [109; 64] });
        assert_noop!(
            DIDModule::add_keys(
                Origin::signed(alice),
                add_keys,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::InvalidSig
        );

        // Using wrong key_id should fail
        let add_keys = AddKeys {
            did: did_2.clone(),
            keys: vec![DidKey {
                public_key: pk_secp_1.clone(),
                ver_rels: VerRelType::NONE.into(),
            }],
            nonce: 7 + 1,
        };
        let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr_1);
        assert_noop!(
            DIDModule::add_keys(
                Origin::signed(alice),
                add_keys,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 2u32.into(),
                    sig
                }
            ),
            Error::<Test>::InvalidSig
        );

        // Using wrong key type should fail
        let add_keys = AddKeys {
            did: did_2.clone(),
            keys: vec![DidKey {
                public_key: pk_secp_1.clone(),
                ver_rels: VerRelType::KEY_AGREEMENT.into(),
            }],
            nonce: 7 + 1,
        };
        let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr_1);
        assert_noop!(
            DIDModule::add_keys(
                Origin::signed(alice),
                add_keys,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::IncompatibleVerificationRelation
        );

        // Add x25519 key
        let add_keys = AddKeys {
            did: did_2.clone(),
            keys: vec![DidKey {
                public_key: PublicKey::x25519(pk_ed_1),
                ver_rels: VerRelType::KEY_AGREEMENT.into(),
            }],
            nonce: 7 + 1,
        };
        let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr_1);
        assert_ok!(DIDModule::add_keys(
            Origin::signed(alice),
            add_keys,
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 2, 0, 1, 8);

        // Add many keys
        let add_keys = AddKeys {
            did: did_2.clone(),
            keys: vec![
                DidKey {
                    public_key: PublicKey::x25519(pk_sr_2),
                    ver_rels: VerRelType::KEY_AGREEMENT.into(),
                },
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed_1),
                    ver_rels: VerRelType::ASSERTION.into(),
                },
                DidKey {
                    public_key: pk_secp_2,
                    ver_rels: (VerRelType::AUTHENTICATION | VerRelType::ASSERTION).into(),
                },
            ],
            nonce: 8 + 1,
        };

        // Controller uses a key without the capability to update DID
        let sig = SigValue::ed25519(&add_keys.to_state_change().encode(), &pair_ed_2);
        assert_noop!(
            DIDModule::add_keys(
                Origin::signed(alice),
                add_keys.clone(),
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 3u32.into(),
                    sig
                }
            ),
            Error::<Test>::InsufficientVerificationRelationship
        );

        // Controller uses the correct key
        let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr_2);
        assert_ok!(DIDModule::add_keys(
            Origin::signed(alice),
            add_keys,
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 2u32.into(),
                sig
            }
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 5, 0, 1, 9);
    });
}

#[test]
fn remove_keys_from_did() {
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();
        let did_2: Did = [52; Did::BYTE_SIZE].into();

        // Add keys to a DID that has not been registered yet should fail
        let (pair_sr_1, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr_1 = pair_sr_1.public().0;
        let (pair_sr_2, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr_2 = pair_sr_2.public().0;
        let (pair_ed_1, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed_1 = pair_ed_1.public().0;
        let (pair_ed_2, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed_2 = pair_ed_2.public().0;
        let (_, pk_secp_1) = get_secp256k1_keypair(&[21; 32]);
        let (_, pk_secp_2) = get_secp256k1_keypair(&[22; 32]);

        run_to_block(2);
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![
                DidKey::new_with_all_relationships(PublicKey::sr25519(pk_sr_1)),
                DidKey::new_with_all_relationships(PublicKey::ed25519(pk_ed_1)),
                DidKey::new(PublicKey::ed25519(pk_ed_2), VerRelType::ASSERTION),
                DidKey::new(PublicKey::sr25519(pk_sr_2), VerRelType::AUTHENTICATION),
            ],
            vec![did_2].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_1, 4, 2, 2, 2);

        run_to_block(5);

        // This DID does not control itself
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed_1),
                    ver_rels: VerRelType::AUTHENTICATION.into()
                },
                DidKey::new_with_all_relationships(PublicKey::sr25519(pk_sr_1))
            ],
            vec![did_1].into_iter().map(Controller).collect()
        ));
        check_did_detail(&did_2, 2, 1, 2, 5);

        run_to_block(10);

        // Nonce should be 1 greater than existing 7, i.e. 8
        for nonce in vec![1, 2, 4, 5, 10, 10000] {
            let remove_keys = RemoveKeys {
                did: did_2.clone(),
                keys: vec![2u32.into()].into_iter().collect(),
                nonce,
            };
            let sig = SigValue::sr25519(&remove_keys.to_state_change().encode(), &pair_sr_1);
            assert_noop!(
                DIDModule::remove_keys(
                    Origin::signed(alice),
                    remove_keys,
                    DidSignature {
                        did: Controller(did_1.clone()),
                        key_id: 1u32.into(),
                        sig
                    }
                ),
                Error::<Test>::IncorrectNonce
            );
        }

        // Since did_2 does not control itself, it cannot add keys to itself
        let remove_keys = RemoveKeys {
            did: did_1.clone(),
            keys: vec![1u32.into(), 3u32.into(), 5u32.into()]
                .into_iter()
                .collect(),
            nonce: 3,
        };
        let sig = SigValue::ed25519(&remove_keys.to_state_change().encode(), &pair_ed_1);
        assert_noop!(
            DIDModule::remove_keys(
                Origin::signed(alice),
                remove_keys,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 2u32.into(),
                    sig
                }
            ),
            Error::<Test>::NoKeyForDid
        );
        let remove_keys = RemoveKeys {
            did: did_1.clone(),
            keys: vec![1u32.into()].into_iter().collect(),
            nonce: 3,
        };
        let sig = SigValue::ed25519(&remove_keys.to_state_change().encode(), &pair_ed_1);
        assert_ok!(DIDModule::remove_keys(
            Origin::signed(alice),
            remove_keys,
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 2u32.into(),
                sig
            }
        ));
        check_did_detail(&did_1, 4, 1, 2, 3);

        let remove_keys = RemoveKeys {
            did: did_1.clone(),
            keys: vec![3u32.into()].into_iter().collect(),
            nonce: 4,
        };
        let sig = SigValue::sr25519(&remove_keys.to_state_change().encode(), &pair_sr_1);
        assert_ok!(DIDModule::remove_keys(
            Origin::signed(alice),
            remove_keys,
            DidSignature {
                did: Controller(did_2.clone()),
                key_id: 2u32.into(),
                sig
            }
        ));

        let did_5: Did = [54; Did::BYTE_SIZE].into();
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_5.clone(),
            vec![DidKey::new_with_all_relationships(PublicKey::sr25519(
                pk_sr_1
            ))]
            .into_iter()
            .collect(),
            vec![did_2].into_iter().map(Controller).collect()
        ));
        check_did_detail(&did_5, 1, 1, 2, 10);

        let remove_keys = RemoveKeys {
            did: did_5.clone(),
            keys: vec![1u32.into()].into_iter().collect(),
            nonce: 11,
        };
        let sig = SigValue::sr25519(&remove_keys.to_state_change().encode(), &pair_sr_1);
        assert_ok!(DIDModule::remove_keys(
            Origin::signed(alice),
            remove_keys,
            DidSignature {
                did: Controller(did_5.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        check_did_detail(&did_5, 1, 0, 1, 11);

        let remove_controllers = RemoveControllers {
            did: did_5.clone(),
            controllers: vec![did_2].into_iter().map(Controller).collect(),
            nonce: 12,
        };
        let sig = SigValue::sr25519(
            &StateChange::RemoveControllers(Cow::Borrowed(&remove_controllers)).encode(),
            &pair_sr_1,
        );
        assert_ok!(DIDModule::remove_controllers(
            Origin::signed(alice),
            remove_controllers,
            DidSignature {
                did: Controller(did_2.clone()),
                key_id: 2u32.into(),
                sig
            }
        ));
        check_did_detail(&did_5, 1, 0, 0, 12);
    });
}

#[test]
fn remove_controllers_from_did() {
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();
        let did_2: Did = [52; Did::BYTE_SIZE].into();
        let did_3: Did = [53; Did::BYTE_SIZE].into();

        // Add keys to a DID that has not been registered yet should fail
        let (pair_sr_1, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr_1 = pair_sr_1.public().0;
        let (pair_sr_2, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr_2 = pair_sr_2.public().0;
        let (pair_ed_1, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed_1 = pair_ed_1.public().0;
        let (pair_ed_2, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed_2 = pair_ed_2.public().0;
        let (_, pk_secp_1) = get_secp256k1_keypair(&[21; 32]);
        let (_, pk_secp_2) = get_secp256k1_keypair(&[22; 32]);

        run_to_block(2);
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![
                DidKey::new_with_all_relationships(PublicKey::sr25519(pk_sr_1)),
                DidKey::new_with_all_relationships(PublicKey::ed25519(pk_ed_1)),
                DidKey::new(PublicKey::ed25519(pk_ed_2), VerRelType::ASSERTION),
                DidKey::new(PublicKey::sr25519(pk_sr_2), VerRelType::AUTHENTICATION),
            ],
            vec![did_2].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_1, 4, 2, 2, 2);

        run_to_block(5);

        // This DID does not control itself
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed_1),
                    ver_rels: VerRelType::AUTHENTICATION.into()
                },
                DidKey::new_with_all_relationships(PublicKey::sr25519(pk_sr_1))
            ],
            vec![did_1].into_iter().map(Controller).collect()
        ));
        check_did_detail(&did_2, 2, 1, 2, 5);

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_3.clone(),
            vec![].into_iter().collect(),
            vec![did_1, did_2, did_3]
                .into_iter()
                .map(Controller)
                .collect()
        ));
        check_did_detail(&did_3, 0, 0, 3, 5);

        run_to_block(10);

        // Nonce should be 1 greater than existing 7, i.e. 8
        for nonce in vec![1, 2, 4, 5, 10, 10000] {
            let remove_controllers = RemoveControllers {
                did: did_2.clone(),
                controllers: vec![did_1].into_iter().map(Controller).collect(),
                nonce,
            };
            let sig = SigValue::sr25519(
                &StateChange::RemoveControllers(Cow::Borrowed(&remove_controllers)).encode(),
                &pair_sr_1,
            );
            assert_noop!(
                DIDModule::remove_controllers(
                    Origin::signed(alice),
                    remove_controllers,
                    DidSignature {
                        did: Controller(did_1.clone()),
                        key_id: 1u32.into(),
                        sig
                    }
                ),
                Error::<Test>::IncorrectNonce
            );
        }

        // Since did_2 does not control itself, it cannot add keys to itself
        let remove_controllers = RemoveControllers {
            did: did_1.clone(),
            controllers: vec![did_1, did_2, did_3, [53; Did::BYTE_SIZE].into()]
                .into_iter()
                .map(Controller)
                .collect(),
            nonce: 3,
        };
        let sig = SigValue::ed25519(
            &StateChange::RemoveControllers(Cow::Borrowed(&remove_controllers)).encode(),
            &pair_ed_1,
        );
        assert_noop!(
            DIDModule::remove_controllers(
                Origin::signed(alice),
                remove_controllers,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 2u32.into(),
                    sig
                }
            ),
            Error::<Test>::NoControllerForDid
        );
        let remove_controllers = RemoveControllers {
            did: did_1.clone(),
            controllers: vec![did_1].into_iter().map(Controller).collect(),
            nonce: 3,
        };
        let sig = SigValue::ed25519(
            &StateChange::RemoveControllers(Cow::Borrowed(&remove_controllers)).encode(),
            &pair_ed_1,
        );
        assert_ok!(DIDModule::remove_controllers(
            Origin::signed(alice),
            remove_controllers,
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 2u32.into(),
                sig
            }
        ));
        assert!(!DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_1, 4, 2, 1, 3);

        let remove_controllers = RemoveControllers {
            did: did_1.clone(),
            controllers: vec![did_2].into_iter().map(Controller).collect(),
            nonce: 4,
        };
        let sig = SigValue::sr25519(
            &StateChange::RemoveControllers(Cow::Borrowed(&remove_controllers)).encode(),
            &pair_sr_1,
        );
        assert_ok!(DIDModule::remove_controllers(
            Origin::signed(alice),
            remove_controllers,
            DidSignature {
                did: Controller(did_2.clone()),
                key_id: 2u32.into(),
                sig
            }
        ));
        check_did_detail(&did_1, 4, 2, 0, 4);

        let remove_controllers = RemoveControllers {
            did: did_3.clone(),
            controllers: vec![did_2].into_iter().map(Controller).collect(),
            nonce: 6,
        };
        let sig = SigValue::sr25519(
            &StateChange::RemoveControllers(Cow::Borrowed(&remove_controllers)).encode(),
            &pair_sr_1,
        );
        assert_ok!(DIDModule::remove_controllers(
            Origin::signed(alice),
            remove_controllers,
            DidSignature {
                did: Controller(did_2.clone()),
                key_id: 2u32.into(),
                sig
            }
        ));
        run_to_block(22);
        let remove_controllers = RemoveControllers {
            did: did_3.clone(),
            controllers: vec![did_1].into_iter().map(Controller).collect(),
            nonce: 7,
        };
        check_did_detail(&did_3, 0, 0, 2, 6);
        let sig = SigValue::sr25519(
            &StateChange::RemoveControllers(Cow::Borrowed(&remove_controllers)).encode(),
            &pair_sr_1,
        );
        assert_err!(
            DIDModule::remove_controllers(
                Origin::signed(alice),
                remove_controllers,
                DidSignature {
                    did: Controller(did_2.clone()),
                    key_id: 2u32.into(),
                    sig
                }
            ),
            Error::<Test>::OnlyControllerCanUpdate
        );
    });
}

#[test]
fn add_controllers_to_did() {
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();
        let did_2: Did = [52; Did::BYTE_SIZE].into();
        let did_3: Did = [53; Did::BYTE_SIZE].into();
        let did_4: Did = [54; Did::BYTE_SIZE].into();
        let did_5: Did = [55; Did::BYTE_SIZE].into();

        // Add keys to a DID that has not been registered yet should fail
        let (pair_sr, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr = pair_sr.public().0;
        let (pair_ed, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed = pair_ed.public().0;
        let (sk_secp_1, pk_secp_1) = get_secp256k1_keypair(&[21; 32]);
        let (sk_secp_2, pk_secp_2) = get_secp256k1_keypair(&[22; 32]);

        run_to_block(5);

        // At least one controller must be provided
        let add_controllers = AddControllers {
            did: did_1.clone(),
            controllers: vec![].into_iter().collect(),
            nonce: 5,
        };
        let sig = SigValue::sr25519(
            &StateChange::AddControllers(Cow::Borrowed(&add_controllers)).encode(),
            &pair_sr,
        );
        assert_noop!(
            DIDModule::add_controllers(
                Origin::signed(alice),
                add_controllers,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::NoControllerProvided
        );

        let add_controllers = AddControllers {
            did: did_1.clone(),
            controllers: vec![did_2].into_iter().map(Controller).collect(),
            nonce: 5,
        };
        let sig = SigValue::sr25519(
            &StateChange::AddControllers(Cow::Borrowed(&add_controllers)).encode(),
            &pair_sr,
        );
        assert_noop!(
            DIDModule::add_controllers(
                Origin::signed(alice),
                add_controllers,
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::OnlyControllerCanUpdate
        );

        // This DID controls itself
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![
                DidKey {
                    public_key: pk_secp_1.clone(),
                    ver_rels: VerRelType::NONE.into()
                },
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed),
                    ver_rels: VerRelType::AUTHENTICATION.into()
                },
            ],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_1, 2, 1, 1, 5);

        run_to_block(6);

        // This DID is controlled by itself and another DID as well
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_3.clone(),
            vec![DidKey {
                public_key: pk_secp_2.clone(),
                ver_rels: VerRelType::NONE.into()
            },],
            vec![did_1].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_3, 1, 1, 2, 6);

        run_to_block(10);
        // This DID does not control itself
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr),
                ver_rels: VerRelType::AUTHENTICATION.into()
            }],
            vec![did_1].into_iter().map(Controller).collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 1, 0, 1, 10);

        run_to_block(15);

        // Since did_2 does not control itself, it cannot controller to itself
        let add_controllers = AddControllers {
            did: did_2.clone(),
            controllers: vec![did_3].into_iter().map(Controller).collect(),
            nonce: 10 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::AddControllers(Cow::Borrowed(&add_controllers)).encode(),
            &pair_sr,
        );
        assert_noop!(
            DIDModule::add_controllers(
                Origin::signed(alice),
                add_controllers,
                DidSignature {
                    did: Controller(did_2.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::OnlyControllerCanUpdate
        );

        // Nonce should be 1 greater than existing 10, i.e. 11
        for nonce in vec![8, 9, 10, 12, 25000] {
            let add_controllers = AddControllers {
                did: did_2.clone(),
                controllers: vec![did_3].into_iter().map(Controller).collect(),
                nonce,
            };
            let sig = SigValue::secp256k1(
                &StateChange::AddControllers(Cow::Borrowed(&add_controllers)).encode(),
                &sk_secp_1,
            );
            assert_noop!(
                DIDModule::add_controllers(
                    Origin::signed(alice),
                    add_controllers,
                    DidSignature {
                        did: Controller(did_1.clone()),
                        key_id: 1u32.into(),
                        sig
                    }
                ),
                Error::<Test>::IncorrectNonce
            );
        }

        // Invalid signature should fail
        let add_controllers = AddControllers {
            did: did_2.clone(),
            controllers: vec![did_3].into_iter().map(Controller).collect(),
            nonce: 10 + 1,
        };
        let sig = SigValue::Secp256k1(Bytes65 { value: [35; 65] });
        assert_noop!(
            DIDModule::add_controllers(
                Origin::signed(alice),
                add_controllers.clone(),
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::InvalidSig
        );

        // Valid signature should work
        let sig = SigValue::secp256k1(
            &StateChange::AddControllers(Cow::Borrowed(&add_controllers)).encode(),
            &sk_secp_1,
        );
        assert_ok!(DIDModule::add_controllers(
            Origin::signed(alice),
            add_controllers,
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 1, 0, 2, 11);

        run_to_block(15);

        // Add many controllers
        let add_controllers = AddControllers {
            did: did_2.clone(),
            controllers: vec![did_4, did_5].into_iter().map(Controller).collect(),
            nonce: 11 + 1,
        };
        let sig = SigValue::secp256k1(
            &StateChange::AddControllers(Cow::Borrowed(&add_controllers)).encode(),
            &sk_secp_2,
        );
        assert_ok!(DIDModule::add_controllers(
            Origin::signed(alice),
            add_controllers,
            DidSignature {
                did: Controller(did_3.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 1, 0, 4, 12);
    });
}

#[test]
fn becoming_controller() {
    // A DID that was not a controller of its DID during creation can become one
    // when either a key is added with `capabilityInvocation`
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();
        let did_2: Did = [52; Did::BYTE_SIZE].into();

        let (pair_sr, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr = pair_sr.public().0;
        let (pair_ed, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed = pair_ed.public().0;
        let (sk_secp, pk_secp) = get_secp256k1_keypair(&[21; 32]);

        run_to_block(5);

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr),
                ver_rels: VerRelType::NONE.into()
            },],
            vec![].into_iter().collect()
        ));

        run_to_block(10);

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![DidKey {
                public_key: PublicKey::x25519(pk_ed),
                ver_rels: VerRelType::KEY_AGREEMENT.into()
            },],
            vec![did_1].into_iter().map(Controller).collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 1, 0, 1, 10);

        run_to_block(15);

        let add_keys = AddKeys {
            did: did_2.clone(),
            keys: vec![DidKey {
                public_key: PublicKey::ed25519(pk_ed),
                ver_rels: VerRelType::ASSERTION.into(),
            }],
            nonce: 10 + 1,
        };
        let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr);
        assert_ok!(DIDModule::add_keys(
            Origin::signed(alice),
            add_keys,
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 2, 0, 1, 11);

        run_to_block(20);

        let add_keys = AddKeys {
            did: did_2.clone(),
            keys: vec![DidKey {
                public_key: pk_secp.clone(),
                ver_rels: VerRelType::CAPABILITY_INVOCATION.into(),
            }],
            nonce: 11 + 1,
        };
        let sig = SigValue::sr25519(&add_keys.to_state_change().encode(), &pair_sr);
        assert_ok!(DIDModule::add_keys(
            Origin::signed(alice),
            add_keys,
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        assert!(DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 3, 1, 2, 12);
    });

    // TODO:
}

#[test]
fn any_controller_can_update() {
    // For a DID with many controllers, any controller can update it by adding keys, controllers.
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();
        let did_2: Did = [52; Did::BYTE_SIZE].into();
        let did_3: Did = [53; Did::BYTE_SIZE].into();
        let did_4: Did = [54; Did::BYTE_SIZE].into();

        let (pair_sr, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr = pair_sr.public().0;
        let (pair_ed, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed = pair_ed.public().0;
        let (_, pk_secp) = get_secp256k1_keypair(&[21; 32]);

        run_to_block(5);

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![DidKey {
                public_key: PublicKey::ed25519(pk_ed),
                ver_rels: VerRelType::NONE.into()
            },],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_1, 1, 1, 1, 5);

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr),
                ver_rels: VerRelType::NONE.into()
            },],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 1, 1, 1, 5);

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_3.clone(),
            vec![DidKey {
                public_key: pk_secp.clone(),
                ver_rels: VerRelType::NONE.into()
            },],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_3));
        check_did_detail(&did_3, 1, 1, 1, 5);

        run_to_block(7);

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_4.clone(),
            vec![DidKey {
                public_key: pk_secp.clone(),
                ver_rels: VerRelType::NONE.into()
            },],
            vec![did_2].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_4));
        check_did_detail(&did_4, 1, 1, 2, 7);

        run_to_block(14);

        let add_controllers = AddControllers {
            did: did_4.clone(),
            controllers: vec![did_1].into_iter().map(Controller).collect(),
            nonce: 7 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::AddControllers(Cow::Borrowed(&add_controllers)).encode(),
            &pair_sr,
        );
        assert_ok!(DIDModule::add_controllers(
            Origin::signed(alice),
            add_controllers,
            DidSignature {
                did: Controller(did_2.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        check_did_detail(&did_4, 1, 1, 3, 8);

        run_to_block(15);

        let add_keys = AddKeys {
            did: did_4.clone(),
            keys: vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr),
                ver_rels: VerRelType::NONE.into(),
            }],
            nonce: 8 + 1,
        };
        let sig = SigValue::ed25519(&add_keys.to_state_change().encode(), &pair_ed);
        assert_ok!(DIDModule::add_keys(
            Origin::signed(alice),
            add_keys,
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
    });
}

#[test]
fn any_controller_can_remove() {
    // For a DID with many controllers, any controller can remove it.
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();

        // TODO:
    });
}

#[test]
fn service_endpoints() {
    // Adding and removing service endpoints to a DID
    ext().execute_with(|| {
        let alice = 1u64;
        let did: Did = [51; Did::BYTE_SIZE].into();

        let endpoint_1_id: WrappedBytes = vec![102; 50].into();
        let origins_1: Vec<WrappedBytes> = vec![vec![112; 100].into()];
        let endpoint_2_id: WrappedBytes = vec![202; 90].into();
        let origins_2: Vec<WrappedBytes> = vec![vec![212; 150].into(), vec![225; 30].into()];

        let (pair_sr, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr = pair_sr.public().0;
        let (pair_ed, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed = pair_ed.public().0;

        run_to_block(5);

        let add_service_endpoint = AddServiceEndpoint {
            did: did.clone(),
            id: endpoint_1_id.clone(),
            endpoint: ServiceEndpoint {
                types: ServiceEndpointType::LINKED_DOMAINS,
                origins: origins_1.clone(),
            },
            nonce: 5 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::AddServiceEndpoint(Cow::Borrowed(&add_service_endpoint)).encode(),
            &pair_sr,
        );

        // DID does not exist yet, thus no controller
        assert_noop!(
            DIDModule::add_service_endpoint(
                Origin::signed(alice),
                add_service_endpoint.clone(),
                DidSignature {
                    did: Controller(did.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::OnlyControllerCanUpdate
        );

        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did.clone(),
            vec![
                DidKey {
                    public_key: PublicKey::sr25519(pk_sr),
                    ver_rels: VerRelType::NONE.into()
                },
                DidKey {
                    public_key: PublicKey::ed25519(pk_ed),
                    ver_rels: (VerRelType::AUTHENTICATION | VerRelType::ASSERTION).into()
                },
            ],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did));
        check_did_detail(&did, 2, 1, 1, 5);

        run_to_block(10);

        // Non-control key cannot add endpoint
        let add_service_endpoint = AddServiceEndpoint {
            did: did.clone(),
            id: endpoint_1_id.clone(),
            endpoint: ServiceEndpoint {
                types: ServiceEndpointType::LINKED_DOMAINS,
                origins: origins_1.clone(),
            },
            nonce: 5 + 1,
        };
        let sig = SigValue::ed25519(
            &StateChange::AddServiceEndpoint(Cow::Borrowed(&add_service_endpoint)).encode(),
            &pair_ed,
        );

        assert_noop!(
            DIDModule::add_service_endpoint(
                Origin::signed(alice),
                add_service_endpoint.clone(),
                DidSignature {
                    did: Controller(did.clone()),
                    key_id: 2u32.into(),
                    sig
                }
            ),
            Error::<Test>::InsufficientVerificationRelationship
        );

        // Trying to add invalid endpoint fails
        for (id, ep) in vec![
            (
                vec![].into(), // Empty id not allowed
                ServiceEndpoint {
                    types: ServiceEndpointType::LINKED_DOMAINS,
                    origins: origins_1.clone(),
                },
            ),
            (
                vec![20; 512].into(), // too big id not allowed
                ServiceEndpoint {
                    types: ServiceEndpointType::LINKED_DOMAINS,
                    origins: origins_1.clone(),
                },
            ),
            (
                endpoint_1_id.clone(),
                ServiceEndpoint {
                    types: ServiceEndpointType::NONE, // Empty type not allowed
                    origins: origins_1.clone(),
                },
            ),
            (
                endpoint_1_id.clone(),
                ServiceEndpoint {
                    types: ServiceEndpointType::LINKED_DOMAINS,
                    origins: vec![], // Empty origin not allowed
                },
            ),
            (
                endpoint_1_id.clone(),
                ServiceEndpoint {
                    types: ServiceEndpointType::LINKED_DOMAINS,
                    origins: vec![vec![].into()], // Empty origin not allowed
                },
            ),
            (
                endpoint_1_id.clone(),
                ServiceEndpoint {
                    types: ServiceEndpointType::LINKED_DOMAINS,
                    origins: vec![vec![45; 55].into(), vec![].into()], // All provided origins mut be non-empty
                },
            ),
            (
                endpoint_1_id.clone(),
                ServiceEndpoint {
                    types: ServiceEndpointType::LINKED_DOMAINS,
                    origins: vec![vec![30; 561].into()], // too big origin not allowed
                },
            ),
            (
                endpoint_1_id.clone(),
                ServiceEndpoint {
                    types: ServiceEndpointType::LINKED_DOMAINS,
                    origins: vec![vec![30; 20].into(); 300], // too many origins not allowed
                },
            ),
        ] {
            let add_service_endpoint = AddServiceEndpoint {
                did: did.clone(),
                id: id.into(),
                endpoint: ep,
                nonce: 5 + 1,
            };
            let sig = SigValue::sr25519(
                &StateChange::AddServiceEndpoint(Cow::Borrowed(&add_service_endpoint)).encode(),
                &pair_sr,
            );

            assert_noop!(
                DIDModule::add_service_endpoint(
                    Origin::signed(alice),
                    add_service_endpoint.clone(),
                    DidSignature {
                        did: Controller(did.clone()),
                        key_id: 1u32.into(),
                        sig
                    }
                ),
                Error::<Test>::InvalidServiceEndpoint
            );
        }

        assert!(DIDModule::did_service_endpoints(&did, &endpoint_1_id).is_none());

        let add_service_endpoint = AddServiceEndpoint {
            did: did.clone(),
            id: endpoint_1_id.clone(),
            endpoint: ServiceEndpoint {
                types: ServiceEndpointType::LINKED_DOMAINS,
                origins: origins_1.clone(),
            },
            nonce: 5 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::AddServiceEndpoint(Cow::Borrowed(&add_service_endpoint)).encode(),
            &pair_sr,
        );

        assert_ok!(DIDModule::add_service_endpoint(
            Origin::signed(alice),
            add_service_endpoint.clone(),
            DidSignature {
                did: Controller(did.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));

        assert_eq!(
            DIDModule::did_service_endpoints(&did, &endpoint_1_id).unwrap(),
            ServiceEndpoint {
                types: ServiceEndpointType::LINKED_DOMAINS,
                origins: origins_1.clone(),
            }
        );
        check_did_detail(&did, 2, 1, 1, 6);

        run_to_block(15);

        // Adding new endpoint with existing id fails
        let add_service_endpoint = AddServiceEndpoint {
            did: did.clone(),
            id: endpoint_1_id.clone(),
            endpoint: ServiceEndpoint {
                types: ServiceEndpointType::LINKED_DOMAINS,
                origins: origins_2.clone(),
            },
            nonce: 6 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::AddServiceEndpoint(Cow::Borrowed(&add_service_endpoint)).encode(),
            &pair_sr,
        );

        assert_noop!(
            DIDModule::add_service_endpoint(
                Origin::signed(alice),
                add_service_endpoint.clone(),
                DidSignature {
                    did: Controller(did.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::ServiceEndpointAlreadyExists
        );

        let add_service_endpoint = AddServiceEndpoint {
            did: did.clone(),
            id: endpoint_2_id.clone(),
            endpoint: ServiceEndpoint {
                types: ServiceEndpointType::LINKED_DOMAINS,
                origins: origins_2.clone(),
            },
            nonce: 6 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::AddServiceEndpoint(Cow::Borrowed(&add_service_endpoint)).encode(),
            &pair_sr,
        );

        assert_ok!(DIDModule::add_service_endpoint(
            Origin::signed(alice),
            add_service_endpoint.clone(),
            DidSignature {
                did: Controller(did.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));

        assert_eq!(
            DIDModule::did_service_endpoints(&did, &endpoint_2_id).unwrap(),
            ServiceEndpoint {
                types: ServiceEndpointType::LINKED_DOMAINS,
                origins: origins_2.clone(),
            }
        );
        check_did_detail(&did, 2, 1, 1, 7);

        run_to_block(16);

        // Non-control key cannot remove endpoint
        let rem_service_endpoint = RemoveServiceEndpoint {
            did: did.clone(),
            id: endpoint_1_id.clone(),
            nonce: 7 + 1,
        };
        let sig = SigValue::ed25519(
            &StateChange::RemoveServiceEndpoint(Cow::Borrowed(&rem_service_endpoint)).encode(),
            &pair_ed,
        );

        assert_noop!(
            DIDModule::remove_service_endpoint(
                Origin::signed(alice),
                rem_service_endpoint.clone(),
                DidSignature {
                    did: Controller(did.clone()),
                    key_id: 2u32.into(),
                    sig
                }
            ),
            Error::<Test>::InsufficientVerificationRelationship
        );

        // Invalid endpoint id fails
        let rem_service_endpoint = RemoveServiceEndpoint {
            did: did.clone(),
            id: vec![].into(),
            nonce: 7 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::RemoveServiceEndpoint(Cow::Borrowed(&rem_service_endpoint)).encode(),
            &pair_sr,
        );

        assert_noop!(
            DIDModule::remove_service_endpoint(
                Origin::signed(alice),
                rem_service_endpoint.clone(),
                DidSignature {
                    did: Controller(did.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::InvalidServiceEndpoint
        );

        let rem_service_endpoint = RemoveServiceEndpoint {
            did: did.clone(),
            id: endpoint_1_id.clone(),
            nonce: 7 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::RemoveServiceEndpoint(Cow::Borrowed(&rem_service_endpoint)).encode(),
            &pair_sr,
        );

        assert_ok!(DIDModule::remove_service_endpoint(
            Origin::signed(alice),
            rem_service_endpoint.clone(),
            DidSignature {
                did: Controller(did.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        assert!(DIDModule::did_service_endpoints(&did, &endpoint_1_id).is_none());
        check_did_detail(&did, 2, 1, 1, 8);

        // id already removed, removing again fails
        let rem_service_endpoint = RemoveServiceEndpoint {
            did: did.clone(),
            id: endpoint_1_id.clone(),
            nonce: 8 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::RemoveServiceEndpoint(Cow::Borrowed(&rem_service_endpoint)).encode(),
            &pair_sr,
        );
        assert_noop!(
            DIDModule::remove_service_endpoint(
                Origin::signed(alice),
                rem_service_endpoint.clone(),
                DidSignature {
                    did: Controller(did.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::ServiceEndpointDoesNotExist
        );

        let rem_service_endpoint = RemoveServiceEndpoint {
            did: did.clone(),
            id: endpoint_2_id.clone(),
            nonce: 8 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::RemoveServiceEndpoint(Cow::Borrowed(&rem_service_endpoint)).encode(),
            &pair_sr,
        );

        assert_ok!(DIDModule::remove_service_endpoint(
            Origin::signed(alice),
            rem_service_endpoint.clone(),
            DidSignature {
                did: Controller(did.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        assert!(DIDModule::did_service_endpoints(&did, &endpoint_2_id).is_none());
        check_did_detail(&did, 2, 1, 1, 9);

        let rem_did = DidRemoval {
            did: did.clone(),
            nonce: 9 + 1,
        };
        let sig = SigValue::ed25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_ed,
        );

        assert_noop!(
            DIDModule::remove_onchain_did(
                Origin::signed(alice),
                rem_did.clone(),
                DidSignature {
                    did: Controller(did.clone()),
                    key_id: 2u32.into(),
                    sig
                }
            ),
            Error::<Test>::InsufficientVerificationRelationship
        );

        check_did_detail(&did, 2, 1, 1, 9);

        let rem_did = DidRemoval {
            did: did.clone(),
            nonce: 9 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_sr,
        );

        assert_ok!(DIDModule::remove_onchain_did(
            Origin::signed(alice),
            rem_did.clone(),
            DidSignature {
                did: Controller(did.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        ensure_onchain_did_gone(&did);
    });
}

#[test]
fn did_removal() {
    // Removing a DID
    ext().execute_with(|| {
        let alice = 1u64;
        let did_1: Did = [51; Did::BYTE_SIZE].into();
        let did_2: Did = [52; Did::BYTE_SIZE].into();
        let did_3: Did = [53; Did::BYTE_SIZE].into();
        let did_4: Did = [54; Did::BYTE_SIZE].into();

        let (pair_sr, _, _) = sr25519::Pair::generate_with_phrase(None);
        let pk_sr = pair_sr.public().0;
        let (pair_ed, _, _) = ed25519::Pair::generate_with_phrase(None);
        let pk_ed = pair_ed.public().0;

        run_to_block(5);

        // did_1 controls itself
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_1.clone(),
            vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr),
                ver_rels: VerRelType::NONE.into()
            }],
            vec![].into_iter().collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_1));
        check_did_detail(&did_1, 1, 1, 1, 5);

        run_to_block(10);

        // did_2 does not control itself but controlled by did_1
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_2.clone(),
            vec![DidKey {
                public_key: PublicKey::ed25519(pk_ed),
                ver_rels: VerRelType::AUTHENTICATION.into()
            }],
            vec![did_1.clone()].into_iter().map(Controller).collect()
        ));
        assert!(!DIDModule::is_self_controlled(&did_2));
        check_did_detail(&did_2, 1, 0, 1, 10);

        run_to_block(15);

        // did_3 controls itself and also controlled by did_1
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_3.clone(),
            vec![DidKey {
                public_key: PublicKey::ed25519(pk_ed),
                ver_rels: VerRelType::NONE.into()
            }],
            vec![did_1.clone()].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_3));
        check_did_detail(&did_3, 1, 1, 2, 15);

        run_to_block(20);

        // did_4 controls itself and also controlled by did_3
        assert_ok!(DIDModule::new_onchain(
            Origin::signed(alice),
            did_4.clone(),
            vec![DidKey {
                public_key: PublicKey::sr25519(pk_sr),
                ver_rels: VerRelType::NONE.into()
            }],
            vec![did_3.clone()].into_iter().map(Controller).collect()
        ));
        assert!(DIDModule::is_self_controlled(&did_4));
        check_did_detail(&did_4, 1, 1, 2, 20);

        // did_2 does not control itself so it cannot remove itself
        let rem_did = DidRemoval {
            did: did_2.clone(),
            nonce: 10 + 1,
        };
        let sig = SigValue::ed25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_ed,
        );
        assert_noop!(
            DIDModule::remove_onchain_did(
                Origin::signed(alice),
                rem_did.clone(),
                DidSignature {
                    did: Controller(did_2.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::OnlyControllerCanUpdate
        );
        check_did_detail(&did_2, 1, 0, 1, 10);

        // did_2 is controlled by did_1 so it can be removed by did_1
        let sig = SigValue::sr25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_sr,
        );
        assert_ok!(DIDModule::remove_onchain_did(
            Origin::signed(alice),
            rem_did.clone(),
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        ensure_onchain_did_gone(&did_2);

        // Nonce should be correct when its deleted
        let rem_did = DidRemoval {
            did: did_3.clone(),
            nonce: 15,
        };
        let sig = SigValue::sr25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_sr,
        );
        assert_noop!(
            DIDModule::remove_onchain_did(
                Origin::signed(alice),
                rem_did.clone(),
                DidSignature {
                    did: Controller(did_1.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::IncorrectNonce
        );
        check_did_detail(&did_3, 1, 1, 2, 15);

        // did_3 is controlled by itself and did_1 and thus did_1 can remove it
        let rem_did = DidRemoval {
            did: did_3.clone(),
            nonce: 15 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_sr,
        );
        assert_ok!(DIDModule::remove_onchain_did(
            Origin::signed(alice),
            rem_did.clone(),
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        ensure_onchain_did_gone(&did_3);

        // did_4 is controlled by itself and did_3 but did_3 has been removed so it can no
        // longer remove did_4
        let rem_did = DidRemoval {
            did: did_4.clone(),
            nonce: 20 + 1,
        };
        let sig = SigValue::ed25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_ed,
        );
        assert_noop!(
            DIDModule::remove_onchain_did(
                Origin::signed(alice),
                rem_did.clone(),
                DidSignature {
                    did: Controller(did_3.clone()),
                    key_id: 1u32.into(),
                    sig
                }
            ),
            Error::<Test>::NoKeyForDid
        );
        check_did_detail(&did_4, 1, 1, 2, 20);

        // did_4 removes itself
        let rem_did = DidRemoval {
            did: did_4.clone(),
            nonce: 20 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_sr,
        );
        assert_ok!(DIDModule::remove_onchain_did(
            Origin::signed(alice),
            rem_did.clone(),
            DidSignature {
                did: Controller(did_4.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        ensure_onchain_did_gone(&did_4);

        // did_1 removes itself
        let rem_did = DidRemoval {
            did: did_1.clone(),
            nonce: 5 + 1,
        };
        let sig = SigValue::sr25519(
            &StateChange::DidRemoval(Cow::Borrowed(&rem_did)).encode(),
            &pair_sr,
        );
        assert_ok!(DIDModule::remove_onchain_did(
            Origin::signed(alice),
            rem_did.clone(),
            DidSignature {
                did: Controller(did_1.clone()),
                key_id: 1u32.into(),
                sig
            }
        ));
        ensure_onchain_did_gone(&did_1);
    });
}
// TODO: Add test for events DidAdded, KeyUpdated, DIDRemoval
