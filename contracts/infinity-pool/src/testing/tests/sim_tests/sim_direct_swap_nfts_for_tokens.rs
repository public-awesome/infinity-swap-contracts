use crate::msg::ExecuteMsg;
use crate::msg::QueryMsg::SimDirectSwapNftsForTokens;
use crate::msg::SwapResponse;
use crate::msg::{NftSwap, SwapParams};
use crate::state::PoolType;
use crate::testing::setup::templates::{_minter_template_30_pct_fee, standard_minter_template};
use crate::testing::tests::sim_tests::helpers::{
    check_nft_sale, deposit_nfts_and_tokens, get_sim_swap_message, set_pool_active,
    setup_swap_pool, DepositNftsResult, SwapPoolResult, SwapPoolSetup, VendingTemplateSetup,
};
use cosmwasm_std::StdError;
use cosmwasm_std::StdError::GenericErr;
use cosmwasm_std::StdResult;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{coins, Uint128};
use cw_multi_test::Executor;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use std::vec;

#[test]
fn cant_swap_inactive_pool() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Token,
        spot_price,
        finders_fee_bps: None,
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        1000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    set_pool_active(
        &mut router,
        false,
        spr.pool.clone(),
        spr.creator,
        spr.infinity_pool,
    );
    let swap_msg = get_sim_swap_message(spr.pool, dnr.token_id_1, 1000, true, spr.user2, None);
    let res: StdResult<SwapResponse> = router.wrap().query_wasm_smart(dnr.infinity_pool, &swap_msg);

    let res = res.unwrap_err();
    assert_eq!(
        res,
        StdError::GenericErr {
            msg: "Querier contract error: Generic error: Invalid pool: pool is not active"
                .to_string()
        }
    );
}

#[test]
fn can_swap_active_pool() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Trade,
        spot_price,
        finders_fee_bps: None,
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        2000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let sale_price = 1000_u128;
    let swap_msg = get_sim_swap_message(
        spr.pool.clone(),
        dnr.token_id_1,
        sale_price,
        true,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    let swap_price_plus_delta = spot_price + 100;
    let expected_royalty_fee = Uint128::from(swap_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price + 100)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();
    check_nft_sale(
        spot_price + 100,
        expected_royalty_fee,
        expected_network_fee,
        0,
        swaps,
        spr.creator,
        spr.user2,
        dnr.token_id_1.to_string(),
    )
}

#[test]
fn invalid_nft_pool_can_not_deposit() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Nft,
        spot_price,
        finders_fee_bps: None,
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    let dnr = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        1000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator,
    );
    let error_res = dnr.err().unwrap();
    assert_eq!(
        error_res.root_cause().to_string(),
        "Invalid pool: cannot deposit tokens into nft pool"
    );
}

#[test]
fn not_enough_deposit_no_swap() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Trade,
        spot_price,
        finders_fee_bps: None,
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        500_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );
    let sale_price = 1000_u128;
    let swap_msg = get_sim_swap_message(
        spr.pool.clone(),
        dnr.token_id_1,
        sale_price,
        false,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    let error_msg = res.err().unwrap();

    let expected_error = GenericErr {
        msg: "Querier contract error: \
        Generic error: Swap error: pool cannot offer quote"
            .to_string(),
    };
    assert_eq!(error_msg, expected_error);

    let msg = ExecuteMsg::DepositTokens {
        pool_id: spr.pool.id,
    };
    let _ = router.execute_contract(
        spr.creator.clone(),
        spr.infinity_pool.clone(),
        &msg,
        &coins(1000_u128, NATIVE_DENOM),
    );

    // swap wil be valid now there there is sufficent payment
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    let swaps = res.unwrap().swaps;
    let spot_price_plus_delta = spot_price + 100;
    let expected_royalty_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();
    check_nft_sale(
        spot_price_plus_delta,
        expected_royalty_fee,
        expected_network_fee,
        0,
        swaps,
        spr.creator,
        spr.user2,
        dnr.token_id_1.to_string(),
    )
}

