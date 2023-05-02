use crate::msg::ExecuteMsg;
use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let _api = deps.api;

    // match msg {};

    Ok(Response::new())
}
