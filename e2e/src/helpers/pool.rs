use super::chain::Chain;
use super::constants::{INFINITY_POOL_NAME, LISTING_FEE, SG721_NAME};
use super::fixtures::get_pool_fixtures;
use crate::helpers::nft::{approve_all_nfts, mint_nfts};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::ExecResponse;
use cosm_orc::orchestrator::{ExecReq, SigningKey};
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, PoolsByIdResponse, QueryMsg as InfinityPoolQueryMsg,
};
use infinity_pool::state::Pool;
use serde::Deserialize;

pub fn pool_execute_message(
    chain: &mut Chain,
    execute_msg: InfinityPoolExecuteMsg,
    op_name: &str,
    funds: Vec<OrcCoin>,
    user: &SigningKey,
) -> ExecResponse {
    let mut reqs = vec![];
    reqs.push(ExecReq {
        contract_name: INFINITY_POOL_NAME.to_string(),
        msg: Box::new(execute_msg),
        funds,
    });

    chain.orc.execute_batch(op_name, reqs, user).unwrap()
}

pub fn pool_query_message<T: for<'a> Deserialize<'a>>(
    chain: &Chain,
    query_msg: InfinityPoolQueryMsg,
) -> T {
    chain
        .orc
        .query(INFINITY_POOL_NAME, &query_msg)
        .unwrap()
        .data()
        .unwrap()
}

pub fn create_active_pool(
    chain: &mut Chain,
    user: &SigningKey,
    pool_deposit_amount: u128,
    num_nfts: u32,
    create_pool_msg: InfinityPoolExecuteMsg,
) -> Pool {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();

    let resp = mint_nfts(chain, num_nfts, &user);
    let mut token_ids: Vec<String> = vec![];

    for res in resp {
        let tags = res
            .res
            .find_event_tags("wasm".to_string(), "token_id".to_string());
        token_ids.append(
            &mut tags
                .iter()
                .map(|tag| tag.value.clone())
                .collect::<Vec<String>>(),
        );
    }

    approve_all_nfts(
        chain,
        chain.orc.contract_map.address(INFINITY_POOL_NAME).unwrap(),
        &user,
    );

    let resp = pool_execute_message(
        chain,
        create_pool_msg,
        "infinity-pool-create-pool",
        vec![OrcCoin {
            amount: LISTING_FEE,
            denom: denom.parse().unwrap(),
        }],
        &user,
    );

    let tag = resp
        .res
        .find_event_tags("wasm-create-pool".to_string(), "id".to_string())[0];
    let pool_id = tag.value.parse::<u64>().unwrap();
    let mut resp: PoolsByIdResponse = pool_query_message(
        chain,
        InfinityPoolQueryMsg::PoolsById {
            pool_ids: vec![pool_id],
        },
    );
    let pool = &resp.pools.pop().unwrap().1.unwrap();

    if pool.can_sell_nfts() {
        pool_execute_message(
            chain,
            InfinityPoolExecuteMsg::DepositNfts {
                pool_id,
                collection: collection.clone(),
                nft_token_ids: token_ids.clone(),
            },
            "infinity-pool-deposit-nfts",
            vec![],
            &user,
        );
    }

    if pool.can_buy_nfts() {
        pool_execute_message(
            chain,
            InfinityPoolExecuteMsg::DepositTokens { pool_id },
            "infinity-pool-deposit-tokens",
            vec![OrcCoin {
                amount: pool_deposit_amount,
                denom: denom.parse().unwrap(),
            }],
            &user,
        );
    }

    pool_execute_message(
        chain,
        InfinityPoolExecuteMsg::SetActivePool {
            is_active: true,
            pool_id,
        },
        "infinity-pool-activate",
        vec![],
        &user,
    );

    let mut resp: PoolsByIdResponse = pool_query_message(
        chain,
        InfinityPoolQueryMsg::PoolsById {
            pool_ids: vec![pool_id],
        },
    );
    let pool = resp.pools.pop().unwrap().1.unwrap();
    pool
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
