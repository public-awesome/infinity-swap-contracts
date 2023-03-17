use crate::error::ContractError;
use crate::state::PoolType;
use crate::testing::helpers::deposit::deposit_tokens;
use crate::testing::helpers::execute_messages::get_direct_swap_nfts_for_tokens_msg;
use crate::testing::helpers::msg::SwapPoolResult;
use crate::testing::helpers::msg::SwapPoolSetup;
use crate::testing::helpers::msg::VendingTemplateSetup;
use crate::testing::helpers::nft_functions::approve;
use crate::testing::helpers::nft_functions::mint;
use crate::testing::helpers::setup_swap_pool::set_pool_active;
use crate::testing::helpers::setup_swap_pool::setup_swap_pool;
use crate::testing::setup::templates::standard_minter_template;
use cw_multi_test::Executor;
use std::vec;

#[test]
fn cant_swap_inactive_pool() {
    let spot_price = 1000_u128;
    let finders_fee_bps = 50_u128;
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
        finders_fee_bps: Some(finders_fee_bps.try_into().unwrap()),
    }];
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(&mut router, vts, swap_pool_configs, None);

    let spr: SwapPoolResult = swap_results.pop().unwrap().unwrap();

    let _ = deposit_tokens(
        &mut router,
        2500_u128,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    );
    let token_id_1 = mint(&mut router, &spr.creator.clone(), &spr.minter.clone());
    approve(
        &mut router,
        &spr.creator.clone(),
        &spr.collection,
        &spr.infinity_pool.clone(),
        token_id_1,
    );

    set_pool_active(
        &mut router,
        false,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );
    let swap_msg = get_direct_swap_nfts_for_tokens_msg(
        spr.pool,
        token_id_1,
        900,
        true,
        Some(spr.user1.to_string()),
    );
    let res = router.execute_contract(spr.creator.clone(), spr.infinity_pool, &swap_msg, &[]);
    let expected_error = ContractError::InvalidPool("pool is not active".to_string());
    assert_eq!(
        res.unwrap_err().root_cause().to_string(),
        expected_error.to_string()
    );
}
