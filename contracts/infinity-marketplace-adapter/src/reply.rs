use crate::{state::FORWARD_NFTS, ContractError};
use cosmwasm_std::{DepsMut, Env, Reply};
use sg_marketplace_common::transfer_nft;
use sg_std::Response;

pub const BUY_NOW_REPLY_ID: u64 = 1111;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        BUY_NOW_REPLY_ID => reply_buy_now(deps, env, msg),
        id => unreachable!("unknown reply ID: `{id}`"),
    }
}

pub fn reply_buy_now(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let result = msg.result.unwrap();

    let event = result
        .events
        .iter()
        .find(|e| e.ty == "wasm-finalize-sale")
        .unwrap();

    let collection_str = &event
        .attributes
        .iter()
        .find(|a| a.key == "collection")
        .unwrap()
        .value;

    let collection = deps.api.addr_validate(&collection_str)?;

    let token_id = &event
        .attributes
        .iter()
        .find(|a| a.key == "token_id")
        .unwrap()
        .value;

    let forward_to = FORWARD_NFTS.load(deps.storage, (collection.clone(), token_id.to_string()))?;
    FORWARD_NFTS.remove(deps.storage, (collection.clone(), token_id.to_string()));

    Ok(Response::new().add_submessage(transfer_nft(&collection, token_id, &forward_to)))
}
