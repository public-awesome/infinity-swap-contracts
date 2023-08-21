use crate::helpers::generate_instantiate_2_addr;
use crate::msg::{NextPairResponse, QueryMsg};
use crate::state::{INFINITY_GLOBAL, SENDER_COUNTER};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};
use infinity_global::load_global_config;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::NextPair {
            sender,
        } => to_binary(&query_next_pair(deps, env, deps.api.addr_validate(&sender)?)?),
    }
}

pub fn query_next_pair(deps: Deps, env: Env, sender: Addr) -> StdResult<NextPairResponse> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let global_config = load_global_config(&deps.querier, &infinity_global)?;
    let counter = SENDER_COUNTER.may_load(deps.storage, sender.clone())?.unwrap_or_default();

    let (pair, salt) = generate_instantiate_2_addr(
        deps,
        &env,
        &sender,
        counter,
        global_config.infinity_pair_code_id,
    )
    .unwrap();

    Ok(NextPairResponse {
        sender,
        pair,
        salt,
    })
}
