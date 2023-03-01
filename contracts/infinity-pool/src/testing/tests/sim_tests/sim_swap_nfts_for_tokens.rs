use crate::msg::SwapResponse;
use crate::state::PoolType;
use crate::testing::setup::templates::standard_minter_template;
use crate::testing::tests::sim_tests::helpers::{
    check_nft_sale, deposit_nfts_and_tokens, get_sim_swap_message, set_pool_active,
    setup_swap_pool, DepositNftsResult, SwapPoolResult, SwapPoolSetup, VendingTemplateSetup,
};
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use std::vec;

#[test]
fn can_swap_two_active_pools() {
    let spot_price = 1000_u128;
    let vt = standard_minter_template(5000);
    let mut router = vt.router;
    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: vt.accts.creator,
        user1: vt.accts.bidder,
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
    let (sale_price, royalty_price) = (1000_u128, 100_u128);
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
    let expected_network_fee = Uint128::from(spot_price)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();
    check_nft_sale(
        spot_price,
        royalty_price,
        expected_network_fee,
        0,
        swaps,
        spr.pool,
        spr.creator,
        spr.user2,
        dnr.token_id_1.to_string(),
    )
}
