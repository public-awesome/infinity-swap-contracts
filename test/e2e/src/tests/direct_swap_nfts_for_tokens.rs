use crate::helpers::{
    chain::Chain,
    constants::INFINITY_POOL_NAME,
    helper::{gen_users, latest_block_time},
    instantiate::instantiate_minter,
    nft::{approve_all_nfts, mint_and_transfer_nfts},
    pool::{create_pools_from_fixtures, pool_execute_message, pool_query_message},
};
use cosmwasm_std::Uint128;
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, NftSwap, QueryMsg as InfinityPoolQueryMsg, SwapParams,
    SwapResponse,
};
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
    let mut users = gen_users(chain, 2, balance);
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

    let pools = create_pools_from_fixtures(
        chain,
        &maker,
        pool_deposit_amount,
        10,
        &Some(asset_account_addr.to_string()),
        150,
        300,
    );

    let mut bidder_token_ids = mint_and_transfer_nfts(chain, 50, &maker, taker_addr.as_ref());
    approve_all_nfts(
        chain,
        chain.orc.contract_map.address(INFINITY_POOL_NAME).unwrap(),
        &taker,
    );

    for pool in pools.iter() {
        if !pool.can_buy_nfts() {
            continue;
        }
        let num_swaps = 3;
        let nfts_to_swap: Vec<NftSwap> = bidder_token_ids
            .drain(0..num_swaps)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
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
        assert!(!sim_res.swaps.is_empty());

        let exec_resp = pool_execute_message(
            chain,
            InfinityPoolExecuteMsg::DirectSwapNftsForTokens {
                pool_id: pool.id,
                nfts_to_swap,
                swap_params: SwapParams {
                    deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                    robust: false,
                    asset_recipient: None,
                    finder: None,
                },
            },
            "infinity-pool-direct-swap-nfts-for-tokens",
            vec![],
            &taker,
        );

        let tags = exec_resp
            .res
            .find_event_tags("wasm-swap".to_string(), "pool_id".to_string());
        println!("{:?}", tags);
        assert!(tags.len() == num_swaps);
    }
}
