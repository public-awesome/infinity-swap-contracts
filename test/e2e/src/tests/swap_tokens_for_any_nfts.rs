use crate::helpers::{
    chain::Chain,
    constants::SG721_NAME,
    helper::{gen_users, latest_block_time},
    instantiate::instantiate_minter,
    pool::{create_pools_from_fixtures, pool_execute_message, pool_query_message},
};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosmwasm_std::Uint128;
use infinity_swap::interface::{SwapParams, SwapResponse};
use infinity_swap::msg::{ExecuteMsg as InfinitySwapExecuteMsg, QueryMsg as InfinitySwapQueryMsg};
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

    let _pools = create_pools_from_fixtures(
        chain,
        &maker,
        pool_deposit_amount,
        10,
        &Some(asset_account_addr.to_string()),
        150,
        300,
    );
    let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();

    let num_swaps: u8 = 10;
    let orders: Vec<Uint128> = vec![Uint128::from(1_000_000u128); num_swaps as usize];

    let sim_res: SwapResponse = pool_query_message(
        chain,
        InfinitySwapQueryMsg::SimSwapTokensForAnyNfts {
            collection: collection.to_string(),
            orders: orders.clone(),
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

    let total_amount: Uint128 = orders.iter().sum();

    let exec_resp = pool_execute_message(
        chain,
        InfinitySwapExecuteMsg::SwapTokensForAnyNfts {
            collection,
            orders,
            swap_params: SwapParams {
                deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                robust: false,
                asset_recipient: None,
                finder: None,
            },
        },
        "infinity-swap-swap-nfts-for-tokens",
        vec![OrcCoin {
            amount: total_amount.u128(),
            denom: denom.parse().unwrap(),
        }],
        &taker,
    );

    let tags = exec_resp
        .res
        .find_event_tags("wasm-swap".to_string(), "pool_id".to_string());
    assert!(tags.len() == num_swaps as usize);
}
