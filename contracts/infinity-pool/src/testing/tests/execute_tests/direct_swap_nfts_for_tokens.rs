use crate::msg::SwapResponse;
use crate::state::PoolType;
use crate::testing::helpers::deposit::deposit_nfts_and_tokens;
use crate::testing::helpers::execute_messages::get_direct_swap_nfts_for_tokens_msg;
use crate::testing::helpers::msg::SwapPoolResult;
use crate::testing::helpers::msg::SwapPoolSetup;
use crate::testing::helpers::msg::VendingTemplateSetup;
use crate::testing::helpers::setup_swap_pool::set_pool_active;
use crate::testing::helpers::setup_swap_pool::setup_swap_pool;
use crate::testing::setup::templates::standard_minter_template;
use cosmwasm_std::StdError::GenericErr;
use cosmwasm_std::StdResult;
use cw_multi_test::Executor;
use std::vec;

#[test]
fn cant_swap_inactive_pool() {
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

    let token_id_1 = deposit_nfts_and_tokens(
        &mut router,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
        1000_u128,
    )
    .token_id_1;
    println!("after deposit nfts");
    set_pool_active(
        &mut router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );
    let swap_msg = get_direct_swap_nfts_for_tokens_msg(spr.pool, token_id_1, 1000, true, None);
    println!("before res");
    let res = router.execute_contract(spr.infinity_pool.clone(), spr.infinity_pool, &swap_msg, &[]);
    println!("res is {:?}", res.unwrap_err());
    // let res: StdResult<SwapResponse> = router.wrap().query_wasm_smart(spr.infinity_pool, &swap_msg);

    // let res = res.unwrap_err();
    // assert_eq!(
    //     res,
    //     GenericErr {
    //         msg: "Querier contract error: Generic error: No quote for pool: pool 1 cannot offer quote"
    //             .to_string()
    //     }
    // );
}
