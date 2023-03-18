use crate::helpers::{
    chain::Chain,
    constants::SG721_NAME,
    helper::{gen_users, latest_block_time},
    instantiate::instantiate_minter,
    pool::{create_pools_from_fixtures, pool_execute_message, pool_query_message},
};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosmwasm_std::Uint128;
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, NftSwap, PoolNftSwap, QueryMsg as InfinityPoolQueryMsg,
    SwapParams, SwapResponse,
};
use infinity_pool::state::Pool;
use itertools::Itertools;
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

    let pool_chunks: Vec<Vec<&Pool>> = pools
        .iter()
        .filter(|&p| p.can_sell_nfts())
        .chunks(3 as usize)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        let mut sender_amount = Uint128::zero();

        let swaps_per_chunk: u8 = 3;
        for pool in &chunk {
            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: pool
                    .nft_token_ids
                    .iter()
                    .take(swaps_per_chunk as usize)
                    .map(|token_id| {
                        let nft_swap = NftSwap {
                            nft_token_id: token_id.to_string(),
                            token_amount: Uint128::from(100_000u128),
                        };
                        sender_amount += nft_swap.token_amount;
                        nft_swap
                    })
                    .collect(),
            });
        }

        let sim_res: SwapResponse = pool_query_message(
            chain,
            InfinityPoolQueryMsg::SimSwapTokensForSpecificNfts {
                collection: collection.to_string(),
                pool_nfts_to_swap_for: pool_nfts_to_swap_for.clone(),
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

        let exec_resp = pool_execute_message(
            chain,
            InfinityPoolExecuteMsg::SwapTokensForSpecificNfts {
                collection: collection.to_string(),
                pool_nfts_to_swap_for: pool_nfts_to_swap_for.clone(),
                swap_params: SwapParams {
                    deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                    robust: false,
                    asset_recipient: None,
                    finder: None,
                },
            },
            "infinity-pool-swap-tokens-for-specific-nfts",
            vec![OrcCoin {
                amount: sender_amount.u128(),
                denom: denom.parse().unwrap(),
            }],
            &taker,
        );

        let tags = exec_resp
            .res
            .find_event_tags("wasm-swap".to_string(), "pool_id".to_string());
        println!("{:?}", tags);
        assert!(tags.len() == chunk.len() * swaps_per_chunk as usize);
    }
}
