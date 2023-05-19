use crate::{msg::ExecuteMsg, state::SWAP_CONTEXT, ContractError};

use cosmwasm_std::{to_binary, DepsMut, Env, Reply, WasmMsg};
use sg_std::{Response, SubMsg};

pub const NFT_TO_TOKEN_REPLY_ID: u64 = 1111;
pub const TOKEN_TO_SPECIFIC_NFT_REPLY_ID: u64 = 2222;
pub const TOKEN_TO_ANY_NFT_REPLY_ID: u64 = 3333;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        NFT_TO_TOKEN_REPLY_ID => reply_nft_to_token_swap(deps, env, msg),
        TOKEN_TO_SPECIFIC_NFT_REPLY_ID => reply_token_to_specific_nft_swap(deps, env, msg),
        TOKEN_TO_ANY_NFT_REPLY_ID => reply_token_to_any_nft_swap(deps, env, msg),
        id => unreachable!("unknown reply ID: `{id}`"),
    }
}

pub fn reply_nft_to_token_swap(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    let swap_context = SWAP_CONTEXT.load(deps.storage)?;

    if msg.result.is_err() {
        if swap_context.robust {
            return Ok(Response::new());
        } else {
            return Err(ContractError::SwapFailed);
        }
    }

    let mut response = Response::new();

    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SwapNftsForTokensInternal {})?,
        funds: vec![],
    }));

    Ok(response)
}

pub fn reply_token_to_specific_nft_swap(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    let swap_context = SWAP_CONTEXT.load(deps.storage)?;

    if msg.result.is_err() && !swap_context.robust {
        return Err(ContractError::SwapFailed);
    }

    let mut response = Response::new();

    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SwapTokensForSpecificNftsInternal {})?,
        funds: vec![],
    }));

    Ok(response)
}

pub fn reply_token_to_any_nft_swap(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    let swap_context = SWAP_CONTEXT.load(deps.storage)?;

    if msg.result.is_err() {
        if swap_context.robust {
            return Ok(Response::new());
        } else {
            return Err(ContractError::SwapFailed);
        }
    }

    let mut response = Response::new();

    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SwapTokensForAnyNftsInternal {})?,
        funds: vec![],
    }));

    Ok(response)
}
