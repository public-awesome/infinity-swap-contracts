use super::helpers::ProcessSwapPoolResultsResponse;
use crate::msg::SwapResponse;
use crate::state::PoolType;
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::setup::templates::{_minter_template_30_pct_fee, standard_minter_template};
use crate::testing::tests::sim_tests::get_messages::get_swap_tokens_for_any_nfts_msg;
use crate::testing::tests::sim_tests::helpers::{
    check_nft_sale, deposit_nfts, process_swap_results, set_pool_active, setup_swap_pool,
    NftSaleCheckParams, SwapPoolResult, SwapPoolSetup, VendingTemplateSetup, ASSET_ACCOUNT,
};
use cosmwasm_std::Addr;
use cosmwasm_std::StdError::GenericErr;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
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

    let _ = deposit_nfts(
        &mut router,
        spr.minter,
        spr.collection.clone(),
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    );

    let sale_price = 2000_u128;

    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        spr.collection,
        vec![sale_price.into()],
        false,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router.wrap().query_wasm_smart(spr.infinity_pool, &swap_msg);
    assert_eq!(res.unwrap().swaps, []);
}

#[test]
fn can_swap_active_pool() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;

    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_3,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_2,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_1,
            finders_fee_bps: None,
        },
    ];
    let deposit_amounts = vec![spot_price_3, spot_price_3, spot_price_3];
    let pspr: ProcessSwapPoolResultsResponse = process_swap_results(
        &mut router,
        vts,
        swap_pool_configs,
        deposit_amounts,
        None,
        None,
    );
    let token_id_1 = pspr.token_ids.first().unwrap();

    let sale_price = 2000_u128;
    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        pspr.collection,
        vec![sale_price.into()],
        false,
        pspr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(pspr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let swap_price_plus_delta = spot_price_1;
    let expected_royalty_fee = Uint128::from(swap_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price_1,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: 0,
        swaps,
        creator: pspr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: token_id_1.to_string(),
        expected_nft_payer: Addr::unchecked(pspr.user2.clone()),
        expected_finder: pspr.user2,
    };
    check_nft_sale(nft_sale_check_params);
}

#[test]
fn pool_type_must_be_pool_trade_or_nft_error() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Token,
            spot_price: spot_price_3,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Token,
            spot_price: spot_price_2,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Token,
            spot_price: spot_price_1,
            finders_fee_bps: None,
        },
    ];
    let deposit_amounts = vec![spot_price_3, spot_price_3, spot_price_3];
    let pspr: ProcessSwapPoolResultsResponse = process_swap_results(
        &mut router,
        vts,
        swap_pool_configs,
        deposit_amounts,
        None,
        None,
    );

    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        pspr.collection,
        vec![spot_price_1.into()],
        false,
        pspr.user2,
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(pspr.infinity_pool, &swap_msg);

    assert_eq!(res.unwrap().swaps, []);
}

#[test]
fn insuficient_nfts_error() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };

    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Token,
            spot_price: spot_price_3,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Token,
            spot_price: spot_price_2,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Token,
            spot_price: spot_price_1,
            finders_fee_bps: None,
        },
    ];
    let deposit_amounts = vec![spot_price_3, spot_price_3, spot_price_3];
    let pspr: ProcessSwapPoolResultsResponse = process_swap_results(
        &mut router,
        vts,
        swap_pool_configs,
        deposit_amounts,
        None,
        Some(true),
    );

    let token_id_1 = mint(&mut router, &pspr.user1.clone(), &pspr.minter);
    approve(
        &mut router,
        &pspr.user1.clone(),
        &pspr.collection.clone(),
        &pspr.infinity_pool.clone(),
        token_id_1,
    );

    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        pspr.collection,
        vec![spot_price_1.into()],
        false,
        pspr.user2,
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(pspr.infinity_pool.clone(), &swap_msg);
    assert_eq!(res.unwrap().swaps, []);
}

#[test]
fn sale_price_above_expected_error() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;

    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_3,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_2,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_1,
            finders_fee_bps: None,
        },
    ];
    let deposit_amounts = vec![spot_price_3, spot_price_3, spot_price_3];
    let pspr: ProcessSwapPoolResultsResponse = process_swap_results(
        &mut router,
        vts,
        swap_pool_configs,
        deposit_amounts,
        None,
        None,
    );

    let sale_price = 50_u128;
    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        pspr.collection,
        vec![sale_price.into()],
        false,
        pspr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(pspr.infinity_pool, &swap_msg);

    let expected_error = GenericErr {
            msg: "Querier contract error: Generic error: Swap error: pool sale price is above max expected"
                .to_string(),
        };
    let error_msg = res.err().unwrap();
    assert_eq!(error_msg, expected_error);
}