#[test]
fn invalid_sale_price_below_min_expected() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Token,
        spot_price,
        finders_fee_bps: None,
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        1000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let swap_msg = get_sim_swap_message(
        spr.pool.clone(),
        dnr.token_id_1,
        1200,
        false,
        spr.user2,
        None,
    );

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator,
        spr.infinity_pool.clone(),
    );

    let error_res: StdResult<SwapResponse> =
        router.wrap().query_wasm_smart(spr.infinity_pool, &swap_msg);
    let error_msg = error_res.err().unwrap();

    let expected_error = GenericErr {
        msg: "Querier contract error: Generic error: Swap error: \
        pool sale price is below min expected"
            .to_string(),
    };
    assert_eq!(error_msg, expected_error);
}

#[test]
fn robust_query_does_not_revert_whole_tx_on_error() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Trade,
        spot_price,
        finders_fee_bps: None,
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        2000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let sale_price_too_high = 1200_u128;
    let sale_price_valid = 900_u128;
    let swap_msg = SimDirectSwapNftsForTokens {
        pool_id: spr.pool.id,
        nfts_to_swap: vec![
            NftSwap {
                nft_token_id: dnr.token_id_2.to_string(),
                token_amount: Uint128::new(sale_price_valid),
            },
            NftSwap {
                nft_token_id: dnr.token_id_1.to_string(),
                token_amount: Uint128::new(sale_price_too_high), // won't swap bc price too high
            },
        ],
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
        },
        token_recipient: spr.user2.to_string(),
        finder: None,
    };

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let res: StdResult<SwapResponse> = router.wrap().query_wasm_smart(spr.infinity_pool, &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    let spot_price_plus_delta = spot_price + 100_u128;
    let expected_royalty_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();
    check_nft_sale(
        spot_price_plus_delta,
        expected_royalty_fee,
        expected_network_fee,
        0,
        swaps,
        spr.creator,
        spr.user2,
        dnr.token_id_2.to_string(),
    )
}

#[test]
fn network_fee_is_applied_correctly() {
    let spot_price = 20000_u128;
    let trading_fee = 500_u64;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Trade,
        spot_price,
        finders_fee_bps: None,
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, Some(trading_fee));

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        30000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let sale_price = 1000_u128;
    let swap_msg = get_sim_swap_message(
        spr.pool.clone(),
        dnr.token_id_1,
        sale_price,
        true,
        spr.user2.clone(),
        None,
    );

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    let spot_price_plus_delta = spot_price + 100_u128;
    let expected_network_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(5_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_royalty_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    check_nft_sale(
        spot_price_plus_delta,
        expected_royalty_fee,
        expected_network_fee,
        0,
        swaps,
        spr.creator,
        spr.user2,
        dnr.token_id_1.to_string(),
    )
}

#[test]
fn royalty_fee_applied_correctly() {
    let spot_price = 20000_u128;
    let vt = _minter_template_30_pct_fee(5000);

    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Trade,
        spot_price,
        finders_fee_bps: None,
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        25000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let sale_price = 1000_u128;
    let swap_msg = get_sim_swap_message(
        spr.pool.clone(),
        dnr.token_id_1,
        sale_price,
        true,
        spr.user2.clone(),
        None,
    );

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    let spot_price_plus_delta = spot_price + 100;
    let expected_network_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_royalty_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(30_u128, 100_u128)
        .unwrap()
        .u128();
    check_nft_sale(
        spot_price_plus_delta,
        expected_royalty_fee,
        expected_network_fee,
        0,
        swaps,
        spr.creator,
        spr.user2,
        dnr.token_id_1.to_string(),
    )
}

#[test]
fn finders_fee_is_applied_correctly() {
    let finders_fee_bps = 2_u128;
    let spot_price = 20000_u128;

    let vt = standard_minter_template(5000);

    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Trade,
        spot_price,
        finders_fee_bps: Some(finders_fee_bps.try_into().unwrap()),
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut router,
        spr.user1,
        25000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let sale_price = 1000_u128;
    let swap_msg = get_sim_swap_message(
        spr.pool.clone(),
        dnr.token_id_1,
        sale_price,
        true,
        spr.user2.clone(),
        Some(spr.user2.to_string()),
    );

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    let spot_price_plus_delta = spot_price + 100;
    let expected_network_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_royalty_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_finders_fee = Uint128::from(spot_price_plus_delta)
        .checked_multiply_ratio(finders_fee_bps, Uint128::new(100).u128())
        .unwrap();
    check_nft_sale(
        spot_price_plus_delta,
        expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee.u128(),
        swaps,
        spr.creator,
        spr.user2,
        dnr.token_id_1.to_string(),
    )
}
