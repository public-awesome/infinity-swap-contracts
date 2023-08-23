use crate::helpers::generate_instantiate_2_addr;
use crate::msg::{NextPairResponse, QueryMsg};
use crate::state::{INFINITY_GLOBAL, SENDER_COUNTER};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult};
use infinity_global::load_global_config;
use sg_index_query::{QueryBound, QueryOptions, QueryOptionsInternal};
use std::cmp::{max, min};

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

    let num_pairs = num_pairs_option.unwrap();

    let QueryOptionsInternal {
        limit,
        order,
        ..
    } = query_options.unpack(&(|&offset| offset), None, None);

    let qo_min = query_options.min.unwrap_or(QueryBound::Inclusive(0u64));
    let min_index = match qo_min {
        QueryBound::Inclusive(min_index) => min_index,
        QueryBound::Exclusive(min_index) => min_index + 1,
    };

    let qo_max = query_options.max.unwrap_or(QueryBound::Inclusive(u64::MAX));
    let max_index = min(
        match qo_max {
            QueryBound::Inclusive(max_index) => max_index,
            QueryBound::Exclusive(max_index) => max_index - 1,
        },
        num_pairs - 1,
    );

    let (start, end): (u64, u64) = match order {
        Order::Ascending => (min_index, min(min_index + limit as u64, max_index)),
        Order::Descending => (max_index, max(max_index - limit as u64, min_index)),
    };

    let mut retval: Vec<(u64, Addr)> = vec![];

    let code_id = deps.querier.query_wasm_contract_info(&env.contract.address)?.code_id;

    for idx in start..end {
        let (pair, _) = generate_instantiate_2_addr(deps, &env, &owner, idx, code_id).unwrap();
        retval.push((idx, pair));
    }

    Ok(retval)
}
