use crate::msg::ExecuteMsg;
use crate::{error::ContractError, state::CONFIG};

use cosmwasm_std::{to_binary, Addr, DepsMut, Env, MessageInfo, SubMsg, WasmMsg};
use cw721::Cw721ExecuteMsg;
use infinity_index::msg::{PoolQuoteResponse, QueryMsg};
use infinity_pool::msg::ExecuteMsg as InfinityPoolExecuteMsg;
use infinity_shared::interface::NftOrder;
use infinity_shared::shared::only_nft_owner;
use sg_marketplace_common::transfer_nft;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::SwapNftsForTokens {
            collection,
            sender,
            nft_orders,
        } => execute_swap_nfts_for_tokens(
            deps,
            env,
            info,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            nft_orders,
        ),
    }
}

pub fn execute_swap_nfts_for_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    collection: Addr,
    mut nft_orders: Vec<NftOrder>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut response = Response::new();

    let nft_order = nft_orders.remove(0);

    only_nft_owner(&deps.querier, deps.api, &sender, &collection, &nft_order.token_id)?;

    let sell_quote = deps
        .querier
        .query_wasm_smart::<PoolQuoteResponse>(
            &config.infinity_index,
            &QueryMsg::QuoteSellToPool {
                collection: collection.to_string(),
                limit: 1,
            },
        )?
        .pool_quotes
        .pop();

    if !sell_quote.is_some() {
        return Ok(response);
    }

    let sell_quote = sell_quote.unwrap();

    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::ApproveAll {
            operator: sell_quote.pool.to_string(),
            expires: None,
        })?,
        funds: vec![],
    }));

    response = response.add_submessage(transfer_nft(
        &collection,
        &nft_order.token_id,
        &env.contract.address,
    ));

    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: sell_quote.pool.to_string(),
        msg: to_binary(&InfinityPoolExecuteMsg::SwapNftsForTokens {
            token_id: nft_order.token_id,
            min_output: nft_order.amount,
            asset_recipient: sender.to_string(),
            finder: None,
        })?,
        funds: vec![],
    }));

    if nft_orders.len() > 0 {
        response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::SwapNftsForTokens {
                collection: collection.to_string(),
                sender: sender.to_string(),
                nft_orders,
            })?,
            funds: vec![],
        }));
    }

    Ok(response)
}