#[test]
fn robust_query_does_not_revert_whole_tx() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_3,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_2,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_1,
            finders_fee_bps: None,
        },
    ];
    let deposit_amounts = vec![spot_price_3, spot_price_3, spot_price_3];
    let pspr: ProcessSwapPoolResultsResponse = process_swap_results(
        &mut router,
        vts,
        swap_pool_configs,
        deposit_amounts,
        None,
        None,
    );
    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        pspr.collection,
        vec![spot_price_3.into(), 0_u128.into(), 0_u128.into()], // second two quotes are invalid
        true,
        pspr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(pspr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let swap_price_plus_delta = spot_price_1;
    let expected_royalty_fee = Uint128::from(swap_price_plus_delta)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price_1,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: 0,
        swaps,
        creator: pspr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: pspr.token_ids[0].to_string(),
        expected_nft_payer: Addr::unchecked(pspr.user2.clone()),
        expected_finder: pspr.user2,
    };
    check_nft_sale(nft_sale_check_params);
}

#[test]
fn trading_fee_is_applied_correctly() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;
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

    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_3,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_2,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_1,
            finders_fee_bps: None,
        },
    ];
    let deposit_amounts = vec![spot_price_3, spot_price_3, spot_price_3];
    let pspr: ProcessSwapPoolResultsResponse = process_swap_results(
        &mut router,
        vts,
        swap_pool_configs,
        deposit_amounts,
        Some(trading_fee),
        None,
    );

    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        pspr.collection,
        vec![spot_price_3.into()],
        true,
        pspr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(pspr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let expected_royalty_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(5_u128, 100_u128)
        .unwrap()
        .u128();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price_1,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: 0,
        swaps,
        creator: pspr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: pspr.token_ids[0].to_string(),
        expected_nft_payer: Addr::unchecked(pspr.user2.clone()),
        expected_finder: pspr.user2,
    };
    check_nft_sale(nft_sale_check_params);
}

#[test]
fn royalty_fee_applied_correctly() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;
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
    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_3,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_2,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_1,
            finders_fee_bps: None,
        },
    ];
    let deposit_amounts = vec![spot_price_3, spot_price_3, spot_price_3];
    let pspr: ProcessSwapPoolResultsResponse = process_swap_results(
        &mut router,
        vts,
        swap_pool_configs,
        deposit_amounts,
        Some(trading_fee),
        None,
    );

    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        pspr.collection,
        vec![spot_price_3.into()],
        true,
        pspr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(pspr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let expected_royalty_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(30_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(5_u128, 100_u128)
        .unwrap()
        .u128();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price_1,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: 0,
        swaps,
        creator: pspr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: pspr.token_ids[0].to_string(),
        expected_nft_payer: Addr::unchecked(pspr.user2.clone()),
        expected_finder: pspr.user2,
    };
    check_nft_sale(nft_sale_check_params);
}

#[test]
fn finders_fee_is_applied_correctly() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;
    let finders_fee_bps = 2_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;

    let vts = VendingTemplateSetup {
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
        user2: vt.accts.owner,
        collection: vt.collection_response_vec[0].collection.as_ref().unwrap(),
    };
    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_3,
            finders_fee_bps: Some(finders_fee_bps.try_into().unwrap()),
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_2,
            finders_fee_bps: Some(finders_fee_bps.try_into().unwrap()),
        },
        SwapPoolSetup {
            pool_type: PoolType::Nft,
            spot_price: spot_price_1,
            finders_fee_bps: Some(finders_fee_bps.try_into().unwrap()),
        },
    ];
    let deposit_amounts = vec![spot_price_3, spot_price_3, spot_price_3];
    let pspr: ProcessSwapPoolResultsResponse = process_swap_results(
        &mut router,
        vts,
        swap_pool_configs,
        deposit_amounts,
        None,
        None,
    );

    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        pspr.collection,
        vec![spot_price_3.into()],
        true,
        pspr.user2.clone(),
        Some(pspr.user1.to_string()),
    );
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(pspr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;

    let expected_royalty_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(10_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_network_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();
    let expected_finders_fee = Uint128::from(spot_price_1)
        .checked_multiply_ratio(finders_fee_bps, Uint128::new(100).u128())
        .unwrap();

    let nft_sale_check_params = NftSaleCheckParams {
        expected_spot_price: spot_price_1,
        expected_royalty_price: expected_royalty_fee,
        expected_network_fee,
        expected_finders_fee: expected_finders_fee.u128(),
        swaps,
        creator: pspr.creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: pspr.token_ids[0].to_string(),
        expected_nft_payer: Addr::unchecked(pspr.user2.clone()),
        expected_finder: pspr.user1,
    };
    check_nft_sale(nft_sale_check_params);
}
