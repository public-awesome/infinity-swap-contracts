use super::chain::Chain;
use super::constants::{INFINITY_SWAP_NAME, LISTING_FEE, SG721_NAME};
use super::fixtures::get_pool_fixtures;
use crate::helpers::nft::{approve_all_nfts, mint_nfts};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::ExecResponse;
use cosm_orc::orchestrator::{ExecReq, SigningKey};
use infinity_swap::msg::{
    ExecuteMsg as InfinitySwapExecuteMsg, PoolsByIdResponse, QueryMsg as InfinitySwapQueryMsg,
};
use infinity_swap::state::Pool;
use serde::Deserialize;

pub fn pool_execute_message(
    chain: &mut Chain,
    execute_msg: InfinitySwapExecuteMsg,
    op_name: &str,
    funds: Vec<OrcCoin>,
    user: &SigningKey,
) -> ExecResponse {
    let reqs = vec![ExecReq {
        contract_name: INFINITY_SWAP_NAME.to_string(),
        msg: Box::new(execute_msg),
        funds,
    }];

    chain.orc.execute_batch(op_name, reqs, user).unwrap()
}

pub fn pool_query_message<T: for<'a> Deserialize<'a>>(
    chain: &Chain,
    query_msg: InfinitySwapQueryMsg,
) -> T {
    chain
        .orc
        .query(INFINITY_SWAP_NAME, &query_msg)
        .unwrap()
        .data()
        .unwrap()
}

pub fn create_active_pool(
    chain: &mut Chain,
    user: &SigningKey,
    pool_deposit_amount: u128,
    num_nfts: u32,
    create_pool_msg: InfinitySwapExecuteMsg,
) -> Pool {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();

    let token_ids = mint_nfts(chain, num_nfts, user);

    approve_all_nfts(
        chain,
        chain.orc.contract_map.address(INFINITY_SWAP_NAME).unwrap(),
        user,
    );

    let resp = pool_execute_message(
        chain,
        create_pool_msg,
        "infinity-swap-create-pool",
        vec![OrcCoin {
            amount: LISTING_FEE,
            denom: denom.parse().unwrap(),
        }],
        user,
    );

    let tag = resp
        .res
        .find_event_tags("wasm-create-pool".to_string(), "id".to_string())[0];
    let pool_id = tag.value.parse::<u64>().unwrap();
    let mut resp: PoolsByIdResponse = pool_query_message(
        chain,
        InfinitySwapQueryMsg::PoolsById {
            pool_ids: vec![pool_id],
        },
    );
    let pool = &resp.pools.pop().unwrap().1.unwrap();

    if pool.can_sell_nfts() {
        pool_execute_message(
            chain,
            InfinitySwapExecuteMsg::DepositNfts {
                pool_id,
                collection,
                nft_token_ids: token_ids,
            },
            "infinity-swap-deposit-nfts",
            vec![],
            user,
        );
    }

    if pool.can_buy_nfts() {
        pool_execute_message(
            chain,
            InfinitySwapExecuteMsg::DepositTokens { pool_id },
            "infinity-swap-deposit-tokens",
            vec![OrcCoin {
                amount: pool_deposit_amount,
                denom: denom.parse().unwrap(),
            }],
            user,
        );
    }

    pool_execute_message(
        chain,
        InfinitySwapExecuteMsg::SetActivePool {
            is_active: true,
            pool_id,
        },
        "infinity-swap-activate",
        vec![],
        user,
    );

    let mut resp: PoolsByIdResponse = pool_query_message(
        chain,
        InfinitySwapQueryMsg::PoolsById {
            pool_ids: vec![pool_id],
        },
    );
    resp.pools.pop().unwrap().1.unwrap()
}

pub fn create_pools_from_fixtures(
    chain: &mut Chain,
    user: &SigningKey,
    pool_deposit_amount: u128,
    num_nfts: u32,
    asset_account: &Option<String>,
    finders_fee_bps: u64,
    swap_fee_bps: u64,
) -> Vec<Pool> {
    let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();

    let pool_fixtures =
        get_pool_fixtures(&collection, asset_account, finders_fee_bps, swap_fee_bps);

    let mut pools: Vec<Pool> = vec![];
    for fixt in pool_fixtures {
        pools.push(create_active_pool(
            chain,
            user,
            pool_deposit_amount,
            num_nfts,
            fixt,
        ));
    }
    pools
}
