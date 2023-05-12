#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::msg::SudoMsg;
use crate::{error::ContractError, state::GLOBAL_CONFIG};
use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Event, Uint128};
use cw_utils::maybe_addr;
use sg_std::Response;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        SudoMsg::UpdateConfig {
            infinity_index,
            infinity_factory,
            min_price,
            pool_creation_fee,
            trading_fee_bps,
        } => sudo_update_config(
            deps,
            env,
            maybe_addr(api, infinity_index)?,
            maybe_addr(api, infinity_factory)?,
            min_price,
            pool_creation_fee,
            trading_fee_bps,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sudo_update_config(
    deps: DepsMut,
    _env: Env,
    infinity_index: Option<Addr>,
    infinity_factory: Option<Addr>,
    min_price: Option<Uint128>,
    pool_creation_fee: Option<Uint128>,
    trading_fee_bps: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = GLOBAL_CONFIG.load(deps.storage)?;

    let mut event = Event::new("sudo-update-params");

    if let Some(infinity_index) = infinity_index {
        config.infinity_index = infinity_index;
        event = event.add_attribute("infinity_index", &config.infinity_index);
    }
    if let Some(infinity_factory) = infinity_factory {
        config.infinity_factory = infinity_factory;
        event = event.add_attribute("infinity_factory", &config.infinity_factory);
    }
    if let Some(min_price) = min_price {
        config.min_price = min_price;
        event = event.add_attribute("min_price", config.min_price.to_string());
    }
    if let Some(pool_creation_fee) = pool_creation_fee {
        config.pool_creation_fee = pool_creation_fee;
        event = event.add_attribute("pool_creation_fee", config.pool_creation_fee.to_string());
    }
    if let Some(trading_fee_bps) = trading_fee_bps {
        config.trading_fee_percent = Decimal::percent(trading_fee_bps);
        event = event.add_attribute("trading_fee_percent", config.trading_fee_percent.to_string());
    }

    GLOBAL_CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_event(event))
}
