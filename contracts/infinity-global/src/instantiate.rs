use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::{GlobalConfig, GLOBAL_CONFIG};

use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo};
use cw2::set_contract_version;
use sg_std::Response;

pub const CONTRACT_NAME: &str = "crates.io:infinity-global";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    GLOBAL_CONFIG.save(
        deps.storage,
        &GlobalConfig {
            infinity_index: deps.api.addr_validate(&msg.infinity_index)?,
            infinity_factory: deps.api.addr_validate(&msg.infinity_factory)?,
            min_price: msg.min_price,
            pool_creation_fee: msg.pool_creation_fee,
            trading_fee_percent: Decimal::percent(msg.trading_fee_bps),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("infinity_index", msg.infinity_index)
        .add_attribute("infinity_factory", msg.infinity_factory)
        .add_attribute("min_price", msg.min_price)
        .add_attribute("pool_creation_fee", msg.pool_creation_fee)
        .add_attribute("trading_fee_percent", msg.trading_fee_bps.to_string()))
}
