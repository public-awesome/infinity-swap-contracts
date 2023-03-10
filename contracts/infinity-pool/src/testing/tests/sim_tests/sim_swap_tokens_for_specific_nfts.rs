use crate::msg::PoolNftSwap;
use crate::msg::QueryMsg::SimSwapTokensForSpecificNfts;
use crate::msg::{NftSwap, SwapParams, SwapResponse};
use crate::state::PoolType;
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::deposit_tokens;
use crate::testing::setup::templates::{_minter_template_30_pct_fee, standard_minter_template};
use crate::testing::tests::sim_tests::get_messages::get_swap_tokens_for_specific_nfts_msg;
use crate::testing::tests::sim_tests::helpers::{
    check_nft_sale, deposit_nfts, set_pool_active, setup_swap_pool, NftSaleCheckParams,
    SwapPoolResult, SwapPoolSetup, VendingTemplateSetup, ASSET_ACCOUNT,
};
use cosmwasm_std::StdError::GenericErr;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use cosmwasm_std::{Addr, Timestamp};
use sg_std::GENESIS_MINT_START_TIME;
use std::vec;

#[test]
fn error_inactive_pool() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
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
        setup_swap_pool(&mut router, vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        false,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.minter,
        spr.collection.clone(),
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let sale_price = 2000_u128;
    let swap_msg = get_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        spr.collection,
        token_id_1,
        sale_price,
        true,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router.wrap().query_wasm_smart(spr.infinity_pool, &swap_msg);
    let error_msg = res.err().unwrap();

    let expected_error = GenericErr {
        msg: "Querier contract error: Generic error: Invalid pool: pool is not active".to_string(),
    };
    assert_eq!(error_msg, expected_error);
}

#[test]
fn can_swap_active_pool() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
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
        setup_swap_pool(&mut router, vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.minter,
        spr.collection.clone(),
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let sale_price = 2000_u128;
    let swap_msg = get_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        spr.collection,
        token_id_1,
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

    let swap_price_plus_delta = spot_price;
    let expected_royalty_fee = Uint128::from(swap_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: 0,
        swaps,
        creator: spr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: token_id_1.to_string(),
        expected_nft_payer: Addr::unchecked(spr.user2.clone()),
        expected_finder: spr.user2,
    };
    check_nft_sale(nft_sale_check_params);
}

#[test]
fn pool_type_must_be_pool_trade_or_nft_error() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
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
        setup_swap_pool(&mut router, vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let _ = deposit_tokens(
        &mut router,
        spr.infinity_pool.clone(),
        spr.creator,
        spr.pool.id,
        2500_u128.into(),
    );
    let token_id_1 = mint(&mut router, &spr.user1.clone(), &spr.minter);
    approve(
        &mut router,
        &spr.user1.clone(),
        &spr.collection.clone(),
        &spr.infinity_pool.clone(),
        token_id_1,
    );

    let sale_price = 2000_u128;
    let swap_msg = get_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        spr.collection,
        token_id_1,
        sale_price,
        true,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    let expected_error = GenericErr {
        msg: "Querier contract error: Generic error: Invalid pool: pool cannot sell nfts"
            .to_string(),
    };
    let error_msg = res.err().unwrap();
    assert_eq!(error_msg, expected_error);
}

#[test]
fn insuficient_nfts_error() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
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
        setup_swap_pool(&mut router, vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = mint(&mut router, &spr.user1.clone(), &spr.minter);
    approve(
        &mut router,
        &spr.user1.clone(),
        &spr.collection.clone(),
        &spr.infinity_pool.clone(),
        token_id_1,
    );

    let sale_price = 2000_u128;
    let swap_msg = get_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        spr.collection,
        token_id_1,
        sale_price,
        false,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);

    let expected_error = GenericErr {
        msg: "Querier contract error: Generic error: Invalid pool: pool cannot offer quote"
            .to_string(),
    };
    let error_msg = res.err().unwrap();
    assert_eq!(error_msg, expected_error);
}

#[test]
fn sale_price_above_expected_error() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
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
        setup_swap_pool(&mut router, vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.minter,
        spr.collection.clone(),
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let sale_price = 500_u128;
    let swap_msg = get_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        spr.collection,
        token_id_1,
        sale_price,
        false,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router.wrap().query_wasm_smart(spr.infinity_pool, &swap_msg);

    let expected_error = GenericErr {
            msg: "Querier contract error: Generic error: Swap error: pool sale price is above max expected"
                .to_string(),
        };
    let error_msg = res.err().unwrap();
    assert_eq!(error_msg, expected_error);
}

