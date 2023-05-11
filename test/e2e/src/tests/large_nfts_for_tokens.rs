use crate::helpers::{
    chain::Chain,
    constants::{
        INFINITY_INDEX_NAME, INFINITY_POOL_NAME, INFINITY_ROUTER_NAME, MARKETPLACE_NAME, SG721_NAME,
    },
    instantiate::{instantiate_infinity_index, instantiate_infinity_router, instantiate_minter},
    nft::{approve_all_nfts, mint_nfts},
    pool::{create_active_pool, pool_query_message},
};

use cosm_orc::orchestrator::ExecReq;
use cosmwasm_std::{Addr, Uint128};
use infinity_index::msg::{PoolQuoteResponse, QueryMsg as InfinityIndexQueryMsg};
use infinity_pool::msg::{InstantiateMsg as InfinityPoolInstantiateMsg, PoolInfo};
use infinity_pool::state::{BondingCurve, PoolType};
use infinity_router::msg::ExecuteMsg as InfinityRouterExecuteMsg;
use infinity_shared::interface::NftOrder;
use test_context::test_context;

#[allow(dead_code)]
const LARGE_NUM_SWAPS: usize = 250;

#[test_context(Chain)]
#[test]
#[ignore]
fn large_single_pool_nft_for_token_swap(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let creator = chain.cfg.users[0].clone();
    let user = chain.cfg.users[1].clone();

    // init minter
    instantiate_minter(
        &mut chain.orc,
        // set creator address as user to allow for minting on base minter
        user.account.address.to_string(),
        &creator.key,
        &denom,
    )
    .unwrap();

    let marketplace = chain.orc.contract_map.address(MARKETPLACE_NAME).unwrap();
    let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();
    let infinity_factory = Addr::unchecked("infinity_factory");

    instantiate_infinity_index(&mut chain.orc, &creator, &marketplace).unwrap();
    let infinity_index = chain.orc.contract_map.address(INFINITY_INDEX_NAME).unwrap();

    instantiate_infinity_router(&mut chain.orc, &creator, &infinity_index).unwrap();

    let pool_addr = create_active_pool(
        chain,
        &creator,
        10_000_000u128,
        InfinityPoolInstantiateMsg {
            marketplace,
            infinity_index: infinity_index.clone(),
            pool_info: PoolInfo {
                collection: collection.clone(),
                owner: creator.account.address.to_string(),
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
        },
    );

    let bidder_token_ids = mint_nfts(chain, 100, &user.key);

    approve_all_nfts(
        chain,
        chain.orc.contract_map.address(INFINITY_ROUTER_NAME).unwrap(),
        &user.key,
    );

    let pool_quote_response: PoolQuoteResponse = chain
        .orc
        .query(
            INFINITY_INDEX_NAME,
            &InfinityIndexQueryMsg::QuoteSellToPool {
                collection: collection.to_string(),
                limit: 1,
            },
        )
        .unwrap()
        .data()
        .unwrap();
    assert_eq!(pool_quote_response.pool_quotes.len(), 1);

    let reqs = vec![ExecReq {
        contract_name: INFINITY_ROUTER_NAME.to_string(),
        msg: Box::new(InfinityRouterExecuteMsg::SwapNftsForTokens {
            collection: collection.to_string(),
            sender: user.account.address.to_string(),
            nft_orders: bidder_token_ids
                .iter()
                .map(|token_id| NftOrder {
                    token_id: token_id.to_string(),
                    amount: Uint128::from(1u128),
                })
                .collect(),
        }),
        funds: vec![],
    }];
    let response =
        chain.orc.execute_batch("infinity-router-swap-nfts-for-tokens", reqs, &user.key).unwrap();
    println!("response: {:?}", response);

    // let response = router
    //     .execute_contract(
    //         accts.bidder.clone(),
    //         infinity_pool,
    //         &InfinityIndexExecuteMsg::SwapNftsForTokens {
    //             token_id: bidder_token_ids.pop().unwrap().to_string(),
    //             min_output: Uint128::from(1u128),
    //             asset_recipient: accts.bidder.to_string(),
    //             finder: None,
    //         },
    //         &[],
    //     )
    //     .unwrap();

    // let mut query_response: PoolQuoteResponse = router
    //     .wrap()
    //     .query_wasm_smart(
    //         infinity_index.clone(),
    //         &InfinityIndexQueryMsg::QuoteSellToPool {
    //             collection: collection.to_string(),
    //             limit: 1,
    //         },
    //     )
    //     .unwrap();
    // let pool_quote = query_response.pool_quotes.pop().unwrap();
    // assert_eq!(
    //     pool_quote,
    //     PoolQuote {
    //         pool: Addr::unchecked("contract6",),
    //         collection: Addr::unchecked("contract2",),
    //         quote_price: Uint128::from(9990u128),
    //     },
    // );

    // let nft_orders: Vec<NftOrder> = bidder_token_ids
    //     .iter()
    //     .map(|token_id| NftOrder {
    //         token_id: token_id.to_string(),
    //         amount: Uint128::from(1u128),
    //     })
    //     .collect();

    // approve_all(&mut router, &accts.bidder, &collection, &infinity_router);
    // let response = router.execute_contract(
    //     accts.bidder.clone(),
    //     infinity_router,
    //     &InfinityRouterExecuteMsg::SwapNftsForTokens {
    //         collection: collection.to_string(),
    //         sender: accts.bidder.to_string(),
    //         nft_orders,
    //     },
    //     &[],
    // );

    // println!("{:?}", response);
}
