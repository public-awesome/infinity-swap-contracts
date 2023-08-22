use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::PAIR_COUNTER;

use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    PAIR_COUNTER.save(deps.storage, &0)?;

    let sudo_params = msg.sudo_params.str_to_addr(deps.api)?;
    sudo_params.save(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}
