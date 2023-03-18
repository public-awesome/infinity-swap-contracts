use super::chain::Chain;
use super::constants::{BASE_MINTER_NAME, MINT_PRICE, SG721_NAME};
use base_minter::msg::ExecuteMsg as BaseMinterExecuteMsg;
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::ExecResponse;
use cosm_orc::orchestrator::{ExecReq, SigningKey};
use itertools::Itertools;
use sg721_base::ExecuteMsg as SG721ExecuteMsg;

const MINTS_PER_TX: usize = 15;
const TXS_PER_BLOCK: usize = 5;

pub fn mint_nfts(chain: &mut Chain, num_nfts: u32, user: &SigningKey) -> Vec<String> {
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

    let chunked_reqs: Vec<Vec<ExecReq>> = reqs
        .into_iter()
        .chunks(MINTS_PER_TX)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    let chunked_chunked_reqs: Vec<Vec<Vec<ExecReq>>> = chunked_reqs
        .into_iter()
        .chunks(TXS_PER_BLOCK)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    let mut mint_ctr = 0;
    let mut token_ids: Vec<String> = vec![];

    for chunked_chunked_req in chunked_chunked_reqs {
        for chunked_req in chunked_chunked_req {
            mint_ctr += chunked_req.len() as u32;

            let resp = chain
                .orc
                .execute_batch("base_minter_batch_mint", chunked_req, user)
                .unwrap();

            let tags = resp
                .res
                .find_event_tags("wasm".to_string(), "token_id".to_string());
            token_ids.append(
                &mut tags
                    .iter()
                    .map(|tag| tag.value.clone())
                    .collect::<Vec<String>>(),
            );
        }

        println!("Minted {} NFTs", mint_ctr);
        // chain
        //     .orc
        //     .poll_for_n_blocks(1, Duration::from_secs(10), true)
        //     .unwrap();
    }

    token_ids
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
