use crate::{
    msg::{GlobalConfigResponse, QueryMsg},
    state::GLOBAL_CONFIG,
};

use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GlobalConfig {} => to_binary(&query_global_config(deps, env)?),
    }
}

pub fn query_global_config(deps: Deps, _env: Env) -> StdResult<GlobalConfigResponse> {
    let config = GLOBAL_CONFIG.load(deps.storage)?;
    Ok(GlobalConfigResponse {
        config,
    })
}
