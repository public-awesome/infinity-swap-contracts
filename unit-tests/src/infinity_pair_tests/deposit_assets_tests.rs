use crate::helpers::nft_functions::{approve_all, assert_nft_owner, mint_to, transfer};
use crate::helpers::pair_functions::{create_pair, create_pair_with_deposits};
use crate::helpers::utils::assert_error;
use crate::setup::setup_accounts::MarketAccounts;
use crate::setup::setup_infinity_contracts::UOSMO;
use crate::setup::templates::{
    minter_two_collections, setup_infinity_test, standard_minter_template, InfinityTestSetup,
};

use cosmwasm_std::{coin, Addr, Decimal, Uint128};
use cw_multi_test::Executor;
use infinity_pair::msg::{ExecuteMsg as InfinityPairExecuteMsg, QueryMsg as InfinityPairQueryMsg};
use infinity_pair::pair::Pair;
use infinity_pair::state::{BondingCurve, PairConfig, PairType};
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

    let (pair_addr, _pair) =
        create_pair(&mut router, &infinity_global, &infinity_factory, &collection, &accts.owner);

    let token_id = mint_to(&mut router, &accts.creator, &accts.owner, &minter);

    approve_all(&mut router, &accts.owner, &collection, &pair_addr);
    let response = router.execute_contract(
        accts.owner,
        pair_addr.clone(),
        &InfinityPairExecuteMsg::DepositNfts {
            collection: collection.to_string(),
            token_ids: vec![token_id.clone()],
        },
        &[],
    );
    assert!(response.is_ok());

    assert_nft_owner(&router, &collection, token_id, &pair_addr);

    let pair =
        router.wrap().query_wasm_smart::<Pair>(pair_addr, &InfinityPairQueryMsg::Pair {}).unwrap();
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

    let (pair_addr, _pair) =
        create_pair(&mut router, &infinity_global, &infinity_factory, &collection, &accts.owner);

    let num_nfts: usize = 10;
    let mut token_ids: Vec<String> = vec![];
    for _ in 0..num_nfts {
        let token_id = mint_to(&mut router, &accts.creator.clone(), &accts.owner.clone(), &minter);
        token_ids.push(token_id);
    }

    approve_all(&mut router, &accts.owner.clone(), &collection, &pair_addr);
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::DepositNfts {
            collection: collection.to_string(),
            token_ids: token_ids.clone(),
        },
        &[],
    );
    assert!(response.is_ok());

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
            collection: collection.to_string(),
            token_ids: withdraw_nfts.clone(),
            asset_recipient: None,
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
            collection: collection.to_string(),
            token_ids: withdraw_nfts.clone(),
            asset_recipient: None,
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
            collection: collection.to_string(),
            limit: 100u32,
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Withdraw rest of the token ids to asset recipient
    let asset_recipient = Addr::unchecked("asset_recipient");
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawAnyNfts {
            collection: collection.to_string(),
            limit: 100u32,
            asset_recipient: Some(asset_recipient.to_string()),
        },
        &[],
    );
    assert!(response.is_ok());

    let withdraw_nfts = token_ids[num_nfts / 2..].to_vec();
    for withdraw_token_id in withdraw_nfts {
        assert_nft_owner(&router, &collection, withdraw_token_id, &asset_recipient.clone());
    }

    let pair =
        router.wrap().query_wasm_smart::<Pair>(pair_addr, &InfinityPairQueryMsg::Pair {}).unwrap();
    assert!(pair.internal.total_nfts == 0);
}

