use crate::msg::QueryMsg;

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
