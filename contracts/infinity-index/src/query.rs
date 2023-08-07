use crate::msg::{PairQuoteOffset, QueryMsg};
use crate::state::{buy_from_pair_quotes, sell_to_pair_quotes, PairQuote};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};
use sg_index_query::{QueryOptions, QueryOptionsInternal};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SellToPairQuotes {
            collection,
            denom,
            query_options,
        } => to_binary(&query_sell_to_pair_quotes(
            deps,
            deps.api.addr_validate(&collection)?,
            denom,
            query_options.unwrap_or(QueryOptions::default()),
        )?),
        QueryMsg::BuyFromPairQuotes {
            collection,
            denom,
            query_options,
        } => to_binary(&query_buy_from_pair_quotes(
            deps,
            deps.api.addr_validate(&collection)?,
            denom,
            query_options.unwrap_or(QueryOptions::default()),
        )?),
    }
}

pub fn query_sell_to_pair_quotes(
    deps: Deps,
    collection: Addr,
    denom: String,
    query_options: QueryOptions<PairQuoteOffset>,
) -> StdResult<Vec<PairQuote>> {
    let QueryOptionsInternal {
        limit,
        order,
        min,
        max,
    } = query_options.unpack(
        &(|offset| (offset.amount, Addr::unchecked(offset.pair.clone()))),
        None,
        None,
    );

    let results = sell_to_pair_quotes()
        .idx
        .collection_quote
        .sub_prefix((collection, denom))
        .range_raw(deps.storage, min, max, order)
        .take(limit)
        .map(|res| res.map(|(_, pq)| pq))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(results)
}

pub fn query_buy_from_pair_quotes(
    deps: Deps,
    collection: Addr,
    denom: String,
    query_options: QueryOptions<PairQuoteOffset>,
) -> StdResult<Vec<PairQuote>> {
    let QueryOptionsInternal {
        limit,
        order,
        min,
        max,
    } = query_options.unpack(
        &(|offset| (offset.amount, Addr::unchecked(offset.pair.clone()))),
        None,
        None,
    );

    let results = buy_from_pair_quotes()
        .idx
        .collection_quote
        .sub_prefix((collection, denom))
        .range_raw(deps.storage, min, max, order)
        .take(limit)
        .map(|res| res.map(|(_, pq)| pq))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(results)
}
