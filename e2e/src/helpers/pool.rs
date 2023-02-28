use super::chain::Chain;
use super::constants::{INFINITY_POOL_NAME, LISTING_FEE};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::ExecResponse;
use cosm_orc::orchestrator::{ExecReq, SigningKey};
use infinity_pool::msg::{ExecuteMsg as InfinityPoolExecuteMsg, QueryMsg as InfinityPoolQueryMsg};
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
        funds: funds,
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

pub fn create_pool(
    chain: &mut Chain,
    create_pool_msg: InfinityPoolExecuteMsg,
    user: &SigningKey,
) -> ExecResponse {
    let denom = &chain.cfg.orc_cfg.chain_cfg.denom;

    let mut reqs = vec![];
    reqs.push(ExecReq {
        contract_name: INFINITY_POOL_NAME.to_string(),
        msg: Box::new(create_pool_msg),
        funds: vec![OrcCoin {
            amount: LISTING_FEE,
            denom: denom.parse().unwrap(),
        }],
    });

    chain
        .orc
        .execute_batch("infinity_pool_create_pool", reqs, user)
        .unwrap()
}

pub fn deposit_nfts(
    chain: &mut Chain,
    pool_id: u64,
    collection: String,
    nft_token_ids: Vec<String>,
    user: &SigningKey,
) -> ExecResponse {
    let mut reqs = vec![];
    reqs.push(ExecReq {
        contract_name: INFINITY_POOL_NAME.to_string(),
        msg: Box::new(InfinityPoolExecuteMsg::DepositNfts {
            pool_id,
            collection,
            nft_token_ids,
        }),
        funds: vec![],
    });

    chain
        .orc
        .execute_batch("infinity_pool_deposit_nfts", reqs, user)
        .unwrap()
}
