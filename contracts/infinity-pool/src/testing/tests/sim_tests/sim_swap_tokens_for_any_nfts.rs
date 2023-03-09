use crate::msg::{NftSwap, SwapParams, SwapResponse};
use crate::state::Pool;
use crate::state::PoolType;
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::deposit_tokens;
use crate::testing::setup::templates::{_minter_template_30_pct_fee, standard_minter_template};
use crate::testing::tests::sim_tests::helpers::{
    check_nft_sale, deposit_nfts, deposit_one_nft, get_swap_tokens_for_any_nfts_msg,
    set_pool_active, setup_swap_pool, NftSaleCheckParams, SwapPoolResult, SwapPoolSetup,
    VendingTemplateSetup, ASSET_ACCOUNT,
};
use cosmwasm_std::StdError::GenericErr;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use cosmwasm_std::{Addr, Timestamp};
use sg_multi_test::StargazeApp;
use sg_std::GENESIS_MINT_START_TIME;
use std::mem::swap;
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

fn process_swap_result(
    router: &mut StargazeApp,
    swap_result: Result<SwapPoolResult, anyhow::Error>,
) -> (u32, Addr, Addr, Addr, Pool, Addr, Addr) {
    let r = swap_result.unwrap();

    set_pool_active(
        router,
        true,
        r.pool.clone(),
        r.creator.clone(),
        r.infinity_pool.clone(),
    );
    let token_id = deposit_one_nft(
        router,
        r.minter.clone(),
        r.collection.clone(),
        r.infinity_pool.clone(),
        r.pool.clone(),
        r.creator.clone(),
    );
    println!(
        "spot price {:?} token id: {:?}",
        r.pool.spot_price, token_id
    );
    // minter = Some(result.minter);
    // collection = Some(result.collection);
    // infinity_pool = Some(result.infinity_pool);
    // pool = Some(result.pool);
    // creator = Some(result.creator);
    // user2 = Some(result.user2);
    (
        token_id,
        r.minter,
        r.collection,
        r.infinity_pool,
        r.pool,
        r.creator,
        r.user2,
    )
}
fn process_swap_results(
    router: &mut StargazeApp,
    swap_results: Vec<Result<SwapPoolResult, anyhow::Error>>,
) -> Vec<u32> {
    let mut token_ids = vec![];
    for result in swap_results {
        let token_id = process_swap_result(router, result).0;
        token_ids.append(&mut vec![token_id]);
    }
    token_ids.reverse();
    token_ids
}

#[test]
fn can_swap_active_pool() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 2000_u128;
    let spot_price_3 = 3000_u128;

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
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);
    let swap_result = swap_results.pop().unwrap();
    let (token_id, _, collection, infinity_pool, _, creator, user2) =
        process_swap_result(&mut router, swap_result);
    let mut token_ids_2 = process_swap_results(&mut router, swap_results);

    let mut token_ids = vec![token_id];
    token_ids.append(&mut token_ids_2);
    let token_id_1 = token_ids.first().unwrap();

    let sale_price = 2000_u128;
    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        collection,
        vec![sale_price.into()],
        false,
        user2.clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &swap_msg);
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
        creator,
        expected_seller: Addr::unchecked(ASSET_ACCOUNT),
        token_id: token_id_1.to_string(),
        expected_nft_payer: Addr::unchecked(user2.clone()),
        expected_finder: user2,
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
        router: &mut router,
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
    let swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let mut minter: Option<Addr> = None;
    let mut collection: Option<Addr> = None;
    let mut infinity_pool: Option<Addr> = None;
    let mut pool: Option<Pool> = None;
    let mut creator: Option<Addr> = None;
    let mut user2: Option<Addr> = None;
    let mut token_ids = vec![];

    for spr in swap_results {
        let result = spr.unwrap();
        set_pool_active(
            &mut router,
            true,
            result.pool.clone(),
            result.creator.clone(),
            result.infinity_pool.clone(),
        );
        minter = Some(result.minter);
        collection = Some(result.collection);
        infinity_pool = Some(result.infinity_pool);
        pool = Some(result.pool);
        creator = Some(result.creator);
        user2 = Some(result.user2);

        let this_pool = pool.as_ref().unwrap().clone();

        let token_id = deposit_one_nft(
            &mut router,
            minter.clone().unwrap(),
            collection.clone().unwrap(),
            infinity_pool.clone().unwrap(),
            this_pool.clone(),
            creator.clone().unwrap(),
        );
        token_ids.push(token_id);

        let _ = deposit_tokens(
            &mut router,
            infinity_pool.clone().unwrap(),
            creator.unwrap().clone(),
            this_pool.id,
            2500_u128.into(),
        );
    }

    let swap_msg = get_swap_tokens_for_any_nfts_msg(
        collection.unwrap(),
        vec![spot_price_1.into()],
        false,
        user2.unwrap().clone(),
        None,
    );

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.unwrap(), &swap_msg);

    assert_eq!(res.unwrap().swaps, []);
}