#[test]
fn try_withdraw_other_collection_nfts() {
    let vt = minter_two_collections(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts:
                    MarketAccounts {
                        creator,
                        owner,
                        ..
                    },
            },
        infinity_global,
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Trade {
                swap_fee_percent: Decimal::zero(),
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
            bonding_curve: BondingCurve::Linear {
                spot_price: Uint128::from(10_000_000u128),
                delta: Uint128::from(1_000_000u128),
            },
            is_active: true,
            asset_recipient: None,
        },
        10u64,
        Uint128::from(100_000_000u128),
    );
    assert!(test_pair.pair.internal.sell_to_pair_quote_summary.is_some());
    assert!(test_pair.pair.internal.buy_from_pair_quote_summary.is_some(),);

    // Send NFT from other collection to pair
    let other_collection = &collection_response_vec[1].collection.clone().unwrap();
    let other_minter = &collection_response_vec[1].minter.clone().unwrap();
    let token_id = mint_to(&mut router, &creator.clone(), &owner.clone(), other_minter);
    transfer(&mut router, &owner, &test_pair.address, other_collection, &token_id);
    assert_nft_owner(&router, other_collection, token_id.clone(), &test_pair.address);

    // Non owner cannot withdraw other collection nfts
    let response = router.execute_contract(
        creator.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::WithdrawNfts {
            collection: other_collection.to_string(),
            token_ids: vec![token_id.clone()],
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Owner can withdraw other collection nfts
    let response = router.execute_contract(
        owner.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::WithdrawNfts {
            collection: other_collection.to_string(),
            token_ids: vec![token_id.clone()],
            asset_recipient: None,
        },
        &[],
    );
    assert!(response.is_ok());
    assert_nft_owner(&router, other_collection, token_id, &owner);

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(test_pair.address.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();
    assert_eq!(test_pair.pair, pair);
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

    let (pair_addr, _pair) =
        create_pair(&mut router, &infinity_global, &infinity_factory, &collection, &accts.owner);

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
        accts.owner,
        pair_addr.clone(),
        &InfinityPairExecuteMsg::DepositTokens {},
        &[coin(deposit_amount, NATIVE_DENOM)],
    );
    assert!(response.is_ok());

    let pair =
        router.wrap().query_wasm_smart::<Pair>(pair_addr, &InfinityPairQueryMsg::Pair {}).unwrap();
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

    let (pair_addr, _pair) =
        create_pair(&mut router, &infinity_global, &infinity_factory, &collection, &accts.owner);

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
            funds: vec![coin(10_000_000u128, NATIVE_DENOM)],
            asset_recipient: None,
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
            funds: vec![coin(10_000_000u128, NATIVE_DENOM)],
            asset_recipient: None,
        },
        &[],
    );
    assert!(response.is_ok());

    // Non owner cannot withdraw all tokens
    let response = router.execute_contract(
        accts.creator.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawAllTokens {
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Owner can withdraw all tokens to asset recipient
    let asset_recipient = Addr::unchecked("asset_recipient");
    let response = router.execute_contract(
        accts.owner,
        pair_addr.clone(),
        &InfinityPairExecuteMsg::WithdrawAllTokens {
            asset_recipient: Some(asset_recipient.to_string()),
        },
        &[],
    );
    assert!(response.is_ok());

    let pair =
        router.wrap().query_wasm_smart::<Pair>(pair_addr, &InfinityPairQueryMsg::Pair {}).unwrap();
    assert_eq!(pair.total_tokens.u128(), 0u128);
}

#[test]
fn try_withdraw_other_denom_tokens() {
    let vt = minter_two_collections(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts:
                    MarketAccounts {
                        creator,
                        owner,
                        ..
                    },
            },
        infinity_global,
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Trade {
                swap_fee_percent: Decimal::zero(),
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
            bonding_curve: BondingCurve::Linear {
                spot_price: Uint128::from(10_000_000u128),
                delta: Uint128::from(1_000_000u128),
            },
            is_active: true,
            asset_recipient: None,
        },
        10u64,
        Uint128::from(100_000_000u128),
    );
    assert!(test_pair.pair.internal.sell_to_pair_quote_summary.is_some());
    assert!(test_pair.pair.internal.buy_from_pair_quote_summary.is_some(),);

    // Send other denom tokens to pair
    let other_denom_funds = coin(2_000_000u128, UOSMO);
    let response =
        router.send_tokens(owner.clone(), test_pair.address.clone(), &[other_denom_funds.clone()]);
    assert!(response.is_ok());

    // Non owner cannot withdraw denom tokens
    let response = router.execute_contract(
        creator.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::WithdrawTokens {
            funds: vec![other_denom_funds.clone()],
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Owner can withdraw other denom tokens
    let response = router.execute_contract(
        owner.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::WithdrawTokens {
            funds: vec![other_denom_funds],
            asset_recipient: None,
        },
        &[],
    );
    assert!(response.is_ok());

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(test_pair.address.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();
    assert_eq!(test_pair.pair, pair);
}
