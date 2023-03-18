use crate::helpers::{
    chain::Chain,
    constants::{LISTING_FEE, MINT_PRICE, SG721_NAME},
    helper::{gen_users, latest_block_time},
    instantiate::instantiate_minter,
    nft::mint_nfts,
    pool::{create_pools_from_fixtures, pool_execute_message, pool_query_message},
};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosmwasm_std::Uint128;
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, NftSwap, PoolQuoteResponse, PoolsByIdResponse,
    QueryMsg as InfinityPoolQueryMsg, QueryOptions, SwapParams, SwapResponse,
};
use infinity_pool::state::BondingCurve;
use test_context::test_context;

#[test_context(Chain)]
#[test]
#[ignore]
fn swap_small(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let prefix = chain.cfg.orc_cfg.chain_cfg.prefix.clone();
    let master_account = chain.cfg.users[1].clone();

    let pool_deposit_amount = 1_000_000;
    let balance = 1_000_000_000;
    let mut users = gen_users(chain, 1, balance);
    let maker = users.pop().unwrap();
    let maker_addr = maker.to_addr(&prefix).unwrap();
    let taker = users.pop().unwrap();
    let taker_addr = taker.to_addr(&prefix).unwrap();

    let asset_account = gen_users(chain, 1, 1)[0].clone();
    let asset_account_addr = asset_account.to_addr(&prefix).unwrap();

    // init minter
    instantiate_minter(
        &mut chain.orc,
        // set creator address as maker to allow for minting on base minter
        maker_addr.to_string(),
        &master_account.key,
        &denom,
    )
    .unwrap();
    let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();

    let pools = create_pools_from_fixtures(
        chain,
        &maker,
        pool_deposit_amount,
        10,
        &Some(asset_account_addr.to_string()),
        150,
        300,
    );

    let mut bidder_token_ids = mint_nfts(chain, 10, &taker);

    for pool in pools.iter() {
        if !pool.can_buy_nfts() {
            continue;
        }
        let num_swaps = 3;
        let nfts_to_swap: Vec<NftSwap> = bidder_token_ids
            .drain(0..(num_swaps as usize))
            .map(|token_id| NftSwap {
                nft_token_id: token_id.to_string(),
                token_amount: Uint128::from(10u128),
            })
            .collect();

        let sim_res: SwapResponse = pool_query_message(
            chain,
            InfinityPoolQueryMsg::SimDirectSwapNftsForTokens {
                pool_id: pool.id,
                nfts_to_swap: nfts_to_swap.clone(),
                sender: taker_addr.to_string(),
                swap_params: SwapParams {
                    deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                    robust: false,
                    asset_recipient: None,
                    finder: None,
                },
            },
        );
        assert!(sim_res.swaps.len() > 0);

        let total_amount = nfts_to_swap
            .iter()
            .fold(Uint128::zero(), |acc, nft_swap| acc + nft_swap.token_amount);

        let exec_resp = pool_execute_message(
            chain,
            InfinityPoolExecuteMsg::DirectSwapNftsForTokens {
                pool_id: pool.id,
                nfts_to_swap: nfts_to_swap,
                swap_params: SwapParams {
                    deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                    robust: false,
                    asset_recipient: None,
                    finder: None,
                },
            },
            "infinity-pool-swap-tokens-for-any-nfts",
            vec![OrcCoin {
                amount: total_amount.u128(),
                denom: denom.parse().unwrap(),
            }],
            &taker,
        );

        let tags = exec_resp
            .res
            .find_event_tags("wasm-swap".to_string(), "pool_id".to_string());
        println!("{:?}", tags);
        assert!(tags.len() == num_swaps);
    }

    // let pool_deposit_amount = 1_000_000;
    // let balance = (MINT_PRICE + LISTING_FEE + pool_deposit_amount) * 2_000_000;
    // let mut users = gen_users(chain, 2, balance);
    // let maker = users.pop().unwrap();
    // let maker_addr = maker.to_addr(&prefix).unwrap();
    // let taker = users.pop().unwrap();

    // // init minter
    // instantiate_minter(
    //     &mut chain.orc,
    //     // set creator address as maker to allow for minting on base minter
    //     maker_addr.to_string(),
    //     &master_account.key,
    //     &denom,
    // )
    // .unwrap();

    // let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();

    // let pool = create_active_pool(
    //     chain,
    //     &maker,
    //     pool_deposit_amount,
    //     50 * 2,
    //     InfinityPoolExecuteMsg::CreateTradePool {
    //         collection: collection.to_string(),
    //         asset_recipient: None,
    //         bonding_curve: BondingCurve::Linear,
    //         spot_price: Uint128::from(100_u128),
    //         delta: Uint128::zero(),
    //         finders_fee_bps: 0,
    //         swap_fee_bps: 100,
    //         reinvest_tokens: true,
    //         reinvest_nfts: true,
    //     },
    // );

    // let resp: PoolsByIdResponse = pool_query_message(
    //     chain,
    //     InfinityPoolQueryMsg::PoolsById {
    //         pool_ids: vec![pool.id],
    //     },
    // );
    // println!("{:?}", resp);

    // let resp: PoolQuoteResponse = pool_query_message(
    //     chain,
    //     InfinityPoolQueryMsg::PoolQuotesSell {
    //         collection: collection.to_string(),
    //         query_options: QueryOptions {
    //             descending: None,
    //             start_after: None,
    //             limit: None,
    //         },
    //     },
    // );
    // println!("{:?}", resp);

    // let token_input = vec![Uint128::from(100u128); 50];
    // let total_amount: Uint128 = token_input.iter().sum();
    // println!("total_amount: {:?}", total_amount);
    // let resp = pool_execute_message(
    //     chain,
    //     InfinityPoolExecuteMsg::SwapTokensForAnyNfts {
    //         collection: collection.to_string(),
    //         max_expected_token_input: token_input.clone(),
    //         swap_params: SwapParams {
    //             deadline: latest_block_time(&chain.orc).plus_seconds(10),
    //             robust: true,
    //             asset_recipient: None,
    //             finder: None,
    //         },
    //     },
    //     "infinity-pool-swap-tokens-for-any-nfts",
    //     vec![OrcCoin {
    //         amount: total_amount.u128(),
    //         denom: denom.parse().unwrap(),
    //     }],
    //     &taker,
    // );

    // for event in resp.res.events {
    //     println!("{:?}", event);
    // }
    // // let tags = resp
    // //     .res
    // //     .find_event_tags("wasm-swap".to_string(), "pool_id".to_string());
    // // println!("{:?}", tags);
    // // assert!(tags.len() == 2);

    // let resp: PoolsByIdResponse = pool_query_message(
    //     chain,
    //     InfinityPoolQueryMsg::PoolsById {
    //         pool_ids: vec![pool.id],
    //     },
    // );
    // println!("{:?}", resp);

    // let resp: PoolQuoteResponse = pool_query_message(
    //     chain,
    //     InfinityPoolQueryMsg::PoolQuotesSell {
    //         collection: collection.to_string(),
    //         query_options: QueryOptions {
    //             descending: None,
    //             start_after: None,
    //             limit: None,
    //         },
    //     },
    // );
    // println!("{:?}", resp);
    // assert!(false);
}
