use crate::msg::{QueryMsg};
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, Env, StdResult};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    to_binary(&{})
}
