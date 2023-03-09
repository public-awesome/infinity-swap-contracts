use crate::msg::SwapResponse;
use crate::state::PoolType;
use crate::testing::setup::templates::standard_minter_template;
use crate::testing::tests::sim_tests::helpers::{
    check_nft_sale, deposit_nfts, get_swap_tokens_for_any_nfts_msg, process_swap_results,
    set_pool_active, setup_swap_pool, NftSaleCheckParams, SwapPoolResult, SwapPoolSetup,
    VendingTemplateSetup, ASSET_ACCOUNT,
};
use cosmwasm_std::Addr;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use std::vec;

use super::helpers::ProcessSwapPoolResultsResponse;

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
    let pspr: ProcessSwapPoolResultsResponse =
        process_swap_results(&mut router, vts, swap_pool_configs);
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

    let pspr: ProcessSwapPoolResultsResponse =
        process_swap_results(&mut router, vts, swap_pool_configs);

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
