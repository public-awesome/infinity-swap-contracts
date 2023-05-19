use crate::{msg::QueryMsg, state::INFINITY_GLOBAL};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::InfinityGlobal {} => to_binary(&query_infinity_global(deps)?),
    }
}

pub fn query_infinity_global(deps: Deps) -> StdResult<Addr> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    Ok(infinity_global)
}
