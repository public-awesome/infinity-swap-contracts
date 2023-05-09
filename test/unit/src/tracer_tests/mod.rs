use crate::helpers::nft_functions::{approve_all, validate_nft_owner};
use crate::helpers::tracer_functions::{create_pool, SwapTestSetup};
use crate::helpers::{nft_functions::mint_and_approve_many, tracer_functions::setup_swap_test};
// use crate::helpers::tracer_functions::{setup_swap_test, validate_swap_outcome, SwapTestSetup};
use crate::helpers::utils::get_native_balances;

use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_multi_test::Executor;
use infinity_index::msg::{PoolQuoteResponse, QueryMsg as InfinityIndexQueryMsg};
use infinity_index::state::PoolQuote;
use infinity_pool::msg::{ExecuteMsg as InfinityIndexExecuteMsg, PoolInfo};
use infinity_pool::state::{BondingCurve, PoolType};
use infinity_router::msg::ExecuteMsg as InfinityRouterExecuteMsg;
use infinity_shared::interface::{NftOrder, SwapParams, SwapResponse};
use sg_std::GENESIS_MINT_START_TIME;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn infinity_redesign_tracer() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_router,
        infinity_pool_code_id,
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let response = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );
    assert!(response.is_ok());

    let infinity_pool = response.unwrap();

    let mut bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_pool,
        101,
    );

    let mut query_response: PoolQuoteResponse = router
        .wrap()
        .query_wasm_smart(
            infinity_index.clone(),
            &InfinityIndexQueryMsg::QuoteSellToPool {
                collection: collection.to_string(),
                limit: 1,
            },
        )
        .unwrap();
    let pool_quote = query_response.pool_quotes.pop().unwrap();
    assert_eq!(
        pool_quote,
        PoolQuote {
            pool: Addr::unchecked("contract6",),
            collection: Addr::unchecked("contract2",),
            quote_price: Uint128::from(10_000u128),
        },
    );

    let response = router
        .execute_contract(
            accts.bidder.clone(),
            infinity_pool,
            &InfinityIndexExecuteMsg::SwapNftsForTokens {
                token_id: bidder_token_ids.pop().unwrap().to_string(),
                min_output: Uint128::from(1u128),
                asset_recipient: accts.bidder.to_string(),
                finder: None,
            },
            &[],
        )
        .unwrap();

    let mut query_response: PoolQuoteResponse = router
        .wrap()
        .query_wasm_smart(
            infinity_index.clone(),
            &InfinityIndexQueryMsg::QuoteSellToPool {
                collection: collection.to_string(),
                limit: 1,
            },
        )
        .unwrap();
    let pool_quote = query_response.pool_quotes.pop().unwrap();
    assert_eq!(
        pool_quote,
        PoolQuote {
            pool: Addr::unchecked("contract6",),
            collection: Addr::unchecked("contract2",),
            quote_price: Uint128::from(9990u128),
        },
    );

    let nft_orders: Vec<NftOrder> = bidder_token_ids
        .iter()
        .map(|token_id| NftOrder {
            token_id: token_id.to_string(),
            amount: Uint128::from(1u128),
        })
        .collect();

    approve_all(&mut router, &accts.bidder, &collection, &infinity_router);
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_router,
        &InfinityRouterExecuteMsg::SwapNftsForTokens {
            collection: collection.to_string(),
            sender: accts.bidder.to_string(),
            nft_orders,
        },
        &[],
    );

    println!("{:?}", response);
}
