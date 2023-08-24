use crate::helpers::{generate_instantiate_2_addr, index_range_from_query_options};
use crate::msg::{NextPairResponse, QueryMsg};
use crate::state::{INFINITY_GLOBAL, SENDER_COUNTER};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};
use infinity_global::load_global_config;
use sg_index_query::QueryOptions;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::NextPair {
            sender,
        } => to_binary(&query_next_pair(deps, env, deps.api.addr_validate(&sender)?)?),
        QueryMsg::PairsByOwner {
            owner,
            query_options,
        } => to_binary(&query_pairs_by_owner(
            deps,
            env,
            deps.api.addr_validate(&owner)?,
            query_options.unwrap_or_default(),
        )?),
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
        counter,
        salt,
        pair,
    })
}

pub fn query_pairs_by_owner(
    deps: Deps,
    env: Env,
    owner: Addr,
    query_options: QueryOptions<u64>,
) -> StdResult<Vec<(u64, Addr)>> {
    let num_pairs_option = SENDER_COUNTER.may_load(deps.storage, owner.clone())?;
    if num_pairs_option.is_none() {
        return Ok(vec![]);
    }

    let range = index_range_from_query_options(num_pairs_option.unwrap(), query_options);

    let mut retval: Vec<(u64, Addr)> = vec![];

    let code_id = deps.querier.query_wasm_contract_info(&env.contract.address)?.code_id;

    for idx in range {
        let (pair, _) = generate_instantiate_2_addr(deps, &env, &owner, idx, code_id).unwrap();
        retval.push((idx, pair));
    }

    Ok(retval)
}
