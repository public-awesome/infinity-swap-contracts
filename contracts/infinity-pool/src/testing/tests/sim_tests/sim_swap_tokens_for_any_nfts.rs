use crate::msg::{ExecuteMsg, QueryMsg, SwapParams, SwapResponse};
use crate::state::BondingCurve;
use crate::testing::helpers::nft_functions::mint_and_approve_many;
use crate::testing::helpers::pool_functions::{prepare_pool_variations, prepare_swap_pool};
use crate::testing::helpers::swap_functions::{setup_swap_test, validate_swap_fees, SwapTestSetup};
use crate::testing::setup::setup_accounts::setup_addtl_account;
use cosmwasm_std::{coins, StdError, StdResult, Timestamp, Uint128};
use cw_multi_test::Executor;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_marketplace::msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::msg::VendingTemplateResponse;

#[test]
fn cant_swap_inactive_pools() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        false,
        0,
        0,
    );

    let max_expected_token_input: Vec<Uint128> =
        vec![Uint128::from(1_000_000u128), Uint128::from(1_000_000u128)];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: None,
        },
    };
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &sim_msg);

    assert!(res.unwrap().swaps.is_empty());
}

#[test]
fn can_swap_active_pools() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let max_expected_token_input: Vec<Uint128> =
        vec![Uint128::from(1_000_000u128), Uint128::from(1_000_000u128)];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: None,
        },
    };
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &sim_msg);

    assert!(res.is_ok());
    assert!(!res.unwrap().swaps.is_empty());
}

#[test]
fn sale_price_above_max_expected() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let max_expected_token_input: Vec<Uint128> = vec![Uint128::from(10u128), Uint128::from(10u128)];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: None,
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &sim_msg);

    assert_eq!(
            res.unwrap_err(),
            StdError::GenericErr {
                msg: "Querier contract error: Generic error: Swap error: pool sale price is above max expected"
                    .to_string()
            }
        );
}

#[test]
fn robust_query_does_not_revert_whole_tx() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let max_expected_token_input: Vec<Uint128> = vec![
        Uint128::from(320u128),
        Uint128::from(340u128),
        Uint128::from(350u128),
    ];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
            asset_recipient: None,
            finder: None,
        },
    };
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &sim_msg);

    assert!(res.is_ok());
    assert_eq!(res.unwrap().swaps.len(), 2);
}

#[test]
fn minimal_fee_tx_is_handled_correctly() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        marketplace,
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let marketplace_params: ParamsResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})
        .unwrap();
    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    let max_expected_token_input: Vec<Uint128> = vec![
        Uint128::from(1_000_000u128),
        Uint128::from(1_000_000u128),
        Uint128::from(1_000_000u128),
    ];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
            asset_recipient: None,
            finder: None,
        },
    };
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &sim_msg);

    for swap in res.unwrap().swaps {
        let pool = pools.iter().find(|p| p.id == swap.pool_id).unwrap();
        validate_swap_fees(
            &swap,
            pool,
            &marketplace_params,
            &collection_info.royalty_info,
        );
    }
}

#[test]
fn finders_and_swap_fee_tx_is_handled_correctly() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        marketplace,
    } = setup_swap_test(5000).unwrap();
    let user2 = setup_addtl_account(&mut router, "asset", 100u128).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        250,
        300,
    );

    let marketplace_params: ParamsResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})
        .unwrap();
    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    let max_expected_token_input: Vec<Uint128> = vec![
        Uint128::from(1_000_000u128),
        Uint128::from(1_000_000u128),
        Uint128::from(1_000_000u128),
    ];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
            asset_recipient: None,
            finder: Some(user2.to_string()),
        },
    };
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &sim_msg);

    for swap in res.unwrap().swaps {
        let pool = pools.iter().find(|p| p.id == swap.pool_id).unwrap();
        validate_swap_fees(
            &swap,
            pool,
            &marketplace_params,
            &collection_info.royalty_info,
        );
    }
}

#[test]
fn trades_are_routed_correctly() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        500,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        20,
        true,
        0,
        0,
    );

    let num_swaps: usize = 50;
    let max_expected_token_input: Vec<Uint128> = vec![Uint128::from(1_000_000u128); num_swaps];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
            asset_recipient: None,
            finder: None,
        },
    };
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &sim_msg);
    let swaps = res.unwrap().swaps;
    assert_eq!(swaps.len(), num_swaps);

    for (idx, swap) in swaps.iter().enumerate() {
        if idx == 0 {
            continue;
        }
        assert!(swaps[idx - 1].spot_price <= swap.spot_price);
    }
}

#[test]
fn constant_product_pools_with_little_nfts() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let _pool = prepare_swap_pool(
        &mut router,
        &infinity_pool,
        &accts.owner,
        Uint128::from(1_000u128),
        owner_token_ids.to_vec().drain(0..4).collect(),
        true,
        ExecuteMsg::CreateTradePool {
            collection: collection.to_string(),
            asset_recipient: None,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::from(0u64),
            delta: Uint128::from(0u64),
            finders_fee_bps: 0,
            swap_fee_bps: 0,
            reinvest_nfts: true,
            reinvest_tokens: true,
        },
    )
    .unwrap();

    let num_swaps: usize = 4;
    let max_expected_token_input: Vec<Uint128> = vec![Uint128::from(100_000_000u128); num_swaps];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input: max_expected_token_input.clone(),
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
            asset_recipient: None,
            finder: None,
        },
    };
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &sim_msg);

    let _swaps = res.unwrap().swaps;

    let exec_msg = ExecuteMsg::SwapTokensForAnyNfts {
        collection: collection.to_string(),
        max_expected_token_input: max_expected_token_input.clone(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
            asset_recipient: None,
            finder: None,
        },
    };

    let funds: Uint128 = max_expected_token_input.iter().sum();
    let exec_res = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &exec_msg,
        &coins(funds.u128(), NATIVE_DENOM),
    );

    assert!(exec_res.is_ok());
}
