use crate::helpers::nft_functions::{assert_nft_owner, mint_to};
use crate::helpers::pair_functions::create_pair;
use crate::helpers::utils::assert_error;
use crate::setup::templates::{setup_infinity_test, standard_minter_template, InfinityTestSetup};

use cosmwasm_std::{coin, to_binary, Empty, Uint128};
use cw721::Cw721ExecuteMsg;
use cw_multi_test::Executor;
use infinity_pair::msg::{ExecuteMsg as InfinityPairExecuteMsg, QueryMsg as InfinityPairQueryMsg};
use infinity_pair::pair::Pair;
use infinity_shared::InfinityError;
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_deposit_nft() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        infinity_global,
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let (pair_addr, _pair) = create_pair(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &collection,
        &accts.owner.clone(),
    );

    let token_id = mint_to(&mut router, &accts.creator.clone(), &accts.owner.clone(), &minter);

    let response = router.execute_contract(
        accts.owner.clone(),
        collection.clone(),
        &Cw721ExecuteMsg::SendNft {
            contract: pair_addr.to_string(),
            token_id: token_id.clone(),
            msg: to_binary(&Empty {}).unwrap(),
        },
        &[],
    );
    assert!(response.is_ok());

    assert_nft_owner(&router, &collection, token_id, &pair_addr.clone());

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(pair_addr.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();
    assert!(pair.internal.total_nfts == 1);
}

#[test]
fn try_withdraw_nfts() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        infinity_global,
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let (pair_addr, _pair) = create_pair(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &collection,
        &accts.owner.clone(),
    );

    let num_nfts: usize = 10;
    let mut token_ids: Vec<String> = vec![];
    for _ in 0..num_nfts {
        let token_id = mint_to(&mut router, &accts.creator.clone(), &accts.owner.clone(), &minter);

        let response = router.execute_contract(
            accts.owner.clone(),
            collection.clone(),
            &Cw721ExecuteMsg::SendNft {
                contract: pair_addr.to_string(),
                token_id: token_id.clone(),
                msg: to_binary(&Empty {}).unwrap(),
            },
            &[],
        );
        assert!(response.is_ok());

        token_ids.push(token_id)
    }

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(pair_addr.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();
    assert!(pair.internal.total_nfts == num_nfts as u64);

    let withdraw_nfts = token_ids[0..num_nfts / 2].to_vec();

    // Non owner cannot withdraw
    let response = router.execute_contract(
        accts.creator.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawNfts {
            token_ids: withdraw_nfts.clone(),
        },
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Withdraw half of the token ids
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawNfts {
            token_ids: withdraw_nfts.clone(),
        },
        &[],
    );
    assert!(response.is_ok());

    for withdraw_token_id in withdraw_nfts {
        assert_nft_owner(&router, &collection, withdraw_token_id, &accts.owner.clone());
    }

    // Non owner cannot withdraw any
    let response = router.execute_contract(
        accts.creator.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawAnyNfts {
            limit: 100u32,
        },
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Withdraw rest of the token ids
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawAnyNfts {
            limit: 100u32,
        },
        &[],
    );
    assert!(response.is_ok());

    let withdraw_nfts = token_ids[num_nfts / 2..].to_vec();
    for withdraw_token_id in withdraw_nfts {
        assert_nft_owner(&router, &collection, withdraw_token_id, &accts.owner.clone());
    }

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(pair_addr.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();
    assert!(pair.internal.total_nfts == 0);
}

#[test]
fn try_deposit_tokens() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        infinity_global,
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let _minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let (pair_addr, _pair) = create_pair(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &collection,
        &accts.owner.clone(),
    );

    // Non owner cannot deposit tokens
    let response = router.execute_contract(
        accts.creator.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::DepositTokens {},
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Cannot invoke with no funds
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::DepositTokens {},
        &[],
    );
    assert_error(response, "No funds sent".to_string());

    // Invokable with funds
    let deposit_amount = 100_000_000u128;
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::DepositTokens {},
        &[coin(deposit_amount, NATIVE_DENOM)],
    );
    assert!(response.is_ok());

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(pair_addr.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();
    assert_eq!(pair.total_tokens.u128(), deposit_amount);
}

#[test]
fn try_withdraw_tokens() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        infinity_global,
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let _minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let (pair_addr, _pair) = create_pair(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &collection,
        &accts.owner.clone(),
    );

    let deposit_amount = 100_000_000u128;
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::DepositTokens {},
        &[coin(deposit_amount, NATIVE_DENOM)],
    );
    assert!(response.is_ok());

    // Non owner cannot withdraw tokens
    let response = router.execute_contract(
        accts.creator.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawTokens {
            amount: Uint128::from(10_000_000u128),
        },
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Owner can withdraw tokens
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawTokens {
            amount: Uint128::from(10_000_000u128),
        },
        &[],
    );
    assert!(response.is_ok());

    // Non owner cannot withdraw all tokens
    let response = router.execute_contract(
        accts.creator.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawAllTokens {},
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Owner can withdraw all tokens
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawAllTokens {},
        &[],
    );
    assert!(response.is_ok());

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(pair_addr.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();
    assert_eq!(pair.total_tokens.u128(), 0u128);
}
