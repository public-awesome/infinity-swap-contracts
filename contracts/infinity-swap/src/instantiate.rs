use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::{Config, CONFIG, POOL_COUNTER};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;
use cw_utils::maybe_addr;
use sg_std::Response;

pub const CONTRACT_NAME: &str = "crates.io:infinity-swap";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    POOL_COUNTER.save(deps.storage, &1)?;
    CONFIG.save(
        deps.storage,
        &Config {
            denom: msg.denom.clone(),
            marketplace_addr: deps.api.addr_validate(&msg.marketplace_addr)?,
            developer: maybe_addr(deps.api, msg.developer)?,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("denom", msg.denom)
        .add_attribute("marketplace_addr", msg.marketplace_addr))
}
