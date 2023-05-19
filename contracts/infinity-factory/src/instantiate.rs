use crate::msg::InstantiateMsg;
use crate::{error::ContractError, state::INFINITY_GLOBAL};

use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;
use sg_std::Response;

pub const CONTRACT_NAME: &str = "crates.io:infinity-factory";
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

    INFINITY_GLOBAL.save(deps.storage, &deps.api.addr_validate(&msg.infinity_global)?)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("infinity_global", msg.infinity_global))
}
