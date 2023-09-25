use crate::helpers::{generate_instantiate_2_addr, index_range_from_query_options};
use crate::msg::{NextPairResponse, QueryMsg, QuotesResponse};
use crate::state::{INFINITY_GLOBAL, SENDER_COUNTER};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdError, StdResult, Uint128};
use infinity_global::{load_global_config, GlobalConfig};
use infinity_pair::helpers::load_payout_context;
use infinity_pair::pair::Pair;
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
            code_id,
            query_options,
        } => to_binary(&query_pairs_by_owner(
            deps,
            env,
            deps.api.addr_validate(&owner)?,
            code_id,
            query_options.unwrap_or_default(),
        )?),
        QueryMsg::SimSellToPairQuotes {
            pair,
            limit,
        } => to_binary(&query_sim_sell_to_pair_quotes(deps, pair, limit)?),
        QueryMsg::SimBuyFromPairQuotes {
            pair,
            limit,
        } => to_binary(&query_sim_buy_from_pair_quotes(deps, pair, limit)?),
    }
}

pub fn query_next_pair(deps: Deps, env: Env, sender: Addr) -> StdResult<NextPairResponse> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let GlobalConfig {
        infinity_pair_code_id: code_id,
        ..
    } = load_global_config(&deps.querier, &infinity_global)?;
    let counter_key = (sender.clone(), code_id);
    let counter = SENDER_COUNTER.may_load(deps.storage, counter_key)?.unwrap_or_default();

    let (pair, salt) = generate_instantiate_2_addr(deps, &env, &sender, counter, code_id).unwrap();

    Ok(NextPairResponse {
        sender,
        code_id,
        counter,
        salt,
        pair,
    })
}

pub fn query_pairs_by_owner(
    deps: Deps,
    env: Env,
    owner: Addr,
    code_id: u64,
    query_options: QueryOptions<u64>,
) -> StdResult<Vec<(u64, Addr)>> {
    let counter_key = (owner.clone(), code_id);
    let num_pairs_option = SENDER_COUNTER.may_load(deps.storage, counter_key)?;
    if num_pairs_option.is_none() {
        return Ok(vec![]);
    }

    let range = index_range_from_query_options(num_pairs_option.unwrap(), query_options);

    let mut retval: Vec<(u64, Addr)> = vec![];

    for idx in range {
        let (pair, _) = generate_instantiate_2_addr(deps, &env, &owner, idx, code_id).unwrap();
        retval.push((idx, pair));
    }

    Ok(retval)
}

pub fn query_sim_sell_to_pair_quotes(
    deps: Deps,
    mut pair: Pair,
    limit: u32,
) -> StdResult<QuotesResponse> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let payout_context = load_payout_context(
        deps,
        &infinity_global,
        &pair.immutable.collection,
        &pair.immutable.denom,
    )
    .map_err(|_| StdError::generic_err("failed to load payout context".to_string()))?;

    let mut quotes: Vec<Uint128> = vec![];

    let mut idx = 0u32;

    pair.update_sell_to_pair_quote_summary(&payout_context);
    while idx < limit {
        if let Some(quote_summary) = &pair.internal.sell_to_pair_quote_summary {
            quotes.push(quote_summary.seller_amount);
        } else {
            break;
        }

        pair.sim_swap_nft_for_tokens(&payout_context);

        idx += 1;
    }

    Ok(QuotesResponse {
        denom: pair.immutable.denom,
        quotes,
    })
}

pub fn query_sim_buy_from_pair_quotes(
    deps: Deps,
    mut pair: Pair,
    limit: u32,
) -> StdResult<QuotesResponse> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let payout_context = load_payout_context(
        deps,
        &infinity_global,
        &pair.immutable.collection,
        &pair.immutable.denom,
    )
    .map_err(|_| StdError::generic_err("failed to load payout context".to_string()))?;

    let mut quotes: Vec<Uint128> = vec![];

    let mut idx = 0u32;

    pair.update_buy_from_pair_quote_summary(&payout_context);
    while idx < limit {
        if let Some(quote_summary) = &pair.internal.buy_from_pair_quote_summary {
            quotes.push(quote_summary.total());
        } else {
            break;
        }

        pair.sim_swap_tokens_for_nft(&payout_context);

        idx += 1;
    }

    Ok(QuotesResponse {
        denom: pair.immutable.denom,
        quotes,
    })
}
