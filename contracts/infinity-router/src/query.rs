use crate::msg::QueryMsg;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    // match msg {};

    Ok(to_binary(&"query not supported")?)
}
