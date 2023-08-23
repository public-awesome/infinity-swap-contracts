use crate::{
    helpers::{load_pair, load_payout_context},
    msg::{NftDepositsResponse, QueryMsg, QuotesResponse},
    pair::Pair,
    state::{INFINITY_GLOBAL, NFT_DEPOSITS, PAIR_IMMUTABLE},
};

use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult, Uint128};
use sg_index_query::{QueryOptions, QueryOptionsInternal};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pair {} => to_binary(&query_pair(deps, env)?),
        QueryMsg::NftDeposits {
            query_options,
        } => to_binary(&query_nft_deposits(deps, query_options.unwrap_or_default())?),
        QueryMsg::SellToPairQuotes {
            limit,
        } => to_binary(&query_sell_to_pair_quotes(deps, env, limit)?),
        QueryMsg::BuyFromPairQuotes {
            limit,
        } => to_binary(&query_buy_from_pair_quotes(deps, env, limit)?),
    }
}

pub fn query_pair(deps: Deps, env: Env) -> StdResult<Pair> {
    let pair = load_pair(&env.contract.address, deps.storage, &deps.querier)
        .map_err(|_| StdError::generic_err("failed to load pair".to_string()))?;

    Ok(pair)
}

pub fn query_nft_deposits(
    deps: Deps,
    query_options: QueryOptions<String>,
) -> StdResult<NftDepositsResponse> {
    let collection = PAIR_IMMUTABLE.load(deps.storage)?.collection;

    let QueryOptionsInternal {
        limit,
        order,
        min,
        max,
    } = query_options.unpack(&(|offset| offset.clone()), None, None);

    let token_ids = NFT_DEPOSITS
        .range(deps.storage, min, max, order)
        .take(limit)
        .map(|res| res.map(|(k, _)| k))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(NftDepositsResponse {
        collection,
        token_ids,
    })
}

pub fn query_sell_to_pair_quotes(deps: Deps, env: Env, limit: u32) -> StdResult<QuotesResponse> {
    let mut pair = load_pair(&env.contract.address, deps.storage, &deps.querier)
        .map_err(|_| StdError::generic_err("failed to load pair".to_string()))?;

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

pub fn query_buy_from_pair_quotes(deps: Deps, env: Env, limit: u32) -> StdResult<QuotesResponse> {
    let mut pair = load_pair(&env.contract.address, deps.storage, &deps.querier)
        .map_err(|_| StdError::generic_err("failed to load pair".to_string()))?;

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