#[test]
fn robust_query_does_not_revert_whole_tx() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
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
        setup_swap_pool(&mut router, vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.minter,
        spr.collection.clone(),
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let swap_msg = SimSwapTokensForSpecificNfts {
        pool_nfts_to_swap_for: vec![PoolNftSwap {
            pool_id: spr.pool.id,
            nft_swaps: vec![
                NftSwap {
                    nft_token_id: token_id_1.to_string(),
                    token_amount: Uint128::new(1200),
                },
                NftSwap {
                    nft_token_id: token_id_1.to_string(),
                    token_amount: Uint128::new(500),
                },
            ],
        }],
        sender: spr.user2.to_string(),
        collection: spr.collection.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
            asset_recipient: None,
            finder: None,
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let swap_price_plus_delta = spot_price;
    let expected_royalty_fee = Uint128::from(swap_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: 0,
        swaps,
        creator: spr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: token_id_1.to_string(),
        expected_nft_payer: Addr::unchecked(spr.user2.clone()),
        expected_finder: spr.user2,
    };
    check_nft_sale(nft_sale_check_params);
}

#[test]
fn trading_fee_is_applied_correctly() {
    let spot_price = 1000_u128;
    let trading_fee = 500_u64;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
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
        setup_swap_pool(&mut router, vts, swap_pool_configs, Some(trading_fee));

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.minter,
        spr.collection.clone(),
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let sale_price = 2000_u128;
    let swap_msg = get_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        spr.collection,
        token_id_1,
        sale_price,
        false,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let swap_price_plus_delta = spot_price;
    let expected_royalty_fee = Uint128::from(swap_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price)
        .checked_multiply_ratio(5_u128, 100_u128)
        .unwrap()
        .u128();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: 0,
        swaps,
        creator: spr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: token_id_1.to_string(),
        expected_nft_payer: Addr::unchecked(spr.user2.clone()),
        expected_finder: spr.user2,
    };
    check_nft_sale(nft_sale_check_params);
}

#[test]
fn royalty_fee_applied_correctly() {
    let spot_price = 1000_u128;
    let trading_fee = 500_u64;
    let vt = _minter_template_30_pct_fee(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
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
        setup_swap_pool(&mut router, vts, swap_pool_configs, Some(trading_fee));

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.minter,
        spr.collection.clone(),
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let sale_price = 2000_u128;
    let swap_msg = get_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        spr.collection,
        token_id_1,
        sale_price,
        false,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let swap_price_plus_delta = spot_price;
    let expected_royalty_fee = Uint128::from(swap_price_plus_delta)
        .checked_multiply_ratio(30_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price)
        .checked_multiply_ratio(5_u128, 100_u128)
        .unwrap()
        .u128();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: 0,
        swaps,
        creator: spr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: token_id_1.to_string(),
        expected_nft_payer: Addr::unchecked(spr.user2.clone()),
        expected_finder: spr.user2,
    };
    check_nft_sale(nft_sale_check_params);
}

#[test]
fn finders_fee_is_applied_correctly() {
    let finders_fee_bps = 2_u128;
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![SwapPoolSetup {
        pool_type: PoolType::Nft,
        spot_price,
        finders_fee_bps: Some(finders_fee_bps.try_into().unwrap()),
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(&mut router, vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.minter,
        spr.collection.clone(),
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let sale_price = 1000_u128;
    let swap_msg = get_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        spr.collection,
        token_id_1,
        sale_price,
        false,
        spr.user2.clone(),
        Some(spr.user1.to_string()),
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let expected_royalty_fee = Uint128::from(spot_price)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_finders_fee = Uint128::from(spot_price)
        .checked_multiply_ratio(finders_fee_bps, Uint128::new(100).u128())
        .unwrap();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: expected_finders_fee.u128(),
        swaps,
        creator: spr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: token_id_1.to_string(),
        expected_nft_payer: Addr::unchecked(spr.user2.clone()),
        expected_finder: spr.user1,
    };
    check_nft_sale(nft_sale_check_params);
}
