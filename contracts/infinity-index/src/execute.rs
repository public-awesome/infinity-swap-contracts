use crate::msg::ExecuteMsg;
use crate::state::PoolQuote;
use crate::{
    error::ContractError,
    state::{buy_from_pool_quotes, sell_to_pool_quotes},
};

use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Uint128};
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
        ExecuteMsg::UpdateBuyFromPoolQuote {
            collection,
            quote_price,
        } => execute_update_buy_from_pool_quote(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            quote_price,
        ),
        ExecuteMsg::UpdateSellToPoolQuote {
            collection,
            quote_price,
        } => execute_update_sell_to_pool_quote(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            quote_price,
        ),
    }
}

pub fn execute_update_buy_from_pool_quote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    quote_price: Option<Uint128>,
) -> Result<Response, ContractError> {
    if let Some(_quote_price) = quote_price {
        buy_from_pool_quotes().save(
            deps.storage,
            info.sender.clone(),
            &PoolQuote {
                pool: info.sender,
                collection,
                quote_price: _quote_price,
            },
        )?;
    } else {
        buy_from_pool_quotes().remove(deps.storage, info.sender)?;
    }

    Ok(Response::default())
}

pub fn execute_update_sell_to_pool_quote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    quote_price: Option<Uint128>,
) -> Result<Response, ContractError> {
    if let Some(_quote_price) = quote_price {
        sell_to_pool_quotes().save(
            deps.storage,
            info.sender.clone(),
            &PoolQuote {
                pool: info.sender,
                collection,
                quote_price: _quote_price,
            },
        )?;
    } else {
        sell_to_pool_quotes().remove(deps.storage, info.sender)?;
    }

    Ok(Response::default())
}
