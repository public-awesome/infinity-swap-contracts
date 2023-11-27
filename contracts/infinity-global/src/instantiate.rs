use crate::{
    constants::{CONTRACT_NAME, CONTRACT_VERSION},
    msg::InstantiateMsg,
    state::{GLOBAL_CONFIG, MIN_PRICES},
};

use cosmwasm_std::{DepsMut, Env, MessageInfo, StdError};
use cw2::set_contract_version;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let global_config = msg.global_config.str_to_addr(deps.api)?;
    GLOBAL_CONFIG.save(deps.storage, &global_config)?;

    for min_price in msg.min_prices {
        if MIN_PRICES.has(deps.storage, min_price.denom.clone()) {
            return Err(StdError::generic_err("Duplicate min price"));
        } else {
            MIN_PRICES.save(deps.storage, min_price.denom, &min_price.amount)?;
        }
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}
