use crate::helpers::validate_infinity_pool;
use crate::msg::ExecuteMsg;
use crate::state::PoolQuote;
use crate::{
    error::ContractError,
    state::{buy_from_pool_quotes, sell_to_pool_quotes},
};

use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Uint128};
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
    _env: Env,
    info: MessageInfo,
    collection: Addr,
    quote_price: Option<Uint128>,
) -> Result<Response, ContractError> {
    validate_infinity_pool(deps.as_ref(), &info.sender)?;

    let event = match quote_price {
        Some(quote_price) => {
            buy_from_pool_quotes().save(
                deps.storage,
                info.sender.clone(),
                &PoolQuote {
                    pool: info.sender.clone(),
                    collection: collection.clone(),
                    quote_price,
                },
            )?;
            Event::new("update-buy-from-pool-quote")
                .add_attribute("pool", info.sender.to_string())
                .add_attribute("collection", collection.to_string())
                .add_attribute("quote_price", quote_price.to_string())
        },
        None => {
            buy_from_pool_quotes().remove(deps.storage, info.sender.clone())?;
            Event::new("remove-buy-from-pool-quote")
                .add_attribute("pool", info.sender.to_string())
                .add_attribute("collection", collection.to_string())
        },
    };

    Ok(Response::new().add_event(event))
}

pub fn execute_update_sell_to_pool_quote(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: Addr,
    quote_price: Option<Uint128>,
) -> Result<Response, ContractError> {
    validate_infinity_pool(deps.as_ref(), &info.sender)?;

    let event = match quote_price {
        Some(quote_price) => {
            sell_to_pool_quotes().save(
                deps.storage,
                info.sender.clone(),
                &PoolQuote {
                    pool: info.sender.clone(),
                    collection: collection.clone(),
                    quote_price,
                },
            )?;
            Event::new("update-sell-to-pool-quote")
                .add_attribute("pool", info.sender.to_string())
                .add_attribute("collection", collection.to_string())
                .add_attribute("quote_price", quote_price.to_string())
        },
        None => {
            sell_to_pool_quotes().remove(deps.storage, info.sender.clone())?;
            Event::new("remove-sell-to-pool-quote")
                .add_attribute("pool", info.sender.to_string())
                .add_attribute("collection", collection.to_string())
        },
    };

    Ok(Response::new().add_event(event))
}
