use super::chain::Chain;
use super::constants::{BASE_MINTER_NAME, MINT_PRICE, SG721_NAME};
use base_minter::msg::ExecuteMsg as BaseMinterExecuteMsg;
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::ExecResponse;
use cosm_orc::orchestrator::{ExecReq, SigningKey};
use sg721_base::ExecuteMsg as SG721ExecuteMsg;

pub fn mint_nfts(chain: &mut Chain, num_nfts: u32, user: &SigningKey) -> ExecResponse {
    let denom = &chain.cfg.orc_cfg.chain_cfg.denom;

    let mut reqs = vec![];
    for i in 1..(num_nfts + 1) {
        reqs.push(ExecReq {
            contract_name: BASE_MINTER_NAME.to_string(),
            msg: Box::new(BaseMinterExecuteMsg::Mint {
                token_uri: format!(
                    "ipfs://bafybeideczllcb5kz75hgy25irzevarybvazgdiaeiv2xmgqevqgo6d3ua/{}.png",
                    i
                ),
            }),
            funds: vec![OrcCoin {
                amount: MINT_PRICE,
                denom: denom.parse().unwrap(),
            }],
        });
    }

    chain
        .orc
        .execute_batch("base_minter_batch_mint", reqs, user)
        .unwrap()
}

pub fn approve_all_nfts(
    chain: &mut Chain,
    approve_addr: String,
    user: &SigningKey,
) -> ExecResponse {
    let mut reqs = vec![];
    reqs.push(ExecReq {
        contract_name: SG721_NAME.to_string(),
        msg: Box::new(SG721ExecuteMsg::ApproveAll {
            operator: approve_addr.clone(),
            expires: None,
        }),
        funds: vec![],
    });

    chain
        .orc
        .execute_batch("sg721_approve_all", reqs, user)
        .unwrap()
}
