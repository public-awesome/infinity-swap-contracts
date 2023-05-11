use super::chain::{Chain, SigningAccount};
use super::constants::{INFINITY_POOL_NAME, LISTING_FEE, SG721_NAME};
use crate::helpers::nft::{approve_all_nfts, mint_nfts};
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::ExecResponse;
use cosm_orc::orchestrator::{Coin as OrcCoin, Denom};
use cosm_orc::orchestrator::{ExecReq, SigningKey};
use cosm_tome::chain::request::TxOptions;
use cosm_tome::modules::bank::model::SendRequest;
use cosmwasm_std::Addr;
use infinity_pool::msg::{
    ExecuteMsg as InfinityIndexExecuteMsg, InstantiateMsg as InfinityPoolInstantiateMsg,
    QueryMsg as InfinityPoolQueryMsg,
};
use infinity_pool::state::PoolConfig;
use serde::Deserialize;

pub fn pool_execute_message(
    chain: &mut Chain,
    execute_msg: InfinityIndexExecuteMsg,
    op_name: &str,
    funds: Vec<OrcCoin>,
    user: &SigningKey,
) -> ExecResponse {
    let reqs = vec![ExecReq {
        contract_name: INFINITY_POOL_NAME.to_string(),
        msg: Box::new(execute_msg),
        funds,
    }];

    chain.orc.execute_batch(op_name, reqs, user).unwrap()
}

pub fn pool_query_message<T: for<'a> Deserialize<'a>>(
    chain: &Chain,
    query_msg: InfinityPoolQueryMsg,
) -> T {
    chain.orc.query(INFINITY_POOL_NAME, &query_msg).unwrap().data().unwrap()
}
pub fn create_active_pool(
    chain: &mut Chain,
    user: &SigningAccount,
    pool_deposit_amount: u128,
    msg: InfinityPoolInstantiateMsg,
) -> String {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();

    let response = chain
        .orc
        .instantiate(
            INFINITY_POOL_NAME,
            &format!("{}_inst", INFINITY_POOL_NAME),
            &msg,
            &user.key,
            None,
            vec![],
        )
        .unwrap();

    let pool_addr =
        response.res.find_event_tags("wasm".to_string(), "_contract_address".to_string())[0]
            .value
            .clone();

    let send_request = SendRequest {
        from: user.account.address.parse().unwrap(),
        to: pool_addr.parse().unwrap(),
        amounts: vec![OrcCoin {
            denom: denom.parse().unwrap(),
            amount: pool_deposit_amount,
        }],
    };

    tokio_block(async {
        chain.orc.client.bank_send(send_request, &user.key, &TxOptions::default()).await
    })
    .unwrap();

    // pool_execute_message(
    //     chain,
    //     InfinityIndexExecuteMsg::SetIsActive {
    //         is_active: true,
    //     },
    //     "infinity-pool-set-is-active",
    //     vec![],
    //     &user.key,
    // );

    pool_addr
}
