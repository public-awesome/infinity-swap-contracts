use crate::helpers::{
    chain::Chain,
    constants::SG721_NAME,
    helper::{gen_users, latest_block_time},
    instantiate::instantiate_minter,
    pool::{create_pools_from_fixtures, pool_execute_message, pool_query_message},
};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosmwasm_std::Uint128;
use infinity_swap::interface::{NftOrder, SwapParams, SwapResponse};
use infinity_swap::msg::{
    ExecuteMsg as InfinitySwapExecuteMsg, NftTokenIdsResponse, QueryMsg as InfinitySwapQueryMsg,
    QueryOptions,
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

    let mut nft_orders: Vec<NftOrder> = vec![];
    let mut sender_amount = Uint128::zero();

    for pool in &pools {
        let nft_token_ids_res: NftTokenIdsResponse = pool_query_message(
            chain,
            InfinitySwapQueryMsg::PoolNftTokenIds {
                pool_id: pool.id,
                query_options: QueryOptions {
                    descending: None,
                    start_after: None,
                    limit: None,
                },
            },
        );

        nft_orders.extend(nft_token_ids_res.nft_token_ids.iter().map(|token_id| {
            let nft_swap = NftOrder {
                token_id: token_id.to_string(),
                amount: Uint128::from(1_000_000u128),
            };
            sender_amount += nft_swap.amount;
            nft_swap
        }));
    }

    let sim_res: SwapResponse = pool_query_message(
        chain,
        InfinitySwapQueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            nft_orders: nft_orders.clone(),
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
        InfinitySwapExecuteMsg::SwapTokensForSpecificNfts {
            collection: collection.to_string(),
            nft_orders: nft_orders.clone(),
            swap_params: SwapParams {
                deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                robust: false,
                asset_recipient: None,
                finder: None,
            },
        },
        "infinity-swap-swap-tokens-for-specific-nfts",
        vec![OrcCoin {
            amount: sender_amount.u128(),
            denom: denom.parse().unwrap(),
        }],
        &taker,
    );

    let tags = exec_resp
        .res
        .find_event_tags("wasm-swap".to_string(), "pool_id".to_string());
    assert!(tags.len() == nft_orders.len() as usize);
}
