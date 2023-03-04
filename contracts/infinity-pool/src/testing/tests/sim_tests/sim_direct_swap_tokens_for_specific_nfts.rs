use crate::msg::SwapResponse;
use crate::state::PoolType;
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::deposit_tokens;
use crate::testing::setup::templates::standard_minter_template;
use crate::testing::tests::sim_tests::helpers::{
    check_nft_sale, deposit_nfts, get_sim_direct_swap_tokens_for_specific_nfts_msg,
    set_pool_active, setup_swap_pool, NftSaleCheckParams, SwapPoolResult, SwapPoolSetup,
    VendingTemplateSetup, ASSET_ACCOUNT,
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

    set_pool_active(
        &mut router,
        false,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.user1,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let sale_price = 2000_u128;
    let swap_msg = get_sim_direct_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
        token_id_1,
        sale_price,
        true,
        spr.user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);

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

    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let token_id_1 = deposit_nfts(
        &mut router,
        spr.user1,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .token_id_1;

    let sale_price = 2000_u128;
    let swap_msg = get_sim_direct_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
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
    let swap_msg = get_sim_direct_swap_tokens_for_specific_nfts_msg(
        spr.pool.clone(),
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
        msg: "Querier contract error: Generic error: Invalid pool: pool does not sell NFTs"
            .to_string(),
    };
    let error_msg = res.err().unwrap();
    assert_eq!(error_msg, expected_error);
}
